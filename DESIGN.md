# Direction de design — TuxTeams v1

Demande utilisateur (2026-11-02) : « une interface moderne mais proche de Teams, jolie et intuitive ».
Les tokens sont portés par `crates/tuxteams-ui/ui/theme.slint` (source unique, global `Theme`).

## Intention

Familier pour un utilisateur de Teams dès la première seconde (même structure, mêmes repères),
mais plus calme et plus net : moins de chrome, plus d'air, une hiérarchie typographique claire,
des transitions courtes. Natif Linux : respecte le thème clair/sombre du système, Wayland,
HiDPI, clavier.

## Structure (identique à Teams)

```
┌──┬──────────────────────────────────────────────────────────────┐
│  │  Barre de titre : ← →   [ Rechercher (Ctrl+K) ]       ● Avatar│
│R ├──────────────┬───────────────────────────────────────────────┤
│a │ Liste        │ En-tête : nom · présence · onglets             │
│i │ (Chats /     │ [Conversation] [Fichiers] [Notes]       ⟳  ⋯   │
│l │  Équipes)    ├───────────────────────────────────────────────┤
│  │ filtres      │ Fil de messages (bulles, regroupées par auteur │
│76│ 320 px       │ et par jour, séparateur « Nouveaux messages ») │
│px│ redim.       ├───────────────────────────────────────────────┤
│  │              │ Zone de saisie (mise en forme, @, emoji, 📎)   │
└──┴──────────────┴───────────────────────────────────────────────┘
```

- Rail gauche 76 px (68 px à l'origine, porté à 76 pour que le libellé « Conversation » tienne
  entier en 10 px) : Activité, Conversation, Calendrier (icône + libellé court, pastille de
  non-lus), en bas : Paramètres. Chaque élément : bouton 64 × 56 centré, icône 24 px centrée,
  libellé 10 px centré dessous. Élément actif : icône Filled, fond `accent-doux` arrondi 6 px,
  barre verticale d'accent 3 px collée au bord gauche du rail (hors du bouton), libellé en
  `texte-rail-actif`. Survol : `surface-survol`. Séparateur 1 px `bordure` à droite.
- Barre supérieure 48 px (à droite du rail) : champ « Rechercher (Ctrl+K) » centré (480 px max,
  loupe, rayon 4 px, fond `fond`), avatar rond 32 px à droite (initiales + pastille de présence ;
  « ?? » hors ligne tant que la connexion n'existe pas).
- Contenu sans sélection : état vide centré propre à la vue (icône Fluent 48 px en `accent` sur
  disque `accent-doux` 96 px, titre 18 px 600, phrase 14 px `texte-secondaire`).
- Panneau de liste en chargement : titre de vue 20 px 700, actions discrètes (filtrer, nouvelle
  conversation), 7 lignes squelette (avatar rond + 2 barres arrondies `surface-survol`).
- Panneau de liste 320 px, redimensionnable (240–480), repliable.
- Calendrier : le panneau de liste disparaît, grille pleine largeur.
- Fenêtre étroite (< 900 px) : panneau de liste et contenu en navigation empilée.

## Tokens (Fluent 2 adapté)

| Token | Clair | Sombre | Usage |
|---|---|---|---|
| `accent` | #5B5FC7 | #7F85F5 | marque Teams, éléments actifs, boutons primaires |
| `accent-doux` | #EBEBF8 | #2F2F4A | sélection, rail actif, bulles « moi » |
| `texte-rail-actif` | #242424 | #FFFFFF | libellé de l'élément actif du rail, sur `accent-doux` (l'`accent` seul y donnait 4.04:1 en sombre, refusé) ; icône et barre restent en `accent` (non textuel, 3:1) |
| `fond` | #F5F5F5 | #1F1F1F | arrière-plan de l'application |
| `surface` | #FFFFFF | #292929 | panneaux, cartes, bulles des autres |
| `surface-survol` | #F0F0F0 | #333333 | survol |
| `texte` | #242424 | #FFFFFF | texte principal |
| `texte-secondaire` | #616161 | #ADADAD | horodatage, métadonnées |
| `bordure` | #E0E0E0 | #3D3D3D | séparateurs 1 px |
| `texte-sur-accent` | #FFFFFF (5.38:1 sur #5B5FC7) | #1F1F1F (5.16:1 sur #7F85F5) | texte des boutons primaires et éléments pleins accent (blanc sur #7F85F5 = 3.19:1, refusé) |
| `danger` | #C4314B | #F1707B | suppression, erreurs |
| `presence-disponible / occupe / absent / ndp` | #6BB700 / #C50F1F / #FFAA44 / #C50F1F (barre) | idem | pastilles de présence |
| `mention` | #EBEBF8 + barre accent | #2F2F4A | message qui me mentionne |

- Contraste texte ≥ 4.5:1 (AA) dans les deux thèmes ; thème contraste élevé dérivé.
- Rayons : 4 px (contrôles), 8 px (bulles, cartes), 12 px (dialogues). Ombres : 2 niveaux seulement.
- Espacements : échelle 4 px (4, 8, 12, 16, 24, 32).

## Typographie

- Police : Segoe UI n'est pas distribuable sous Linux → **Inter** (licence OFL-1.1, embarquée ;
  voir `THIRD_PARTY_LICENSES.md`), repli Cantarell / Ubuntu / sans-serif.
