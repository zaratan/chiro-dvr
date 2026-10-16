# Outillage du dépôt

Ce qui n'est écrit nulle part ailleurs : outils du poste, hooks, CI, langue des messages et
distribution. La chaîne de construction (toolchain, FFmpeg lié en statique, lints, charte,
dépendances), les niveaux de tests, la couverture, les licences et l'encodage sont dans
[11](11-migration-rust.md) ; les commandes de tous les jours dans [CONTRIBUTING.md](../CONTRIBUTING.md)
et [CLAUDE.md](../CLAUDE.md).

## Outils du poste

- **mise** fixe la version de chaque outil (`mise.toml`) : lefthook, `cargo-llvm-cov`,
  `cargo-deny`, `cargo-shear`, et Python 3.14.8, qui ne sert qu'aux scripts en ligne de
  `rust:rules` (règles de la charte) et de `rust:data` (récupération de la référence). La
  toolchain Rust vient de `rust-toolchain.toml`, installée par rustup.
- **lefthook** (`lefthook.yml`) : `rustfmt` sur les fichiers indexés avant chaque commit,
  `mise run check` avant chaque push. Les hooks s'installent une fois, dans le dossier git
  commun à tous les worktrees ; `LEFTHOOK=0 git push` saute le hook d'un push.
- **Git LFS** porte les trois vidéos de test (`.gitattributes`) ; lefthook appelle aussi les
  hooks de Git LFS, qui envoient les objets au push.

## CI

`.github/workflows/ci.yml` : sept tâches, toutes sur le Rust. Avec la matrice, les six
premières font huit exécutions, et `package` six de plus, celles de `package.yml`.

| Tâche | Systèmes | Ce qu'elle joue |
| --- | --- | --- |
| `rust` | macOS, Linux | `rust:static`, `rust:coverage` |
| `rust-slow` | macOS, Linux | `rust:slow`, `hardware_*` sous macOS |
| `rust-reference` | Linux | `rust:reference` |
| `rust-settings` | Linux | `rust:settings` |
| `rust-windows` | Windows | construction MSVC, tests rapides, clippy |
| `rust-windows-slow` | Windows | tests `slow_*` |
| `package` | macOS 26, Ubuntu 22.04, Windows 2025 | `package.yml` : les trois archives `release`, `check.sh`, les sources LGPL, `SHA256SUMS` |

- Les actions sont épinglées par SHA. L'installation commune est dans
  `.github/actions/setup` (mise, rustup) ; `.github/actions/ffmpeg-key` calcule les clés des
  caches de FFmpeg et de `target/`, `.github/actions/windows` prépare MSYS2.
- Les vidéos LFS sont mises en cache (`.git/lfs`) pour ménager le quota : chaque tâche ne
  récupère que ce qu'elle lit.
- zizmor ne tourne pas en CI : il se lance à la main sur les workflows. Au dernier passage
  noté, il ne signalait que l'appel des actions locales (`self-repository`, niveau le plus
  bas).
