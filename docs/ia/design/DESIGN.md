# DESIGN.md — TuxTeams

Référence de design de l'interface, lue par l'étape `ui` de l'orchestrateur (tokens, composants
génériques, espacements, typographie, accessibilité, états). Elle fait foi sur toute autre
description (`design.md` du run, ancienne version racine).

Sources : maquettes fournies par l'utilisateur le 2026-11-02, 1440×900, thème clair :

| Maquette | Fichier | Écran |
|---|---|---|
| Conversation A — Chat clair | `maquettes/conversation-chat-clair.png` | Conversation 1:1 ouverte, panneau contextuel |
| Calendrier A — Semaine | `maquettes/calendrier-semaine.png` | Vue Semaine de travail, détail d'une réunion |
| Calendrier B — Agenda du jour | `maquettes/calendrier-agenda-du-jour.png` | Vue Agenda (tableau de bord du jour) |

Les valeurs ci-dessous ont été relevées au pixel sur ces maquettes (couleurs exactes,
séparateurs mesurés). Les tailles de police et rayons sont estimés à ±1 px. Le thème sombre et
les écrans non maquettés (Activité, canal, Fichiers, Notes, connexion, Paramètres) sont
**dérivés** des mêmes règles ; ils sont marqués « dérivé ».

## 1. Principes

1. **Familier** : structure Teams (rail, liste, contenu, panneau contextuel). Un utilisateur Teams
   doit tout trouver sans apprentissage.
2. **Calme** : fonds blancs et gris-lavande très clairs, une seule couleur d'accent (indigo),
   couleurs de catégorie en aplats pastel avec texte foncé de la même teinte.
3. **Aéré et arrondi** : rayons 8–16 px, cartes plutôt que traits, séparateurs 1 px seulement
   entre les grandes zones.
4. **Orienté action** : l'action principale de chaque écran est un bouton accent plein
   (« Nouvelle réunion », « Rejoindre », envoyer) ; les secondaires sont en contour.
5. **Information contextuelle à droite** : chaque vue de détail a un panneau latéral (prochaine
   réunion et fichiers d'une conversation, détail d'un événement).

## 2. Tokens

### 2.1 Couleurs — thème clair (relevées)

| Token | Valeur | Usage observé |
|---|---|---|
| `accent` | `#5B5FC7` | boutons primaires, bulle « moi », pastille non lus, jour courant, barre de semaine |
| `accent-fort` | `#4E52A5` | contour de l'événement sélectionné, survol/pression du primaire |
| `accent-texte` | `#4B4FB8` | liens, heure de réunion, « Ouvrir », « 3 réponses dans le fil » (6.75:1 sur blanc) |
| `accent-doux` | `#ECECFA` | élément actif du rail, ligne sélectionnée, réaction à moi, carte « prochaine réunion », élément en cours du programme |
| `accent-pale` | `#C9CAF0` | barres non sélectionnées du graphe « Ma semaine » (décoratif, jamais pour du texte) |
| `fond` | `#F4F5F9` | fond du fil de messages, fond général de la vue Agenda, champs de saisie secondaires |
| `surface` | `#FFFFFF` | rail, liste, barre supérieure, cartes, bulles des autres, panneaux latéraux |
| `surface-calme` | `#FAFAFE` | colonnes de jours (heures hors programme) |
| `bordure` | `#E6E7EE` | séparateurs de zones, contours de cartes, bulles, puces, lignes de la grille |
| `bordure-pointillee` | `#C9CBD6` (dérivé) | « Temps de concentration » (contour pointillé) |
| `texte` | `#1B1C22` | titres, noms, corps de message (16.99:1) |
| `texte-secondaire` | `#5F6270` | aperçus, heures, sous-titres, sections en capitales, libellés du rail (6.06:1 sur blanc, 5.56:1 sur `fond`) |
| `texte-indice` | `#6E7080` | placeholders. **Écart corrigé** : la maquette utilise `#848487` (3.73:1, sous AA) ; remplacé par `#6E7080` (4.89:1) |
| `texte-sur-accent` | `#FFFFFF` | texte sur `accent` (5.38:1) et sur `heros` |
| `puce-active-fond` / `puce-active-texte` | `#1B1C22` / `#FFFFFF` | filtre sélectionné (« Tout ») |
| `heros` | `#23264F` | carte « prochaine réunion » de l'Agenda (bleu nuit) |
| `heros-signal` | `#7EE29A` | point « Commence dans 12 min » sur `heros` |
| `danger` | `#C4314B` | ligne « maintenant », pastilles de notification du rail, présence occupé |
| `danger-doux` | `#F1CDD3` | halo des pastilles de notification |

