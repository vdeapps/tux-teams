# TuxTeams — consignes du dépôt

Client Teams natif Linux en Rust + Slint. Langue du code, des commentaires et de la doc : français
(termes techniques sans équivalent en anglais : token, cache, hook).

## Architecture hexagonale (crates dans `crates/<nom>/`)

- `tuxteams-domaine` : règles métier pures. Sans I/O, sans tokio/reqwest/rusqlite/slint.
- `tuxteams-application` : cas d'usage et ports (`src/ports/`), interfaces de toute I/O.
- Adaptateurs : `tuxteams-teams`, `tuxteams-graph`, `tuxteams-auth`, `tuxteams-cache`. Ils implémentent les ports.
- `tuxteams-ui` : Slint, composants minces qui appellent un cas d'usage. Tokens dans `ui/theme.slint` (source unique).
- `tuxteams` : binaire, composition des couches.
- Dépendances : domaine ← application ← adaptateurs/ui ← binaire. Jamais l'inverse.
- Toute I/O (HTTP, fichiers, secrets, horloge, navigateur, notifications) passe par un port.

## Nommage

`principes.md` demande du `camelCase`, mais rustc (`non_snake_case`) et `clippy -D warnings` imposent le
`snake_case` : la configuration du langage prime. Méthodes `verbe_anglais_nom_francais`
(`fetch_conversations`, `send_message`), types `PascalCase`, constantes `MAJUSCULES_AVEC_UNDERSCORES`.
Tests : nom français qui énonce la règle (`refuse_un_corps_d_envoi_avec_champ_supplementaire`).

## Commandes

- `make test` : `cargo test --workspace`
- `make lint` : `cargo fmt --all -- --check` puis `cargo clippy --workspace --all-targets -- -D warnings`
- `make format` / `make format-check` / `make typecheck`
- `make test-integration` : tests qui exigent un Secret Service de session D-Bus (`cargo test -p tuxteams-auth --features integration --test coffre`), hors `make test`. Ils utilisent la collection `session` (en mémoire) et l'attribut `application=tuxteams-test` : aucun secret réel n'est touché.
- `make qualite` : test, lint, format-check, typecheck

## Règles

- TDD : test vu rouge pour la bonne raison, puis code minimal.
- `unsafe_code` interdit (`forbid`) ; exception : `tuxteams-ui` en `deny`, le code généré par Slint portant `#[allow(unsafe_code)]`, et son module `genere` autorise `clippy::unwrap_used` ; `unwrap` interdit hors tests (`clippy.toml`).
- Interdiction de copier du code tiers (purple-teams GPL-3.0, mautrix-teams AGPL-3.0, squads sans licence connue) : les
  sources servent de référence de format en lecture seule.
- Fixtures : procédure dans `docs/fixtures.md` (capture réelle anonymisée, jamais de secret ni de contenu réel).
- Journalisation sans secret : jamais de token, de contenu de message ni d'identifiant personnel dans les logs.
- Licences tierces : `THIRD_PARTY_LICENSES.md`.

## Lancer la connexion réelle (recette)

`make run` relance silencieusement la session si un refresh token est au trousseau ; sinon il ouvre
l'écran de connexion (device code Microsoft) puis la fenêtre principale avec les initiales du compte.

- Compte professionnel ou scolaire uniquement ; un compte personnel est refusé avec un message.
- Surcharges (variables d'environnement) : `TUXTEAMS_CLIENT_ID` (UUID), `TUXTEAMS_TENANT`
  (défaut `organizations`). Journaux : `RUST_LOG` (défaut `warn`), jamais de secret dans les logs.
- Persistance : le refresh token (seul secret conservé) est dans le trousseau du bureau (Secret Service,
  crate `oo7`), attributs `application=tuxteams`, `type=refresh_token`, `tenant=<tid>`.
  Vérifier : `secret-tool search application tuxteams` (paquet `libsecret-tools`) ; après « Se déconnecter »
  (menu de l'avatar) la commande ne renvoie plus rien.
- Sans trousseau, un bandeau « Session non persistée » s'affiche : la session ne survit pas au redémarrage.
- Session révoquée (`invalid_grant`) : retour à l'écran de connexion, cache conservé.
- À vérifier tôt : l'accès conditionnel du tenant. Une réponse `AADSTS53003` affiche « Votre
  organisation interdit la connexion par code » ; la webview PKCE (tâche 29) passe alors après la tâche 4.
- Les tests n'ouvrent jamais de connexion réelle (wiremock, doubles de ports).
