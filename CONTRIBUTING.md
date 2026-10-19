# Contribuer

Ce guide est pour le développeur. Le mode d'emploi pour la naturaliste est dans le [README](README.md).
Les règles du dépôt et la carte du code sont dans [CLAUDE.md](CLAUDE.md).
Ce que le code ne dit pas (construction, licences, charte, tests, pièges de chaque crate) est dans [docs/developpement.md](docs/developpement.md).

## Construire depuis les sources

Prérequis : [mise](https://mise.jdx.dev/), [rustup](https://rustup.rs/), Git LFS, `make`, un compilateur C et C++, `pkg-config` et `curl`.
Sous Linux, il faut aussi `xz`.
On n'installe pas FFmpeg. Il se construit une fois par poste dans `~/.cache/batdetect/` (50 s sur un M1 Max) puis se lie dans le binaire.

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
`batdetect -h` donne l'essentiel. `batdetect --help` donne toutes les options avec leur valeur par défaut, qui font foi.

Hors d'un tag, `batdetect --version` affiche `X.Y.Z-N-gSHA` : N commits après le tag `vX.Y.Z`.
Il ajoute `-dirty` quand l'arbre a des modifications non commitées.
La version gravée vient de `BATDETECT_VERSION` si elle est posée (`BATDETECT_VERSION=0.2.0 cargo build --release -p batdetect`).
À défaut, elle vient de `git describe`, puis de `Cargo.toml` ([détail](docs/developpement.md#cli)).

## Vérifier

```bash
mise run check                     # rust:static, rust:test et page:js, 20 s à 1 min : vert avant chaque push
mise run rust:reference            # la 092 entière, ses 14 pistes
cargo run --release -p bench -- tests/fixtures/video_092_original.mp4   # banc de mesure
```

La CI de chaque PR joue en plus `rust:coverage` pour la couverture, `rust:slow` pour l'extrait et `rust:settings` pour les 16 cas de réglages.
`mise run check:ci` les enchaîne en local.
Les niveaux de tests et les gardes figées sont décrits dans [docs/developpement.md](docs/developpement.md#tests).
Le banc s'y lit à [Banc de mesure](docs/developpement.md#banc-de-mesure).
Les images des docs se refont par `tools/docs-images.sh` ([docs/images/](docs/images/README.md)).

## Outillage

- **mise** fixe dans `mise.toml` la version de chaque outil : lefthook, `cargo-llvm-cov`, `cargo-deny`, `cargo-shear`, `cargo-about`, Node et Python.
  Node ne sert qu'aux tests du JavaScript de la page (`page:js`).
  Python fait tourner les scripts de `rust:rules`, de `notice` et de `rust:data`.
  `packaging/release-guards.sh` et `package.yml` s'en servent aussi pour lire la version.
  La toolchain Rust vient de `rust-toolchain.toml`, installée par rustup.
- Quand une tâche mise a plusieurs commandes, `mise run TÂCHE -- ARGS` ne passe les arguments qu'à la dernière.
  Une sous-tâche `{ task = … }` ne les reçoit jamais. Ainsi `rust:static -- X` ne les donne qu'à `cargo shear`.
- **lefthook** (`lefthook.yml`) lance `rustfmt` sur les fichiers indexés avant chaque commit et `mise run check` avant chaque push.
  Les hooks s'installent une fois dans le dossier git commun à tous les worktrees. `LEFTHOOK=0 git push` saute le hook d'un push.
- **Git LFS** porte les trois vidéos de test (`.gitattributes`). lefthook appelle aussi ses hooks, qui envoient les objets au push.

### CI

`.github/workflows/ci.yml` a sept tâches sur le Rust et une sur le JavaScript de la page, entre `changes` et `verdict`.

| Tâche | Systèmes | Ce qu'elle joue |
| --- | --- | --- |
| `changes` | Linux | liste les fichiers de la PR, dit si les tâches du Rust et `page-js` tournent |
| `rust` | macOS, Linux | `rust:static`, `rust:coverage` |
| `page-js` | Linux | `page:js` |
| `rust-slow` | macOS, Linux | `rust:slow`, `hardware_*` sous macOS |
| `rust-reference` | Linux | `rust:reference` |
| `rust-settings` | Linux | `rust:settings` |
| `rust-windows` | Windows | construction MSVC, tests rapides, clippy |
| `rust-windows-slow` | Windows | tests `slow_*` |
| `package` | macOS 26, Ubuntu 22.04, Windows 2025 | `package.yml` : les trois archives `release`, `check.sh`, les sources LGPL, `SHA256SUMS` |
| `verdict` | Linux | rouge si une tâche a échoué ou a été annulée, vert sinon |

- **`verdict` est le seul contrôle exigé** par le ruleset `main PR`.
  Une tâche ajoutée à `ci.yml` doit s'ajouter à ses `needs`, sinon elle n'est pas exigée.
  GitHub compte comme réussi un contrôle exigé qui a été sauté. `verdict` tourne donc par `if: always()`.
- **PR de docs seules** : une PR dont chaque fichier finit en `.md` ou est sous `docs/images/` saute `page-js` et les sept tâches du Rust.
  Aucune tâche ne lit ces fichiers. Un test qui en lirait un devra resserrer le filtre.
  Un `push` sur `main` n'est jamais filtré, car `packaging/release-guards.sh` exige une CI complète sur le commit du tag.
- **PR de détection** : elle met à jour les écarts figés contre la référence v0.2.0 et donne la cause de chaque écart ([gardes](docs/developpement.md#gardes-de-détection)).
  Elle joint le rapport de `compare local --reference v0.1.1`, qui compare à l'implémentation indépendante du Python.
  Elle met à jour [resultats-092.md](docs/resultats-092.md) si les pistes de la 092 bougent et [detection.md](docs/detection.md) pour l'étape changée.
  Elle refait `tests/fixtures/rust/…/tracks.csv` si le suivi ou la détection de l'extrait change.
  Les passages de la naturaliste sur la 092 ne bougent jamais : un passage manqué est une régression, pas un écart à figer.
- **Images** : « Linux » est `ubuntu-26.04`, nommée plutôt que `ubuntu-latest`.
  Seules les archives de `package` restent sur `ubuntu-22.04` pour la glibc.
  Les clés des caches portent l'image.
  En changer reconstruit FFmpeg, `target/` et les outils de mise au premier passage, puis une seconde fois sur `main`.
  actionlint 1.7.12 ne connaît pas `ubuntu-26.04`. `.github/actionlint.yaml` le déclare comme `self-hosted-runner`.
- La tâche `rust` installe zsh sous Linux pour le test `zsh -n` du script de complétion.
- Les actions sont épinglées par SHA. Les vidéos LFS sont mises en cache et chaque tâche ne récupère que celles qu'elle lit.
- zizmor ne tourne pas en CI.
  Quand un workflow change, on le lance à la main par `uvx zizmor --offline .github/` sans appliquer ses corrections automatiques.
  Il ne signale que les appels locaux `uses: ./…`, gardés parce qu'actionlint refuse la forme qu'il propose.

## Publier une version

1. `workspace.package.version` de `Cargo.toml` prend la version. `Cargo.lock` suit.
2. La section `[Non publiée]` de `CHANGELOG.md` devient `[X.Y.Z] - date`.
3. En bas du CHANGELOG, le lien `[X.Y.Z]` compare `v<précédente>...vX.Y.Z`.
   Si la section `[Non publiée]` reste, son lien compare `vX.Y.Z...HEAD`.
4. Une fois ce commit sur `main` et la CI verte, on pousse le tag `vX.Y.Z`.

`release.yml` ne tourne que sur un tag `vX.Y.Z`. `packaging/release-guards.sh` refuse le tag dans six cas, dans cet ordre :

- Il ne vaut pas `v` + la version de `Cargo.toml`.
- Le CHANGELOG n'a pas sa section.
- Le lien `[X.Y.Z]` manque, est en double ou ne part pas de la version précédente du CHANGELOG.
- La section `[Non publiée]` ou son lien existe et ce lien ne compare pas `vX.Y.Z...HEAD`.
- Le commit n'est pas sur `main`.
- `ci.yml` n'y est pas passé au vert.

Puis `package.yml` reconstruit tout sans cache : aucun cache n'entre dans le livrable.
`publish` crée le brouillon de release avec les trois archives, les sources de FFmpeg et d'openh264 et `SHA256SUMS`, puis le publie.
Le texte de la release est la section du CHANGELOG, extraite par `packaging/release-notes.sh`.
Une relance complète un brouillon existant et refuse une release déjà publiée. Hors tag, rien ne peut publier.
`ci.yml` n'a que `contents: read` et un workflow appelé ne reçoit jamais plus de droits que celui qui l'appelle.

### Homebrew

`brew install zaratan/bat-tools/batdetect` installe la version publiée depuis la formule `Formula/batdetect.rb` du tap `zaratan/homebrew-bat-tools`.

- La formule télécharge `batdetect-darwin-arm64.tar.gz` et vérifie son `sha256`.
  Elle installe le binaire et `licenses/`, auquel `NOTICE` renvoie.
  Elle n'a pas de `depends_on "ffmpeg"`, car FFmpeg est lié en statique. Sa licence est `all_of: ["MIT", "LGPL-2.1-or-later"]`.
  Elle demande Apple Silicon (`arch: :arm64`) et macOS 14 (`macos: :sonoma`).
- Elle installe le script de complétion zsh par `generate_completions_from_executable(bin/"batdetect", "completion", shells: [:zsh])`.
  Son `caveats` rappelle la ligne `compinit` de `~/.zshrc`.
  Le script est dans la langue du système à l'installation, ce qui reste à vérifier sur un Mac.
- Le workflow `bump-formulae.yml` du tap ouvre la PR `batdetect X.Y.Z` à son premier passage de 6 h UTC après la release, ou à la demande.
  Il ne change que la ligne `version "…"` et le `sha256` du bloc `on_macos do … on_arm do … end end`.
  La formule doit donc garder cette forme.
  `brew audit --strict` juge la ligne `version` redondante, mais le script en a besoin.
  Quand l'archive ou son installation changent de forme, on retouche la formule à la main.
