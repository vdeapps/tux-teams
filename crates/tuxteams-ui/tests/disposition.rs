//! Disposition de la fenêtre (rail, barre supérieure, panneau de liste, contenu), d'après
//! DESIGN.md, via le backend de test Slint (sans affichage).

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use slint::{ComponentHandle, LogicalSize};
use tuxteams_ui::Fenetre;

/// Valeurs de DESIGN.md (source indépendante du code).
const LARGEUR_RAIL: f32 = 76.0;
const LARGEUR_BOUTON_RAIL: f32 = 64.0;
const HAUTEUR_BARRE_SUPERIEURE: f32 = 48.0;
const LARGEUR_PANNEAU: f32 = 320.0;
const LARGEUR_MAX_RECHERCHE: f32 = 480.0;
const TAILLE_AVATAR: f32 = 32.0;
const TOLERANCE: f32 = 1.0;

/// Tailles de fenêtre visées : bureau courant et fenêtre étroite.
const TAILLES: [(f32, f32); 2] = [(1280.0, 800.0), (900.0, 700.0)];

fn fenetre(largeur: f32, hauteur: f32) -> Fenetre {
    i_slint_backend_testing::init_no_event_loop();
    let fenetre = Fenetre::new().expect("fenêtre créable");
    fenetre
        .window()
        .set_size(LogicalSize::new(largeur, hauteur));
    fenetre.show().expect("fenêtre affichable");
    fenetre
}

fn par_libelle(fenetre: &Fenetre, libelle: &str) -> Vec<ElementHandle> {
    ElementHandle::find_by_accessible_label(fenetre, libelle).collect()
}

fn unique(fenetre: &Fenetre, libelle: &str) -> ElementHandle {
    let trouves = par_libelle(fenetre, libelle);
    assert_eq!(trouves.len(), 1, "« {libelle} » attendu une fois");
    trouves.into_iter().next().expect("élément présent")
}

fn par_id(fenetre: &Fenetre, id: &str) -> Vec<ElementHandle> {
    ElementHandle::find_by_element_id(fenetre, id).collect()
}

fn active_onglet(fenetre: &Fenetre, libelle: &str) {
    ElementHandle::find_by_accessible_label(fenetre, libelle)
        .find(|element| element.accessible_role() == Some(AccessibleRole::Tab))
        .unwrap_or_else(|| panic!("onglet « {libelle} » introuvable"))
        .invoke_accessible_default_action();
}

/// (x, y, largeur, hauteur) absolus.
fn cadre(element: &ElementHandle) -> (f32, f32, f32, f32) {
    let position = element.absolute_position();
    let taille = element.size();
    (position.x, position.y, taille.width, taille.height)
}

fn centre_x(element: &ElementHandle) -> f32 {
    let (x, _, largeur, _) = cadre(element);
    x + largeur / 2.0
}

fn est_contenu_dans(interieur: &ElementHandle, exterieur: &ElementHandle) -> bool {
    let (xi, yi, li, hi) = cadre(interieur);
    let (xe, ye, le, he) = cadre(exterieur);
    xi >= xe - TOLERANCE
        && yi >= ye - TOLERANCE
        && xi + li <= xe + le + TOLERANCE
        && yi + hi <= ye + he + TOLERANCE
}

#[test]
fn le_rail_mesure_76_px_de_large_et_part_du_bord_gauche() {
    let fenetre = fenetre(1280.0, 800.0);
    let (x, y, largeur, hauteur) = cadre(&unique(&fenetre, "Navigation principale"));
    assert_eq!((x, y), (0.0, 0.0));
    assert_eq!(largeur, LARGEUR_RAIL);
    assert_eq!(hauteur, 800.0);
}

#[test]
fn l_icone_de_chaque_element_du_rail_est_centree_horizontalement_dans_le_rail() {
    let fenetre = fenetre(1280.0, 800.0);
    let icones = par_id(&fenetre, "ElementRail::icone");
    assert_eq!(icones.len(), 4, "une icône par élément du rail");
    for icone in &icones {
        let ecart = centre_x(icone) - LARGEUR_RAIL / 2.0;
        assert!(ecart.abs() <= TOLERANCE, "icône décentrée de {ecart} px");
        assert_eq!(icone.size().width, 24.0);
    }
}

#[test]
fn le_libelle_de_chaque_element_du_rail_est_centre_sous_son_icone() {
    let fenetre = fenetre(1280.0, 800.0);
    let icones = par_id(&fenetre, "ElementRail::icone");
    let libelles = par_id(&fenetre, "ElementRail::libelle");
    assert_eq!(libelles.len(), 4, "un libellé par élément du rail");
    for (icone, libelle) in icones.iter().zip(&libelles) {
        let ecart = centre_x(libelle) - LARGEUR_RAIL / 2.0;
        assert!(ecart.abs() <= TOLERANCE, "libellé décentré de {ecart} px");
        let (_, y_icone, _, h_icone) = cadre(icone);
        assert!(
            cadre(libelle).1 >= y_icone + h_icone,
            "libellé au-dessus du bas de l'icône"
        );
    }
}

