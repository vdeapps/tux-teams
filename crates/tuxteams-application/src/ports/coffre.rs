//! Port du coffre de secrets (trousseau du bureau) : conservation du refresh token.

use async_trait::async_trait;
use thiserror::Error;
use tuxteams_domaine::session::Compte;

use super::jetons::SecretRefresh;

/// Erreur du coffre ; ne porte jamais de secret.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErreurCoffre {
    /// Pas de trousseau joignable, verrouillé ou refusé par l'utilisateur.
    #[error("trousseau indisponible")]
    Indisponible,
    #[error("erreur du trousseau")]
    Inattendu,
}

#[async_trait]
pub trait PortCoffre: Send + Sync {
    /// Enregistre (en remplaçant) le refresh token du compte.
    async fn save_refresh(
        &self,
        compte: &Compte,
        jeton: &SecretRefresh,
    ) -> Result<(), ErreurCoffre>;
    /// Refresh token conservé, `None` si aucun (v1 : un seul compte).
    async fn load_refresh(&self) -> Result<Option<SecretRefresh>, ErreurCoffre>;
    /// Supprime tout refresh token conservé ; sans secret, réussit.
    async fn delete_refresh(&self) -> Result<(), ErreurCoffre>;
}
