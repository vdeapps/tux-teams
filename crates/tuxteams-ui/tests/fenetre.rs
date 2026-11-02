//! Fenêtre et rail de navigation, via le backend de test Slint (sans affichage).

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use slint::ComponentHandle;
use tuxteams_ui::Fenetre;

fn fenetre() -> Fenetre {
    i_slint_backend_testing::init_no_event_loop();
    let fenetre = Fenetre::new().expect("fenêtre créable");
    fenetre.show().expect("fenêtre affichable");
    fenetre
}

/// Élément du rail (rôle onglet) : un titre de panneau peut porter le même libellé.
fn element(fenetre: &Fenetre, libelle: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(fenetre, libelle)
        .find(|element| element.accessible_role() == Some(AccessibleRole::Tab))
        .unwrap_or_else(|| panic!("élément « {libelle} » introuvable"))
}

fn ordonnee(fenetre: &Fenetre, libelle: &str) -> f32 {
    element(fenetre, libelle).absolute_position().y
}

#[test]
fn la_fenetre_affiche_le_rail_activite_conversation_calendrier() {
    let fenetre = fenetre();
    let activite = ordonnee(&fenetre, "Activité");
    let conversation = ordonnee(&fenetre, "Conversation");
    let calendrier = ordonnee(&fenetre, "Calendrier");
    assert!(
        activite < conversation,
        "Activité doit précéder Conversation"
    );
    assert!(
        conversation < calendrier,
        "Conversation doit précéder Calendrier"
    );
}

#[test]
fn le_rail_porte_parametres_en_bas() {
    let fenetre = fenetre();
    let parametres = element(&fenetre, "Paramètres");
    let hauteur_fenetre = fenetre.window().size().height as f32;
    let bas = parametres.absolute_position().y + parametres.size().height;
    assert!(parametres.absolute_position().y > ordonnee(&fenetre, "Calendrier"));
    assert!(bas <= hauteur_fenetre, "Paramètres déborde de la fenêtre");
    assert!(
        bas > hauteur_fenetre - 32.0,
        "Paramètres n'est pas ancré en bas : {bas}"
    );
}

#[test]
fn cliquer_un_element_du_rail_le_rend_actif() {
    let fenetre = fenetre();
    assert_eq!(
        element(&fenetre, "Activité").accessible_checked(),
        Some(true)
    );
    assert_eq!(
        element(&fenetre, "Conversation").accessible_checked(),
        Some(false)
    );

    element(&fenetre, "Conversation").invoke_accessible_default_action();

    assert_eq!(
        element(&fenetre, "Conversation").accessible_checked(),
        Some(true)
    );
    assert_eq!(
        element(&fenetre, "Activité").accessible_checked(),
        Some(false)
    );
    assert_eq!(fenetre.get_onglet_actif(), 1);
}

#[test]
fn le_theme_sombre_change_le_fond() {
    let fenetre = fenetre();
    let fond_clair = fenetre.get_fond_fenetre();
    fenetre.set_sombre(true);
    let fond_sombre = fenetre.get_fond_fenetre();
    assert_ne!(fond_clair, fond_sombre);
    assert_eq!(fond_clair, slint::Color::from_rgb_u8(0xF5, 0xF5, 0xF5));
    assert_eq!(fond_sombre, slint::Color::from_rgb_u8(0x1F, 0x1F, 0x1F));
}
