//! Bandeau « session non persistée » de la fenêtre principale (backend de test Slint).

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use slint::ComponentHandle;
use tuxteams_ui::Fenetre;

const LIBELLE_BANDEAU: &str = "Session non persistée";

fn fenetre() -> Fenetre {
    i_slint_backend_testing::init_no_event_loop();
    let fenetre = Fenetre::new().expect("fenêtre créable");
    fenetre.show().expect("fenêtre affichable");
    fenetre
}

fn bandeaux(fenetre: &Fenetre) -> Vec<ElementHandle> {
    ElementHandle::find_by_accessible_label(fenetre, LIBELLE_BANDEAU).collect()
}

fn bouton_fermer(fenetre: &Fenetre) -> ElementHandle {
    ElementHandle::find_by_accessible_label(fenetre, "Fermer le bandeau")
        .find(|element| element.accessible_role() == Some(AccessibleRole::Button))
        .expect("bouton de fermeture du bandeau")
}

#[test]
fn le_bandeau_session_non_persistee_s_affiche_si_le_trousseau_manque() {
    let fenetre = fenetre();
    assert!(bandeaux(&fenetre).is_empty(), "absent par défaut");

    fenetre.set_session_non_persistee(true);

    let bandeaux = bandeaux(&fenetre);
    assert_eq!(bandeaux.len(), 1);
    assert!(bandeaux[0].size().width > 0.0 && bandeaux[0].size().height > 0.0);
}

#[test]
fn le_bandeau_se_place_sous_la_barre_et_laisse_le_rail_utilisable() {
    let fenetre = fenetre();
    fenetre.set_session_non_persistee(true);

    let barre = ElementHandle::find_by_accessible_label(&fenetre, "Barre supérieure")
        .next()
        .expect("barre supérieure");
    let bandeau = &bandeaux(&fenetre)[0];
    assert!(
        bandeau.absolute_position().y >= barre.absolute_position().y + barre.size().height,
        "le bandeau doit suivre la barre supérieure"
    );
    ElementHandle::find_by_accessible_label(&fenetre, "Conversation")
        .find(|element| element.accessible_role() == Some(AccessibleRole::Tab))
        .expect("rail toujours présent")
        .invoke_accessible_default_action();
    assert_eq!(fenetre.get_onglet_actif(), 1, "le bandeau ne bloque pas");
}

#[test]
fn fermer_le_bandeau_le_masque() {
    let fenetre = fenetre();
    fenetre.set_session_non_persistee(true);

    bouton_fermer(&fenetre).invoke_accessible_default_action();

    assert!(bandeaux(&fenetre).is_empty());
    assert!(!fenetre.get_session_non_persistee());
}
