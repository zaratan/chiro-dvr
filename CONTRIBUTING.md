# Contribuer

Ce guide s'adresse au développeur ; le mode d'emploi pour la naturaliste est dans le
[README](README.md). Les règles du dépôt et la carte du code sont dans [CLAUDE.md](CLAUDE.md),
la base de connaissances du projet dans [docs/](docs/README.md).

## Construire depuis les sources

Prérequis : [mise](https://mise.jdx.dev/), [rustup](https://rustup.rs/), Git LFS, `make`, un
compilateur C et C++, `pkg-config`, `curl`, et `xz` sous Linux. FFmpeg n'est pas à installer :
il se construit une fois par poste dans `~/.cache/batdetect/` (50 s sur un M1 Max) et se lie
dans le binaire.

```bash
git lfs install            # une fois par machine, avant le clone
git lfs pull               # après un clone fait sans LFS : les trois vidéos de test
mise install               # lefthook, les outils cargo et le Python des scripts de mise.toml
lefthook install           # dans le dépôt principal seulement : les hooks sont communs aux worktrees
rustup toolchain install   # la version de rust-toolchain.toml
mise run rust:ffmpeg       # FFmpeg 7.1.1 (LGPL) et openh264 (BSD), liés en statique
cargo build --release -p batdetect   # le binaire dans target/release/batdetect
```

Depuis les sources, la commande se lance par `target/release/batdetect` ou par
`cargo run --release -p batdetect -- <vidéos>`. `batdetect -h` donne l'essentiel,
`batdetect --help` toutes les options avec leur valeur par défaut : c'est lui qui fait foi.

## Vérifier

```bash
mise run check                     # rust:static et rust:test, 20 s à 1 min : vert avant chaque push
mise run rust:reference            # la 092 entière, ses 14 pistes
cargo run --release -p bench -- tests/fixtures/video_092_original.mp4   # banc de mesure (docs/08)
```

La CI de chaque PR joue en plus la couverture (`rust:coverage`), l'extrait (`rust:slow`) et
les 16 cas de réglages (`rust:settings`) : `mise run check:ci` les enchaîne en local. La
commande renvoie un code non nul dès qu'une entrée est refusée ou qu'une vidéo échoue ; hors
d'un terminal, elle ne traite rien quand une entrée est refusée, sauf avec `--skip-refused`. Les
images des docs se refont par `tools/docs-images.sh` ([docs/images/](docs/images/README.md)).

## Publier une version

Mettre la version dans `Cargo.toml`, renommer la section `[Non publiée]` du CHANGELOG, puis
pousser le tag `vX.Y.Z` sur `main` une fois la CI verte. `release.yml` contrôle le tag,
construit et contrôle les trois archives et publie la release. Le détail est dans
[docs/05](docs/05-outillage.md#distribution). Au premier passage de 6 h UTC qui suit, le tap
`zaratan/homebrew-bat-tools` ouvre la PR de mise à jour de la formule, qui ne change que sa
version et son `sha256`. Quand l'archive ou son installation changent de forme, la formule
se retouche à la main.
