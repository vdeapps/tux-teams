//! État « chargement » de la fenêtre principale pendant la relance silencieuse (backend de test Slint).

use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;
use tuxteams_ui::Fenetre;

fn fenetre() -> Fenetre {
    i_slint_backend_testing::init_no_event_loop();
    let fenetre = Fenetre::new().expect("fenêtre créable");
    fenetre.show().expect("fenêtre affichable");
    fenetre
}

fn textes(fenetre: &Fenetre, libelle: &str) -> usize {
    ElementHandle::find_by_accessible_label(fenetre, libelle).count()
}

#[test]
fn la_fenetre_principale_s_affiche_en_chargement_pendant_la_relance() {
    let fenetre = fenetre();
    assert!(!fenetre.get_chargement(), "pas de chargement par défaut");
    assert_eq!(textes(&fenetre, "Reconnexion en cours"), 0);

    fenetre.set_chargement(true);

    assert_eq!(textes(&fenetre, "Reconnexion en cours"), 1);
    // Rail, barre et panneau squelette restent présents ; l'avatar reste neutre.
    assert_eq!(textes(&fenetre, "Barre supérieure"), 1);
    assert_eq!(textes(&fenetre, "Panneau de liste"), 1);
    assert_eq!(fenetre.get_initiales(), "??");
    let zone = ElementHandle::find_by_accessible_label(&fenetre, "Reconnexion en cours")
        .next()
        .expect("zone de reconnexion");
    assert!(zone.size().width > 0.0 && zone.size().height > 0.0);

    fenetre.set_chargement(false);
    assert_eq!(textes(&fenetre, "Reconnexion en cours"), 0);
}
