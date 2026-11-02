//! Cas d'usage : persistance de la session et relance silencieuse (refresh token au coffre).

use std::sync::Arc;
use std::time::Duration;

use tuxteams_domaine::session::{Compte, RaisonEchec};

use crate::ports::coffre::PortCoffre;
use crate::ports::horloge::Horloge;
use crate::ports::jetons::{Authentifie, ErreurAuth, PortRenouvellement};

/// Sort du refresh token après une authentification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Persistance {
    /// Conservé dans le trousseau : la session survivra au redémarrage.
    Active,
    /// Trousseau indisponible : rien n'est écrit, l'utilisateur doit en être prévenu.
    Indisponible,
}

/// Issue de la tentative de relance au démarrage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultatRelance {
    Connecte {
        compte: Compte,
        persistance: Persistance,
    },
    /// Écran de connexion ; `raison` explique pourquoi (session révoquée, réseau), s'il y en a une.
    AConnecter { raison: Option<RaisonEchec> },
}

/// Délai maximal de la relance silencieuse, distinct du délai HTTP général (30 s) : au-delà,
/// l'utilisateur est renvoyé à l'écran de connexion plutôt que de rester devant un chargement.
pub const DELAI_RELANCE: Duration = Duration::from_secs(10);

pub struct CasSession {
    coffre: Arc<dyn PortCoffre>,
    renouvellement: Arc<dyn PortRenouvellement>,
    horloge: Arc<dyn Horloge>,
}

impl CasSession {
    pub fn new(
        coffre: Arc<dyn PortCoffre>,
        renouvellement: Arc<dyn PortRenouvellement>,
        horloge: Arc<dyn Horloge>,
    ) -> Self {
        Self {
            coffre,
            renouvellement,
            horloge,
        }
    }

    /// Relance silencieuse : relit le refresh token et l'échange, sans nouveau code.
    pub async fn restore(&self) -> ResultatRelance {
        let refresh = match self.coffre.load_refresh().await {
            Ok(Some(refresh)) => refresh,
            Ok(None) => return ResultatRelance::AConnecter { raison: None },
            Err(erreur) => {
                tracing::warn!(%erreur, "trousseau illisible : relance impossible");
                return ResultatRelance::AConnecter { raison: None };
            }
        };
        let echange = tokio::select! {
            resultat = self.renouvellement.refresh(&refresh) => resultat,
            () = self.horloge.sleep(DELAI_RELANCE) => {
                tracing::warn!("relance silencieuse trop longue : retour à la connexion");
                Err(ErreurAuth::Reseau)
            }
        };
        match echange {
            Ok(authentifie) => {
                // Rotation : le nouveau refresh token remplace l'ancien, sinon l'ancien reste valable.
                let persistance = match &authentifie.refresh {
                    Some(_) => self.save(&authentifie).await,
                    None => Persistance::Active,
                };
                ResultatRelance::Connecte {
                    compte: authentifie.compte,
                    persistance,
                }
            }
            Err(ErreurAuth::SessionRevoquee) => {
                tracing::info!("session révoquée ou expirée : reconnexion requise");
                if let Err(erreur) = self.coffre.delete_refresh().await {
                    tracing::warn!(%erreur, "suppression du secret périmé impossible");
                }
                ResultatRelance::AConnecter {
                    raison: Some(RaisonEchec::SessionExpiree),
                }
            }
            Err(erreur) => ResultatRelance::AConnecter {
                raison: Some(erreur.raison()),
            },
        }
    }

