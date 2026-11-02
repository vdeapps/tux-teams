//! Écran de connexion : relie la fenêtre Slint au cas d'usage `CasConnexion`.
//!
//! Composant mince : aucun appel réseau ni règle métier ici, seulement de la présentation.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use slint::ComponentHandle;
use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use tuxteams_application::cas::connexion::{CasConnexion, ErreurBureau};
use tuxteams_application::ports::horloge::Horloge;
use tuxteams_domaine::session::{Compte, EtatConnexion};

use url::Url;

use crate::FenetreConnexion;

/// Valeurs de la propriété `etat` de l'écran.
pub const ETAT_ACCUEIL: i32 = 0;
pub const ETAT_DEMANDE: i32 = 1;
pub const ETAT_CODE: i32 = 2;
pub const ETAT_ECHEC: i32 = 3;

/// Reflète un état de connexion dans la fenêtre. `maintenant` sert au texte d'expiration.
pub fn afficher_etat(fenetre: &FenetreConnexion, etat: &EtatConnexion, maintenant: DateTime<Utc>) {
    fenetre.set_message_erreur("".into());
    match etat {
        EtatConnexion::NonConnecte | EtatConnexion::Connecte { .. } => {
            fenetre.set_etat(ETAT_ACCUEIL);
        }
        EtatConnexion::CodeAffiche {
            code,
            url,
            expiration,
        } => {
            fenetre.set_code(code.as_str().into());
            fenetre.set_url(url.as_str().into());
            fenetre.set_expiration_texte(texte_expiration(*expiration - maintenant).into());
            fenetre.set_etat(ETAT_CODE);
        }
        EtatConnexion::Echec(raison) => {
            fenetre.set_message_erreur(raison.message().into());
            fenetre.set_etat(ETAT_ECHEC);
        }
    }
}

fn texte_expiration(restant: chrono::Duration) -> String {
    // Arrondi supérieur, au moins une minute.
    let minutes = ((restant.num_seconds() + 59) / 60).max(1);
    let unite = if minutes > 1 { "minutes" } else { "minute" };
    format!("Ce code expire dans {minutes} {unite}.")
}

/// Branche les boutons de l'écran sur le cas d'usage. `sur_connecte` est appelé sur le fil de
/// l'interface lorsque la connexion aboutit.
pub fn brancher(
    fenetre: &FenetreConnexion,
    cas: Arc<CasConnexion>,
    horloge: Arc<dyn Horloge>,
    runtime: Handle,
    sur_connecte: Arc<dyn Fn(Compte) + Send + Sync>,
) {
    // Une tentative = une génération : les notifications d'une tentative abandonnée sont ignorées.
    let generation = Arc::new(AtomicU64::new(0));
    let annulation: Rc<RefCell<Option<CancellationToken>>> = Rc::new(RefCell::new(None));

    fenetre.on_se_connecter({
        let faible = fenetre.as_weak();
        let cas = cas.clone();
        let generation = generation.clone();
        let annulation = annulation.clone();
        move || {
            let Some(fenetre) = faible.upgrade() else {
                return;
            };
            let jeton = CancellationToken::new();
            if let Some(ancien) = annulation.borrow_mut().replace(jeton.clone()) {
                ancien.cancel();
            }
            let ma_generation = generation.fetch_add(1, Ordering::SeqCst) + 1;
            fenetre.set_message_erreur("".into());
            fenetre.set_etat(ETAT_DEMANDE);

            let cas = cas.clone();
            let faible = faible.clone();
            let generation = generation.clone();
            let horloge = horloge.clone();
            let sur_connecte = sur_connecte.clone();
            runtime.spawn(async move {
                let notifier = move |etat: EtatConnexion| {
                    let maintenant = horloge.now();
                    let faible = faible.clone();
                    let generation = generation.clone();
                    let sur_connecte = sur_connecte.clone();
                    let envoi = slint::invoke_from_event_loop(move || {
                        if generation.load(Ordering::SeqCst) != ma_generation {
                            return;
                        }
                        let Some(fenetre) = faible.upgrade() else {
                            return;
                        };
                        if let EtatConnexion::Connecte { compte } = etat {
                            sur_connecte(compte);
                        } else {
                            afficher_etat(&fenetre, &etat, maintenant);
                        }
                    });
                    if envoi.is_err() {
                        tracing::debug!("boucle d'événements arrêtée : état de connexion ignoré");
                    }
                };
                cas.start_connexion(jeton, &notifier).await;
            });
        }
    });

    fenetre.on_copier_et_ouvrir({
        let faible = fenetre.as_weak();
        let cas = cas.clone();
        move || {
            let Some(fenetre) = faible.upgrade() else {
                return;
            };
            let Ok(url) = Url::parse(fenetre.get_url().as_str()) else {
                return;
            };
            let message = match cas.open_verification(fenetre.get_code().as_str(), &url) {
                Ok(()) => "",
                Err(ErreurBureau::Copie) => {
                    "Copie impossible : sélectionnez le code ci-dessus et copiez-le à la main."
                }
                Err(ErreurBureau::Navigateur) => {
                    "Ouverture du navigateur impossible : ouvrez l'adresse ci-dessus à la main."
                }
            };
            fenetre.set_message_erreur(message.into());
        }
    });

    fenetre.on_annuler({
        let faible = fenetre.as_weak();
        move || {
            if let Some(jeton) = annulation.borrow_mut().take() {
                jeton.cancel();
            }
            generation.fetch_add(1, Ordering::SeqCst);
            if let Some(fenetre) = faible.upgrade() {
                afficher_etat(&fenetre, &EtatConnexion::NonConnecte, Utc::now());
            }
        }
    });
}