#[test]
fn la_barre_d_accent_active_est_collee_au_bord_gauche_du_rail_hors_du_bouton() {
    let fenetre = fenetre(1280.0, 800.0);
    for libelle in ["Activité", "Conversation", "Calendrier", "Paramètres"] {
        active_onglet(&fenetre, libelle);
        let barres = par_id(&fenetre, "ElementRail::barre-accent");
        assert_eq!(barres.len(), 1, "une seule barre visible ({libelle})");
        let (x, _, largeur, _) = cadre(&barres[0]);
        assert_eq!(x, 0.0, "barre d'accent hors du bord gauche ({libelle})");
        assert_eq!(largeur, 3.0);
        let bord_gauche_bouton = (LARGEUR_RAIL - LARGEUR_BOUTON_RAIL) / 2.0;
        assert!(
            x + largeur <= bord_gauche_bouton,
            "la barre empiète sur le bouton ({libelle})"
        );
        let icones = par_id(&fenetre, "ElementRail::icone");
        assert!(
            icones.iter().all(|icone| cadre(icone).0 >= x + largeur),
            "la barre est dessinée sur une icône ({libelle})"
        );
    }
}

#[test]
fn la_barre_superieure_contient_la_recherche_centree_et_l_avatar() {
    let fenetre = fenetre(TAILLES[0].0, TAILLES[0].1);
    for (largeur, hauteur) in TAILLES {
        fenetre
            .window()
            .set_size(LogicalSize::new(largeur, hauteur));
        let barre = unique(&fenetre, "Barre supérieure");
        assert_eq!(
            cadre(&barre),
            (
                LARGEUR_RAIL,
                0.0,
                largeur - LARGEUR_RAIL,
                HAUTEUR_BARRE_SUPERIEURE
            )
        );

        let recherche = unique(&fenetre, "Rechercher (Ctrl+K)");
        assert!(
            est_contenu_dans(&recherche, &barre),
            "recherche hors de la barre"
        );
        assert!(recherche.size().width <= LARGEUR_MAX_RECHERCHE);
        let ecart = centre_x(&recherche) - centre_x(&barre);
        assert!(
            ecart.abs() <= TOLERANCE,
            "recherche décentrée de {ecart} px"
        );

        let avatar = unique(&fenetre, "Profil : non connecté");
        assert!(est_contenu_dans(&avatar, &barre), "avatar hors de la barre");
        assert_eq!(avatar.size().width, TAILLE_AVATAR);
        assert_eq!(avatar.size().height, TAILLE_AVATAR);
        let (xr, _, lr, _) = cadre(&recherche);
        assert!(
            cadre(&avatar).0 > xr + lr,
            "avatar à gauche de la recherche"
        );
    }
}

#[test]
fn le_panneau_de_liste_existe_en_activite_et_conversation_mais_pas_en_calendrier() {
    let fenetre = fenetre(1280.0, 800.0);
    for (libelle, attendu) in [
        ("Conversation", true),
        ("Calendrier", false),
        ("Activité", true),
    ] {
        active_onglet(&fenetre, libelle);
        let panneaux = par_libelle(&fenetre, "Panneau de liste");
        assert_eq!(panneaux.len(), usize::from(attendu), "panneau en {libelle}");
        let contenu = unique(&fenetre, "Contenu");
        if attendu {
            assert_eq!(
                cadre(&panneaux[0]),
                (
                    LARGEUR_RAIL,
                    HAUTEUR_BARRE_SUPERIEURE,
                    LARGEUR_PANNEAU,
                    800.0 - HAUTEUR_BARRE_SUPERIEURE
                )
            );
            assert_eq!(cadre(&contenu).0, LARGEUR_RAIL + LARGEUR_PANNEAU);
        } else {
            assert_eq!(cadre(&contenu).0, LARGEUR_RAIL, "contenu pleine largeur");
        }
    }
}

#[test]
fn l_etat_vide_affiche_le_titre_propre_a_la_vue_active() {
    let fenetre = fenetre(1280.0, 800.0);
    let titres = [
        ("Activité", "Aucune activité pour le moment"),
        ("Conversation", "Sélectionnez une conversation"),
        ("Calendrier", "Votre calendrier s'affichera ici"),
    ];
    for (onglet, titre) in titres {
        active_onglet(&fenetre, onglet);
        let contenu = unique(&fenetre, "Contenu");
        let affiche = unique(&fenetre, titre);
        assert!(
            est_contenu_dans(&affiche, &contenu),
            "titre hors du contenu"
        );
        let ecart = centre_x(&affiche) - centre_x(&contenu);
        assert!(ecart.abs() <= TOLERANCE, "titre décentré de {ecart} px");
        for (_, autre) in titres.iter().filter(|(_, autre)| *autre != titre) {
            assert!(
                par_libelle(&fenetre, autre).is_empty(),
                "« {autre} » en {onglet}"
            );
        }
    }
}