- **Trois workflows pour la publication**, parce qu'un workflow appelé ne peut pas
  recevoir plus de droits que celui qui l'appelle (`ci.yml` n'a que `contents: read`) :
  - `package.yml` (`workflow_call` et `workflow_dispatch`, lecture seule) :
    - une tâche par système, aux étiquettes figées : `macos-26`, `ubuntu-22.04` (glibc
      2.35) et `windows-2025-vs2026` ;
    - chacune construit FFmpeg, puis l'archive (`packaging/package.sh`), la contrôle
      (`packaging/check.sh`, [11](11-migration-rust.md#empaquetage)) et la téléverse ;
    - `sources` télécharge `ffmpeg-7.1.1.tar.xz` et `openh264-2.6.0.tar.gz`, et vérifie
      leurs sha256 épinglés (`packaging/sources.sh`) ;
    - `sums` écrit `SHA256SUMS` des cinq fichiers et les réunit dans l'artefact
      `release-assets` ;
    - le cache FFmpeg sert sauf avec l'entrée `fresh`. `BATDETECT_VERSION` vaut l'entrée
      `version`, sinon la version de `Cargo.toml`.
  - `ci.yml` l'appelle sur chaque PR et sur `main`. Une PR qui change le suivi ou la
    détection de l'extrait doit donc refaire `tests/fixtures/rust/…/tracks.csv`.
  - `release.yml`, sur un tag `vX.Y.Z` seulement :
    - `guard` lance `packaging/release-guards.sh` ;
    - `package.yml` reconstruit, en `fresh`, sans aucun cache dans le livrable ;
    - `publish` (`contents: write`) crée le brouillon avec les six fichiers et le texte
      du CHANGELOG, puis le publie. Une relance complète un brouillon existant et refuse
      une release déjà publiée.

    Hors tag, rien ne peut publier.

## Langue des messages

L'aide, les erreurs d'arguments, les lignes d'étape, le bilan, le bandeau de l'image résumé
et les noms des fichiers produits sont en français quand la locale l'est, en anglais sinon.

- Les textes vivent dans deux catalogues Fluent, `crates/cli/locales/fr.ftl` et `en.ftl`,
  lus par `texts.rs` dans le binaire ; aucun texte vu par l'utilisateur n'est dans le code.
- **Choix de la langue** (`crates/cli/src/language.rs`) : la première variable non vide parmi
  `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` et `LANG`, sinon la langue du système ; dans une liste
  (`LANGUAGE=fr:en`), la première langue `fr…` ou `en…`. Français si c'est `fr…`, anglais
  sinon.
- Virgule décimale dans les nombres français (« 1,5 s »), tableau des pistes compris, qui
  sépare alors ses colonnes par un point-virgule ([docs/02](02-methode.md#sorties)) ; les
  noms des fichiers n'ont jamais d'accent.

## Distribution

- **Homebrew installe la 0.2.0** (`brew install zaratan/bat-tools/batdetect`), par la
  formule `Formula/batdetect.rb` du tap `zaratan/homebrew-bat-tools` (#225).
  - La formule télécharge `batdetect-darwin-arm64.tar.gz` de la release et vérifie son
    `sha256`.
  - Elle installe le binaire et `licenses/`, auquel `NOTICE` renvoie. Homebrew copie
    d'office `LICENSE` et `NOTICE` dans le keg, mais pas un dossier.
  - Elle n'a plus de `depends_on "ffmpeg"` : FFmpeg est lié en statique. Sa licence est
    `all_of: ["MIT", "LGPL-2.1-or-later"]`.
  - Ses planchers : Apple Silicon (`arch: :arm64`) et macOS 14 (`macos: :sonoma`).
- **La 0.1.1**, binaire Python sous GPL v3, est décrite par le `packaging/NOTICE` du tag
  `v0.1.1`, et son mode d'emploi par le README de ce tag.
- **Le binaire Rust se publie par un tag** (#223), sous MIT et LGPL 2.1 ou ultérieure :
  `packaging/NOTICE` et `packaging/licenses/` le décrivent
  ([11](11-migration-rust.md#licences), #224).
- **Publier une version** :
  1. `workspace.package.version` de `Cargo.toml` vaut la version, et `Cargo.lock` suit.
  2. La section `[Non publiée]` de `CHANGELOG.md` devient `[X.Y.Z] - date`.
  3. Une fois ce commit sur `main` et la CI verte, on pousse le tag `vX.Y.Z`.
- `packaging/release-guards.sh` refuse le tag dans quatre cas, dans cet ordre :
  - il ne vaut pas `v` + la version de `Cargo.toml` ;
  - le CHANGELOG n'a pas sa section ;
  - le commit n'est pas sur `main` ;
  - `ci.yml` n'y est pas passé au vert.
- La release porte les trois archives, les deux sources LGPL (FFmpeg et openh264) et
  `SHA256SUMS`. Son texte est la section du CHANGELOG, extraite par
  `packaging/release-notes.sh`.
- **Mise à jour de la formule** : `bump-formulae.yml`, dans le tap, ouvre la PR
  `batdetect X.Y.Z` au premier passage de 6 h UTC après la release, ou à la demande par
  `workflow_dispatch`.
  - Il ne change que deux lignes : la ligne `version "…"` à la racine, et le `sha256` du
    bloc `on_macos do … on_arm do … end end`, recalculé sur l'archive.
  - La formule garde donc cette forme. `brew audit --strict` la juge redondante avec la
    version lue dans l'URL, comme pour la 0.1.1, mais le script en a besoin.
  - Quand la forme de l'archive ou de l'installation change, la formule se retouche à la
    main, comme à #225.
