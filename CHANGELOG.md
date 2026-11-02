# Journal des changements

## Tâche 1 — Socle

- Workspace Rust (8 crates), qualité (`make test lint format-check typecheck qualite`).
- Tokens de design (`theme.slint`, clair/sombre) et test de contraste WCAG AA.
- Fenêtre Slint avec rail (Activité, Conversation, Calendrier, Paramètres), police Inter, icônes Fluent.
- Reprise visuelle : rail 76 px (icônes et libellés centrés, barre d'accent au bord gauche),
  barre supérieure (recherche, avatar), panneau de liste en chargement (Activité, Conversation),
  états vides par vue ; tests de disposition headless (`tests/disposition.rs`).

## Tâche 2 — Connexion par device code (sans persistance)

- Domaine `session` : états de connexion, compte, refus des comptes personnels (tenant MSA), initiales.
- Application : ports `PortConnexion`, `Horloge`, `PortNavigateur`, `PortPressePapiers` et cas d'usage
  `CasConnexion` (intervalle annoncé, `slow_down` +5 s, expiration, annulation).
- `tuxteams-auth` : client device code maison sur reqwest 0.13 (client_id `1fec8e78-…`, autorité
  `organizations`, surcharges `TUXTEAMS_CLIENT_ID` / `TUXTEAMS_TENANT`), erreurs OAuth typées dont `AADSTS53003`.
- UI : écran de connexion (code en grand, copier et ouvrir le navigateur, annuler, expiration, refus) ;
  avatar du compte dans la fenêtre principale ; binaire : connexion puis fenêtre principale ; `make run`.

## Tâche 3 — Persistance Secret Service, relance, déconnexion

- Port `PortCoffre` (adaptateur `oo7` 0.6.0, Secret Service ou portail Flatpak) : le refresh token est
  conservé au trousseau (`application=tuxteams`), type `SecretRefresh` à `Debug` masqué et mémoire
  effacée (`zeroize`) ; jamais exposé à l'interface ni aux logs.
- Refresh grant (`grant_type=refresh_token`) avec rotation persistée ; `invalid_grant` renvoie à l'écran de
  connexion (cache conservé). Relance silencieuse au démarrage : la fenêtre principale s'affiche aussitôt en
  état « Reconnexion… » (squelettes, avatar neutre) ; délai maximal de 10 s (`DELAI_RELANCE`, horloge injectée),
  au-delà retour à l'écran de connexion avec le message réseau, secret conservé.
- Déconnexion depuis le menu de l'avatar (nom, e-mail, « Se déconnecter ») : secret supprimé, cache vidé
  (port `PortCache`, adaptateur à venir en tâche 6).
- Bandeau non bloquant « Session non persistée » si le trousseau est indisponible.
- Journalisation `tracing-subscriber` (`RUST_LOG`, défaut `warn`) ; test de non-fuite du refresh token.
- `make test-integration` : test du coffre contre un Secret Service de session.

