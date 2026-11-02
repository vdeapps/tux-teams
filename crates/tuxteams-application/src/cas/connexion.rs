//! Cas d'usage : connexion par device code (demande, sondage, expiration, annulation).

use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use tuxteams_domaine::session::{EtatConnexion, RaisonEchec};
use url::Url;

use crate::cas::session::{CasSession, Persistance};
use crate::ports::horloge::Horloge;
use crate::ports::jetons::{EtatSondage, PortConnexion};
use crate::ports::navigateur::{PortNavigateur, PortPressePapiers};

/// Majoration de l'intervalle à chaque `slow_down` (spécification OAuth device flow).
const MAJORATION_SLOW_DOWN: Duration = Duration::from_secs(5);
/// Plafond de l'intervalle de sondage après des `slow_down` répétés.
const INTERVALLE_MAX: Duration = Duration::from_secs(60);

/// Notification d'un nouvel état de connexion, appelée depuis la tâche de sondage.
pub type Notification<'a> = &'a (dyn Fn(EtatConnexion) + Send + Sync);

pub struct CasConnexion {
    connexion: Arc<dyn PortConnexion>,
    horloge: Arc<dyn Horloge>,
    navigateur: Arc<dyn PortNavigateur>,
    presse_papiers: Arc<dyn PortPressePapiers>,
    session: Arc<CasSession>,
}

impl CasConnexion {
    pub fn new(
        connexion: Arc<dyn PortConnexion>,
        horloge: Arc<dyn Horloge>,
        navigateur: Arc<dyn PortNavigateur>,
        presse_papiers: Arc<dyn PortPressePapiers>,
        session: Arc<CasSession>,
    ) -> Self {
        Self {
            connexion,
            horloge,
            navigateur,
            presse_papiers,
            session,
        }
    }

    /// Demande un code puis sonde jusqu'à l'issue ; chaque état est notifié, le dernier est rendu.
    /// L'annulation interrompt aussi l'attente en cours et rend `NonConnecte`.
    pub async fn start_connexion(
        &self,
        annulation: CancellationToken,
        notifier: Notification<'_>,
    ) -> EtatConnexion {
        let final_ = tokio::select! {
            biased;
            () = annulation.cancelled() => EtatConnexion::NonConnecte,
            etat = self.sonder(notifier) => etat,
        };
        notifier(final_.clone());
        final_
    }

    async fn sonder(&self, notifier: Notification<'_>) -> EtatConnexion {
        let demande = match self.connexion.start_device_code().await {
            Ok(demande) => demande,
            Err(erreur) => return EtatConnexion::Echec(erreur.raison()),
        };
        let expiration = self.horloge.now() + demande.expire_dans;
        notifier(EtatConnexion::CodeAffiche {
            code: demande.code_utilisateur.clone(),
            url: demande.url_verification.clone(),
            expiration,
        });
        let mut intervalle = demande.intervalle;
        loop {
            self.horloge.sleep(intervalle).await;
            if self.horloge.now() >= expiration {
                return EtatConnexion::Echec(RaisonEchec::Expire);
            }
            match self.connexion.poll_device_code(&demande).await {
                Ok(EtatSondage::EnAttente) => {}
                Ok(EtatSondage::PlusLent) => {
                    intervalle = (intervalle + MAJORATION_SLOW_DOWN).min(INTERVALLE_MAX);
                }
                Ok(EtatSondage::Termine(authentifie)) => {
                    let persistance = self.session.save(&authentifie).await;
                    return EtatConnexion::Connecte {
                        compte: authentifie.compte,
                        session_persistee: persistance == Persistance::Active,
                    };
                }
                Err(erreur) => return EtatConnexion::Echec(erreur.raison()),
            }
        }
    }

    /// Copie le code dans le presse-papiers puis ouvre la page de vérification.
    /// Les deux actions sont tentées ; la première erreur est rendue.
    pub fn open_verification(&self, code: &str, url: &Url) -> Result<(), ErreurBureau> {
        let copie = self
            .presse_papiers
            .copy_text(code)
            .map_err(|_| ErreurBureau::Copie);
        let ouverture = self
            .navigateur
            .open_url(url)
            .map_err(|_| ErreurBureau::Navigateur);
        copie.and(ouverture)
    }
}

