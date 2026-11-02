//! Écran de connexion via le backend de test Slint (sans affichage ni réseau).

mod commun;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use slint::ComponentHandle;
use tuxteams_application::cas::connexion::CasConnexion;
use tuxteams_application::ports::horloge::Horloge;
use tuxteams_application::ports::jetons::{DemandeCode, ErreurAuth, EtatSondage, PortConnexion};
use tuxteams_application::ports::navigateur::{
    ErreurNavigateur, ErreurPressePapiers, PortNavigateur, PortPressePapiers,
};
use tuxteams_domaine::session::{EtatConnexion, RaisonEchec};
use tuxteams_ui::connexion::{ETAT_ACCUEIL, ETAT_CODE, ETAT_ECHEC, afficher_etat, brancher};
use tuxteams_ui::{Fenetre, FenetreConnexion};
use url::Url;

/// Service de connexion inutilisé par ces tests (aucun clic sur « Se connecter »).
struct ConnexionInerte;

#[async_trait]
impl PortConnexion for ConnexionInerte {
    async fn start_device_code(&self) -> Result<DemandeCode, ErreurAuth> {
        Err(ErreurAuth::Reseau)
    }
    async fn poll_device_code(&self, _: &DemandeCode) -> Result<EtatSondage, ErreurAuth> {
        Err(ErreurAuth::Reseau)
    }
}

struct HorlogeFixe;

#[async_trait]
impl Horloge for HorlogeFixe {
    fn now(&self) -> DateTime<Utc> {
        maintenant()
    }
    async fn sleep(&self, _: Duration) {}
}

#[derive(Default)]
struct Bureau {
    copies: Mutex<Vec<String>>,
    urls: Mutex<Vec<String>>,
}

impl PortNavigateur for Bureau {
    fn open_url(&self, url: &Url) -> Result<(), ErreurNavigateur> {
        self.urls.lock().expect("verrou").push(url.to_string());
        Ok(())
    }
}

impl PortPressePapiers for Bureau {
    fn copy_text(&self, texte: &str) -> Result<(), ErreurPressePapiers> {
        self.copies.lock().expect("verrou").push(texte.to_owned());
        Ok(())
    }
}

fn maintenant() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 2, 9, 0, 0).unwrap()
}

fn code_affiche() -> EtatConnexion {
    EtatConnexion::CodeAffiche {
        code: "ABCD1234".into(),
        url: Url::parse("https://login.microsoft.com/device").expect("url"),
        expiration: maintenant() + chrono::Duration::minutes(15),
    }
}

struct Banc {
    fenetre: FenetreConnexion,
    bureau: Arc<Bureau>,
    _runtime: tokio::runtime::Runtime,
}

fn banc() -> Banc {
    i_slint_backend_testing::init_no_event_loop();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime");
    let bureau = Arc::new(Bureau::default());
    let cas = Arc::new(CasConnexion::new(
        Arc::new(ConnexionInerte),
        Arc::new(HorlogeFixe),
        bureau.clone(),
        bureau.clone(),
        commun::session_sans_trousseau(),
    ));
    let fenetre = FenetreConnexion::new().expect("fenêtre créable");
    brancher(
        &fenetre,
        cas,
        Arc::new(HorlogeFixe),
        runtime.handle().clone(),
        Arc::new(|_, _| {}),
    );
    fenetre.show().expect("fenêtre affichable");
    Banc {
        fenetre,
        bureau,
        _runtime: runtime,
    }
}

fn par_libelle(fenetre: &FenetreConnexion, libelle: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(fenetre, libelle)
        .next()
        .unwrap_or_else(|| panic!("élément « {libelle} » introuvable"))
}

fn bouton(fenetre: &FenetreConnexion, libelle: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(fenetre, libelle)
        .find(|element| element.accessible_role() == Some(AccessibleRole::Button))
        .unwrap_or_else(|| panic!("bouton « {libelle} » introuvable"))
}

#[test]
fn l_ecran_de_connexion_affiche_le_code_et_l_url() {
    let banc = banc();

    afficher_etat(&banc.fenetre, &code_affiche(), maintenant());

    assert_eq!(banc.fenetre.get_etat(), ETAT_CODE);
    let code = par_libelle(&banc.fenetre, "Code de connexion");
    assert_eq!(code.accessible_value().as_deref(), Some("ABCD1234"));
    let url = par_libelle(&banc.fenetre, "Adresse de connexion");
    assert_eq!(
        url.accessible_value().as_deref(),
        Some("https://login.microsoft.com/device")
    );
    assert_eq!(
        banc.fenetre.get_expiration_texte().as_str(),
        "Ce code expire dans 15 minutes."
    );
}

#[test]
fn le_bouton_copier_et_ouvrir_appelle_le_navigateur() {
    let banc = banc();
    afficher_etat(&banc.fenetre, &code_affiche(), maintenant());

    bouton(&banc.fenetre, "Copier le code et ouvrir le navigateur")
        .invoke_accessible_default_action();

    assert_eq!(*banc.bureau.copies.lock().expect("verrou"), ["ABCD1234"]);
    assert_eq!(
        *banc.bureau.urls.lock().expect("verrou"),
        ["https://login.microsoft.com/device"]
    );
}

#[test]
fn le_bouton_annuler_revient_a_l_accueil() {
    let banc = banc();
    afficher_etat(&banc.fenetre, &code_affiche(), maintenant());
    assert_eq!(banc.fenetre.get_etat(), ETAT_CODE);

    bouton(&banc.fenetre, "Annuler").invoke_accessible_default_action();

    assert_eq!(banc.fenetre.get_etat(), ETAT_ACCUEIL);
    // Retour à l'accueil : le bouton de connexion est de nouveau proposé.
    bouton(&banc.fenetre, "Se connecter");
}

#[test]
fn un_code_expire_affiche_le_message_d_expiration() {
    let banc = banc();

    afficher_etat(
        &banc.fenetre,
        &EtatConnexion::Echec(RaisonEchec::Expire),
        maintenant(),
    );

    assert_eq!(banc.fenetre.get_etat(), ETAT_ECHEC);
    assert!(banc.fenetre.get_message_erreur().contains("expiré"));
    let alerte = par_libelle(&banc.fenetre, "Erreur de connexion");
    assert_eq!(alerte.accessible_role(), Some(AccessibleRole::Groupbox));
    bouton(&banc.fenetre, "Réessayer");
}

#[test]
fn un_compte_personnel_affiche_un_message_de_refus() {
    let banc = banc();

    afficher_etat(
        &banc.fenetre,
        &EtatConnexion::Echec(RaisonEchec::ComptePersonnel),
        maintenant(),
    );

    let message = banc.fenetre.get_message_erreur();
    assert!(message.contains("personnels"), "message : {message}");
    assert!(message.contains("professionnel"), "message : {message}");
}

#[test]
fn la_fenetre_principale_porte_le_libelle_du_profil_connecte() {
    i_slint_backend_testing::init_no_event_loop();
    let fenetre = Fenetre::new().expect("fenêtre créable");
    fenetre.show().expect("fenêtre affichable");
    assert_eq!(
        ElementHandle::find_by_accessible_label(&fenetre, "Profil : non connecté").count(),
        1,
        "sans compte, l'avatar reste « non connecté »"
    );

    fenetre.set_libelle_profil("Profil : Alex Example".into());

    assert_eq!(
        ElementHandle::find_by_accessible_label(&fenetre, "Profil : Alex Example").count(),
        1
    );
}
