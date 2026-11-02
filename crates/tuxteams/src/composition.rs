//! Composition : adaptateurs du bureau et assemblage des cas d'usage (connexion, session, déconnexion).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tuxteams_application::cas::connexion::CasConnexion;
use tuxteams_application::cas::deconnexion::CasDeconnexion;
use tuxteams_application::cas::session::CasSession;
use tuxteams_application::ports::cache::{ErreurCache, PortCache};
use tuxteams_application::ports::horloge::Horloge;
use tuxteams_application::ports::jetons::ErreurAuth;
use tuxteams_application::ports::navigateur::{
    ErreurNavigateur, ErreurPressePapiers, PortNavigateur, PortPressePapiers,
};
use tuxteams_auth::{ClientDeviceCode, CoffreSecretService, ConfigAuth};
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

/// Cache local : le cache SQLite n'existe pas encore, il n'y a donc rien à vider.
// TODO(tache-6): remplacer par l'adaptateur `tuxteams-cache` (vidage de la base à la déconnexion).
pub struct CacheAbsent;

#[async_trait]
impl PortCache for CacheAbsent {
    async fn clear_all(&self) -> Result<(), ErreurCache> {
        Ok(())
    }
}

/// Cas d'usage assemblés avec les adaptateurs réels.
pub struct Services {
    pub horloge: Arc<HorlogeSysteme>,
    pub connexion: Arc<CasConnexion>,
    pub session: Arc<CasSession>,
    pub deconnexion: Arc<CasDeconnexion>,
}

pub fn assembler(config: ConfigAuth) -> Result<Services, ErreurAuth> {
    let horloge = Arc::new(HorlogeSysteme);
    let identite = Arc::new(ClientDeviceCode::new(config)?);
    let coffre = Arc::new(CoffreSecretService::new());
    let session = Arc::new(CasSession::new(
        coffre.clone(),
        identite.clone(),
        horloge.clone(),
    ));
    let connexion = Arc::new(CasConnexion::new(
        identite,
        horloge.clone(),
        Arc::new(NavigateurSysteme),
        Arc::new(PressePapiersSysteme::default()),
        session.clone(),
    ));
    let deconnexion = Arc::new(CasDeconnexion::new(coffre, Arc::new(CacheAbsent)));
    Ok(Services {
        horloge,
        connexion,
        session,
        deconnexion,
    })
}
