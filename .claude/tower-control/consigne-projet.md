Dépôt unique : issues, board et code dans `zaratan/chiro-dvr`. Tu travailles dans ton
worktree, sous `~/Projects/dvr-wt/`. Ton compte rendu s'écrit dans
`/Users/zaratan/Projects/dvr-wt/rapports/rapport-<N>.md`, jamais dans le worktree.

**Vérifications avant de rendre**

- `mise run check` vert (lint, format, types, tests).
- Si tu touches à `parallel.py`, `video.py`, `detect.py`, `track.py`, `stability.py` ou
  `median.py` : `uv run pytest -m slow`, puis la référence
  `uv run batdetect /Users/zaratan/Projects/dvr/in/video_092_original.mp4 -o out/ref`,
  avec le nombre de pistes et les temps de début, avant et après. La consigne de l'issue
  donne les chiffres de départ.
- Tout changement de détection ou de suivi se mesure aussi au banc, avant et après :
  `uv run batdetect-bench /Users/zaratan/Projects/dvr/in/video_092_original.mp4`
  (docs/08-banc-de-mesure.md).

**Mesures : une à la fois, verrouillées**

- Plusieurs agents mesurent sur la même machine. Toute commande lourde (`batdetect` ou
  `batdetect-bench` sur une vidéo entière, `ffprobe`/`ffmpeg` sur une vidéo entière, tout
  chronométrage, `pytest -m slow` et `pytest -m reference`) passe par le verrou commun, une
  commande par appel : `lockf /Users/zaratan/Projects/dvr-wt/mesure.lock <commande>`. Règle
  complète dans `/Users/zaratan/Projects/dvr-wt/MESURES.md`. Les séries durent plus longtemps :
  n'écourte rien pour compenser.
- Avant chaque traitement d'une vidéo entière, vérifie qu'il reste au moins 20 Go libres
  (`df -g .`) ; au-delà de 200 pistes, supprime les extraits produits et garde le CSV.
- Un réglage bon sur la 092 et au banc n'est pas un réglage : avant de le proposer, lance-le
  sur toutes les vidéos de `in/` sans défaut connu et donne le nombre de pistes de chacune.
- Un verdict à l'œil se consigne avec le temps de la piste (mm:ss.ii), jamais son numéro, qui
  change à chaque réglage.
- Les temps réels sont invalides si la machine s'est mise en veille : publie le temps CPU
  (`/usr/bin/time -l`) à côté.
- Un test fondé sur une vidéo fabriquée est « à confirmer au premier passage en CI » dans le
  compte rendu : ffmpeg 6 du runner ne reproduit pas tout ce que ffprobe 9 fait sur Mac.
- Le compte rendu ne s'écrit qu'à la toute fin, quand plus aucune mesure ne tourne en arrière-plan.

**Vidéos : lecture seule**

- `/Users/zaratan/Projects/dvr/in/` et `/Users/zaratan/dossier sans titre/img_0000/`
  contiennent les vidéos de l'utilisateur : ne rien y écrire, supprimer ni déplacer.
  Toute sortie va dans `out/` de ton worktree (hors git), par `-o`.
- `tests/fixtures/video_092_original_3m24-4m05.mp4` ne se réencode pas et ne se remplace pas.
- Aucune vidéo ajoutée au dépôt : il est public.

**Règles du dépôt** (détail dans `CLAUDE.md`)

- Zéro commentaire, zéro `noqa`, zéro `type: ignore`. Code en anglais, docs en français.
- Un module, une responsabilité, son fichier de tests. Pas de mock d'OpenCV ni de subprocess.
- Un changement de comportement met à jour `docs/` dans la même issue ; si les 14 pistes
  de la 092 changent, `docs/03-resultats-092.md` aussi.
- `docs/10-logique-de-detection.md` explique la logique étape par étape (outil, choix,
  limite). Toute issue qui change une étape de la détection ou du suivi met à jour sa
  section dans la même issue : l'outil, la raison du choix, la limite, et la ligne du
  tableau des hypothèses si elle change. Le compte rendu dit quelles sections ont bougé.
- Ce qui est ou n'est pas une chauve-souris ne se tranche pas dans le code : c'est une
  question pour la naturaliste, à faire remonter.
- La qualité de détection passe avant le temps de calcul ; 16 Go de mémoire est la limite.

**Issues de 8 points** : avant de coder, écris ton plan dans
`/Users/zaratan/Projects/dvr-wt/rapports/plan-<N>.md`, fais-le relire par `tech-architect`
et `lead-engineer-reviewer` en parallèle, puis arrête-toi : la tour le fait valider.
Ensuite, arrête-toi à la fin de chaque sous-phase du plan avec ses chiffres.