### 2.2 Couleurs de catégorie (événements, puces de statut, tuiles de fichiers)

Toujours un aplat pastel + texte foncé de la même teinte (contraste ≥ 6:1) :

| Catégorie | Fond | Texte | Usage |
|---|---|---|---|
| `succes` | `#DDF1E8` | `#175C42` | événements récurrents (Daily), « Accepté », bouton Accepter actif, tuile XLS |
| `alerte` | `#FCE7D6` | `#8A3F0B` | événements projet, « Préparer », tuile PDF (`#FDEBDD`) |
| `violet` | `#E4E5FB` | `#3E419B` | réunions de travail, ateliers, tuile DOC (`#E8EEFD` / texte `#2B4BA8` dérivé) |
| `neutre` | `#F1F2F6` | `#5F6270` | temps de concentration (avec contour pointillé), éléments terminés |

Couleur d'un événement : catégorie Outlook de l'événement si présente (table de correspondance
couleurs Outlook → ces 4 familles, dérivé), sinon `violet` ; `showAs = free/focus` → `neutre`.

### 2.3 Présence

| État | Couleur | Forme |
|---|---|---|
| Disponible | `#13A10E` | pastille pleine |
| Occupé / En réunion / Ne pas déranger | `#C4314B` | pleine (NPD : avec trait blanc, dérivé) |
| Absent / De retour bientôt | `#C19C00` | pleine |
| Hors ligne / inconnu | `#8A8D99` (dérivé) | pleine grise |

Pastille 10 px, anneau blanc 2 px (`surface`), en bas à droite de l'avatar.

### 2.4 Avatars

- Forme : **carré arrondi** (rayon 10 px pour 40 px, 8 px pour 32 px) pour les personnes et groupes
  dans les listes ; **cercle** pour l'avatar de l'utilisateur (barre supérieure) et les piles de
  participants.
- Sans photo : initiales (2 lettres, 600, 13 px pour 40 px) sur aplat pastel, texte foncé de la
  même teinte. Teinte choisie de façon stable par hachage de l'identifiant parmi : vert
  (`#E3F3EC`/`#175C42`), rose (`#F6E5F1`/`#8A2C6B`), pêche (`#FDEBDD`/`#8A3F0B`), lavande
  (`#E8E9FB`/`#3E419B`), gris (`#EEEFF3`/`#4A4D5A`) ; textes dérivés à ≥ 4.5:1.
- Pile de participants : cercles 24 px chevauchés de 6 px, anneau blanc, « +3 » sur `accent`.

### 2.5 Thème sombre (dérivé, non maquetté)

| Token | Valeur | | Token | Valeur |
|---|---|---|---|---|
| `accent` | `#7F85F5` | | `fond` | `#17181D` |
| `accent-texte` | `#A3A8FF` | | `surface` | `#1F2026` |
| `accent-doux` | `#2D2F4D` | | `surface-calme` | `#1B1C22` |
| `texte` | `#F2F3F7` | | `bordure` | `#33353F` |
| `texte-secondaire` | `#A7AAB8` | | `texte-indice` | `#8C8FA0` |
| `texte-sur-accent` | `#1B1C22` (sur `#7F85F5`) | | `heros` | `#2D3170` |

Catégories en sombre : fond = teinte à 18 % d'opacité sur `surface`, texte = teinte claire
(`#8FD9B6`, `#F5B98A`, `#B9BCFF`). Toute paire texte/fond est testée ≥ 4.5:1 dans les deux
thèmes (test `contraste.rs`).

### 2.6 Typographie

- Police : **Inter** (embarquée, OFL) ; chiffres tabulaires pour les heures (`font-feature
  tnum` si disponible dans Slint, sinon accepté).
- Échelle :

