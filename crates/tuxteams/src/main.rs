//! Binaire TuxTeams : composition, relance silencieuse, écran de connexion puis fenêtre principale.

mod composition;

use std::cell::RefCell;
use std::sync::Arc;

use chrono::Utc;
use slint::{CloseRequestResponse, ComponentHandle};
use tokio::runtime::Handle;
use tracing_subscriber::EnvFilter;
use tuxteams_application::cas::session::{Persistance, ResultatRelance};
use tuxteams_auth::ConfigAuth;
use tuxteams_domaine::session::{Compte, EtatConnexion, RaisonEchec};
use tuxteams_ui::connexion::{afficher_etat, brancher};
use tuxteams_ui::{Fenetre, FenetreConnexion};

use composition::{Services, assembler};

thread_local! {
    /// Fenêtres : gardent les composants en vie tant qu'ils sont affichés.
    static FENETRE_PRINCIPALE: RefCell<Option<Fenetre>> = const { RefCell::new(None) };
    static FENETRE_CONNEXION: RefCell<Option<FenetreConnexion>> = const { RefCell::new(None) };
}

/// Ce qu'il faut pour passer d'un écran à l'autre ; clonable et transmissible entre fils.
#[derive(Clone)]
struct Contexte {
    services: Arc<Services>,
    runtime: Handle,
}

/// Quitte l'application quand l'utilisateur ferme la fenêtre ; une fermeture programmatique
/// (changement d'écran) ne passe pas par ce rappel.
fn quitter_a_la_fermeture(fenetre: &impl ComponentHandle) {
    fenetre.window().on_close_requested(|| {
        if let Err(erreur) = slint::quit_event_loop() {
            tracing::warn!(%erreur, "arrêt de la boucle d'événements impossible");
        }
        CloseRequestResponse::HideWindow
    });
}

/// Affiche l'écran de connexion (avec la raison du retour à cet écran, s'il y en a une), puis
/// ferme la fenêtre principale éventuelle.
fn afficher_connexion(ctx: &Contexte, raison: Option<RaisonEchec>) {
    let ouverture = FenetreConnexion::new().and_then(|connexion| {
        quitter_a_la_fermeture(&connexion);
        let contexte = ctx.clone();
        brancher(
            &connexion,
            ctx.services.connexion.clone(),
            ctx.services.horloge.clone(),
            ctx.runtime.clone(),
            Arc::new(move |compte, persistee| {
                ouvrir_fenetre_principale(&contexte, &compte, persistee);
            }),
        );
        if let Some(raison) = raison {
            afficher_etat(&connexion, &EtatConnexion::Echec(raison), Utc::now());
        }
        connexion.show()?;
        FENETRE_CONNEXION.with(|cellule| *cellule.borrow_mut() = Some(connexion));
        Ok(())
    });
    match ouverture {
        Ok(()) => {
            let ancienne = FENETRE_PRINCIPALE.with(|cellule| cellule.borrow_mut().take());
            if let Some(Err(erreur)) = ancienne.map(|fenetre| fenetre.hide()) {
                tracing::warn!(%erreur, "fermeture de la fenêtre principale impossible");
            }
        }
        Err(erreur) => tracing::error!(%erreur, "ouverture de l'écran de connexion impossible"),
    }
}

/// Renseigne la fenêtre principale avec le compte connecté et la sort de l'état de chargement.
fn renseigner_compte(ctx: &Contexte, fenetre: &Fenetre, compte: &Compte, persistee: bool) {
    fenetre.set_chargement(false);
    fenetre.set_initiales(compte.initiales().into());
    if let Some(nom) = &compte.nom {
        fenetre.set_libelle_profil(format!("Profil : {nom}").into());
        fenetre.set_nom_compte(nom.as_str().into());
    }
    if let Some(email) = &compte.email {
        fenetre.set_email_compte(email.as_str().into());
    }
    fenetre.set_session_non_persistee(!persistee);
    let contexte = ctx.clone();
    fenetre.on_se_deconnecter(move || deconnecter(&contexte));
}

