//! États de connexion et comptes (règles pures, sans I/O).

use chrono::{DateTime, Utc};
use url::Url;

/// Identifiant de tenant des comptes Microsoft personnels (`tid` de l'id_token).
pub const TENANT_COMPTE_PERSONNEL: &str = "9188040d-6c67-4c5b-b112-36a304b66dad";

/// Famille du compte connecté.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCompte {
    Professionnel,
    Personnel,
}

/// Compte connecté. Seules les données utiles à l'affichage sont conservées.
#[derive(Clone, PartialEq, Eq)]
pub struct Compte {
    pub nom: Option<String>,
    pub email: Option<String>,
    pub tenant: String,
}

/// Debug manuel : nom et e-mail sont des données personnelles, jamais journalisées.
impl std::fmt::Debug for Compte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Compte")
            .field("nom", &self.nom.as_ref().map(|_| "<masqué>"))
            .field("email", &self.email.as_ref().map(|_| "<masqué>"))
            .field("tenant", &self.tenant)
            .finish()
    }
}

/// Raison d'un échec de connexion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaisonEchec {
    Expire,
    Refuse,
    ComptePersonnel,
    AccesConditionnel,
    Reseau,
    Inattendu,
}

/// État de la connexion, tel que l'écran de connexion le présente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EtatConnexion {
    NonConnecte,
    CodeAffiche {
        code: String,
        url: Url,
        expiration: DateTime<Utc>,
    },
    Connecte {
        compte: Compte,
    },
    Echec(RaisonEchec),
}

impl TypeCompte {
    /// Déduit la famille du compte depuis le tenant de son id_token.
    pub fn depuis_tenant(tenant: &str) -> Self {
        if tenant.eq_ignore_ascii_case(TENANT_COMPTE_PERSONNEL) {
            Self::Personnel
        } else {
            Self::Professionnel
        }
    }
}

impl Compte {
    /// Construit le compte depuis les claims d'identité ; refuse un compte personnel.
    pub fn depuis_identite(
        tenant: &str,
        nom: Option<&str>,
        email: Option<&str>,
    ) -> Result<Self, RaisonEchec> {
        if TypeCompte::depuis_tenant(tenant) == TypeCompte::Personnel {
            return Err(RaisonEchec::ComptePersonnel);
        }
        Ok(Self {
            nom: nom.map(str::to_owned),
            email: email.map(str::to_owned),
            tenant: tenant.to_owned(),
        })
    }

    pub fn type_compte(&self) -> TypeCompte {
        TypeCompte::depuis_tenant(&self.tenant)
    }

    /// Initiales (deux lettres au plus) pour l'avatar ; « ?? » sans nom.
    pub fn initiales(&self) -> String {
        let nom = self.nom.as_deref().unwrap_or_default();
        let mut mots = nom.split_whitespace();
        let premiere = mots.next().and_then(|mot| mot.chars().next());
        let derniere = mots.next_back().and_then(|mot| mot.chars().next());
        let initiales: String = [premiere, derniere]
            .into_iter()
            .flatten()
            .flat_map(char::to_uppercase)
            .collect();
        if initiales.is_empty() {
            "??".to_owned()
        } else {
            initiales
        }
    }
}

impl RaisonEchec {
    /// Message destiné à l'utilisateur : sans détail technique ni secret.
    pub fn message(&self) -> &'static str {
        match self {
            Self::Expire => "Le code a expiré. Demandez un nouveau code pour réessayer.",
            Self::Refuse => "La connexion a été refusée sur l'autre appareil.",
            Self::ComptePersonnel => {
                "Les comptes Microsoft personnels ne sont pas pris en charge : \
                 connectez-vous avec un compte professionnel ou scolaire."
            }
            Self::AccesConditionnel => {
                "Votre organisation interdit la connexion par code. \
                 Une autre méthode de connexion sera nécessaire."
            }
            Self::Reseau => "Le service de connexion est injoignable. Vérifiez votre réseau.",
            Self::Inattendu => "Réponse inattendue du service de connexion. Réessayez plus tard.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_compte_personnel_est_refuse_avec_un_message_explicite() {
        let resultat = Compte::depuis_identite(TENANT_COMPTE_PERSONNEL, Some("Alex Example"), None);
        let raison = resultat.expect_err("un compte personnel doit être refusé");
        assert_eq!(raison, RaisonEchec::ComptePersonnel);
        let message = raison.message();
        assert!(message.contains("personnel"), "message : {message}");
        assert!(message.contains("professionnel"), "message : {message}");
    }

    #[test]
    fn le_debug_d_un_compte_ne_revele_ni_nom_ni_email() {
        let compte = Compte::depuis_identite(
            "00000000-0000-0000-0000-0000000000aa",
            Some("Alex Example"),
            Some("alex@contoso.example"),
        )
        .expect("compte");
        let sortie = format!("{compte:?}");
        assert!(!sortie.contains("Alex"), "{sortie}");
        assert!(!sortie.contains("contoso"), "{sortie}");
    }

    #[test]
    fn un_compte_d_organisation_est_accepte() {
        let compte = Compte::depuis_identite(
            "00000000-0000-0000-0000-0000000000aa",
            Some("Alex Example"),
            Some("alex@contoso.example"),
        )
        .expect("compte professionnel accepté");
        assert_eq!(compte.type_compte(), TypeCompte::Professionnel);
        assert_eq!(compte.nom.as_deref(), Some("Alex Example"));
    }

    #[test]
    fn le_tenant_microsoft_est_reconnu_sans_tenir_compte_de_la_casse() {
        let majuscules = TENANT_COMPTE_PERSONNEL.to_uppercase();
        assert_eq!(
            TypeCompte::depuis_tenant(&majuscules),
            TypeCompte::Personnel
        );
    }

    #[test]
    fn les_initiales_viennent_du_premier_et_du_dernier_mot_du_nom() {
        let compte = |nom: Option<&str>| {
            Compte::depuis_identite("tenant-a", nom, None).expect("compte accepté")
        };
        assert_eq!(compte(Some("alex Example")).initiales(), "AE");
        assert_eq!(compte(Some("Marie Anne Dupont")).initiales(), "MD");
        assert_eq!(compte(Some("Zoé")).initiales(), "Z");
        assert_eq!(compte(None).initiales(), "??");
    }

    #[test]
    fn un_code_expire_a_un_message_d_expiration() {
        let message = RaisonEchec::Expire.message();
        assert!(message.contains("expiré"), "message : {message}");
    }
}
