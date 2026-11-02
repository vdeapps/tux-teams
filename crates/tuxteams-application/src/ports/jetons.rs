//! Port de connexion : flux « device code » du service d'identité.

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use tuxteams_domaine::session::{Compte, RaisonEchec};
use url::Url;

/// Réponse à la demande de code : ce que l'utilisateur doit saisir et la cadence du sondage.
#[derive(Clone, PartialEq, Eq)]
pub struct DemandeCode {
    /// Code à saisir sur la page de vérification.
    pub code_utilisateur: String,
    /// Code d'appareil (secret) servant au sondage ; jamais affiché ni journalisé.
    pub code_appareil: String,
    pub url_verification: Url,
    pub expire_dans: Duration,
    pub intervalle: Duration,
}

impl fmt::Debug for DemandeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DemandeCode")
            .field("code_appareil", &"<masqué>")
            .field("expire_dans", &self.expire_dans)
            .field("intervalle", &self.intervalle)
            .finish_non_exhaustive()
    }
}

/// Résultat d'un sondage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EtatSondage {
    /// `authorization_pending` : l'utilisateur n'a pas encore terminé.
    EnAttente,
    /// `slow_down` : espacer les sondages.
    PlusLent,
    /// Connexion aboutie. Les jetons ne sont pas encore exposés (persistance : tâche 3).
    Termine(Compte),
}

/// Erreur du service de connexion ; ne porte jamais de secret.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErreurAuth {
    #[error("code expiré")]
    CodeExpire,
    #[error("connexion refusée par l'utilisateur")]
    Refuse,
    #[error("compte personnel refusé")]
    ComptePersonnel,
    #[error("accès conditionnel (AADSTS53003)")]
    AccesConditionnel,
    #[error("service injoignable")]
    Reseau,
    /// Réponse non prévue ; porte le seul code d'erreur OAuth, sans description.
    #[error("réponse inattendue ({0})")]
    Inattendu(String),
}

impl ErreurAuth {
    pub fn raison(&self) -> RaisonEchec {
        match self {
            Self::CodeExpire => RaisonEchec::Expire,
            Self::Refuse => RaisonEchec::Refuse,
            Self::ComptePersonnel => RaisonEchec::ComptePersonnel,
            Self::AccesConditionnel => RaisonEchec::AccesConditionnel,
            Self::Reseau => RaisonEchec::Reseau,
            Self::Inattendu(_) => RaisonEchec::Inattendu,
        }
    }
}

#[async_trait]
pub trait PortConnexion: Send + Sync {
    async fn start_device_code(&self) -> Result<DemandeCode, ErreurAuth>;
    async fn poll_device_code(&self, demande: &DemandeCode) -> Result<EtatSondage, ErreurAuth>;
}
