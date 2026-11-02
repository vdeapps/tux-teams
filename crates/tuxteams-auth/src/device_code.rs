//! Client device code : demande du code puis sondage de l'endpoint de jetons.
//!
//! Requêtes maison sur reqwest 0.13 (pas de crate `oauth2`, qui dépend de reqwest 0.12).
//! Format des échanges : spécification OpenAPI interne (`/{tenant}/oauth2/v2.0/devicecode` et
//! `/token`) ; le statut HTTP du sondage n'est pas inspecté, `access_token` ou `error` décident.

use std::time::Duration;

use async_trait::async_trait;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::Client;
use serde::Deserialize;
use tuxteams_application::ports::jetons::{DemandeCode, ErreurAuth, EtatSondage, PortConnexion};
use tuxteams_domaine::session::Compte;
use url::Url;

use crate::config::ConfigAuth;
use crate::erreurs;

/// Taille maximale lue dans une réponse du service d'identité.
const TAILLE_MAX_REPONSE: usize = 64 * 1024;
/// Intervalle de sondage retenu quand le service annonce une valeur nulle ou absente.
const INTERVALLE_PAR_DEFAUT: u64 = 5;
const GRANT_DEVICE_CODE: &str = "urn:ietf:params:oauth:grant-type:device_code";

pub struct ClientDeviceCode {
    config: ConfigAuth,
    http: Client,
}

#[derive(Deserialize)]
struct ReponseCode {
    device_code: Option<String>,
    user_code: Option<String>,
    verification_uri: Option<String>,
    expires_in: Option<u64>,
    interval: Option<i64>,
}

#[derive(Deserialize)]
struct ReponseJeton {
    access_token: Option<String>,
    id_token: Option<String>,
}

#[derive(Deserialize)]
struct ClaimsIdentite {
    tid: Option<String>,
    name: Option<String>,
    email: Option<String>,
    preferred_username: Option<String>,
}

fn inattendu(code: &str) -> ErreurAuth {
    tracing::warn!(code, "réponse inattendue du service de connexion");
    ErreurAuth::Inattendu(code.to_owned())
}

impl ClientDeviceCode {
    pub fn new(config: ConfigAuth) -> Result<Self, ErreurAuth> {
        let http = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|_| ErreurAuth::Reseau)?;
        Ok(Self { config, http })
    }

    fn endpoint(&self, nom: &str) -> Result<Url, ErreurAuth> {
        let chemin = format!("{}/oauth2/v2.0/{nom}", self.config.tenant);
        self.config
            .autorite
            .join(&chemin)
            .map_err(|_| inattendu("autorite_invalide"))
    }

    /// Poste un formulaire et rend (statut, corps borné à 64 Kio).
    async fn poster(
        &self,
        url: Url,
        champs: &[(&str, &str)],
    ) -> Result<(u16, Vec<u8>), ErreurAuth> {
        let mut reponse = self
            .http
            .post(url)
            .form(champs)
            .send()
            .await
            .map_err(|erreur| {
                tracing::warn!(
                    timeout = erreur.is_timeout(),
                    "service de connexion injoignable"
                );
                ErreurAuth::Reseau
            })?;
        let statut = reponse.status().as_u16();
        let mut corps = Vec::new();
        while let Some(morceau) = reponse.chunk().await.map_err(|_| ErreurAuth::Reseau)? {
            if corps.len() + morceau.len() > TAILLE_MAX_REPONSE {
                return Err(inattendu("reponse_trop_grande"));
            }
            corps.extend_from_slice(&morceau);
        }
        Ok((statut, corps))
    }
}

/// Extrait le compte des claims de l'id_token. La signature n'est pas vérifiée : le jeton vient
/// directement de l'endpoint du service d'identité, en TLS, et ne sert qu'à l'affichage et au
/// refus des comptes personnels.
fn compte_depuis_id_token(id_token: &str) -> Result<Compte, ErreurAuth> {
    let charge = id_token
        .split('.')
        .nth(1)
        .ok_or_else(|| inattendu("id_token_invalide"))?;
    let octets = URL_SAFE_NO_PAD
        .decode(charge)
        .map_err(|_| inattendu("id_token_invalide"))?;
    let claims: ClaimsIdentite =
        serde_json::from_slice(&octets).map_err(|_| inattendu("id_token_invalide"))?;
    let tenant = claims.tid.ok_or_else(|| inattendu("tid_absent"))?;
    let email = claims.email.or(claims.preferred_username);
    Compte::depuis_identite(&tenant, claims.name.as_deref(), email.as_deref())
        .map_err(|_| ErreurAuth::ComptePersonnel)
}

#[async_trait]
impl PortConnexion for ClientDeviceCode {
    async fn start_device_code(&self) -> Result<DemandeCode, ErreurAuth> {
        let url = self.endpoint("devicecode")?;
        let champs = [
            ("client_id", self.config.client_id.as_str()),
            ("scope", self.config.scopes.as_str()),
        ];
        let (statut, corps) = self.poster(url, &champs).await?;
        if statut >= 400 {
            return Err(erreurs::depuis_corps(&corps));
        }
        let reponse: ReponseCode =
            serde_json::from_slice(&corps).map_err(|_| inattendu("corps_illisible"))?;
        let (Some(code_appareil), Some(code_utilisateur)) =
            (reponse.device_code, reponse.user_code)
        else {
            return Err(inattendu("code_absent"));
        };
        let url_verification = reponse
            .verification_uri
            .and_then(|brut| Url::parse(&brut).ok())
            .filter(|url| url.scheme() == "https")
            .ok_or_else(|| inattendu("url_verification_invalide"))?;
        let intervalle = u64::try_from(reponse.interval.unwrap_or(0))
            .ok()
            .filter(|secondes| *secondes > 0)
            .unwrap_or(INTERVALLE_PAR_DEFAUT);
        Ok(DemandeCode {
            code_utilisateur,
            code_appareil,
            url_verification,
            expire_dans: Duration::from_secs(reponse.expires_in.unwrap_or(900)),
            intervalle: Duration::from_secs(intervalle),
        })
    }

    async fn poll_device_code(&self, demande: &DemandeCode) -> Result<EtatSondage, ErreurAuth> {
        let url = self.endpoint("token")?;
        let champs = [
            ("client_id", self.config.client_id.as_str()),
            ("grant_type", GRANT_DEVICE_CODE),
            ("device_code", demande.code_appareil.as_str()),
        ];
        let (_, corps) = self.poster(url, &champs).await?;
        if let Ok(jeton) = serde_json::from_slice::<ReponseJeton>(&corps)
            && jeton.access_token.is_some()
        {
            let id_token = jeton.id_token.ok_or_else(|| inattendu("id_token_absent"))?;
            return compte_depuis_id_token(&id_token).map(EtatSondage::Termine);
        }
        match erreurs::code_oauth(&corps).as_deref() {
            Some("authorization_pending") => Ok(EtatSondage::EnAttente),
            Some("slow_down") => Ok(EtatSondage::PlusLent),
            _ => Err(erreurs::depuis_corps(&corps)),
        }
    }
}
