# Lot 4 : socle langue et noms (#54, #87, #57, #85) — consignes

Un worktree `~/Projects/dvr-wt/socle`, quatre branches à la suite, chacune depuis `origin/main` après
`fetch` (ou empilée sur la précédente si sa PR n'est pas encore mergée). Rapports dans
`~/Projects/dvr-wt/rapports/rapport-<N>.md`. Lot 1 du plan ergonomie ([lot-3.md](lot-3.md)).

Départ mesuré le 10 octobre 2026 sur `main` 742668b : `mise run check` vert (496 tests rapides en 14 s,
4 lents en 1 min 35) ; `LANG=fr_FR.UTF-8 uv run batdetect` répond `usage: batdetect [-h] …` puis
`batdetect: error: the following arguments are required: inputs` ; `--help` : 31 options, 7 sans
description (`-o`, `--bg-step`, `--max-gap`, `--min-hits`, `--box-pad`, `--trail`, `--clip-margin`),
aucune valeur par défaut affichée ; bandeau de l'image résumé de l'extrait : « 4 passages »
(`summary.py:45`) ; sorties de l'extrait : `<nom>.tracks.png`, `<nom>.tracks.csv`, `params.json`,
`split/01_0m04s80.mp4` … ; `msgfmt` présent sur ce poste (`/opt/homebrew/bin/msgfmt`), pas garanti en CI.

Scénarios qui déclenchent encore chaque défaut : #54 Manon tape `--help` et trouve sept réglages sans un
mot ; #87 l'image résumé de la 092 annonce « 14 passages » alors que Manon n'en a confirmé que 13 ; #57
son Mac est en français et l'aide reste en anglais ; #85 elle ouvre `split/` sans savoir que ce sont les
pistes à vérifier.

## #54 — Options décrites, défauts affichés

Issue : `gh issue view 54 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/socle`, branche `produit/54-options-decrites` depuis `origin/main`.

Dans `arguments.py` et `cli.py` : une phrase d'aide par option (effet visible d'abord, unité ensuite),
un `metavar` parlant (`DIR`, `SECONDS`, `PIXELS`…), et la valeur par défaut affichée pour chacune. Pour
les quatre réglages portés par le mode, les deux valeurs lues dans `MODES`, jamais recopiées. Textes en
anglais (langue source, #57 traduit ensuite) ; « track » pour une sortie de l'outil, « point » ou
« detection » pour une détection sur une image, jamais « pass » ni « passage ». `batdetect-bench --help`
partage `add_detection_arguments` et `add_tracking_arguments` : il doit rester cohérent.

Décision prise : les options restent déclarées par `add_argument` ; la table déclarative des options
appartient à #59 (complétion), pas à cette issue. Si un défaut est aujourd'hui calculé à l'exécution,
afficher la règle en une phrase plutôt que la valeur.

**Ne touche pas** aux noms d'options ni aux valeurs par défaut : afficher, pas changer. Pas de tl;dr ni
de sections (#56).

Attendu : `uv run batdetect --help` : 0 option sans description (7 aujourd'hui), chaque option affiche
son défaut, `--threshold` et consorts montrent les deux valeurs des modes ; `uv run batdetect-bench
--help` toujours valide ; un test qui parcourt les actions du parseur et échoue sur une option sans
`help` ; `mise run check` vert (496 + 4 au départ). README : seulement si une option y est citée avec un
texte qui change.

## #87 — Le bandeau dit « pistes »

Issue : `gh issue view 87 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/socle`, branche `produit/87-bandeau-pistes` depuis `origin/main` (ou sur #54).

`summary.py:45` : « piste » / « pistes », singulier pour 0 et 1 ; les trois assertions de
`tests/test_summary.py` (`:46`, `:47`, `:77`) suivent. Le bandeau reste en ASCII (`:78-81`).

**Ne touche pas** au README ni à docs/02 : #81 et #77 reprendront le mot. Pas de traduction (#57).

Attendu : image résumé de l'extrait avec « 4 pistes » ; `grep -rn passage src/` ne trouve plus rien ;
`mise run check` vert.

## #57 — Français ou anglais selon la locale

Issue : `gh issue view 57 --repo zaratan/chiro-dvr` (la piste est détaillée, l'essai gettext a été fait).
Worktree : `~/Projects/dvr-wt/socle`, branche `produit/57-langue` depuis `origin/main` (ou sur #87).

Décisions (9 octobre 2026) : gettext de la bibliothèque standard, domaine `messages` lié par
`bindtextdomain` pour que les textes internes d'argparse suivent ; aucune dépendance Python ; un script
du dépôt écrit le `.mo` depuis le `.po` et un test vérifie qu'ils correspondent (`msgfmt` du poste sert
seulement à comparer, il n'est pas requis) ; choix de la langue par `LC_ALL`, puis `LC_MESSAGES`, puis
`LANG`, français si la valeur commence par `fr`, **anglais sinon et quand aucune n'est posée** ; virgule
décimale dans les messages français par une fonction de formatage du module de langue, sans
`locale.setlocale`. Couvre l'aide (textes de #54), les erreurs d'arguments, les lignes de fin de
traitement, et le bandeau de l'image résumé (« tracks » / « pistes », ASCII seulement : OpenCV ne dessine
pas les accents). Module de langue dédié avec son fichier de tests ; tests sur les deux langues par la
variable d'environnement (`monkeypatch.setenv`), sans mock. `mise run package` embarque le catalogue
(`--collect-data` ou `--add-data`) et `package:check` vérifie `LANG=fr_FR.UTF-8 dist/…/batdetect --help`
en français. docs/05 décrit le mécanisme et le script.

**Ne touche pas** aux noms des fichiers produits ni à `params.json` (#85), ni au contenu du CSV (#84),
ni aux messages que d'autres issues créeront (avancement, bilan) : seulement les textes existants.
`packaging/NOTICE` ne change pas (aucune bibliothèque ajoutée) : le dire dans le compte rendu.

Attendu : `LANG=fr_FR.UTF-8 uv run batdetect` → `utilisation : …` et `batdetect : erreur : arguments
obligatoires manquants : inputs` (ou formulation équivalente, en français) ; `LANG=C uv run batdetect`
et `env -u LANG -u LC_ALL -u LC_MESSAGES uv run batdetect` → anglais ; `LANG=fr_FR.UTF-8 uv run
batdetect --help` sans phrase anglaise (l'agent liste ce qui reste en anglais s'il y a lieu, par exemple
les noms d'options) ; extrait traité sous `fr` : bandeau « 4 pistes » et lignes de fin en français ;
`mise run package` puis le binaire en français ; `mise run check` vert ; `mise run package:check` vert.

## #85 — Noms des fichiers produits selon la langue

Issue : `gh issue view 85 --repo zaratan/chiro-dvr` (sept endroits où les noms sont formés, impacts listés).
Worktree : `~/Projects/dvr-wt/socle`, branche `produit/85-noms` depuis `origin/main` (ou sur #57).

Un module de `output/` qui porte tous les noms (une table par langue, une fonction par fichier), son
fichier de tests, et `cli.py`, `clips.py`, `zoomview.py`, `summary.py`, `render.py` qui passent par lui ;
la langue vient du module de #57. Français : `extraits/piste_01_0m04s.mp4`,
`extraits/piste_01_0m04s_zoom.mp4`, `<nom>_pistes.csv`, `<nom>_resume.png` (et `_000m-010m` au-delà
de 10 min), `<nom>_annotee.mp4` ; anglais : `clips/track_01_0m04s.mp4`, `_zoom`, `<nom>_tracks.csv`,
`<nom>_summary.png`, `<nom>_annotated.mp4` ; `params.json` identique partout. Le temps des extraits est
sans centièmes ; le numéro de piste départage deux pistes de la même seconde. Le ménage avant écriture
efface les noms des deux langues et ceux d'aujourd'hui (`split/`, `.tracks.*`, `_boxes`), dans le dossier
de la vidéo seulement : la langue du dernier calcul fait foi.

Décision prise : aucun accent dans un nom de fichier produit (`_resume`, `_annotee`), pour éviter la
décomposition APFS que l'issue signale.

**Ne touche pas** au contenu du CSV (#84) ni à l'emplacement du dossier de sortie (#86). Mets à jour :
`mise.toml:85` (`package:check` compare les CSV par leur nouveau nom), les tests listés dans l'issue,
README (lignes 107-133), docs/02, docs/03 (`split/09_3m29s63.mp4`), docs/05, docs/06, CLAUDE.md
(lignes 86-87, 114).

Attendu : extrait traité sous `LANG=fr_FR.UTF-8` → `extraits/piste_01_0m04s.mp4` … `piste_04_0m32s.mp4`,
`<nom>_pistes.csv`, `<nom>_resume.png`, `params.json`, rien d'autre ; sous `LANG=C` → `clips/track_01_0m04s.mp4`,
`<nom>_tracks.csv`, `<nom>_summary.png` ; un second passage sous l'autre langue ne laisse aucun fichier de
la première ; le CSV est identique au bit près à celui d'avant (`cmp` avec `main`) ; `mise run check` vert ;
`mise run package:check` vert.
