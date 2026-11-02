//! Port du cache local (conversations, messages) : seul le vidage est nécessaire à la déconnexion.

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("cache inaccessible")]
pub struct ErreurCache;

#[async_trait]
pub trait PortCache: Send + Sync {
    /// Efface toutes les données mises en cache pour le compte.
    async fn clear_all(&self) -> Result<(), ErreurCache>;
}
