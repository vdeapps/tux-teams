//! Binaire TuxTeams : composition, écran de connexion puis fenêtre principale.

mod composition;

use std::cell::RefCell;
use std::sync::Arc;

use slint::ComponentHandle;
use tuxteams_auth::ConfigAuth;
use tuxteams_domaine::session::Compte;
use tuxteams_ui::connexion::brancher;
use tuxteams_ui::{Fenetre, FenetreConnexion};

use composition::{HorlogeSysteme, assembler_connexion};

thread_local! {
    /// Fenêtre principale : garde le composant en vie tant que l'application tourne.
    static FENETRE_PRINCIPALE: RefCell<Option<Fenetre>> = const { RefCell::new(None) };
}

/// Ouvre la fenêtre principale pour le compte connecté, puis ferme l'écran de connexion.
fn ouvrir_fenetre_principale(compte: &Compte, connexion: &FenetreConnexion) {
    let ouverture = Fenetre::new().and_then(|fenetre| {
        fenetre.set_initiales(compte.initiales().into());
        if let Some(nom) = &compte.nom {
            fenetre.set_libelle_profil(format!("Profil : {nom}").into());
        }
        fenetre.show()?;
        FENETRE_PRINCIPALE.with(|cellule| *cellule.borrow_mut() = Some(fenetre));
        Ok(())
    });
    match ouverture {
        // La fenêtre principale est affichée avant de masquer l'écran de connexion : la boucle
        // d'événements ne s'arrête qu'à la fermeture de la dernière fenêtre.
        Ok(()) => {
            if let Err(erreur) = connexion.hide() {
                eprintln!("fermeture de l'écran de connexion impossible : {erreur}");
            }
        }
        Err(erreur) => eprintln!("ouverture de la fenêtre principale impossible : {erreur}"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let config = ConfigAuth::depuis_variables(|nom| std::env::var(nom).ok())?;
    let horloge = Arc::new(HorlogeSysteme);
    let cas = assembler_connexion(config, horloge.clone())?;

    let connexion = FenetreConnexion::new()?;
    let faible = connexion.as_weak();
    brancher(
        &connexion,
        cas,
        horloge,
        runtime.handle().clone(),
        Arc::new(move |compte| {
            if let Some(connexion) = faible.upgrade() {
                ouvrir_fenetre_principale(&compte, &connexion);
            }
        }),
    );
    connexion.show()?;
    slint::run_event_loop()?;
    Ok(())
}
