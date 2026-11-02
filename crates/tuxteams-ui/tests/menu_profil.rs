//! Menu de l'avatar : compte connecté et déconnexion (backend de test Slint).

use std::cell::Cell;
use std::rc::Rc;

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use slint::ComponentHandle;
use tuxteams_ui::Fenetre;

fn fenetre() -> Fenetre {
    i_slint_backend_testing::init_no_event_loop();
    let fenetre = Fenetre::new().expect("fenêtre créable");
    fenetre.set_libelle_profil("Profil : Alex Example".into());
    fenetre.set_nom_compte("Alex Example".into());
    fenetre.set_email_compte("alex@contoso.example".into());
    fenetre.show().expect("fenêtre affichable");
    fenetre
}

fn nombre(fenetre: &Fenetre, libelle: &str) -> usize {
    ElementHandle::find_by_accessible_label(fenetre, libelle).count()
}

fn avatar(fenetre: &Fenetre) -> ElementHandle {
    ElementHandle::find_by_accessible_label(fenetre, "Profil : Alex Example")
        .next()
        .expect("avatar")
}

#[test]
fn le_menu_du_profil_est_ferme_au_demarrage() {
    let fenetre = fenetre();
    assert_eq!(nombre(&fenetre, "Se déconnecter"), 0);
    assert!(!fenetre.get_menu_profil_ouvert());
}

#[test]
fn cliquer_l_avatar_ouvre_le_menu_avec_le_nom_et_l_email_du_compte() {
    let fenetre = fenetre();

    avatar(&fenetre).invoke_accessible_default_action();

    assert!(fenetre.get_menu_profil_ouvert());
    assert_eq!(nombre(&fenetre, "Se déconnecter"), 1);
    assert_eq!(nombre(&fenetre, "Alex Example"), 1, "nom du compte");
    assert_eq!(
        nombre(&fenetre, "alex@contoso.example"),
        1,
        "e-mail du compte"
    );
}

#[test]
fn un_second_clic_sur_l_avatar_ferme_le_menu() {
    let fenetre = fenetre();
    avatar(&fenetre).invoke_accessible_default_action();

    avatar(&fenetre).invoke_accessible_default_action();

    assert_eq!(nombre(&fenetre, "Se déconnecter"), 0);
}

#[test]
fn se_deconnecter_declenche_le_cas_d_usage_et_ferme_le_menu() {
    let fenetre = fenetre();
    let appels = Rc::new(Cell::new(0));
    fenetre.on_se_deconnecter({
        let appels = appels.clone();
        move || appels.set(appels.get() + 1)
    });
    avatar(&fenetre).invoke_accessible_default_action();

    ElementHandle::find_by_accessible_label(&fenetre, "Se déconnecter")
        .find(|element| element.accessible_role() == Some(AccessibleRole::Button))
        .expect("bouton « Se déconnecter »")
        .invoke_accessible_default_action();

    assert_eq!(appels.get(), 1);
    assert!(!fenetre.get_menu_profil_ouvert());
}