/// Affiche aussitôt la fenêtre principale en état de chargement (relance silencieuse en cours).
fn afficher_chargement() {
    let ouverture = Fenetre::new().and_then(|fenetre| {
        quitter_a_la_fermeture(&fenetre);
        fenetre.set_chargement(true);
        fenetre.show()?;
        FENETRE_PRINCIPALE.with(|cellule| *cellule.borrow_mut() = Some(fenetre));
        Ok(())
    });
    if let Err(erreur) = ouverture {
        tracing::error!(%erreur, "ouverture de la fenêtre de chargement impossible");
    }
}

/// Ouvre la fenêtre principale pour le compte connecté (celle du chargement si elle existe),
/// puis ferme l'écran de connexion éventuel.
fn ouvrir_fenetre_principale(ctx: &Contexte, compte: &Compte, persistee: bool) {
    let existante = FENETRE_PRINCIPALE.with(|cellule| cellule.borrow_mut().take());
    let ouverture = match existante {
        Some(fenetre) => Ok(fenetre),
        None => Fenetre::new().inspect(quitter_a_la_fermeture),
    }
    .and_then(|fenetre| {
        renseigner_compte(ctx, &fenetre, compte, persistee);
        fenetre.show()?;
        FENETRE_PRINCIPALE.with(|cellule| *cellule.borrow_mut() = Some(fenetre));
        Ok(())
    });
    match ouverture {
        // La fenêtre principale est affichée avant de masquer l'écran de connexion : la boucle
        // d'événements ne doit jamais se retrouver sans fenêtre.
        Ok(()) => {
            let ancienne = FENETRE_CONNEXION.with(|cellule| cellule.borrow_mut().take());
            if let Some(Err(erreur)) = ancienne.map(|connexion| connexion.hide()) {
                tracing::warn!(%erreur, "fermeture de l'écran de connexion impossible");
            }
        }
        Err(erreur) => tracing::error!(%erreur, "ouverture de la fenêtre principale impossible"),
    }
}

/// Déconnexion : secret et cache supprimés hors du fil de l'interface, puis écran de connexion.
fn deconnecter(ctx: &Contexte) {
    let contexte = ctx.clone();
    ctx.runtime.spawn(async move {
        let bilan = contexte.services.deconnexion.logout().await;
        if !bilan.secret_supprime || !bilan.cache_vide {
            tracing::warn!(?bilan, "déconnexion incomplète");
        }
        let envoi = slint::invoke_from_event_loop(move || afficher_connexion(&contexte, None));
        if envoi.is_err() {
            tracing::debug!("boucle d'événements arrêtée : retour à la connexion ignoré");
        }
    });
}

/// Relance silencieuse au démarrage : la fenêtre s'affiche en chargement, puis fenêtre principale si le refresh token est encore valable,
/// sinon écran de connexion.
fn relancer(ctx: &Contexte) {
    afficher_chargement();
    let contexte = ctx.clone();
    ctx.runtime.spawn(async move {
        let resultat = contexte.services.session.restore().await;
        let envoi = slint::invoke_from_event_loop(move || match resultat {
            ResultatRelance::Connecte {
                compte,
                persistance,
            } => {
                let persistee = persistance == Persistance::Active;
                ouvrir_fenetre_principale(&contexte, &compte, persistee);
            }
            ResultatRelance::AConnecter { raison } => afficher_connexion(&contexte, raison),
        });
        if envoi.is_err() {
            tracing::debug!("boucle d'événements arrêtée : relance ignorée");
        }
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Niveau par `RUST_LOG`, `warn` par défaut. Aucun secret n'est journalisé quel que soit le niveau.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let config = ConfigAuth::depuis_variables(|nom| std::env::var(nom).ok())?;
    let contexte = Contexte {
        services: Arc::new(assembler(config)?),
        runtime: runtime.handle().clone(),
    };
    relancer(&contexte);
    slint::run_event_loop_until_quit()?;
    Ok(())
}
