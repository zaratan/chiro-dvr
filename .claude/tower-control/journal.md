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
