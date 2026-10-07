# Lot 2 : consignes par issue (valeurs de départ mesurées sur `main` e2bd6e4, 7 octobre 2026)

Quatre worktrees sous `~/Projects/dvr-wt/` : `cli` (#4, puis #38, puis #3), `rendu` (#37 : plan
d'abord), `decodage` (#34, puis #36), `ci` (#29, puis #30 : plan d'abord). Rapports et plans :
`~/Projects/dvr-wt/rapports/`. Mesures lourdes sous verrou (`MESURES.md`).
Départ : `mise run check` vert (423 tests, 2 min 06), 092 en `normal` : 14 pistes (0:07.39, 0:33.94,
0:51.62, 0:53.75, 1:12.60, 1:48.93, 2:02.82, 2:24.63, 2:36.99, 2:43.95, 3:29.61, 3:48.29, 3:50.89,
3:57.18), 136,6 s réels, 438 s CPU, 0 s ignorée. #5 fermée sans code : `check_frame_count` couvre le cas.
`cli` et `rendu` touchent tous deux `cli.py:process` : merger `rendu` en dernier, retard mesuré avant.

Scénarios qui déclenchent encore chaque défaut : #4 `--bg-step 40` sur une vidéo à 30 i/s donne un
fond d'une image sans erreur ; #38 passer en rapide demande quatre options ; #3 un réglage explicite
(seuil 12 sans facteur de bruit) sort 1 130 pistes sur la 089 et remplit le disque ; #37 la piste 3 de
la 092 (7 détections, 36 px²) est invisible sur son extrait ; #34 410 lignes `[h264 @ …]` sur la 122 ;
#36 55 s de passe ffprobe sur la 092 saine ; #29 6 min de CI sans seuil de couverture ; #30 aucune
release ni formule.

## #4 — Réglages invalides acceptés

Issue : `gh issue view 4 --repo zaratan/chiro-dvr` (décision en commentaire).
Worktree : `~/Projects/dvr-wt/cli`, branche `bug/4-reglages-invalides` depuis `origin/main`.

Erreur dans les deux cas : `ValueError` quand `bg_step` laisse moins de `MIN_BACKGROUND_FRAMES`
images dans la fenêtre (`DetectConfig`, là où la cadence est connue : `half_window` ou une méthode à
côté, appelée par le même chemin que la commande et le banc) ; `--workers` < 1 refusé par argparse avec un
message, et le `max(1, ns.workers)` de `cli.py` retiré. Documente les deux règles dans le README si les
options y sont décrites. **Ne touche à aucune valeur par défaut.**

Attendu : `uv run batdetect in/x.mp4 --bg-step 40` et `--workers 0` échouent avec un message clair et
un code de sortie non nul ; un test par règle ; `mise run check` vert (423 tests au départ).

## #38 — `--mode quick|normal`

Issue : `gh issue view 38 --repo zaratan/chiro-dvr` (précisions en commentaire).
Worktree : `~/Projects/dvr-wt/cli`, branche `produit/38-mode` depuis `origin/main`.