- Échelle : 12 (méta), 14 (corps), 16 (noms, titres de liste), 20 (titres de vue), 28 (calendrier, mois).
- Poids : 400 corps, 600 noms et non-lus, 700 titres.

## Iconographie

- **Fluent UI System Icons** (dépôt microsoft/fluentui-system-icons ; licence MIT, voir
  `THIRD_PARTY_LICENSES.md`), variantes Regular / Filled (Filled = élément actif), SVG embarqués.
- Avatars : photo, sinon initiales sur couleur dérivée du nom (palette de 10 teintes AA).

## Composants clés

- **Ligne de conversation** : avatar 32 px + pastille présence, nom (600 si non lu), extrait
  1 ligne, heure relative ; pastille non-lus ; menu contextuel (Marquer non lu, Épingler, Masquer).
- **Message** : regroupement des messages consécutifs d'un même auteur (< 5 min) ; bulle « moi »
  alignée à droite en `accent-doux`, autres à gauche en `surface` ; réactions en pastilles sous
  la bulle ; barre d'actions au survol (réagir, répondre, ⋯) ; « modifié » discret.
- **Canal** : publications en cartes avec fil de réponses repliable (« 3 réponses de A, B »).
- **Zone de saisie** : une ligne qui grandit jusqu'à 8 lignes ; barre de mise en forme
  repliable ; @ ouvre l'autocomplétion ; glisser-déposer de fichiers.
- **Activité** : liste à icône de type (mention @, réaction ♥, réponse ↩), filtres en puces.
- **Calendrier** : grille horaire 30 min, ligne « maintenant » rouge, événements en blocs
  arrondis à barre latérale de couleur (réunion Teams = accent), bouton « Rejoindre » visible
  10 min avant.
- **États** : squelettes de chargement (pas de spinner plein écran), états vides illustrés avec
  une action, erreurs en bandeau non bloquant avec « Réessayer », hors ligne signalé en haut.

## Interaction et intuitivité

- Raccourcis : Ctrl+K recherche/palette, Ctrl+1/2/3 rail, Ctrl+N nouveau chat, Ctrl+Maj+X
  mise en forme, Échap ferme, Alt+↑/↓ conversation précédente/suivante, Ctrl+E éditer le dernier message.
- Navigation entièrement au clavier, focus visible (anneau 2 px accent).
- Animations 120–200 ms (ease-out), désactivées si « réduire les animations » du système.
- Densité : Confortable (défaut) / Compacte.
- Notifications bureau groupées par conversation, clic → ouvre la conversation.

## Critères vérifiables

1. Tous les écrans existent en clair et sombre, basculement suit le système.
2. Contraste AA vérifié sur les paires de tokens (test unitaire sur `theme`).
3. Toutes les actions du fil sont atteignables au clavier (test UI headless).
4. Captures de référence par écran dans `docs/captures/` comparées en revue.
