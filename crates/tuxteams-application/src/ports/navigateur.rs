//! Ports d'interaction avec le bureau : navigateur système et presse-papiers.

use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
#[error("impossible d'ouvrir le navigateur")]
pub struct ErreurNavigateur;

#[derive(Debug, Error)]
#[error("impossible d'écrire dans le presse-papiers")]
pub struct ErreurPressePapiers;

/// Ouvre une URL dans le navigateur par défaut.
pub trait PortNavigateur: Send + Sync {
    fn open_url(&self, url: &Url) -> Result<(), ErreurNavigateur>;
}

/// Écrit du texte dans le presse-papiers du bureau.
pub trait PortPressePapiers: Send + Sync {
    fn copy_text(&self, texte: &str) -> Result<(), ErreurPressePapiers>;
}