`--mode` dans `arguments.py`, partagé par la commande et le banc. `quick` = `--work-width 480
--threshold 25 --target-sigma 0 --noise-factor 0`, rien d'autre ; `normal` = défauts actuels, par
défaut. Une option explicite garde la main sur le mode (argparse : le mode pose les défauts, l'option
les remplace). Le mode est écrit dans `params.json`. README : ce que `quick` manque, avec le tableau
« pistes par vidéo en `quick` et en `normal` » que la tour te fournira (balayage des 26 vidéos) ;
`docs/02-methode.md` et `docs/10` : une ligne sur les deux modes. **Pas de mode fin** (#39).

Attendu : `uv run batdetect in/video_092_original.mp4 -o out/quick --mode quick` donne les mêmes
pistes que les quatre options explicites (compare les deux CSV, identiques) ; `--mode normal` et sans
option donnent les 14 pistes ci-dessus ; `params.json` porte `"mode"` ; `mise run check` vert ;
`uv run pytest -m slow` vert (la référence ne change pas).

## #3 — Garde-fou sur le nombre de pistes

Issue : `gh issue view 3 --repo zaratan/chiro-dvr` (décisions et chiffres du balayage en commentaire).
Worktree : `~/Projects/dvr-wt/cli`, branche `produit/3-garde-fou` depuis `origin/main`.

Option `--max-tracks N` (0 désactive), seuil par défaut donné par la tour avec le balayage. Au-delà :
CSV, `params.json` et image résumé écrits ; extraits et vidéo annotée sautés ; une ligne sur la sortie
d'erreur avec le nombre de pistes et le seuil ; la vidéo compte en échec (code de sortie non nul à la
fin du lot) mais les vidéos suivantes sont traitées. Le seuil écrit dans `params.json`. README et
`docs/02` à jour. **Ne change pas le rendu lui-même** (`render.py`, `clips.py` : c'est #37).

Attendu : sur la 089 avec `--threshold 12 --noise-factor 0` (1 130 pistes au lot 1), le dossier n'a ni
`split/` ni `_boxes.mp4`, le CSV et le `.png` existent, code de sortie non nul ; la 092 en `normal`
inchangée (14 pistes, extraits présents) ; `mise run check` vert.

## #37 — Extrait zoomé et ralenti pour les pistes ténues

Issue : `gh issue view 37 --repo zaratan/chiro-dvr` (décisions du 7 octobre en commentaire : elles
remplacent celle du 5 sur le « pour chaque piste »).
Worktree : `~/Projects/dvr-wt/rendu`, branche `produit/37-zoom` depuis `origin/main`.

**Plan d'abord**, dans `~/Projects/dvr-wt/rapports/plan-37.md`, relu par `tech-architect` et
`lead-engineer-reviewer` en parallèle, puis arrête-toi : la tour le fait valider. Le plan fixe, chiffres
à l'appui : les seuils `max_area_px` / `peak_amplitude` du zoom automatique, mesurés sur les 14 pistes
de la 092 (la piste 3, 0:51.62, 7 détections, 36 px², amplitude 34,7, doit avoir son zoom ; la 11,
3:29.61, 290 détections, 338 px², 110, non) et sur les CSV du balayage que la tour te donnera ; la
marge, la taille minimale et le rapport 4:3 du cadre fixe ; l'agrandissement maximal ; la taille de
sortie ; l'écart entre la fin de la trace et la position courante ; l'épaisseur des traits après
agrandissement ; la cadence fps/4 et sa lecture dans QuickTime et VLC ; l'effet sur le temps de rendu de
la 092 et sur `MAX_WRITERS`. **Le passage de 3:42.10 cité par l'issue n'existe pas en mode `normal`** :
ne le cherche pas dans les 14 pistes. Ce qui est ou n'est pas une chauve-souris ne se tranche pas ici.

Ensuite, par sous-phase avec ses chiffres : `--zoom auto|all|none` (`auto` par défaut), fichier
`split/<nom>_zoom.mp4`, trace sans boîte, docs (`02`, `07` si les extraits y sont décrits, README).

Attendu : sur la 092, `--zoom all` donne 14 fichiers `_zoom.mp4` à fps/4 cadrés sur la trajectoire ;
`--zoom auto` n'en donne que pour les pistes sous les seuils validés ; `--zoom none` aucun ; les 14
extraits normaux sont identiques au bit près à ceux de `main` ; temps de rendu avant/après ; `mise run
check` vert ; `pytest -m slow` vert.

## #34 — Journal du décodeur h264

Issue : `gh issue view 34 --repo zaratan/chiro-dvr` (décision en commentaire).
Worktree : `~/Projects/dvr-wt/decodage`, branche `qualite/34-journal-h264` depuis `origin/main`.

Coupe le journal libav pour toute lecture OpenCV : détection (y compris les processus de
`parallel.py`, qui démarrent par `spawn`), fond médian (`output/background.py`), rendu
(`output/render.py`), banc. Mesure d'abord quel mécanisme marche avec `opencv-python-headless` 5
(`OPENCV_FFMPEG_LOGLEVEL`, `cv2.utils.logging.setLogLevel`, autre) et où il doit être posé pour agir
avant la première ouverture. Pas de compteur. `docs/10` : la section « Illisible » dit que le journal
du décodeur est coupé et pourquoi.

Attendu : `uv run batdetect "/Users/zaratan/dossier sans titre/img_0000/video_122.mp4" -o out/122
2>err.txt` : `grep -c "h264 @" err.txt` passe de 410 à 0, avec `--workers 2` aussi ; `params.json` dit
toujours 195 images abîmées ; `mise run check` vert.

## #36 — Passe ffprobe plus rapide

Issue : `gh issue view 36 --repo zaratan/chiro-dvr` (critère en commentaire).
Worktree : `~/Projects/dvr-wt/decodage`, branche `qualite/36-passe-ffprobe` depuis `origin/main`.

Mesure, sous verrou, dans cet ordre : contrôle rapide multifil qui ne déclenche la passe complète que
s'il voit au moins une erreur ; passe découpée en tranches parallèles partant d'une image-clé
(`-read_intervals`). Critère : sur la 093 et la 122, mêmes plages abîmées au numéro d'image près que
`main` (compare `damaged_s` et `damaged_frames` de `params.json`) ; sur la 092, surcoût de la passe
≤ 15 s sur la commande complète. Si aucune piste ne tient, rends les mesures sans changer le code.
`docs/09` (section passe ffprobe) et `docs/10` à jour.

Attendu : `params.json` de la 093 et de la 122 identiques à `main` sur `damaged_s` et
`damaged_frames` (195 pour la 122) ; 092 : temps réel et CPU avant/après (136,6 s / 438 s CPU au
départ, passe seule 55 s) ; `mise run check` vert ; `pytest -m slow` vert.

## #29 — CI en tâches parallèles, couverture

Issue : `gh issue view 29 --repo zaratan/chiro-dvr` (décisions en commentaire).
Worktree : `~/Projects/dvr-wt/ci`, branche `qualite/29-ci` depuis `origin/main`.

Quatre tâches : statique (lint, format, types ; ni ffmpeg ni LFS), tests rapides (`-m 'not slow'`),
tests `slow` (extrait LFS), `reference` inchangée. ffmpeg en cache ou par une action épinglée par
hash, comme les autres. Couverture sur les tests rapides, `--cov-fail-under=95`, rapport dans le
résumé du job. `pytest-xdist` : mesure sur les tests rapides, garde-le seulement si gain > 30 %, et
dis le chiffre. `mise.toml` suit (les tâches locales restent utilisables). **Pas de macOS** (#19).

Attendu : le fichier passe `actionlint` si disponible ; `mise run check` vert en local ; la tour lira
les durées de la CI sur la PR (départ : `check` 5 min 56, `reference` 5 min 15, ffmpeg 24 s).

## #30 — Distribution par Homebrew

Issue : `gh issue view 30 --repo zaratan/chiro-dvr` (décision en commentaire).
Worktree : `~/Projects/dvr-wt/ci`, branche `produit/30-homebrew` depuis `origin/main`.

**Plan d'abord**, `~/Projects/dvr-wt/rapports/plan-30.md`, relu par `tech-architect` et
`lead-engineer-reviewer`, puis arrête-toi. Le plan compare binaire autonome (comme
`~/Projects/chiro-tools/.github/workflows/release.yml`) et formule Python (`python@3.14`, `opencv`,
`numpy` de Homebrew) sur : installation depuis zéro sur un Mac sans Python ni uv (temps, taille),
temps et pistes de la 092 identiques au dépôt, mise à jour par `bump-formulae.yml` du tap
`zaratan/homebrew-bat-tools` (lecture seule : la formule se propose dans le plan, elle n'est pas
poussée). Release sur tag `v*`, version lue du tag, 0.1.0 d'abord ; `batdetect` seul ;
`depends_on "ffmpeg"`. README et `docs/06` (question 9) à jour.

Attendu (après validation du plan) : un `release.yml` qui construit l'artefact sur `macos-latest` ;
la formule proposée dans `rapports/` ; une installation locale de l'artefact qui donne les 14 pistes
de la 092 ; `mise run check` vert.

Les temps du balayage du 7 octobre sont faussés par d'autres processus lourds sur la machine (bancs de
l'utilisateur) : seuls les nombres de pistes comptent ; les temps se remesurent plus tard.
