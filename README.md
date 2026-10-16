# batdetect

Repère et suit les chauves-souris dans les vidéos de jumelles thermiques, puis sort un
extrait par passage pour validation par une naturaliste.

## Installation

Homebrew installe pour l'instant la 0.1.1, la dernière version publiée, sur un Mac Apple Silicon
en macOS 14 ou plus récent :

```bash
brew install zaratan/bat-tools/batdetect   # installe aussi ffmpeg
batdetect --version
```

La 0.1.1 est la version d'avant la réécriture en Rust : ses options (`--mode`, `--encoder`…) et
ses sorties sont décrites dans le [README de la v0.1.1](https://github.com/zaratan/chiro-dvr/blob/v0.1.1/README.md),
pas ici. Son binaire est sous GPL v3 (il embarque FFmpeg compilé avec x264 et x265). La version
décrite ci-dessous se construit depuis les sources, en attendant sa publication (#93).

### Depuis les sources

Prérequis : [mise](https://mise.jdx.dev/), [rustup](https://rustup.rs/), Git LFS, `make`, un
compilateur C et C++, `pkg-config`, `curl`, et `xz` sous Linux. FFmpeg n'est pas à installer :
il se construit une fois par poste dans `~/.cache/batdetect/` (50 s sur un M1 Max) et se lie
dans le binaire.

```bash
git lfs install            # une fois par machine, avant le clone
git lfs pull               # après un clone fait sans LFS : les trois vidéos de test
mise install               # lefthook, les outils cargo et le Python des scripts de mise.toml
lefthook install           # rustfmt avant chaque commit, mise run check avant chaque push
rustup toolchain install   # la version de rust-toolchain.toml
mise run rust:ffmpeg       # FFmpeg 7.1.1 (LGPL) et openh264 (BSD), liés en statique
cargo build --release -p batdetect   # le binaire dans target/release/batdetect
```

## Utilisation

```bash
batdetect nuit-12-oct                  # toutes les vidéos du dossier, résultats dans nuit-12-oct/batdetect/
batdetect nuit-12-oct nuit-13-oct      # plusieurs soirées d'un coup
batdetect nuit-12-oct/video_092.mp4    # une seule vidéo
batdetect nuit-12-oct -o resultats     # résultats dans un autre dossier
batdetect nuit-12-oct --zoom all       # un extrait ralenti et agrandi pour chaque piste
batdetect nuit-12-oct --annotated      # aussi la vidéo entière, toutes les pistes dessinées
```

Depuis les sources, la commande se lance par `target/release/batdetect` ou par
`cargo run --release -p batdetect -- <vidéos>`. `batdetect -h` donne l'essentiel,
`batdetect --help` toutes les options avec leur valeur par défaut : c'est lui qui fait foi.

- `-o`/`--out-dir`, `--zoom auto|all|none`, `--annotated`, `--max-tracks`, `--clip-margin`,
  `--osd-region` (à répéter pour plusieurs zones), `--skip-refused` et `--no-notify`.
- Quatre réglages changent la détection : `--threshold`, `--min-hits`, `--max-blobs` et
  `--work-width`. Les autres réglages sont fixés et écrits dans `params.json`
  ([docs/02](docs/02-methode.md)).

Toutes les entrées sont contrôlées avant le premier calcul. Les entrées refusées (fichier
introuvable, illisible, pas une vidéo…) sont listées ensemble ; dans un terminal, la commande
demande s'il faut traiter quand même les vidéos valides. Hors d'un terminal, rien n'est traité,
sauf avec `--skip-refused`.

Pendant le calcul, une ligne par étape sur la sortie d'erreur (`[2/5] video_092 — détection`),
puis un bilan de la soirée : pour chaque vidéo sa durée, ses pistes, le temps non analysé et le
temps de calcul, puis les vidéos en échec avec ce qu'il faut faire. Le code de sortie n'est pas
nul dès qu'une entrée est refusée ou qu'une vidéo échoue. Un lot interrompu (Ctrl-C) se refait
en relançant la même commande.

Sous macOS, quand la commande est lancée dans un terminal, la fin du lot s'annonce par une
notification qui reprend la première ligne du bilan, et chaque dossier de résultats s'ouvre dans
le Finder. `--no-notify` les éteint ; ni notification ni dossier ouvert quand la sortie est redirigée.

Les jumelles doivent être fixes pendant l'enregistrement. Les moments où le cadre bouge
sont repérés et ignorés (environ 1 s de part et d'autre du mouvement) : ils ne sont pas
analysés et sont à revoir à l'œil. `--max-blobs 0` désactive ce filtre.

Les images que le décodeur a dû réparer (fichier abîmé) sont repérées pendant la lecture,
par les signalements du décodeur et les trous dans les horodatages. Elles sont ignorées
jusqu'à l'image-clé suivante, plus une demi-seconde de chaque côté : le fond médian les voit
encore ([docs/10](docs/10-logique-de-detection.md)). Ce filtre ne se désactive pas. Une
vidéo dont il manque des images sans explication est comptée en échec ; les autres vidéos du
lot continuent.

Au-delà de `--max-tracks` pistes, la vidéo est presque sûrement du bruit : le CSV,
`params.json` et l'image résumé sont écrits, mais ni extraits ni vidéo annotée. Le bilan la
compte à part, avec le nombre de pistes, et les vidéos suivantes sont traitées.

### Fichiers produits

Pour chaque vidéo, un dossier à son nom, dans `batdetect/` à côté des vidéos, ou dans le
dossier donné à `-o`. Les sorties de la v0.1.1, écrites dans `out/` du dossier où la commande
était lancée, restent où elles sont : rien ne les déplace. Les noms suivent la langue de la
locale, comme l'aide : en français sous une locale française, en anglais sinon. Ils ne portent
jamais d'accent.

| Fichier | Français | Anglais |
| --- | --- | --- |
| Image résumé | `<nom>_resume.png` | `<nom>_summary.png` |
| Tableau des pistes | `<nom>_pistes.csv` | `<nom>_tracks.csv` |
| Dossier des extraits | `extraits/piste_01_0m04s.mp4` | `clips/track_01_0m04s.mp4` |
| Extrait zoomé | `extraits/piste_01_0m04s_zoom.mp4` | `clips/track_01_0m04s_zoom.mp4` |
| Vidéo annotée | `<nom>_annotee.mp4` | `<nom>_annotated.mp4` |
| Réglages | `params.json` | `params.json` |

Une fois la détection finie, chaque calcul efface, dans le dossier de la vidéo, les sorties des
deux langues et celles des versions précédentes (`split/`, `<nom>.tracks.*`, `<nom>_boxes.mp4`).
La langue du dernier calcul fait foi. Un CSV qui n'a pas la forme d'un tableau des pistes, ou dont
une case `verdict` est remplie, est laissé tel quel, et le bilan le dit ; le nouveau tableau n'est
alors pas écrit.

- `<nom>_resume.png` : l'image résumé, toutes les trajectoires sur le fond médian de la
  vidéo. Au-delà de 10 min, une image par tranche de 10 min (`<nom>_resume_000m-010m.png`,
  …) ; au-delà de 20 pistes, l'image est coupée en parties (`<nom>_resume_1_sur_3.png`, …).
  Comment la lire : [lire l'image résumé](docs/lire-les-resultats.md#lire-limage-résumé) ;
- `<nom>_pistes.csv` : une ligne par piste, pour un tableur réglé en français (point-virgule,
  virgule décimale), avec `piste`, `début`, `fin`, `durée (s)`, une colonne `verdict` vide à
  remplir, puis `points`, `corde (px)`, `chemin (px)`, `vitesse (px/s)`, `surface max (px)`,
  `amplitude (niveaux de gris)`, `images comblées` et `début (s)`
  ([lire les résultats](docs/lire-les-resultats.md)) ;
- `params.json` : les réglages employés, la version et le FFmpeg lié ;
- `extraits/` : un extrait par piste, nommé par numéro de piste et temps de début à la
  seconde (le numéro départage deux pistes de la même seconde ; deux chiffres, trois dès
  100 pistes, pour que le Finder les range dans l'ordre), avec une boîte, un numéro et la
  trace des pistes visibles ; pour les pistes petites ou faibles, aussi
  `<même nom>_zoom.mp4` : la même scène recadrée sur la trajectoire et agrandie jusqu'à
  ×4, en pleine taille, ralentie ×0,25, avec la trace seule. `--zoom all` en fait un pour
  chaque piste, `--zoom none` aucun ([docs/02](docs/02-methode.md#extrait-zoomé-et-ralenti)) ;
- les périodes ignorées parce que l'image est saturée de taches (jumelles qui bougent)
  sont écrites dans le bandeau de l'image résumé (`hors analyse 0:00 - 0:09`, arrondi vers
  l'extérieur), montrées sur la frise et listées dans `params.json` (`ignored_s`) ;
- les périodes illisibles (images abîmées et leur marge) sont signalées à part : des lignes
  `illisible 1:12 - 1:15` dans le bandeau, la frise, et dans `params.json` la liste
  `damaged_s` et `damaged_frames` (nombre d'images signalées par le décodeur ou qui suivent
  un trou dans les horodatages, pas le nombre d'images écartées). Le bandeau garde trois
  lignes de chaque sorte, puis `+ N autres` ;
- avec `--annotated` seulement, `<nom>_annotee.mp4` : la vidéo annotée complète. Sans
  l'option, celle d'un traitement précédent est supprimée.

Sous macOS, les extraits sont encodés par le moteur multimédia de la puce (VideoToolbox) ; si
celui-ci échoue en cours de route, les extraits touchés sont réécrits par l'encodeur logiciel
openh264, et une ligne le dit. Ailleurs, et pour les extraits zoomés partout, l'encodeur est
openh264.

Le détail de la méthode et des réglages est dans [docs/02-methode.md](docs/02-methode.md).

## Développement

```bash
mise run check                     # rust:static et rust:test, 20 s à 1 min : vert avant chaque push
mise run rust:reference            # la 092 entière, ses 14 pistes
cargo run --release -p bench -- in/video_092_original.mp4   # banc de mesure (docs/08)
```

La CI de chaque PR joue en plus la couverture (`rust:coverage`), l'extrait (`rust:slow`) et
les 16 cas de réglages (`rust:settings`) : `mise run check:ci` les enchaîne en local.
Les règles du dépôt et la carte du code sont dans [CLAUDE.md](CLAUDE.md), la base de
connaissances du projet dans [docs/](docs/README.md).

### Publier une version

Impossible avant #93 : `release.yml` refuse tout tag. Le CHANGELOG garde la section de la
prochaine version, et `packaging/release-notes.sh` en tirera le texte de la release. Une fois
la release publiée, le tap `zaratan/homebrew-bat-tools` ouvre la PR de mise à jour de la
formule le lendemain (`bump-formulae.yml`, chaque jour à 6 h UTC).

## Licence

Code sous MIT, voir [LICENSE](LICENSE). Le binaire 0.1.1 distribué par Homebrew est sous GPL v3,
voir [packaging/NOTICE](packaging/NOTICE). Le binaire Rust lie FFmpeg (LGPL) et openh264 (BSD) :
sa licence se décidera à #93 ([docs/11](docs/11-migration-rust.md#licences)).
