# Contribuer

Ce guide est pour le développeur ; le mode d'emploi pour la naturaliste est dans le [README](README.md).
Les règles du dépôt et la carte du code sont dans [CLAUDE.md](CLAUDE.md), la base de connaissances dans [docs/](docs/README.md).

## Construire depuis les sources

Prérequis : [mise](https://mise.jdx.dev/), [rustup](https://rustup.rs/), Git LFS, `make`, un compilateur C et C++, `pkg-config`, `curl`, et `xz` sous Linux.
On n'installe pas FFmpeg : il se construit une fois par poste dans `~/.cache/batdetect/` (50 s sur un M1 Max), puis se lie dans le binaire.

```bash
git lfs install            # une fois par machine, avant le clone
git lfs pull               # après un clone fait sans LFS : les trois vidéos de test
mise install               # lefthook, les outils cargo et le Python des scripts de mise.toml
lefthook install           # dans le dépôt principal seulement : les hooks sont communs aux worktrees
rustup toolchain install   # la version de rust-toolchain.toml
mise run rust:ffmpeg       # FFmpeg 7.1.1 (LGPL) et openh264 (BSD), liés en statique
cargo build --release -p batdetect   # le binaire dans target/release/batdetect
```

Depuis les sources, on lance `target/release/batdetect` ou `cargo run --release -p batdetect -- <vidéos>`.
`batdetect -h` donne l'essentiel, `batdetect --help` toutes les options avec leur valeur par défaut, qui font foi.

Hors d'un tag, `batdetect --version` affiche `X.Y.Z-N-gSHA`, N commits après le tag `vX.Y.Z`, et ajoute `-dirty` quand l'arbre a des modifications non commitées.
La version gravée vient de `BATDETECT_VERSION` si elle est posée (`BATDETECT_VERSION=0.2.0 cargo build --release -p batdetect`), sinon de `git describe`, sinon de `Cargo.toml`.
Le détail est dans [docs/11](docs/11-migration-rust.md#crates), à la crate `cli` (`params.json`).

## Vérifier

```bash
mise run check                     # rust:static et rust:test, 20 s à 1 min : vert avant chaque push
mise run rust:reference            # la 092 entière, ses 14 pistes
cargo run --release -p bench -- tests/fixtures/video_092_original.mp4   # banc de mesure (docs/08)
```

La CI de chaque PR joue en plus la couverture (`rust:coverage`), l'extrait (`rust:slow`) et les 16 cas de réglages (`rust:settings`), que `mise run check:ci` enchaîne en local.
Une PR qui ne touche que des docs saute tout le Rust ([05](docs/05-outillage.md#ci)).
Les images des docs se refont par `tools/docs-images.sh` ([docs/images/](docs/images/README.md)).

La commande renvoie un code non nul dès qu'une entrée est refusée ou qu'une vidéo échoue.
Hors d'un terminal, elle ne traite rien quand une entrée est refusée, sauf avec `--skip-refused`.

## Publier une version

On met la version dans `Cargo.toml`, on renomme la section `[Non publiée]` du CHANGELOG en ajoutant son lien `[X.Y.Z]` en bas, puis on pousse le tag `vX.Y.Z` sur `main` une fois la CI verte.
`release.yml` contrôle le tag, construit et contrôle les trois archives, et publie la release ([docs/05](docs/05-outillage.md#distribution)).
Le tap `zaratan/homebrew-bat-tools` ouvre ensuite, à son premier passage de 6 h UTC, la PR qui met à jour la version et le `sha256` de la formule.
Quand l'archive ou son installation changent de forme, on retouche la formule à la main.
