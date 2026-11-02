# Journal des changements

## Tâche 1 — Socle

- Workspace Rust (8 crates), qualité (`make test lint format-check typecheck qualite`).
- Tokens de design (`theme.slint`, clair/sombre) et test de contraste WCAG AA.
- Fenêtre Slint avec rail (Activité, Conversation, Calendrier, Paramètres), police Inter, icônes Fluent.
- Reprise visuelle : rail 76 px (icônes et libellés centrés, barre d'accent au bord gauche),
  barre supérieure (recherche, avatar), panneau de liste en chargement (Activité, Conversation),
  états vides par vue ; tests de disposition headless (`tests/disposition.rs`).
