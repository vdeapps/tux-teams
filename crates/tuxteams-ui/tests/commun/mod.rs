//! Doubles partagés des tests d'interface : session sans trousseau ni service d'identité.
#![allow(dead_code)]

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};

use async_trait::async_trait;
use tuxteams_application::cas::session::CasSession;
use tuxteams_application::ports::coffre::{ErreurCoffre, PortCoffre};
use tuxteams_application::ports::horloge::Horloge;
use tuxteams_application::ports::jetons::{
    Authentifie, ErreurAuth, PortRenouvellement, SecretRefresh,
};
use tuxteams_domaine::session::Compte;

/// Trousseau absent : toute opération échoue comme sans Secret Service.
pub struct CoffreAbsent;

#[async_trait]
impl PortCoffre for CoffreAbsent {
    async fn save_refresh(&self, _: &Compte, _: &SecretRefresh) -> Result<(), ErreurCoffre> {
        Err(ErreurCoffre::Indisponible)
    }
    async fn load_refresh(&self) -> Result<Option<SecretRefresh>, ErreurCoffre> {
        Err(ErreurCoffre::Indisponible)
    }
    async fn delete_refresh(&self) -> Result<(), ErreurCoffre> {
        Err(ErreurCoffre::Indisponible)
    }
}

pub struct RenouvellementInerte;

#[async_trait]
impl PortRenouvellement for RenouvellementInerte {
    async fn refresh(&self, _: &SecretRefresh) -> Result<Authentifie, ErreurAuth> {
        Err(ErreurAuth::Reseau)
    }
}

/// Horloge réelle ; la relance n'est pas exercée par ces tests.
pub struct HorlogeSysteme;

#[async_trait]
impl Horloge for HorlogeSysteme {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    async fn sleep(&self, duree: Duration) {
        tokio::time::sleep(duree).await;
    }
}

pub fn session_sans_trousseau() -> Arc<CasSession> {
    Arc::new(CasSession::new(
        Arc::new(CoffreAbsent),
        Arc::new(RenouvellementInerte),
        Arc::new(HorlogeSysteme),
    ))
}
