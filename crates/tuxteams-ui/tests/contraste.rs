//! Contraste WCAG AA (4.5:1) des paires de tokens, lu depuis le global Slint `Theme`.

use slint::{Color, ComponentHandle};
use tuxteams_ui::theme::ratio_contraste;
use tuxteams_ui::{Fenetre, Theme};

const SEUIL_AA: f64 = 4.5;

fn fenetre() -> Fenetre {
    i_slint_backend_testing::init_no_event_loop();
    Fenetre::new().expect("fenêtre créable")
}

fn verifie(nom: &str, texte: Color, fond: Color) {
    let ratio = ratio_contraste(texte, fond);
    assert!(
        ratio >= SEUIL_AA,
        "{nom} : contraste {ratio:.2}:1 < {SEUIL_AA}:1"
    );
}

fn verifie_texte_principal(sombre: bool) {
    let fenetre = fenetre();
    let theme = fenetre.global::<Theme>();
    theme.set_sombre(sombre);
    verifie("texte sur fond", theme.get_texte(), theme.get_fond());
    verifie("texte sur surface", theme.get_texte(), theme.get_surface());
}

#[test]
fn le_texte_principal_respecte_le_contraste_aa_en_clair() {
    verifie_texte_principal(false);
}

#[test]
fn le_texte_principal_respecte_le_contraste_aa_en_sombre() {
    verifie_texte_principal(true);
}

#[test]
fn le_texte_secondaire_respecte_le_contraste_aa_dans_les_deux_themes() {
    let fenetre = fenetre();
    let theme = fenetre.global::<Theme>();
    for sombre in [false, true] {
        theme.set_sombre(sombre);
        verifie(
            "secondaire sur fond",
            theme.get_texte_secondaire(),
            theme.get_fond(),
        );
        verifie(
            "secondaire sur surface",
            theme.get_texte_secondaire(),
            theme.get_surface(),
        );
    }
}

#[test]
fn le_texte_sur_accent_respecte_le_contraste_aa_dans_les_deux_themes() {
    let fenetre = fenetre();
    let theme = fenetre.global::<Theme>();
    for sombre in [false, true] {
        theme.set_sombre(sombre);
        verifie(
            "texte sur accent",
            theme.get_texte_sur_accent(),
            theme.get_accent(),
        );
    }
}

#[test]
fn le_libelle_actif_du_rail_respecte_le_contraste_aa_dans_les_deux_themes() {
    let fenetre = fenetre();
    let theme = fenetre.global::<Theme>();
    for sombre in [false, true] {
        theme.set_sombre(sombre);
        verifie(
            "libellé actif du rail sur accent-doux",
            theme.get_texte_rail_actif(),
            theme.get_accent_doux(),
        );
    }
}

#[test]
fn le_danger_respecte_le_contraste_aa_dans_les_deux_themes() {
    let fenetre = fenetre();
    let theme = fenetre.global::<Theme>();
    for sombre in [false, true] {
        theme.set_sombre(sombre);
        verifie("danger sur fond", theme.get_danger(), theme.get_fond());
        verifie(
            "danger sur surface",
            theme.get_danger(),
            theme.get_surface(),
        );
    }
}

#[test]
fn le_ratio_de_contraste_noir_sur_blanc_vaut_21() {
    // Valeur de référence WCAG : noir sur blanc = 21:1 (formule indépendante du code).
    let ratio = ratio_contraste(
        Color::from_rgb_u8(0, 0, 0),
        Color::from_rgb_u8(255, 255, 255),
    );
    assert!((ratio - 21.0).abs() < 1e-9, "ratio obtenu : {ratio}");
}