| Rôle | Taille / graisse | Exemple |
|---|---|---|
| `titre-page` | 32 / 700, interlettrage −0.5 | « Bonjour Vincent, 4 réunions aujourd'hui » |
| `titre-vue` | 22 / 700 | « Conversation », « 2 – 6 novembre 2026 » (20/700) |
| `titre-carte` | 20–24 / 700 | « Revue pipeline 2.4 », « Atelier architecture » (héros) |
| `titre-section` | 16–17 / 600 | « Programme », « Ma semaine » |
| `nom` | 15 / 600 | noms dans la liste, nom de l'en-tête (16/600) |
| `corps` | 15 / 400, interligne 1.45 | messages |
| `secondaire` | 13 / 400 | aperçus, sous-titres, méta |
| `meta` | 12 / 400–500 | heures, tailles de fichiers, horaires d'événements |
| `surtitre` | 11–13 / 600, CAPITALES, interlettrage +0.6 | « ÉPINGLÉES », « PROCHAINE RÉUNION », « LUNDI 2 NOVEMBRE 2026 » |
| `rail` | 11 / 500 (actif 600) | libellés du rail |

### 2.7 Espacements, rayons, ombres

- Échelle 4 px : 4, 8, 12, 16, 20, 24, 32. Marges internes de zone : 20 px (liste, panneaux),
  24–32 px (fil, vue Agenda).
