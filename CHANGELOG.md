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
