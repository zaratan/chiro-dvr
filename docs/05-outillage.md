# Outillage du dépôt

Ce qui n'est écrit nulle part ailleurs : outils du poste, hooks, CI, langue des messages et
distribution. La chaîne de construction (toolchain, FFmpeg lié en statique, lints, charte,
dépendances), les niveaux de tests, la couverture, les licences et l'encodage sont dans
[11](11-migration-rust.md) ; les commandes de tous les jours dans le [README](../README.md)
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

`.github/workflows/ci.yml` : six tâches, huit exécutions avec la matrice, toutes sur le
Rust.

| Tâche | Systèmes | Ce qu'elle joue |
| --- | --- | --- |
| `rust` | macOS, Linux | `rust:static`, `rust:coverage` |
| `rust-slow` | macOS, Linux | `rust:slow`, contrôle du lien statique (`otool`, `ldd`), `hardware_*` sous macOS |
| `rust-reference` | Linux | `rust:reference` |
| `rust-settings` | Linux | `rust:settings` |
| `rust-windows` | Windows | construction MSVC, tests rapides, clippy, contrôle des DLL |
| `rust-windows-slow` | Windows | tests `slow_*` |

- Les actions sont épinglées par SHA. L'installation commune est dans
  `.github/actions/setup` (mise, rustup) ; `.github/actions/ffmpeg-key` calcule les clés des
  caches de FFmpeg et de `target/`, `.github/actions/windows` prépare MSYS2.
- Les vidéos LFS sont mises en cache (`.git/lfs`) pour ménager le quota : chaque tâche ne
  récupère que ce qu'elle lit.
- zizmor ne tourne pas en CI : il se lance à la main sur les workflows. Au dernier passage
  noté, il ne signalait que l'appel des actions locales (`self-repository`, niveau le plus
  bas).
- `release.yml` refuse tout tag avant #93.

## Langue des messages

L'aide, les erreurs d'arguments, les lignes d'étape, le bilan, le bandeau de l'image résumé
et les noms des fichiers produits sont en français quand la locale l'est, en anglais sinon.

- Les textes vivent dans deux catalogues Fluent, `crates/cli/locales/fr.ftl` et `en.ftl`,
  lus par `texts.rs` dans le binaire ; aucun texte vu par l'utilisateur n'est dans le code.
- **Choix de la langue** (`crates/cli/src/language.rs`) : la première variable non vide parmi
  `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` et `LANG`, sinon la langue du système ; dans une liste
  (`LANGUAGE=fr:en`), la première langue `fr…` ou `en…`. Français si c'est `fr…`, anglais
  sinon.
- Virgule décimale dans les nombres français (« 1,5 s ») ; les temps des pistes (`0:04.80`)
  et les noms des fichiers gardent le point et n'ont jamais d'accent.

## Distribution

- Homebrew installe la 0.1.1 (`brew install zaratan/bat-tools/batdetect`), formule du tap
  `zaratan/homebrew-bat-tools`, binaire de la v0.1.1 décrit par `packaging/NOTICE` et sous
  GPL v3. Son mode d'emploi est le README du tag `v0.1.1`.
- Le binaire Rust se publiera à #93, qui décidera de sa licence (FFmpeg sous LGPL, openh264
  sous BSD : [11](11-migration-rust.md#licences)) et de la formule. D'ici là, `release.yml`
  refuse tout tag.
- Ce qui reste prêt pour #93 : la section de la version dans `CHANGELOG.md`, que
  `packaging/release-notes.sh` extrait pour le texte de la release, et `bump-formulae.yml`
  du tap, qui ouvre chaque jour à 6 h UTC la PR de mise à jour de la formule après une
  release.
