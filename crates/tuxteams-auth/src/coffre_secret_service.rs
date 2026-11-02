//! Coffre du refresh token dans le trousseau du bureau (Secret Service D-Bus, ou portail Flatpak).

use async_trait::async_trait;
use oo7::{Keyring, Secret};
use tokio::sync::OnceCell;
use tuxteams_application::ports::coffre::{ErreurCoffre, PortCoffre};
use tuxteams_application::ports::jetons::SecretRefresh;
use tuxteams_domaine::session::Compte;

const TYPE_REFRESH: &str = "refresh_token";

pub struct CoffreSecretService {
    trousseau: OnceCell<Keyring>,
    application: String,
}

impl CoffreSecretService {
    pub fn new() -> Self {
        Self {
            trousseau: OnceCell::new(),
            application: "tuxteams".to_owned(),
        }
    }

    /// Coffre sur un trousseau et une valeur d'attribut `application` choisis (tests d'intégration).
    pub fn avec_trousseau(trousseau: Keyring, application: &str) -> Self {
        Self {
            trousseau: OnceCell::new_with(Some(trousseau)),
            application: application.to_owned(),
        }
    }
}

impl CoffreSecretService {
    /// Trousseau par défaut, ouvert au premier usage (le Secret Service peut démarrer après nous).
    async fn trousseau(&self) -> Result<&Keyring, ErreurCoffre> {
        self.trousseau
            .get_or_try_init(|| async {
                let trousseau = Keyring::new().await.map_err(vers_erreur)?;
                trousseau.unlock().await.map_err(vers_erreur)?;
                Ok(trousseau)
            })
            .await
    }

    /// Attributs de recherche : `secret-tool search application tuxteams`.
    fn attributs(&self) -> [(&'static str, String); 2] {
        [
            ("application", self.application.clone()),
            ("type", TYPE_REFRESH.to_owned()),
        ]
    }
}

impl Default for CoffreSecretService {
    fn default() -> Self {
        Self::new()
    }
}

/// Le détail de l'erreur D-Bus ne sert qu'au journal : il ne contient jamais de secret.
fn vers_erreur(erreur: oo7::Error) -> ErreurCoffre {
    tracing::warn!(%erreur, "trousseau du bureau inaccessible");
    ErreurCoffre::Indisponible
}

#[async_trait]
impl PortCoffre for CoffreSecretService {
    async fn save_refresh(
        &self,
        compte: &Compte,
        jeton: &SecretRefresh,
    ) -> Result<(), ErreurCoffre> {
        let trousseau = self.trousseau().await?;
        // Un seul compte en v1 : l'ancien secret est retiré, quel que soit son tenant.
        trousseau
            .delete(&self.attributs())
            .await
            .map_err(vers_erreur)?;
        let mut attributs = self.attributs().to_vec();
        attributs.push(("tenant", compte.tenant.clone()));
        trousseau
            .create_item(
                "TuxTeams (session Microsoft)",
                &attributs,
                Secret::text(jeton.exposer()),
                true,
            )
            .await
            .map_err(vers_erreur)
    }

    async fn load_refresh(&self) -> Result<Option<SecretRefresh>, ErreurCoffre> {
        let trousseau = self.trousseau().await?;
        let elements = trousseau
            .search_items(&self.attributs())
            .await
            .map_err(vers_erreur)?;
        let Some(element) = elements.first() else {
            return Ok(None);
        };
        let secret = element.secret().await.map_err(vers_erreur)?;
        let valeur =
            String::from_utf8(secret.as_bytes().to_vec()).map_err(|_| ErreurCoffre::Inattendu)?;
        Ok(Some(SecretRefresh::new(valeur)))
    }

    async fn delete_refresh(&self) -> Result<(), ErreurCoffre> {
        let trousseau = self.trousseau().await?;
        trousseau
            .delete(&self.attributs())
            .await
            .map_err(vers_erreur)
    }
}
