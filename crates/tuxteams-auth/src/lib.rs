//! Adaptateur d'authentification : flux device code du service d'identité Microsoft.

pub mod coffre_secret_service;
pub mod config;
pub mod device_code;
mod erreurs;

pub use coffre_secret_service::CoffreSecretService;
pub use config::{ConfigAuth, ErreurConfig};
pub use device_code::ClientDeviceCode;
