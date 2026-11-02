//! Doubles de test des ports secrets : coffre, renouvellement, cache.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tuxteams_domaine::session::Compte;

use crate::ports::cache::{ErreurCache, PortCache};
use crate::ports::coffre::{ErreurCoffre, PortCoffre};
use crate::ports::horloge::Horloge;
use crate::ports::jetons::{Authentifie, ErreurAuth, PortRenouvellement, SecretRefresh};

pub const VALEUR_STOCKEE: &str = "valeur-stockee-fictive";
pub const VALEUR_ROTATION: &str = "valeur-rotation-fictive";

pub fn compte() -> Compte {
    Compte::depuis_identite(
        "tenant-a",
        Some("Alex Example"),
        Some("alex@contoso.example"),
    )
    .expect("compte")
}

pub fn authentifie(refresh: Option<&str>) -> Authentifie {
    Authentifie {
        compte: compte(),
        refresh: refresh.map(|valeur| SecretRefresh::new(valeur.to_owned())),
    }
}

/// Coffre en mémoire ; `indisponible` simule l'absence de trousseau.
pub struct CoffreFaux {
    pub contenu: Mutex<Option<SecretRefresh>>,
    pub indisponible: bool,
    pub suppressions: Mutex<u32>,
}

impl CoffreFaux {
    pub fn vide() -> Arc<Self> {
        Self::avec(None, false)
    }

    pub fn avec(contenu: Option<&str>, indisponible: bool) -> Arc<Self> {
        Arc::new(Self {
            contenu: Mutex::new(contenu.map(|valeur| SecretRefresh::new(valeur.to_owned()))),
            indisponible,
            suppressions: Mutex::new(0),
        })
    }

    pub fn valeur(&self) -> Option<String> {
        let contenu = self.contenu.lock().expect("verrou");
        contenu.as_ref().map(|secret| secret.exposer().to_owned())
    }
}

#[async_trait]
impl PortCoffre for CoffreFaux {
    async fn save_refresh(&self, _: &Compte, jeton: &SecretRefresh) -> Result<(), ErreurCoffre> {
        if self.indisponible {
            return Err(ErreurCoffre::Indisponible);
        }
        *self.contenu.lock().expect("verrou") = Some(jeton.clone());
        Ok(())
    }

    async fn load_refresh(&self) -> Result<Option<SecretRefresh>, ErreurCoffre> {
        if self.indisponible {
            return Err(ErreurCoffre::Indisponible);
        }
        Ok(self.contenu.lock().expect("verrou").clone())
    }

    async fn delete_refresh(&self) -> Result<(), ErreurCoffre> {
        *self.suppressions.lock().expect("verrou") += 1;
        if self.indisponible {
            return Err(ErreurCoffre::Indisponible);
        }
        *self.contenu.lock().expect("verrou") = None;
        Ok(())
    }
}

/// Renouvellement scripté ; mémorise les refresh tokens reçus.
pub struct RenouvellementFaux {
    pub reponse: Result<Authentifie, ErreurAuth>,
    pub recus: Mutex<Vec<String>>,
}

impl RenouvellementFaux {
    pub fn new(reponse: Result<Authentifie, ErreurAuth>) -> Arc<Self> {
        Arc::new(Self {
            reponse,
            recus: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl PortRenouvellement for RenouvellementFaux {
    async fn refresh(&self, refresh: &SecretRefresh) -> Result<Authentifie, ErreurAuth> {
        self.recus
            .lock()
            .expect("verrou")
            .push(refresh.exposer().to_owned());
        self.reponse.clone()
    }
}

/// Cache qui compte ses vidages.
pub struct CacheFaux {
    pub vidages: Mutex<u32>,
    pub en_echec: bool,
}

impl CacheFaux {
    pub fn new(en_echec: bool) -> Arc<Self> {
        Arc::new(Self {
            vidages: Mutex::new(0),
            en_echec,
        })
    }
}

#[async_trait]
impl PortCache for CacheFaux {
    async fn clear_all(&self) -> Result<(), ErreurCache> {
        *self.vidages.lock().expect("verrou") += 1;
        if self.en_echec {
            Err(ErreurCache)
        } else {
            Ok(())
        }
    }
}

/// Renouvellement qui ne répond jamais (réseau bloqué).
pub struct RenouvellementBloque;

#[async_trait]
impl PortRenouvellement for RenouvellementBloque {
    async fn refresh(&self, _: &SecretRefresh) -> Result<Authentifie, ErreurAuth> {
        std::future::pending().await
    }
}

/// Horloge dont l'attente se termine aussitôt (le délai est déjà écoulé) ; mémorise les durées.
#[derive(Default)]
pub struct HorlogeInstantanee {
    pub pauses: Mutex<Vec<u64>>,
}

#[async_trait]
impl Horloge for HorlogeInstantanee {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }

    async fn sleep(&self, duree: Duration) {
        self.pauses.lock().expect("verrou").push(duree.as_secs());
    }
}

/// Horloge dont l'attente ne se termine jamais : aucun délai ne s'écoule.
pub struct HorlogeSansFin;

#[async_trait]
impl Horloge for HorlogeSansFin {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }

    async fn sleep(&self, _: Duration) {
        std::future::pending().await
    }
}
