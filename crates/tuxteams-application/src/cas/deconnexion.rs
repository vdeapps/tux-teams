//! Cas d'usage : déconnexion (suppression du secret, vidage du cache).

use std::sync::Arc;

use crate::ports::cache::PortCache;
use crate::ports::coffre::PortCoffre;

/// Ce que la déconnexion a effectivement nettoyé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BilanDeconnexion {
    pub secret_supprime: bool,
    pub cache_vide: bool,
}

pub struct CasDeconnexion {
    coffre: Arc<dyn PortCoffre>,
    cache: Arc<dyn PortCache>,
}

impl CasDeconnexion {
    pub fn new(coffre: Arc<dyn PortCoffre>, cache: Arc<dyn PortCache>) -> Self {
        Self { coffre, cache }
    }

    /// Supprime le secret et vide le cache ; les deux sont tentés quoi qu'il arrive.
    pub async fn logout(&self) -> BilanDeconnexion {
        let secret_supprime = match self.coffre.delete_refresh().await {
            Ok(()) => true,
            Err(erreur) => {
                tracing::warn!(%erreur, "suppression du secret impossible");
                false
            }
        };
        let cache_vide = match self.cache.clear_all().await {
            Ok(()) => true,
            Err(erreur) => {
                tracing::warn!(%erreur, "vidage du cache impossible");
                false
            }
        };
        BilanDeconnexion {
            secret_supprime,
            cache_vide,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cas::doubles::{CacheFaux, CoffreFaux, VALEUR_STOCKEE};

    #[tokio::test]
    async fn la_deconnexion_supprime_le_secret_et_vide_le_cache() {
        let coffre = CoffreFaux::avec(Some(VALEUR_STOCKEE), false);
        let cache = CacheFaux::new(false);
        let cas = CasDeconnexion::new(coffre.clone(), cache.clone());

        let bilan = cas.logout().await;

        assert_eq!(coffre.valeur(), None);
        assert_eq!(*cache.vidages.lock().expect("verrou"), 1);
        assert_eq!(
            bilan,
            BilanDeconnexion {
                secret_supprime: true,
                cache_vide: true
            }
        );
    }

    #[tokio::test]
    async fn un_echec_du_trousseau_n_empeche_pas_de_vider_le_cache() {
        let coffre = CoffreFaux::avec(Some(VALEUR_STOCKEE), true);
        let cache = CacheFaux::new(false);
        let cas = CasDeconnexion::new(coffre, cache.clone());

        let bilan = cas.logout().await;

        assert_eq!(*cache.vidages.lock().expect("verrou"), 1);
        assert!(!bilan.secret_supprime && bilan.cache_vide, "{bilan:?}");
    }

    #[tokio::test]
    async fn un_echec_du_cache_n_empeche_pas_de_supprimer_le_secret() {
        let coffre = CoffreFaux::avec(Some(VALEUR_STOCKEE), false);
        let cas = CasDeconnexion::new(coffre.clone(), CacheFaux::new(true));

        let bilan = cas.logout().await;

        assert_eq!(coffre.valeur(), None);
        assert!(bilan.secret_supprime && !bilan.cache_vide, "{bilan:?}");
    }
}
