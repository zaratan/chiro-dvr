# Outillage du dépôt

Ce qui n'est écrit nulle part ailleurs : outils du poste, hooks, CI, langue des messages et distribution.
La chaîne de construction (toolchain, FFmpeg lié en statique, lints, charte, dépendances), les niveaux de tests, la couverture, les licences et l'encodage sont dans [11](11-migration-rust.md) ; les commandes de tous les jours dans [CONTRIBUTING.md](../CONTRIBUTING.md) et [CLAUDE.md](../CLAUDE.md).

## Outils du poste

- **mise** fixe la version de chaque outil (`mise.toml`) : lefthook, `cargo-llvm-cov`, `cargo-deny`, `cargo-shear`, et Python 3.14.8, qui ne sert qu'au script en ligne de `rust:rules` (règles de la charte) et à `tools/reference-data.sh`, lancé par `rust:data` (récupération de la référence) et par le test `bucket_data`.
  La toolchain Rust vient de `rust-toolchain.toml`, installée par rustup.
  Sur une tâche à plusieurs commandes, `mise run TÂCHE -- ARGS` ne passe les arguments qu'à la dernière, et jamais à une sous-tâche `{ task = … }` : `rust:static -- X` les donne à `cargo shear` seul.
- **lefthook** (`lefthook.yml`) : `rustfmt` sur les fichiers indexés avant chaque commit, `mise run check` avant chaque push.
  Les hooks s'installent une fois, dans le dossier git commun à tous les worktrees ; `LEFTHOOK=0 git push` saute le hook d'un push.
- **Git LFS** porte les trois vidéos de test (`.gitattributes`) ; lefthook appelle aussi les hooks de Git LFS, qui envoient les objets au push.

## CI

`.github/workflows/ci.yml` : sept tâches sur le Rust, entre `changes` et `verdict`.
Avec la matrice, les six tâches `rust*` font huit exécutions, et `package` six de plus, celles de `package.yml`.

| Tâche | Systèmes | Ce qu'elle joue |
| --- | --- | --- |
| `changes` | Linux | liste les fichiers de la PR, dit si les sept tâches du Rust tournent |
| `rust` | macOS, Linux | `rust:static`, `rust:coverage` |
| `rust-slow` | macOS, Linux | `rust:slow`, `hardware_*` sous macOS |
| `rust-reference` | Linux | `rust:reference` |
| `rust-settings` | Linux | `rust:settings` |
| `rust-windows` | Windows | construction MSVC, tests rapides, clippy |
| `rust-windows-slow` | Windows | tests `slow_*` |
| `package` | macOS 26, Ubuntu 22.04, Windows 2025 | `package.yml` : les trois archives `release`, `check.sh`, les sources LGPL, `SHA256SUMS` |
| `verdict` | Linux | rouge si une tâche a échoué ou a été annulée, vert sinon |

