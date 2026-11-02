//! Chaîne complète sur une vraie boucle d'événements (backend de test) : clic « Se connecter »,
//! tâche asynchrone, code affiché, puis compte connecté. Un seul test par processus.

mod commun;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;
use tuxteams_application::cas::connexion::CasConnexion;
use tuxteams_application::ports::horloge::Horloge;
use tuxteams_application::ports::jetons::{
    Authentifie, DemandeCode, ErreurAuth, EtatSondage, PortConnexion,
};
use tuxteams_application::ports::navigateur::{
    ErreurNavigateur, ErreurPressePapiers, PortNavigateur, PortPressePapiers,
};
use tuxteams_domaine::session::Compte;
use tuxteams_ui::FenetreConnexion;
use tuxteams_ui::connexion::brancher;
use url::Url;

struct ServiceFictif;

#[async_trait]
impl PortConnexion for ServiceFictif {
    async fn start_device_code(&self) -> Result<DemandeCode, ErreurAuth> {
        Ok(DemandeCode {
            code_utilisateur: "ABCD1234".into(),
            code_appareil: "appareil-fictif".into(),
            url_verification: Url::parse("https://login.microsoft.com/device").expect("url"),
            expire_dans: Duration::from_secs(900),
            intervalle: Duration::from_secs(5),
        })
    }
    async fn poll_device_code(&self, _: &DemandeCode) -> Result<EtatSondage, ErreurAuth> {
        let compte =
            Compte::depuis_identite("tenant-a", Some("Alex Example"), None).expect("compte");
        Ok(EtatSondage::Termine(Authentifie {
            compte,
            refresh: None,
        }))
    }
}

struct HorlogeRapide;

#[async_trait]
impl Horloge for HorlogeRapide {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    async fn sleep(&self, _: Duration) {
        tokio::task::yield_now().await;
    }
}

struct BureauInerte;

impl PortNavigateur for BureauInerte {
    fn open_url(&self, _: &Url) -> Result<(), ErreurNavigateur> {
        Ok(())
    }
}

impl PortPressePapiers for BureauInerte {
    fn copy_text(&self, _: &str) -> Result<(), ErreurPressePapiers> {
        Ok(())
    }
}

#[test]
fn cliquer_se_connecter_mene_au_compte_connecte_sur_la_boucle_d_evenements() {
    i_slint_backend_testing::init_integration_test_with_system_time();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .expect("runtime");
    let cas = Arc::new(CasConnexion::new(
        Arc::new(ServiceFictif),
        Arc::new(HorlogeRapide),
        Arc::new(BureauInerte),
        Arc::new(BureauInerte),
        commun::session_sans_trousseau(),
    ));
    let connecte: Arc<Mutex<Option<Compte>>> = Arc::default();
    let fenetre = FenetreConnexion::new().expect("fenêtre créable");
    brancher(
        &fenetre,
        cas,
        Arc::new(HorlogeRapide),
        runtime.handle().clone(),
        {
            let connecte = connecte.clone();
            Arc::new(move |compte, _| {
                *connecte.lock().expect("verrou") = Some(compte);
                let _ = slint::quit_event_loop();
            })
        },
    );
    fenetre.show().expect("fenêtre affichable");
    // Garde-fou : la boucle s'arrête même si la chaîne est rompue.
    let garde = slint::Timer::default();
    garde.start(
        slint::TimerMode::SingleShot,
        Duration::from_secs(10),
        || {
            let _ = slint::quit_event_loop();
        },
    );

    ElementHandle::find_by_accessible_label(&fenetre, "Se connecter")
        .next()
        .expect("bouton de connexion")
        .invoke_accessible_default_action();
    slint::run_event_loop_until_quit().expect("boucle d'événements");

    let compte = connecte.lock().expect("verrou").clone();
    assert_eq!(
        compte.and_then(|c| c.nom).as_deref(),
        Some("Alex Example"),
        "le compte doit être transmis à `sur_connecte`"
    );
}
