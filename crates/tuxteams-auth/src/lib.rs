//! Adaptateur d'authentification : flux device code du service d'identité Microsoft.

pub mod config;
pub mod device_code;
mod erreurs;

pub use config::{ConfigAuth, ErreurConfig};
pub use device_code::ClientDeviceCode;