- **Images.**
  « Linux » est `ubuntu-26.04`, nommée explicitement dans `ci.yml`, `package.yml` et `release.yml` plutôt que `ubuntu-latest`, qui bascule vers Ubuntu 26.04 entre le 19 octobre et le 19 novembre 2026 (#272) ; seules les archives de `package` restent sur `ubuntu-22.04`, pour la glibc.
  Changer d'image reconstruit au premier passage FFmpeg, `target/` (clés en [11](11-migration-rust.md), « Cache de `target/` ») et les outils de mise, dont les clés portent l'image ; puis une seconde fois sur `main` après le merge, qui ne lit pas les caches d'une PR.
  Le 15 octobre 2026, un changement d'empreinte a eu le même effet sur les quatre tâches Linux : FFmpeg en 1,5 à 2,5 min, tâche la plus longue en 8,6 min, sous un délai de 30.
- **zsh sous Linux.**
  La tâche `rust` installe zsh (`apt-get`) pour le test `zsh -n` du script de complétion (`crates/cli/tests/completion.rs`, #59) ; macOS l'a d'origine, et le test ne se compile pas sous Windows.
- **PR de docs seules.**
  Une PR dont chaque fichier finit en `.md` ou est sous `docs/images/` (`git diff --name-only --no-renames` contre la branche de base, sur le commit de fusion que prépare GitHub ; un renommage compte ses deux chemins) saute les sept tâches du Rust, `package` compris.
  Aucune tâche ne lit ces fichiers : un test qui en lirait un devra resserrer le filtre.
  Un `push` sur `main` n'est jamais filtré : `packaging/release-guards.sh` exige une CI complète sur le commit du tag.
- **`verdict` est le seul contrôle exigé** par le ruleset `main PR`.
  Il dépend de toutes les autres tâches : une tâche ajoutée à `ci.yml` s'ajoute à ses `needs`, sinon elle n'est pas exigée.
  Il tourne par `if: always()`, même quand une tâche échoue ou que la série est annulée : GitHub compte comme réussi un contrôle exigé qui a été sauté.
- Les actions sont épinglées par SHA.
  L'installation commune est dans `.github/actions/setup` (mise, rustup) ; `.github/actions/ffmpeg-key` calcule les clés des caches de FFmpeg et de `target/`, `.github/actions/windows` prépare MSYS2.
- Les vidéos LFS sont mises en cache (`.git/lfs`) pour ménager le quota : chaque tâche ne récupère que ce qu'elle lit.
- zizmor ne tourne pas en CI : quand un workflow ou une action change, il se lance à la main par `uvx zizmor --offline .github/`, sans appliquer ses corrections automatiques.
  Au 16 octobre 2026, il ne signale que les 16 appels locaux `uses: ./…` (`self-repository`, niveau le plus bas).
  Ils restent : la forme `$/…` qu'il propose est refusée par actionlint 1.7.12.
- actionlint 1.7.12 ne connaît pas le label `ubuntu-26.04` : `.github/actionlint.yaml` le déclare (`self-hosted-runner`), à retirer quand une version le connaîtra.
- **PR de détection.**
  Une PR qui change la détection (#276) met à jour les écarts figés contre la référence v0.2.0, vidéo par vidéo et cas par cas, de `slow_extract`, `settings_agreement`, `reference_092`, `local_agreement`, `local_command` et `local_clips`, avec la cause de chaque écart dans la PR ; joint le rapport de `compare local --reference v0.1.1`, contre l'implémentation indépendante du Python ; met à jour [03](03-resultats-092.md) si les pistes de la 092 bougent, et [10](10-logique-de-detection.md) pour l'étape changée.
  Les passages de la naturaliste sur la 092 (`reference_092`) ne bougent pas : un passage manqué est une régression, pas un écart à figer.
- **Trois workflows pour la publication**, parce qu'un workflow appelé ne peut pas recevoir plus de droits que celui qui l'appelle (`ci.yml` n'a que `contents: read`) :
  - `package.yml` (`workflow_call` et `workflow_dispatch`, lecture seule) :
    - une tâche par système, aux étiquettes figées : `macos-26`, `ubuntu-22.04` (glibc 2.35) et `windows-2025-vs2026` ;
    - chacune construit FFmpeg, puis l'archive (`packaging/package.sh`), la contrôle (`packaging/check.sh`, [11](11-migration-rust.md#empaquetage)) et la téléverse ;
    - `sources` télécharge `ffmpeg-7.1.1.tar.xz` et `openh264-2.6.0.tar.gz`, et vérifie leurs sha256 épinglés (`packaging/sources.sh`) ;
    - `sums` écrit `SHA256SUMS` des cinq fichiers et les réunit dans l'artefact `release-assets` ;
    - le cache FFmpeg sert sauf avec l'entrée `fresh`. `BATDETECT_VERSION` vaut l'entrée `version`, sinon la version de `Cargo.toml`.
  - `ci.yml` l'appelle sur chaque PR et sur `main`.
    Une PR qui change le suivi ou la détection de l'extrait doit donc refaire `tests/fixtures/rust/…/tracks.csv`.
  - `release.yml`, sur un tag `vX.Y.Z` seulement :
    - `guard` lance `packaging/release-guards.sh` ;
    - `package.yml` reconstruit, en `fresh`, sans aucun cache dans le livrable ;
    - `publish` (`contents: write`) crée le brouillon avec les six fichiers et le texte du CHANGELOG, puis le publie.
      Une relance complète un brouillon existant et refuse une release déjà publiée.

    Hors tag, rien ne peut publier.

## Langue des messages

L'aide, les erreurs d'arguments, les lignes d'étape, le bilan, le bandeau de l'image résumé et les noms des fichiers produits sont en français quand la locale l'est, en anglais sinon.

- Les textes vivent dans deux catalogues Fluent, `crates/cli/locales/fr.ftl` et `en.ftl`, lus par `texts.rs` dans le binaire ; aucun texte vu par l'utilisateur n'est dans le code.
- **Choix de la langue** (`crates/cli/src/language.rs`) : la première variable non vide parmi `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` et `LANG`, sinon la langue du système ; dans une liste (`LANGUAGE=fr:en`), la première langue `fr…` ou `en…`.
  Français si c'est `fr…`, anglais sinon.
- Virgule décimale dans les nombres français (« 1,5 s »), tableau des pistes compris, qui sépare alors ses colonnes par un point-virgule ([docs/02](02-methode.md#sorties)) ; les noms des fichiers n'ont jamais d'accent.

## Distribution

- **Homebrew installe la 0.2.0** (`brew install zaratan/bat-tools/batdetect`), par la formule `Formula/batdetect.rb` du tap `zaratan/homebrew-bat-tools` (#225).
  - La formule télécharge `batdetect-darwin-arm64.tar.gz` de la release et vérifie son `sha256`.
  - Elle installe le binaire et `licenses/`, auquel `NOTICE` renvoie.
    Homebrew copie d'office `LICENSE` et `NOTICE` dans le keg, mais pas un dossier.
  - Elle n'a plus de `depends_on "ffmpeg"` : FFmpeg est lié en statique. Sa licence est `all_of: ["MIT", "LGPL-2.1-or-later"]`.
  - Ses planchers : Apple Silicon (`arch: :arm64`) et macOS 14 (`macos: :sonoma`).
- **Complétion zsh (#59, #60)** : la formule de la 0.2.1 installera le script par `generate_completions_from_executable(bin/"batdetect", "completion", shells: [:zsh])` (Homebrew ajoute `zsh` en dernier argument), qui écrit `_batdetect` dans `share/zsh/site-functions` ; il ne reste à l'utilisateur que la ligne `compinit` de `~/.zshrc`.
  Le script est dans la langue du système au moment de l'installation : à vérifier au Mac sous Homebrew.
- **La 0.1.1**, binaire Python sous GPL v3, est décrite par le `packaging/NOTICE` du tag `v0.1.1`, et son mode d'emploi par le README de ce tag.
- **Le binaire Rust se publie par un tag** (#223), sous MIT et LGPL 2.1 ou ultérieure : `packaging/NOTICE` et `packaging/licenses/` le décrivent ([11](11-migration-rust.md#licences), #224).
- **Publier une version** :
  1. `workspace.package.version` de `Cargo.toml` vaut la version, et `Cargo.lock` suit.
  2. La section `[Non publiée]` de `CHANGELOG.md` devient `[X.Y.Z] - date`.
  3. En bas du CHANGELOG, le lien `[X.Y.Z]` compare `v<précédente>...vX.Y.Z` (`releases/tag/vX.Y.Z` pour la première version) ; si la section `[Non publiée]` ou son lien reste, ce lien compare `vX.Y.Z...HEAD`.
  4. Une fois ce commit sur `main` et la CI verte, on pousse le tag `vX.Y.Z`.
- `packaging/release-guards.sh` refuse le tag dans six cas, dans cet ordre :
  - il ne vaut pas `v` + la version de `Cargo.toml` ;
  - le CHANGELOG n'a pas sa section ;
  - le lien `[X.Y.Z]` manque, est en double ou ne part pas de la version précédente du CHANGELOG ;
  - la section `[Non publiée]` ou son lien existe, et ce lien ne compare pas `vX.Y.Z...HEAD` ;
  - le commit n'est pas sur `main` ;
  - `ci.yml` n'y est pas passé au vert.
- La release porte les trois archives, les deux sources LGPL (FFmpeg et openh264) et `SHA256SUMS`.
  Son texte est la section du CHANGELOG, extraite par `packaging/release-notes.sh`.
- **Mise à jour de la formule** : `bump-formulae.yml`, dans le tap, ouvre la PR `batdetect X.Y.Z` au premier passage de 6 h UTC après la release, ou à la demande par `workflow_dispatch`.
  - Il ne change que deux lignes : la ligne `version "…"` à la racine, et le `sha256` du bloc `on_macos do … on_arm do … end end`, recalculé sur l'archive.
  - La formule garde donc cette forme.
    `brew audit --strict` la juge redondante avec la version lue dans l'URL, comme pour la 0.1.1, mais le script en a besoin.
  - Quand la forme de l'archive ou de l'installation change, la formule se retouche à la main, comme à #225.