/// Échec d'une action de bureau (copie ou ouverture du navigateur).
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ErreurBureau {
    #[error("copie du code impossible")]
    Copie,
    #[error("ouverture du navigateur impossible")]
    Navigateur,
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::{DateTime, TimeZone, Utc};
    use tuxteams_domaine::session::RaisonEchec;

    use super::*;
    use crate::cas::doubles::{
        CoffreFaux, HorlogeSansFin, RenouvellementFaux, VALEUR_STOCKEE, authentifie,
    };
    use crate::ports::jetons::{DemandeCode, ErreurAuth};
    use crate::ports::navigateur::{ErreurNavigateur, ErreurPressePapiers};

    fn demande(intervalle_s: u64, expire_s: u64) -> DemandeCode {
        DemandeCode {
            code_utilisateur: "ABCD1234".into(),
            code_appareil: "appareil-fictif".into(),
            url_verification: Url::parse("https://login.microsoft.com/device").expect("url"),
            expire_dans: Duration::from_secs(expire_s),
            intervalle: Duration::from_secs(intervalle_s),
        }
    }

    fn termine() -> Result<EtatSondage, ErreurAuth> {
        Ok(EtatSondage::Termine(authentifie(Some(VALEUR_STOCKEE))))
    }

    /// Session avec un coffre vide et un renouvellement inutilisé.
    fn session(coffre: Arc<CoffreFaux>) -> Arc<CasSession> {
        Arc::new(CasSession::new(
            coffre,
            RenouvellementFaux::new(Err(ErreurAuth::Reseau)),
            Arc::new(HorlogeSansFin),
        ))
    }

    /// Port scripté : rend les sondages dans l'ordre, puis `EnAttente`.
    struct ConnexionFausse {
        demande: Result<DemandeCode, ErreurAuth>,
        sondages: Mutex<VecDeque<Result<EtatSondage, ErreurAuth>>>,
        appels: Mutex<u32>,
    }

    impl ConnexionFausse {
        fn new(demande: DemandeCode, sondages: Vec<Result<EtatSondage, ErreurAuth>>) -> Arc<Self> {
            Arc::new(Self {
                demande: Ok(demande),
                sondages: Mutex::new(sondages.into()),
                appels: Mutex::new(0),
            })
        }
    }

    #[async_trait]
    impl PortConnexion for ConnexionFausse {
        async fn start_device_code(&self) -> Result<DemandeCode, ErreurAuth> {
            self.demande.clone()
        }
        async fn poll_device_code(&self, _: &DemandeCode) -> Result<EtatSondage, ErreurAuth> {
            *self.appels.lock().expect("verrou") += 1;
            self.sondages
                .lock()
                .expect("verrou")
                .pop_front()
                .unwrap_or(Ok(EtatSondage::EnAttente))
        }
    }

    /// Horloge fausse : `sleep` avance l'heure sans attendre et mémorise les pauses.
    struct HorlogeFausse {
        maintenant: Mutex<DateTime<Utc>>,
        pauses: Mutex<Vec<Duration>>,
        a_la_premiere_pause: Option<CancellationToken>,
    }

    impl HorlogeFausse {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                maintenant: Mutex::new(Utc.with_ymd_and_hms(2026, 11, 2, 9, 0, 0).unwrap()),
                pauses: Mutex::new(Vec::new()),
                a_la_premiere_pause: None,
            })
        }
        fn pauses(&self) -> Vec<u64> {
            let pauses = self.pauses.lock().expect("verrou");
            pauses.iter().map(Duration::as_secs).collect()
        }
    }

    #[async_trait]
    impl Horloge for HorlogeFausse {
        fn now(&self) -> DateTime<Utc> {
            *self.maintenant.lock().expect("verrou")
        }
        async fn sleep(&self, duree: Duration) {
            self.pauses.lock().expect("verrou").push(duree);
            if let Some(jeton) = &self.a_la_premiere_pause {
                jeton.cancel();
                std::future::pending::<()>().await;
            }
            let mut maintenant = self.maintenant.lock().expect("verrou");
            *maintenant += chrono::Duration::from_std(duree).expect("durée");
        }
    }

    struct BureauFaux {
        urls: Mutex<Vec<String>>,
        copies: Mutex<Vec<String>>,
        echec_copie: bool,
    }

    impl PortNavigateur for BureauFaux {
        fn open_url(&self, url: &Url) -> Result<(), ErreurNavigateur> {
            self.urls.lock().expect("verrou").push(url.to_string());
            Ok(())
        }
    }

    impl PortPressePapiers for BureauFaux {
        fn copy_text(&self, texte: &str) -> Result<(), ErreurPressePapiers> {
            if self.echec_copie {
                return Err(ErreurPressePapiers);
            }
            self.copies.lock().expect("verrou").push(texte.to_owned());
            Ok(())
        }
    }

    fn bureau(echec_copie: bool) -> Arc<BureauFaux> {
        Arc::new(BureauFaux {
            urls: Mutex::new(Vec::new()),
            copies: Mutex::new(Vec::new()),
            echec_copie,
        })
    }

    fn cas(connexion: Arc<ConnexionFausse>, horloge: Arc<HorlogeFausse>) -> CasConnexion {
        let bureau = bureau(false);
        CasConnexion::new(
            connexion,
            horloge,
            bureau.clone(),
            bureau,
            session(CoffreFaux::vide()),
        )
    }

    /// Joue la connexion et rend les états notifiés dans l'ordre.
    async fn jouer(cas: &CasConnexion, annulation: CancellationToken) -> Vec<EtatConnexion> {
        let etats = Mutex::new(Vec::new());
        let notifier = |etat| etats.lock().expect("verrou").push(etat);
        cas.start_connexion(annulation, &notifier).await;
        etats.into_inner().expect("verrou")
    }

    #[tokio::test]
    async fn le_sondage_respecte_l_intervalle_annonce() {
        let connexion = ConnexionFausse::new(demande(7, 900), vec![termine()]);
        let horloge = HorlogeFausse::new();
        let cas = cas(connexion, horloge.clone());

        let etats = jouer(&cas, CancellationToken::new()).await;

        assert_eq!(
            horloge.pauses(),
            vec![7],
            "une pause d'un intervalle avant le premier sondage"
        );
        assert!(
            matches!(etats.last(), Some(EtatConnexion::Connecte { .. })),
            "{etats:?}"
        );
    }

    #[tokio::test]
    async fn authorization_pending_relance_le_sondage() {
        let connexion = ConnexionFausse::new(
            demande(5, 900),
            vec![
                Ok(EtatSondage::EnAttente),
                Ok(EtatSondage::EnAttente),
                termine(),
            ],
        );
        let horloge = HorlogeFausse::new();
        let cas = cas(connexion.clone(), horloge);

        let etats = jouer(&cas, CancellationToken::new()).await;

        assert_eq!(*connexion.appels.lock().expect("verrou"), 3);
        assert!(
            matches!(etats.last(), Some(EtatConnexion::Connecte { .. })),
            "{etats:?}"
        );
    }

    #[tokio::test]
    async fn slow_down_augmente_l_intervalle() {
        let connexion = ConnexionFausse::new(
            demande(5, 900),
            vec![
                Ok(EtatSondage::PlusLent),
                Ok(EtatSondage::EnAttente),
                termine(),
            ],
        );
        let horloge = HorlogeFausse::new();
        let cas = cas(connexion, horloge.clone());

        jouer(&cas, CancellationToken::new()).await;

        assert_eq!(horloge.pauses(), vec![5, 10, 10]);
    }

    #[tokio::test]
    async fn slow_down_repete_ne_depasse_pas_le_plafond() {
        let sondages = (0..20).map(|_| Ok(EtatSondage::PlusLent)).collect();
        let connexion = ConnexionFausse::new(demande(5, 100_000), sondages);
        let horloge = HorlogeFausse::new();
        let cas = cas(connexion, horloge.clone());

        jouer(&cas, CancellationToken::new()).await;

        let maximum = horloge.pauses().into_iter().max().expect("des pauses");
        assert_eq!(maximum, 60);
    }

    #[tokio::test]
    async fn un_code_expire_affiche_un_message_d_expiration() {
        let connexion = ConnexionFausse::new(demande(5, 12), vec![]);
        let horloge = HorlogeFausse::new();
        let cas = cas(connexion, horloge);

        let etats = jouer(&cas, CancellationToken::new()).await;

        let dernier = etats.last().expect("un état final");
        assert_eq!(dernier, &EtatConnexion::Echec(RaisonEchec::Expire));
        let EtatConnexion::Echec(raison) = dernier else {
            unreachable!()
        };
        assert!(raison.message().contains("expiré"));
    }

    #[tokio::test]
    async fn le_code_est_notifie_avec_son_expiration_avant_le_sondage() {
        let connexion = ConnexionFausse::new(demande(5, 900), vec![termine()]);
        let horloge = HorlogeFausse::new();
        let depart = horloge.now();
        let cas = cas(connexion, horloge);

        let etats = jouer(&cas, CancellationToken::new()).await;

        let EtatConnexion::CodeAffiche {
            code,
            url,
            expiration,
        } = &etats[0]
        else {
            panic!("premier état inattendu : {:?}", etats[0]);
        };
        assert_eq!(code, "ABCD1234");
        assert_eq!(url.as_str(), "https://login.microsoft.com/device");
        assert_eq!(*expiration, depart + chrono::Duration::seconds(900));
    }

    #[tokio::test]
    async fn une_connexion_aboutie_conserve_le_refresh_token_et_signale_la_persistance() {
        let coffre = CoffreFaux::vide();
        let connexion = ConnexionFausse::new(demande(5, 900), vec![termine()]);
        let bureau = bureau(false);
        let cas = CasConnexion::new(
            connexion,
            HorlogeFausse::new(),
            bureau.clone(),
            bureau,
            session(coffre.clone()),
        );

        let etats = jouer(&cas, CancellationToken::new()).await;

        assert_eq!(coffre.valeur().as_deref(), Some(VALEUR_STOCKEE));
        assert!(
            matches!(
                etats.last(),
                Some(EtatConnexion::Connecte {
                    session_persistee: true,
                    ..
                })
            ),
            "{etats:?}"
        );
    }

    #[tokio::test]
    async fn sans_trousseau_la_connexion_aboutit_en_signalant_la_session_non_persistee() {
        let coffre = CoffreFaux::avec(None, true);
        let connexion = ConnexionFausse::new(demande(5, 900), vec![termine()]);
        let bureau = bureau(false);
        let cas = CasConnexion::new(
            connexion,
            HorlogeFausse::new(),
            bureau.clone(),
            bureau,
            session(coffre.clone()),
        );

        let etats = jouer(&cas, CancellationToken::new()).await;

        assert!(
            matches!(
                etats.last(),
                Some(EtatConnexion::Connecte {
                    session_persistee: false,
                    ..
                })
            ),
            "{etats:?}"
        );
        assert_eq!(coffre.valeur(), None);
    }

    #[tokio::test]
    async fn l_annulation_arrete_le_sondage() {
        let connexion = ConnexionFausse::new(demande(5, 900), vec![termine()]);
        let annulation = CancellationToken::new();
        let horloge = Arc::new(HorlogeFausse {
            maintenant: Mutex::new(Utc.with_ymd_and_hms(2026, 11, 2, 9, 0, 0).unwrap()),
            pauses: Mutex::new(Vec::new()),
            a_la_premiere_pause: Some(annulation.clone()),
        });
        let cas = cas(connexion.clone(), horloge);

        let etats = jouer(&cas, annulation).await;

        assert_eq!(etats.last(), Some(&EtatConnexion::NonConnecte));
        assert_eq!(
            *connexion.appels.lock().expect("verrou"),
            0,
            "aucun sondage après l'annulation"
        );
    }

    #[tokio::test]
    async fn une_erreur_du_service_devient_un_echec_sans_detail_technique() {
        let connexion = ConnexionFausse::new(
            demande(5, 900),
            vec![Err(ErreurAuth::Inattendu("invalid_client".into()))],
        );
        let cas = cas(connexion, HorlogeFausse::new());

        let etats = jouer(&cas, CancellationToken::new()).await;

        assert_eq!(
            etats.last(),
            Some(&EtatConnexion::Echec(RaisonEchec::Inattendu))
        );
    }

    #[test]
    fn copier_et_ouvrir_ecrit_le_code_et_ouvre_l_url() {
        let bureau = bureau(false);
        let cas = CasConnexion::new(
            ConnexionFausse::new(demande(5, 900), vec![]),
            HorlogeFausse::new(),
            bureau.clone(),
            bureau.clone(),
            session(CoffreFaux::vide()),
        );
        let url = Url::parse("https://login.microsoft.com/device").expect("url");

        cas.open_verification("ABCD1234", &url).expect("succès");

        assert_eq!(
            *bureau.copies.lock().expect("verrou"),
            vec!["ABCD1234".to_owned()]
        );
        assert_eq!(*bureau.urls.lock().expect("verrou"), vec![url.to_string()]);
    }

    #[test]
    fn le_navigateur_s_ouvre_meme_si_la_copie_echoue() {
        let bureau = bureau(true);
        let cas = CasConnexion::new(
            ConnexionFausse::new(demande(5, 900), vec![]),
            HorlogeFausse::new(),
            bureau.clone(),
            bureau.clone(),
            session(CoffreFaux::vide()),
        );
        let url = Url::parse("https://login.microsoft.com/device").expect("url");

        let resultat = cas.open_verification("ABCD1234", &url);

        assert_eq!(resultat, Err(ErreurBureau::Copie));
        assert_eq!(bureau.urls.lock().expect("verrou").len(), 1);
    }
}