- Rayons : 6 (puces d'événements, petits boutons), 8 (boutons, champs, tuiles de fichier),
  10 (lignes de liste, avatars carrés, cartes de pièce jointe), 12 (bulles, zone de saisie,
  cartes), 16 (grandes cartes de l'Agenda, héros), 999 (puces de filtre, pastilles, réactions).
- Ombres : uniquement sur le bouton « Nouvelle réunion » (accent, ombre colorée douce
  `0 4 12 accent@25%`) et sur les menus/popovers (`0 8 24 #1B1C22@12%`). Tout le reste est plat
  avec `bordure`.

### 2.8 Iconographie

Fluent UI System Icons (MIT), trait **Regular** 20 px dans les boutons et la barre d'outils,
24 px dans le rail ; **Filled** pour l'élément actif du rail. Couleur = `texte-secondaire` par
défaut, `accent` si actif, `texte-sur-accent` dans un bouton primaire.

## 3. Mise en page (mesurée à 1440×900)

```
Conversation                                                     Calendrier — Semaine
┌────┬──────────────┬─────────────────────────┬───────────┐   ┌────┬───────────┬─────────────────────────┬──────────┐
│Rail│ Barre supérieure (67 px, recherche centrée ≤ 584)    │   │Rail│ Panneau    │ Barre d'outils (70 px)  │ Détail   │
│ 80 ├──────────────┼─────────────────────────┼───────────┤   │ 80 │ calendrier ├─────────────────────────┤ réunion  │
│    │ Liste 300    │ En-tête conv. (68 px)   │ Contexte  │   │    │ 240        │ Grille horaire          │ 280      │
│    │              ├─────────────────────────┤ 260       │   │    │            │                         │          │
│    │              │ Fil (fond)              │           │   │    │            │                         │          │
│    │              │ Saisie                  │           │   │    │            │                         │          │
└────┴──────────────┴─────────────────────────┴───────────┘   └────┴───────────┴─────────────────────────┴──────────┘
```

- **Rail** 80 px (séparateur à x = 79). La vue Agenda montre un rail **compact** de 72 px
  (icônes seules, avatar utilisateur en bas) : variante « densité compacte » (Paramètres).
- **Barre supérieure** : présente dans Conversation et Activité (dérivé) ; le Calendrier la
  remplace par sa propre barre d'outils ; la vue Agenda n'a pas de barre (en-tête de page).
- **Conversation** : liste 300 px · contenu flexible · panneau contextuel 260 px (repliable,
  masqué sous 1280 px de large).
- **Calendrier Semaine** : panneau 240 px · grille flexible · détail 280 px (visible quand un
  événement est sélectionné, sinon la grille s'étend).
- **Calendrier Agenda** : contenu centré, colonne principale flexible + colonne droite 300 px,
  marges 36 px.
- **Fenêtre étroite** (< 1100 px, dérivé) : panneau contextuel/détail en surcouche ;
  < 900 px : navigation empilée liste → contenu.

## 4. Composants

### 4.1 Rail
- Haut : tuile de marque 40×40, rayon 10, fond `accent`, « T » blanc 700 (marque TuxTeams).
- Éléments Activité, Conversation, Calendrier : bouton 64×56, rayon 10, icône 24 au-dessus du
  libellé `rail`. Actif : fond `accent-doux`, icône Filled `accent`, libellé `accent` 600.
- Pastille de compteur : cercle 16 px `danger`, chiffre blanc 10/700, anneau `surface` 2 px, en
  haut à droite de l'icône (Activité 3, Conversation 5).
- Bas : Paramètres (même style).
- Survol : fond `fond`. Focus clavier : anneau 2 px `accent`, décalé de 2 px.

### 4.2 Barre supérieure
- Hauteur 67, fond `surface`, séparateur bas.
- Recherche : 584×42, rayon 10, fond `fond`, contour `bordure`, icône loupe, placeholder
  « Rechercher des messages, personnes, fichiers… » en `texte-indice`, raccourci « Ctrl K » dans
  une étiquette `surface` contour `bordure` rayon 6 à droite.
- Droite : pilule de présence « ● Disponible » (contour `bordure`, rayon 8, 13/500) qui ouvre le
  sélecteur de présence ; avatar utilisateur rond 40 + pastille de présence (menu compte).

### 4.3 Liste des conversations
- En-tête : « Conversation » `titre-vue` ; bouton icône filtre (ghost) ; bouton carré 36×36
  rayon 8 `accent` « + » (nouvelle conversation).
- Puces de filtre : Tout · Non lus · Personnes · Groupes (+ Équipes, dérivé). Hauteur 30,
  rayon 999, contour `bordure`, 13/500 ; active = `puce-active-*`. Retour à la ligne autorisé.
- Sections `surtitre` : ÉPINGLÉES, RÉCENTES (+ ÉQUIPES pour les canaux, dérivé).
- Ligne (hauteur 60, marge 10, rayon 10) : avatar 40 + présence · nom `nom` · aperçu
  `secondaire` sur 1 ligne avec ellipse · heure `meta` à droite · pastille non lus (cercle 18
  `accent`, chiffre blanc 11/700) sous l'heure. Non lu : nom 700 et aperçu `texte`.
  Sélectionnée : fond `accent-doux`. Aperçu « Vous : … » pour mes messages ; « X est en train
  d'écrire… » si l'information existe (v2).
- Dates relatives : heure (aujourd'hui), « Hier », jour abrégé (« Lun. ») dans la semaine,
  sinon « 28 oct. ».

### 4.4 En-tête de conversation
- Hauteur 68, fond `surface`, séparateur bas, marges 24.
- Avatar 40 · nom 16/600 · sous-titre `secondaire` « Disponible · Ingénieure DevOps »
  (présence + poste, profil) ; groupe : nombre de participants.
- Onglets segmentés **Discussion · Fichiers · Notes** : conteneur rayon 10 fond `fond`, onglet
  actif fond `surface` 600 avec ombre légère, inactifs `texte-secondaire`.
- Actions à droite : « Appel vidéo » (primaire), appel audio (icône contour 40×40),
  « … » (icône contour). **v1** : appels hors périmètre → ces deux boutons sont masqués ; « … »
  ouvre le menu (Marquer non lu, Épingler, Masquer, Ouvrir dans Teams web).

### 4.5 Fil de messages
- Fond `fond`, colonne de lecture max 740 px, marges 32.
- Séparateur de jour : ligne `bordure` + libellé centré `secondaire` (« Aujourd'hui », « Hier »,
  « lundi 26 octobre »).
- **Message des autres** : avatar 32 (carré arrondi) · ligne d'en-tête « **Nom** · 09:12 »
  (13/600 + `meta`) · bulle `surface`, contour `bordure`, rayon 12 (coin haut-gauche 4),
  padding 12×16, max 600 px, `corps`. Messages consécutifs du même auteur < 5 min : sans
  avatar ni en-tête, espacement 4.
- **Mes messages** : alignés à droite, sans avatar ; méta « 09:20 · Lu » au-dessus (`meta`) ;
  bulle `accent`, texte `texte-sur-accent`, rayon 12 (coin haut-droit 4). « Lu » d'après
  l'horizon de lecture des autres membres si disponible, sinon omis.
- **Pièce jointe** : carte `surface` contour `bordure` rayon 10, 360×56 : tuile de type 36×36
  rayon 8 (PDF `alerte`, XLS `succes`, DOC `violet`, autre `neutre`) avec extension 10/700 ;
  nom 15/500 ; taille `meta` ; action « Ouvrir » `accent-texte` 600 à droite.
- **Réactions** : pilules 28 px rayon 999 contour `bordure`, « emoji · n » 13/500 ; la mienne :
  fond `accent-doux` contour `accent` texte `accent-texte` ; bouton « + » rond contour pour
  ajouter.
- **Résumé de fil** (canal ou réponses) : mini-pile d'avatars 18 px + « 3 réponses dans le fil »
  `accent-texte` 600.
- Barre d'actions au survol (dérivé) : réagir, répondre, « … » ; flottante en haut à droite de
  la bulle, `surface` ombre menu.
- Indicateur de saisie (« ••• Camille est en train d'écrire… ») : v2 (hors périmètre v1).

### 4.6 Zone de saisie
- Carte `surface` contour `bordure` rayon 12, marges 32 depuis les bords du fil, hauteur min 110,
  croît jusqu'à 8 lignes.
- Placeholder « Écrire un message à Camille… » (`texte-indice`).
- Barre d'outils bas-gauche : mise en forme (T), joindre (trombone), emoji, mention (@) — icônes
  20 `texte-secondaire`, cible 32×32.
- Bas-droite : indice « Entrée pour envoyer » `meta` + bouton envoyer 38×38 rayon 8 `accent`
  (désactivé : `accent-pale`, cas vide).
- Focus : contour `accent` 1.5 px.

### 4.7 Panneau contextuel (conversation)
- Fond `surface`, séparateur gauche, padding 20, sections `surtitre` espacées de 24.
- **PROCHAINE RÉUNION** : carte `accent-doux` rayon 12 : titre 16/600, horaire `accent-texte`,
  bouton « Rejoindre » primaire compact (ouvre `joinUrl`). Réunion commune avec ce contact,
  sinon section masquée.
- **FICHIERS PARTAGÉS** : 3 derniers fichiers (tuile 32 + nom 13/500 + date `meta`), lien
  « Voir tous les fichiers » → onglet Fichiers.
- **LIENS ÉPINGLÉS** : liens `accent-texte` 13/500. v1 : derniers liens partagés extraits des
  messages (pas d'API d'épinglage) — libellé « LIENS PARTAGÉS ».

### 4.8 Calendrier — panneau gauche
- Bouton « + Nouvelle réunion » pleine largeur 200×44, rayon 10, `accent`, ombre colorée.
- Mini-calendrier : titre mois 15/600 + flèches ; jours 13 px, cellule 28 ; semaine affichée
  sur fond `accent-doux` rayon 6 ; jour courant cercle `accent` texte blanc ; jours hors mois
  `texte-secondaire` à 60 %.
- **MES CALENDRIERS** : liste des calendriers (15/400) avec case de couleur 12 px de la catégorie
  (dérivé : la maquette ne montre pas la puce ; à afficher pour l'accessibilité) ;
  cocher/décocher masque les événements.
- Carte bas « Cette semaine » (`fond`, rayon 12) : « **11 h 30** en réunion » (22/700 + 13),
  barre de progression 6 px (`accent` sur `bordure`), ligne `meta` « 8 h de temps de
  concentration réservé ».

### 4.9 Calendrier — barre d'outils et grille
- Barre : « Aujourd'hui » (bouton contour), ‹ › (icônes), titre de plage 20/700 ; à droite
  sélecteur segmenté **Jour · Semaine de travail · Semaine · Mois** (style §4.4) ;
  « Rejoindre avec un ID » (contour, icône caméra) → v2 (masqué en v1).
- En-têtes de jours : « LUN. » `surtitre` + numéro 17/600 ; aujourd'hui = numéro dans un cercle
  28 `accent` blanc.
- Colonne horaire 56 px, libellés `meta` alignés en haut de l'heure ; 1 h = 64 px ; lignes
  `bordure`.
- Fond : `surface-calme` ; plage hors heures de travail : `#E6E7EE` à 50 % (dérivé de la
  maquette : zone grisée).
- **Événement** : rayon 6, marge 3 dans la colonne, padding 6×8, titre 13/600 couleur texte de
  la catégorie, horaire 11 `meta` même teinte ; fond pastel de la catégorie. Chevauchement :
  colonnes côte à côte.
- **Temps de concentration** : fond `neutre`, contour 1 px pointillé `bordure-pointillee`.
- **Sélectionné** : fond `surface`, contour 2 px `accent-fort`, rayon 8, titre `accent-fort`.
- **Maintenant** : ligne 2 px `danger` sur la colonne du jour + point 8 px à gauche.

### 4.10 Calendrier — détail de réunion
- Étiquette de calendrier (« Professionnel ») : puce `accent-doux` texte `accent-texte` 11/600.
- Titre 22/700 ; date/horaire `secondaire` 15.
- « Rejoindre la réunion » pleine largeur 48 px, `accent`, icône caméra (ouvre `joinUrl`).
- Réponse : 3 boutons segmentés contour **Accepter · Provisoire · Refuser** ; réponse
  courante : fond de la catégorie correspondante (Accepter `succes`, Provisoire `alerte`,
  Refuser `danger` doux), texte de la teinte.
- Lignes d'info icône 18 + texte 15 : lieu, « Organisé par … », rappel.
- **PARTICIPANTS · n** : avatar 28 + nom 15 + statut à droite (Accepté `succes` texte,
  En attente `texte-secondaire`, Refusé `danger`).
- Pied : « Ouvrir la conversation de la réunion » (bouton contour, icône bulle) → chat de la
  réunion.

### 4.11 Calendrier — vue Agenda (du jour)
- En-tête de page : surtitre date (« LUNDI 2 NOVEMBRE 2026 »), `titre-page`
  « Bonjour {prénom}, {n} réunions aujourd'hui » ; à droite sélecteur **Agenda · Semaine ·
  Mois** + « Nouvelle réunion » primaire.
- **Héros** (prochaine réunion du jour) : carte `heros` rayon 16, padding 28 :
  - puce « ● Commence dans 12 min » (fond blanc 10 %, point `heros-signal`, 13/600 blanc) ;
  - titre 24/700 blanc ; méta 14 blanc 80 % (horaire · lieu · organisateur) ;
  - pile de participants + « 5 acceptés · 1 en attente » ;
  - actions : « Rejoindre » (fond blanc, texte `heros`, 260×48), « Conversation » (contour
    blanc 30 %) ; « Tester micro » → v2 (masqué en v1).
- **Programme** : carte `surface` rayon 16 ; frise verticale : heure 13 (`meta`, en cours 600
  `danger`), point 10 px sur une ligne 2 px `bordure` ; marqueur « Maintenant » `danger`.
  Éléments : terminé (titre barré `texte-secondaire` + « Terminé · 30 min »), en cours/à venir
  proche (carte `accent-doux` + bouton « Rejoindre » compact), accepté (carte `fond` + puce
  « Accepté » `succes`), concentration (carte contour pointillé + « Statut « Ne pas déranger » »).
- Colonne droite (cartes `surface` rayon 16, padding 20) :
  - **Ma semaine** : histogramme 5 barres (heures en réunion par jour), jour courant `accent`,
    autres `accent-pale` ; libellés `meta` (jour courant 600) ; phrase de synthèse
    (« Mercredi est votre journée la plus libre ») ; lien « Ouvrir » → vue Semaine.
  - **Trouver un créneau** : champ participants (icône personne), surtitre « DISPONIBLES TOUS
    LES DEUX », 3 créneaux en boutons contour (premier mis en avant `accent-doux` + contour
    `accent`), action « Proposer » → nouvelle réunion préremplie.
  - **Demain** : liste heure `meta` + titre 15 ; puce « Préparer » `alerte` si un document est
    joint ou si je suis organisateur (règle dérivée).

### 4.12 Boutons et contrôles (génériques)

| Variante | Style |
|---|---|
| Primaire | fond `accent`, texte `texte-sur-accent` 14–15/600, rayon 8–10, hauteur 36/44/48 |
| Contour | fond `surface`, contour `bordure`, texte `texte` 14/500 |
| Fantôme (icône) | sans fond, icône `texte-secondaire`, survol `fond` |
| Segmenté | conteneur `fond` rayon 10, élément actif `surface` 600 |
| Lien | `accent-texte` 600, souligné au survol |
| Puce de filtre | rayon 999, contour `bordure` ; active `puce-active-*` |
| Puce de statut | rayon 6, 11–12/600, fond/texte de catégorie |

États : survol (fond assombri de 6 % ou `fond`), pressé (`accent-fort` pour le primaire),
désactivé (opacité 45 %, curseur interdit), focus clavier (anneau 2 px `accent` décalé de 2 px,
jamais supprimé).

## 5. États

- **Chargement** : squelettes à la forme du contenu (lignes de liste, bulles, blocs d'événements)
  en `fond` sur `surface` / `bordure` sur `fond`, pulsation 1.2 s (désactivée si réduction des
  animations). Jamais de spinner plein écran.
- **Vide** (dérivé, conforme à la tâche 1) : icône 48 dans un disque 96 `accent-doux`, titre
  18/600, phrase `secondaire`, action principale si pertinente.
- **Erreur / hors ligne** : bandeau fin en haut de la zone concernée (fond `alerte`, texte
  `alerte`, action « Réessayer ») ; hors ligne global sous la barre supérieure.
- **Connexion** (dérivé, tâche 2) : carte centrée `surface` rayon 16, code en 32/700 chasse
  fixe, bouton primaire pleine largeur.

## 6. Interaction et accessibilité

- Raccourcis : Ctrl+K recherche, Ctrl+1/2/3 rail, Ctrl+N nouvelle conversation, Ctrl+E éditer
  le dernier message, Alt+↑/↓ conversation précédente/suivante, Échap ferme panneau/menu,
  Entrée envoie / Maj+Entrée nouvelle ligne.
- Tout élément interactif : rôle et libellé accessibles, ordre de tabulation logique
  (rail → barre → liste → contenu → panneau), focus visible.
- Contraste AA (4.5:1 texte, 3:1 éléments graphiques) vérifié par test dans les deux thèmes ;
  aucune information portée par la couleur seule (présence : libellé dans l'infobulle et le
  sous-titre ; statuts : texte).
- Animations 120–200 ms ease-out (survol, ouverture de panneau, apparition des messages) ;
  respect de la réduction des animations du système.

## 7. Correspondance avec le périmètre v1

| Élément des maquettes | Statut |
|---|---|
| Rail, recherche, liste, filtres, sections, en-tête, onglets, fil, pièces jointes, réactions, résumé de fil, saisie | v1 (tâches 6–12, 15–16, 23) |
| Panneau contextuel : prochaine réunion, fichiers partagés | v1 (ajout au plan, après tâches 23–24) |
| Liens « épinglés » | v1 sous forme « liens partagés » extraits des messages ; épinglage réel v2 |
| Lu (accusé de lecture) | v1 si l'horizon de lecture des membres est disponible, sinon omis |
| Indicateur de saisie | v2 |
| Appel vidéo / audio, Tester micro, Rejoindre avec un ID | v2 (masqués en v1 ; « Rejoindre » ouvre `joinUrl` dans le navigateur) |
| Calendrier Semaine de travail, Semaine, Jour, détail, réponses, participants | v1 (tâches 24–25) |
| Vue Mois | v1 ajoutée (plan à amender) |
| Vue Agenda du jour (héros, programme, Ma semaine, Demain) | v1 ajoutée (calculs locaux sur `calendarView`, plan à amender) |
| Trouver un créneau | v1 ajoutée si `findMeetingTimes` est accessible avec les jetons Teams (à vérifier), sinon v2 |
| Statistiques « Cette semaine » | v1 ajoutée (calcul local) |
| Rail compact (vue Agenda) | densité compacte (Paramètres), tâche 28 |
| Thème sombre, Activité, canaux, Fichiers, Notes, Paramètres | dérivés de ce document |

## 8. Écarts avec l'interface déjà livrée (tâches 1–2)

À corriger lors d'une tâche d'alignement visuel (avant la tâche 6) : rail 76 → 80 px avec tuile
de marque et pastilles ; libellés rail 10 → 11 px ; barre supérieure 48 → 67 px, recherche
584×42 rayon 10, pilule de présence ; liste 320 → 300 px avec puces de filtre et sections ;
palette : `fond` `#F5F5F5` → `#F4F5F9`, `texte` `#242424` → `#1B1C22`, `texte-secondaire`
`#616161` → `#5F6270`, `bordure` `#E0E0E0` → `#E6E7EE`, `accent-doux` `#EBEBF8` → `#ECECFA` ;
avatars carrés arrondis ; écran de connexion : rayon de carte 16.
