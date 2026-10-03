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

Les jumelles doivent être fixes pendant l'enregistrement : une vidéo où le cadre bouge
n'est pas exploitable.

Pour chaque vidéo, `out/<nom>/` contient :

- `<nom>_boxes.mp4` : la vidéo annotée, une boîte et un numéro par passage ;
- `<nom>.tracks.png` : toutes les trajectoires sur une image ;
- `<nom>.tracks.csv` : début, fin, durée, vitesse de chaque passage ;
- `params.json` : les réglages utilisés ;
- `split/` : un extrait par passage, nommé par numéro et temps de début.

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