    /// Conserve le refresh token d'une authentification réussie.
    pub async fn save(&self, authentifie: &Authentifie) -> Persistance {
        let Some(refresh) = &authentifie.refresh else {
            tracing::warn!("aucun refresh token fourni : session non persistée");
            return Persistance::Indisponible;
        };
        match self.coffre.save_refresh(&authentifie.compte, refresh).await {
            Ok(()) => Persistance::Active,
            Err(erreur) => {
                tracing::warn!(%erreur, "trousseau indisponible : session non persistée");
                Persistance::Indisponible
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::Mutex;

    use super::*;
    use crate::cas::doubles::{
        CoffreFaux, HorlogeInstantanee, HorlogeSansFin, RenouvellementBloque, RenouvellementFaux,
        VALEUR_ROTATION, VALEUR_STOCKEE, authentifie, compte,
    };
    use crate::ports::jetons::ErreurAuth;

    fn cas(coffre: Arc<CoffreFaux>, renouvellement: Arc<RenouvellementFaux>) -> CasSession {
        CasSession::new(coffre, renouvellement, Arc::new(HorlogeSansFin))
    }

    #[tokio::test]
    async fn la_relance_utilise_le_refresh_token_sans_reafficher_le_code() {
        let coffre = CoffreFaux::avec(Some(VALEUR_STOCKEE), false);
        let renouvellement = RenouvellementFaux::new(Ok(authentifie(None)));
        let cas = cas(coffre, renouvellement.clone());

        let resultat = cas.restore().await;

        assert_eq!(
            resultat,
            ResultatRelance::Connecte {
                compte: compte(),
                persistance: Persistance::Active
            }
        );
        assert_eq!(
            *renouvellement.recus.lock().expect("verrou"),
            vec![VALEUR_STOCKEE.to_owned()]
        );
    }

    #[tokio::test]
    async fn la_rotation_du_refresh_token_renvoye_est_persistee() {
        let coffre = CoffreFaux::avec(Some(VALEUR_STOCKEE), false);
        let renouvellement = RenouvellementFaux::new(Ok(authentifie(Some(VALEUR_ROTATION))));
        let cas = cas(coffre.clone(), renouvellement);

        cas.restore().await;

        assert_eq!(coffre.valeur().as_deref(), Some(VALEUR_ROTATION));
    }

    #[tokio::test]
    async fn sans_secret_conserve_on_passe_a_l_ecran_de_connexion() {
        let renouvellement = RenouvellementFaux::new(Ok(authentifie(None)));
        let cas = cas(CoffreFaux::vide(), renouvellement.clone());

        let resultat = cas.restore().await;

        assert_eq!(resultat, ResultatRelance::AConnecter { raison: None });
        assert!(renouvellement.recus.lock().expect("verrou").is_empty());
    }

    #[tokio::test]
    async fn invalid_grant_supprime_le_secret_perime_et_renvoie_a_la_connexion() {
        // Le cas de session n'a aucun accès au port de cache : seul le secret périmé est retiré.
        // TODO(tache-6): prouver la conservation du cache (compteur de vidages à 0) quand le cache existera.
        let coffre = CoffreFaux::avec(Some(VALEUR_STOCKEE), false);
        let renouvellement = RenouvellementFaux::new(Err(ErreurAuth::SessionRevoquee));
        let cas = cas(coffre.clone(), renouvellement);

        let resultat = cas.restore().await;

        assert_eq!(
            resultat,
            ResultatRelance::AConnecter {
                raison: Some(RaisonEchec::SessionExpiree)
            }
        );
        assert_eq!(coffre.valeur(), None, "le secret périmé est supprimé");
    }

    #[tokio::test]
    async fn une_panne_reseau_a_la_relance_conserve_le_secret() {
        let coffre = CoffreFaux::avec(Some(VALEUR_STOCKEE), false);
        let renouvellement = RenouvellementFaux::new(Err(ErreurAuth::Reseau));
        let cas = cas(coffre.clone(), renouvellement);

        let resultat = cas.restore().await;

        assert_eq!(
            resultat,
            ResultatRelance::AConnecter {
                raison: Some(RaisonEchec::Reseau)
            }
        );
        assert_eq!(coffre.valeur().as_deref(), Some(VALEUR_STOCKEE));
    }

    #[tokio::test]
    async fn une_relance_trop_longue_renvoie_a_la_connexion_avec_un_message() {
        let coffre = CoffreFaux::avec(Some(VALEUR_STOCKEE), false);
        let cas = CasSession::new(
            coffre.clone(),
            Arc::new(RenouvellementBloque),
            Arc::new(HorlogeInstantanee::default()),
        );

        let resultat = cas.restore().await;

        assert_eq!(
            resultat,
            ResultatRelance::AConnecter {
                raison: Some(RaisonEchec::Reseau)
            }
        );
        assert_eq!(
            coffre.valeur().as_deref(),
            Some(VALEUR_STOCKEE),
            "un délai dépassé n'est pas une révocation : le secret reste"
        );
    }

    #[tokio::test]
    async fn le_delai_de_relance_est_de_dix_secondes() {
        let horloge = Arc::new(HorlogeInstantanee::default());
        let cas = CasSession::new(
            CoffreFaux::avec(Some(VALEUR_STOCKEE), false),
            Arc::new(RenouvellementBloque),
            horloge.clone(),
        );

        cas.restore().await;

        assert_eq!(*horloge.pauses.lock().expect("verrou"), vec![10]);
    }

    #[tokio::test]
    async fn un_trousseau_indisponible_previent_l_utilisateur_sans_persister() {
        let coffre = CoffreFaux::avec(None, true);
        let cas = cas(
            coffre.clone(),
            RenouvellementFaux::new(Err(ErreurAuth::Reseau)),
        );

        let persistance = cas.save(&authentifie(Some(VALEUR_STOCKEE))).await;

        assert_eq!(persistance, Persistance::Indisponible);
        assert_eq!(coffre.valeur(), None, "rien n'est écrit sans trousseau");
    }

    #[tokio::test]
    async fn une_connexion_enregistre_le_refresh_token_au_trousseau() {
        let coffre = CoffreFaux::vide();
        let cas = cas(
            coffre.clone(),
            RenouvellementFaux::new(Err(ErreurAuth::Reseau)),
        );

        let persistance = cas.save(&authentifie(Some(VALEUR_STOCKEE))).await;

        assert_eq!(persistance, Persistance::Active);
        assert_eq!(coffre.valeur().as_deref(), Some(VALEUR_STOCKEE));
    }

    #[tokio::test]
    async fn une_authentification_sans_refresh_token_n_est_pas_persistee() {
        let cas = cas(
            CoffreFaux::vide(),
            RenouvellementFaux::new(Err(ErreurAuth::Reseau)),
        );

        let persistance = cas.save(&authentifie(None)).await;

        assert_eq!(persistance, Persistance::Indisponible);
    }

    /// Écrivain partagé qui capture tout ce que `tracing` émet.
    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Capture {
        fn write(&mut self, octets: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("verrou").extend_from_slice(octets);
            Ok(octets.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn le_refresh_token_n_est_jamais_journalise() {
        let capture = Capture::default();
        let ecrivain = capture.clone();
        let abonne = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_ansi(false)
            .with_writer(move || ecrivain.clone())
            .finish();
        let _garde = tracing::subscriber::set_default(abonne);
        let mut sorties = Vec::new();

        // Trousseau indisponible, rotation, session révoquée : chaque chemin qui journalise.
        let hors_service = CoffreFaux::avec(Some(VALEUR_STOCKEE), true);
        let c = cas(
            hors_service,
            RenouvellementFaux::new(Ok(authentifie(Some(VALEUR_ROTATION)))),
        );
        sorties.push(format!("{:?}", c.restore().await));
        sorties.push(format!(
            "{:?}",
            c.save(&authentifie(Some(VALEUR_ROTATION))).await
        ));
        let c = cas(
            CoffreFaux::avec(Some(VALEUR_STOCKEE), false),
            RenouvellementFaux::new(Ok(authentifie(Some(VALEUR_ROTATION)))),
        );
        sorties.push(format!("{:?}", c.restore().await));
        let c = cas(
            CoffreFaux::avec(Some(VALEUR_STOCKEE), false),
            RenouvellementFaux::new(Err(ErreurAuth::SessionRevoquee)),
        );
        sorties.push(format!("{:?}", c.restore().await));
        sorties.push(format!("{:?}", authentifie(Some(VALEUR_STOCKEE))));

        let journal = String::from_utf8(capture.0.lock().expect("verrou").clone()).expect("utf8");
        assert!(
            journal.contains("trousseau"),
            "le journal doit contenir l'avertissement du trousseau : {journal:?}"
        );
        for valeur in [VALEUR_STOCKEE, VALEUR_ROTATION] {
            assert!(!journal.contains(valeur), "secret journalisé : {journal}");
            for sortie in &sorties {
                assert!(
                    !sortie.contains(valeur),
                    "secret affiché par Debug : {sortie}"
                );
            }
        }
    }
}
