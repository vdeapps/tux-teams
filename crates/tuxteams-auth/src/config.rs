//! Configuration de la connexion : client_id, tenant, autorité.

use thiserror::Error;
use url::Url;

/// Client Teams « travail » (registration publique de Microsoft, pas un secret).
pub const CLIENT_ID_PAR_DEFAUT: &str = "1fec8e78-bce4-4aaf-ab1b-5451cc387264";
pub const TENANT_PAR_DEFAUT: &str = "organizations";
pub const AUTORITE_PAR_DEFAUT: &str = "https://login.microsoftonline.com";
pub const SCOPES_PAR_DEFAUT: &str =
    "https://api.spaces.skype.com/.default openid profile offline_access";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigAuth {
    pub client_id: String,
    pub tenant: String,
    pub autorite: Url,
    pub scopes: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErreurConfig {
    #[error("valeur invalide pour {0}")]
    ValeurInvalide(&'static str),
}

impl ConfigAuth {
    pub fn par_defaut() -> Self {
        Self {
            client_id: CLIENT_ID_PAR_DEFAUT.to_owned(),
            tenant: TENANT_PAR_DEFAUT.to_owned(),
            autorite: Url::parse(AUTORITE_PAR_DEFAUT).expect("constante valide"),
            scopes: SCOPES_PAR_DEFAUT.to_owned(),
        }
    }

    /// Surcharge `TUXTEAMS_CLIENT_ID` et `TUXTEAMS_TENANT` lues par `lire` (variables d'environnement
    /// en production). L'autorité n'est jamais surchargeable ainsi : elle ne sert qu'aux tests.
    pub fn depuis_variables(lire: impl Fn(&str) -> Option<String>) -> Result<Self, ErreurConfig> {
        let mut config = Self::par_defaut();
        if let Some(client_id) = lire("TUXTEAMS_CLIENT_ID") {
            config.client_id = client_id;
        }
        if let Some(tenant) = lire("TUXTEAMS_TENANT") {
            config.tenant = tenant;
        }
        config.valider()?;
        Ok(config)
    }

    pub fn avec_autorite(mut self, autorite: Url) -> Self {
        self.autorite = autorite;
        self
    }

    /// Les deux valeurs finissent dans une URL ou un corps de requête : jeu de caractères restreint.
    fn valider(&self) -> Result<(), ErreurConfig> {
        let sur = |valeur: &str, extra: char| {
            !valeur.is_empty()
                && valeur.len() <= 128
                && valeur
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == extra)
        };
        if !sur(&self.client_id, '-') {
            return Err(ErreurConfig::ValeurInvalide("TUXTEAMS_CLIENT_ID"));
        }
        if !sur(&self.tenant, '.') {
            return Err(ErreurConfig::ValeurInvalide("TUXTEAMS_TENANT"));
        }
        Ok(())
    }
}
