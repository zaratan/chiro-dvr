# batdetect

Repère et suit les chauves-souris dans les vidéos de jumelles thermiques, puis sort un
extrait par passage pour validation par une naturaliste.

## Installation

Prérequis : [mise](https://mise.jdx.dev/) et ffmpeg (`brew install ffmpeg` sur macOS,
`apt install ffmpeg` sur Debian/Ubuntu ; testé avec ffmpeg 9).

```bash
mise install        # Python 3.14.8, uv, lefthook aux versions de mise.toml
uv sync             # dépendances dans .venv/ depuis uv.lock
lefthook install    # hooks avant commit (ruff)
```

## Utilisation

```bash
uv run batdetect in                 # toutes les vidéos du dossier in/
uv run batdetect a.mp4 b.mov -o res # fichiers précis, sorties dans res/
uv run batdetect --help             # tous les réglages
```

Sur un Mac Apple Silicon, les vidéos sont encodées par le moteur multimédia de la puce ;
ailleurs par `libx264`. `--encoder x264` force l'encodeur logiciel.

Les jumelles doivent être fixes pendant l'enregistrement. Les moments où le cadre bouge
sont repérés et ignorés (environ 1 s de part et d'autre du mouvement) : ils ne sont pas
analysés et sont à revoir à l'œil. `--max-blobs 0` désactive ce filtre.

Pour chaque vidéo, `out/<nom>/` contient :

- `<nom>.tracks.png` : toutes les trajectoires sur le fond médian de la vidéo, une couleur,
  une flèche (sens du vol) et une pastille numérotée par passage, avec une légende à droite
  (numéro et début). Au-delà de 10 min, une image par tranche de 10 min
  (`<nom>.tracks_000m-010m.png`, …). `--osd-region 0,0,1,0.07` y cache l'affichage
  des jumelles Symbion ;
- `<nom>.tracks.csv` : début, fin, durée, vitesse de chaque passage ;
- `params.json` : les réglages utilisés ;
- `split/` : un extrait par passage, nommé par numéro et temps de début, avec une boîte,
  un numéro et la trace des passages visibles ;
- les périodes ignorées parce que l'image est saturée de taches (jumelles qui bougent)
  sont écrites dans l'en-tête du bandeau de l'image résumé (`hors analyse 0:00 - 0:09`,
  arrondi vers l'extérieur), affichées en console et listées dans `params.json`
  (`ignored_s`) ;
- avec `--annotated` seulement, `<nom>_boxes.mp4` : la vidéo annotée complète. Sans
  l'option, celle d'un traitement précédent est supprimée.

Le détail de la méthode et des réglages est dans [docs/02-methode.md](docs/02-methode.md).

## Développement

```bash
mise run check      # lint + format + types + tests, doit être vert avant tout commit
mise run test       # tests seuls (uv run pytest -m "not slow" pour sauter l'extrait réel)
mise run coverage
mise run format
uv run batdetect-bench in/video_092_original.mp4   # banc de mesure (docs/08)
```

La base de connaissances du projet est dans [docs/](docs/README.md).

## Licence

MIT, voir [LICENSE](LICENSE).
