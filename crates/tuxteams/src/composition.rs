//! Composition : adaptateurs du bureau et assemblage du cas d'usage de connexion.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tuxteams_application::cas::connexion::CasConnexion;
use tuxteams_application::ports::horloge::Horloge;
use tuxteams_application::ports::jetons::ErreurAuth;
use tuxteams_application::ports::navigateur::{
    ErreurNavigateur, ErreurPressePapiers, PortNavigateur, PortPressePapiers,
};
use tuxteams_auth::{ClientDeviceCode, ConfigAuth};
use url::Url;

/// Horloge du système, attente par le runtime tokio.
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

/// Navigateur par défaut du bureau.
pub struct NavigateurSysteme;

impl PortNavigateur for NavigateurSysteme {
    fn open_url(&self, url: &Url) -> Result<(), ErreurNavigateur> {
        webbrowser::open(url.as_str()).map_err(|_| ErreurNavigateur)
    }
}

/// Presse-papiers du bureau. Sous Linux, le contenu n'est servi que tant que l'objet `Clipboard`
/// vit : il est donc conservé entre deux copies.
#[derive(Default)]
pub struct PressePapiersSysteme {
    presse_papiers: Mutex<Option<arboard::Clipboard>>,
}

impl PortPressePapiers for PressePapiersSysteme {
    fn copy_text(&self, texte: &str) -> Result<(), ErreurPressePapiers> {
        let mut garde = self
            .presse_papiers
            .lock()
            .map_err(|_| ErreurPressePapiers)?;
        if garde.is_none() {
            *garde = Some(arboard::Clipboard::new().map_err(|_| ErreurPressePapiers)?);
        }
        garde
            .as_mut()
            .ok_or(ErreurPressePapiers)?
            .set_text(texte)
            .map_err(|_| ErreurPressePapiers)
    }
}

/// Assemble le cas d'usage de connexion avec les adaptateurs réels.
pub fn assembler_connexion(
    config: ConfigAuth,
    horloge: Arc<HorlogeSysteme>,
) -> Result<Arc<CasConnexion>, ErreurAuth> {
    Ok(Arc::new(CasConnexion::new(
        Arc::new(ClientDeviceCode::new(config)?),
        horloge,
        Arc::new(NavigateurSysteme),
        Arc::new(PressePapiersSysteme::default()),
    )))
}
