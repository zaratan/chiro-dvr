# Développement

Ce document sert à qui va toucher au code de batdetect, agent ou développeur.
Il garde ce que le code ne dit pas : les contrats, les pièges et les choix en vigueur avec la mesure qui les justifie.
La carte des crates est dans [CLAUDE.md](../CLAUDE.md).
La logique de la détection et la valeur des réglages sont dans [detection.md](detection.md).
Les commandes de tous les jours et la CI sont dans [CONTRIBUTING.md](../CONTRIBUTING.md).

Le Rust doit détecter au moins aussi bien que la v0.1.1 sans la reproduire bit à bit.
La v0.1.1 est la dernière version en Python (tag `python-final`).
Les détections et les pistes se comparent par appariement avec des tolérances. Chaque écart se juge ensuite à l'œil.

Sur le Mac de développement, les commandes lourdes passent par `/Users/zaratan/Projects/dvr-wt/verrou lourd` (`MESURES.md`, hors dépôt).
Ailleurs elles se lancent sans lui.

## Construction

FFmpeg 7.1.1 et openh264 2.6.0 se construisent une fois par poste dans un cache partagé entre worktrees.
Ils se lient en statique dans le binaire. Rien du système n'entre dans la liaison hors d'une courte liste mesurée.

- **Toolchain** : `rust-toolchain.toml` fixe Rust 1.99.0 en édition 2024. C'est la seconde source de versions après `mise.toml`.
  Une montée de version se fait dans un changement à part.
- **Versions** : `packaging/ffmpeg-versions` donne les versions, les adresses et les sha256.
  La 7.1.1 est la version de la roue OpenCV de la v0.1.1. Avec elle le prototype lisait les mêmes images qu'OpenCV sur l'extrait.
  FFmpeg 9.0.2 s'en écartait jusqu'à 3 niveaux.
  L'épinglage rend aussi les drapeaux d'erreur (`decode_error_flags`) reproductibles d'un poste à l'autre.
  nasm 2.16.03 n'est construit que sur x86_64. Il n'entre pas dans le binaire.
- **`configure`** active le démultiplexeur mov, h264 et hevc, le protocole file, swscale, libopenh264 et le muxeur mp4.
  Sur macOS il ajoute VideoToolbox.
  `--disable-autodetect` écarte les bibliothèques du système.
  `--enable-pic` est explicite. Le gcc de Rocky 10 ne produit pas de PIC par défaut et Rust lie en PIE sous Linux.
- **Cache** : `packaging/ffmpeg-unix.sh` construit dans un dossier temporaire puis le pose par `mv`.
  Une construction concurrente garde la cible déjà posée.
  Le cache est `${XDG_CACHE_HOME:-~/.cache}/batdetect/ffmpeg/<empreinte>-<os>-<arch>/`.
  L'empreinte hache le script et les clés `FFMPEG_`, `OPENH264_` et `NASM_` de `ffmpeg-versions`.
  Chaque worktree a un lien `.ffmpeg` vers sa cible, ignoré par git.
  Les `.pc` sont réécrits en `${pcfiledir}/../..` pour marcher à travers le lien.
  `--fingerprint` imprime l'empreinte, clé du cache CI. `--link` (`mise run rust:link`) pose le lien et échoue si le cache manque.
  Ainsi `mise run check` ne construit jamais FFmpeg.
- **Liaison** : `.cargo/config.toml` force `PKG_CONFIG_LIBDIR` vers `.ffmpeg/lib/pkgconfig` et vide `PKG_CONFIG_PATH`, `CPATH` et `C_INCLUDE_PATH`.
  Le FFmpeg de Homebrew ne peut donc pas être pris. `cargo` seul suffit.
  On n'utilise pas `FFMPEG_DIR` : dans ffmpeg-sys-next 9.0.1 cette route oublie openh264 et la bibliothèque C++.
  On n'utilise pas la feature `static` de ffmpeg-next. Elle ajoute 16 frameworks macOS inutiles dont QTKit, absent en arm64.
  Sur macOS le binaire ne lie en dynamique que libc++, libSystem, libiconv, libobjc et six frameworks.
  libobjc et Foundation viennent de ctrlc par objc2.
  Sous Linux il ne lie que libstdc++, libgcc_s, libc et libm. `packaging/check.sh` contrôle ces listes.
- **Version liée** : FFmpeg est construit avec `--extra-version=<empreinte>`. `batdetect --version` affiche donc `FFmpeg 7.1.1-<empreinte>`.
  Un test compare cette empreinte à la cible de `.ffmpeg`.
- **Reconstruction** : cargo ne suit pas les `.a`. Ils sont lus à l'édition de liens et rien ne se refait quand ils changent.
  Le script lance donc `cargo clean -p ffmpeg-sys-next` dans chaque profil quand `.ffmpeg` change de cible, avant de changer le lien.
  Après une montée de version de FFmpeg on fait un `cargo clean` complet, car les liaisons de bindgen changent.
  Un arrêt brutal peut laisser un dossier `.build.*` dans le cache. Il se supprime à la main.
- **Cible processeur** : le Rust se compile en `x86-64-v2` sous Linux et Windows (`.cargo/config.toml`).
  Sans cela le min/max u16 du réseau de médiane ne serait pas vectorisé.
  Le code C de FFmpeg et d'openh264 suit une cible épinglée dans `ffmpeg-unix.sh`, donc dans l'empreinte : `-march=x86-64-v2` sous Linux et `-mcpu=apple-m1` sous macOS.
  Sans elle il suivrait le compilateur du poste.
  Le gcc de Rocky 10 vise `x86-64-v3` et le FFmpeg de lumiel s'arrêtait sous Intel SDE en Nehalem.
  Sous Windows MSVC compile en SSE2 par défaut.
  Ne jamais poser `RUSTFLAGS` : il remplacerait ce réglage.
- **macOS 14 au minimum** : `MACOSX_DEPLOYMENT_TARGET=14.0` vaut pour openh264, FFmpeg et rustc.
- **Profils** : `release` a `lto = "fat"`, `codegen-units = 1` et `panic = "unwind"`.
  `ci` hérite de `release` sans LTO, en 16 unités, avec `overflow-checks` et `debug-assertions`. Les tests tournent en `ci`.
  Les temps se mesurent en `release`.

### Empaquetage

```bash
mise run package          # binaire release, puis dist/batdetect-<cible>.tar.gz (zip sous Windows)
mise run package:check    # contrôles du livrable sur cette archive
```

- Les archives sont `batdetect-darwin-arm64.tar.gz`, `batdetect-linux-x86_64.tar.gz` et `batdetect-windows-x86_64.zip`.
  Elles contiennent le binaire, `LICENSE`, `NOTICE` et `licenses/`. Leur nom n'a pas de version parce que la formule Homebrew en dépend.
- `packaging/package.sh` lit la cible dans la ligne `host:` de `rustc -vV`. Sous MSYS2 `uname` vaut `UCRT64_NT-…`.
  Sans `BATDETECT_VERSION` il prend la version de `[workspace.package]`, celle que `--version` imprime.
- `packaging/check.sh ARCHIVE [VERSION]` joue tous les contrôles. Il échoue à la fin si un seul a échoué.
  Chaque contrôle échoue seul sur une archive altérée.

  | Contrôle | Ce qu'il mesure |
  |---|---|
  | C1 | l'archive contient le binaire, `LICENSE`, `NOTICE` et `licenses/`, rien d'autre |
  | C2 | aucun lien dynamique hors d'une liste mesurée par système : `otool -L`, `readelf -d` (lignes NEEDED), `dumpbin -dependents` |
  | C3 | `minos` ≤ 14.0 sous macOS ; sous Linux, `GLIBC_` ≤ 2.35, `GLIBCXX_` ≤ 3.4.30, `CXXABI_` ≤ 1.3.13 et `GCC_` ≤ 12.0.0, la bibliothèque de l'Ubuntu 22.04 ; sous Windows, versions du système et du sous-système de l'en-tête PE ≤ 10.0 (`dumpbin -headers`), celle de Windows 10, que Windows 11 annonce aussi ; C3-cpu, sous Linux et Windows : l'extrait tourne sous Intel SDE en mode Nehalem (`-nhm`, qui couvre x86-64-v2) et donne le CSV de C6 |
  | C4 | `--version` donne `batdetect <VERSION>`, puis le FFmpeg épinglé, avec l'empreinte du script de construction et la LGPL |
  | C5 | le bloc `Rust crates:` de `NOTICE` égale `cargo tree -p batdetect -e normal,no-proc-macro` sur les trois cibles de `packaging/about.toml` ([Licences](#licences)) |
  | C6 | le CSV de l'extrait, en anglais, est identique à l'octet à `tests/fixtures/rust/video_092_original_3m24-4m05/tracks.csv` |

- C3-cpu traite l'extrait parce que `--version` passe sur un binaire fautif. Le code fautif ne tourne qu'au traitement.
- **Intel SDE** : `packaging/sde.sh` télécharge le kit 10.13.1 de `packaging/sde-versions` et vérifie son sha256.
  Il l'extrait dans `${XDG_CACHE_HOME:-~/.cache}/batdetect/sde/`.
  SDE présente le CPUID de Nehalem et arrête le programme sur toute instruction que Nehalem n'a pas.
  Sa licence permet l'usage et la redistribution sans modification. Elle est acceptée depuis le 17 octobre 2026.
  Sous Ubuntu, Pin demande `kernel.yama.ptrace_scope=0`. `package.yml` le pose par `sudo sysctl`.
  Le kit n'a pas de cache CI : `sde.sh` fait confiance à un kit déjà extrait et un cache restauré contournerait l'épinglage.
  Une release dépend donc du miroir d'Intel. S'il manque, elle échoue en clair (`could not fetch Intel SDE`).
  SDE ne tourne ni sous macOS arm64 ni sous Rocky 10, dont la glibc exige x86-64-v3.
  Là `mise run package:check` passe au rouge sur la sonde `sde64 -nhm -- /bin/true`.
  La preuve se fait dans les tâches `package / build` d'Ubuntu et de Windows.

### Windows

Seule la CI construit la cible `x86_64-pc-windows-msvc`, dans `rust-windows`, `rust-windows-slow` et `package.yml`.
`mise` ne tourne pas sous Windows.
La construction sur un poste Windows est l'objet de #226.

- `packaging/ffmpeg-windows.sh` tourne sous MSYS2 avec MSVC dans le `PATH`.
  Il lit `ffmpeg-versions` plus deux clés `WINDOWS_NASM_*`, hors de l'empreinte Unix.
  Ses fonctions communes sont recopiées du script Unix, car un fichier commun changerait l'empreinte Unix.
  Un correctif du script Unix se reporte à la main.
- **Même bibliothèque C partout** : openh264 et FFmpeg se compilent en `/MT`. Le Rust prend `+crt-static`.
  Le binaire ne demande ni `vcruntime140.dll` ni l'UCRT. Chaque `.lib` demande `LIBCMT` et jamais `MSVCRT` (`dumpbin -directives`).
- **Liaison** : FFmpeg nomme ses archives `libavcodec.a` même sous MSVC.
  rustc passe `avcodec.lib` à `link.exe`, donc le script renomme `libX.a` en `X.lib`.
  pkg-config-rs ignore en silence un `bcrypt.lib` nu et lit `-libpath:` comme `-l` suivi de `ibpath:`.
  Le script réécrit les `.pc` en `-lbcrypt` et `-L`.
- **Pièges** :
  - ffmpeg-sys-next essaie vcpkg avant pkg-config. `msvc-dev-cmd` définit `VCPKG_ROOT`, donc la tâche pose `VCPKGRS_DISABLE=1`.
  - Le `link.exe` de coreutils est renommé avant toute construction. Le script refuse un `link` qui n'est pas celui de Microsoft.
  - MSYS2 réécrit en chemin tout argument qui commence par `/`. Les options de MSVC s'écrivent donc `-x`.
    `cmd /c` passe par `MSYS2_ARG_CONV_EXCL='*'`.
  - openh264 se compile sans `-Zi -FS`. Avec `-Zi` chaque édition de liens affichait 65 avertissements LNK4099.
  - Le lien `.ffmpeg` est une jonction NTFS (`mklink /J`). `fs::read_link` la lit, donc le test de la version liée vaut aussi ici.
  - Le multifil (`HAVE_W32THREADS 1`) n'est gardé qu'à la compilation. Aucun test ne vérifie que le décodage tourne sur plusieurs fils.
- **Fins de ligne** : l'image Windows extrait le texte en CRLF.
  `.gitattributes` force `eol=lf` pour les scripts, les fichiers de versions, la référence v0.1.1 et les textes de licence.
  Les textes du zip sont ainsi ceux des autres archives.

## Licences

Le code de batdetect est sous **MIT** (`LICENSE`). FFmpeg est sous **LGPL 2.1 ou ultérieure** (décision du 16 octobre 2026, #224).
Le livrable porte aussi openh264 en BSD-2, Roboto en OFL-1.1, la bibliothèque standard de Rust et 83 crates tierces.
`packaging/NOTICE` décrit le binaire composant par composant.
`packaging/licenses/` tient les textes, tirés de la source épinglée de chaque composant :

| Fichier | Source |
|---|---|
| `FFmpeg-LGPL-2.1.txt` | `COPYING.LGPLv2.1` de `ffmpeg-7.1.1.tar.xz` |
| `openh264-BSD-2-Clause.txt` | `LICENSE` de l'archive openh264 2.6.0 |
| `Roboto-OFL-1.1.txt` | copie de `crates/draw/fonts/OFL.txt` |
| `Rust-MIT.txt`, `Rust-Apache-2.0.txt` | `LICENSE-MIT` et `LICENSE-APACHE` de `rust-lang/rust` au tag 1.99.0 |
| `crates.txt` | généré par `mise run notice` |

- **FFmpeg** est construit sans `--enable-gpl` ni `--enable-nonfree`. Lié en statique, il oblige à fournir de quoi relier.
  Chaque release joint `ffmpeg-7.1.1.tar.xz` et les sources du tag sont publiques.
  `NOTICE` dit comment relier : changer `FFMPEG_URL` et `FFMPEG_SHA256` dans `ffmpeg-versions`, lancer le script de construction puis `cargo build`.
- **openh264** est construit depuis ses sources, jointes à la release.
  Il n'a donc **pas** la licence de brevets H.264 que Cisco n'accorde qu'à ses binaires.
  Les brevets ont été acceptés le 16 octobre 2026. `NOTICE` dit qu'aucune licence de brevet n'est fournie.
- **Roboto** vient de `ofl/roboto/Roboto[wdth,wght].ttf` du dépôt `google/fonts` au commit `6183fc0` (sha256 `d7598e12…872a134`).
  Elle n'a pas de nom de police réservé.
- **Crates livrées** : l'union de `cargo tree -p batdetect -e normal,no-proc-macro --target <triple>` sur les trois triples de `packaging/about.toml`, hors workspace.
  `mise run notice` croise cette liste avec le JSON de `cargo-about`.
  Il réécrit le bloc `Rust crates:` de `NOTICE` et `licenses/crates.txt`.
  `cargo-about` ne sait pas écarter les proc-macros : c'est la liste de `cargo tree` qui fait foi.
  Une dépendance changée sans `mise run notice` fait échouer C5. C5 a besoin du réseau pour lire les manifestes des trois cibles.
  Un texte que `cargo-about` ne trouve pas seul se donne par `clarify` dans `about.toml`.
  La tâche échoue sur un modèle de texte (`<year>`, `<copyright holders>`).
  Dans une expression « OU », l'ordre de `accepted` choisit la licence retenue.
- **Licences admises** : `deny.toml` n'admet que MIT, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, WTFPL, Zlib et Apache-2.0 (avec ou sans LLVM-exception).
  Chacune sert au moins une crate de `Cargo.lock`. Les autres licences n'y apparaissent qu'en alternative dans une expression « OU ».

## Lints

Les lints se règlent dans `[workspace.lints]` du `Cargo.toml` racine et jamais par attribut.

- **Activés** : `clippy::pedantic` et une liste de restrictions qui porte la charte (`unwrap_used`, `print_stdout`, `allow_attributes`…).
  La table fait foi.
- **Sept lints de `pedantic` coupés** dans la table.
  Les quatre `cast_*` tombaient sur des calculs de pixels où la règle 15 suffit.
  `missing_errors_doc` et `missing_panics_doc` suivent la règle du zéro commentaire.
  `must_use_candidate` signalerait presque chaque accesseur.
- **Refusés** : `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, `missing_docs`, `nursery` et `cargo`.
- **`media` est la seule crate avec `unsafe`.** Sa table est la copie exacte de celle du workspace sans `unsafe_code`.
  Cargo refuse `workspace = true` avec une surcharge locale. Un contrôle compare les deux tables.
- **Règles vérifiées par script** (`mise run rust:rules`, dans `rust:static`) :
  - Aucun `#[allow]`, `#[expect]` ou `cfg_attr(…allow…)`.
  - Chaque crate porte `[lints] workspace = true` sauf `media`.
  - Aucun `ffmpeg_next` hors de `media`. Les dépendances des crates ne viennent que du workspace.
  - Au plus un appel à `catch_unwind` hors des tests, sous `crates/cli/src/`. Pas de `panic = "abort"` dans les profils.
  - Au plus 2 % de lignes de commentaire dans `crates/`.
  - Aucun mot d'imitation (`round_ties_even`, `opencv`, `numpy`, `python`, `pairwise`).

## Charte

But : détecter au moins aussi bien que la v0.1.1, en Rust idiomatique. On ne recopie ligne à ligne ni le Python ni le prototype.

Outillage :

1. Toolchain de `rust-toolchain.toml`, édition 2024. `cargo fmt --all --check` ne signale rien.
2. `cargo clippy --workspace --all-targets --locked -- -D warnings` sans avertissement.
   Les lints se règlent dans `[workspace.lints]` du `Cargo.toml` racine et nulle part ailleurs.
3. Zéro `#[allow]`, `#[expect]` ou `cfg_attr(…allow…)` même si clippy le suggère.
   Un lint gênant se résout en réécrivant le code. S'il paraît faux, on demande.
4. Chaque crate a `[lints] workspace = true`. Seule `media` a sa propre table : la copie exacte de celle du workspace sans `unsafe_code`.

Pas d'imitation :

5. Ni Python ni numpy ni OpenCV ne s'imitent.
   On n'utilise pas `round_ties_even` pour copier `round()` et on ne recopie pas l'ordre des opérations.
   Le calcul s'écrit simple et juste. L'écart se mesure par appariement avec tolérances (`reference`) puis se juge.
   **Exception** (#118) : le gris de la détection reprend la table mesurée sur OpenCV et l'arrondi à 8 bits après la réduction.
   Les seuils de la v0.1.1 sont en effet réglés sur ce gris ([Gris de la détection](#gris-de-la-détection)).

Sûreté et erreurs :

6. `unsafe` seulement dans `media`. Un bloc contient une seule opération et porte une ligne `// SAFETY:` qui dit pourquoi le contrat tient.
7. Aucun type de `ffmpeg_next` ne sort de `media`. Une fonction `extern "C"` ne peut pas paniquer.
8. Chaque bibliothèque a son `enum` d'erreur (`thiserror`). Cet `enum` garde la cause (`#[source]`).
   Ni `anyhow` ni `Box<dyn Error>` ni erreur `String`. Une erreur par module dans `media`.
9. Une entrée refusée (vidéo abîmée, format non géré, réglage hors bornes) rend une erreur typée et jamais une panique.
   `unwrap`, `expect`, `panic!`, `todo!` et `dbg!` sont refusés hors des tests.
10. Une panique signale un bug. Seule la commande la rattrape, une fois par vidéo.
    Les fils passent par `std::thread::scope` ou rayon. Ils ne sont jamais détachés.
11. Aucun résultat jeté : ni `let _ =` sur un `Result` ni `.ok();`.
12. Ni `println!` ni `eprintln!` : on écrit par `writeln!` sur un flux verrouillé et on traite son erreur.
    Tout texte affiché vit dans un seul module de la commande.

Types et calcul :

13. L'index d'image et le `pts` ont chacun leur type (`FrameIndex`, `Pts`).
    L'index suit l'ordre d'affichage, celui où le décodeur rend les images. Pas de sentinelle : `Option<Pts>`.
14. Tout est en pixels de la vidéo d'origine hors de la détection. Seul `WorkScale` passe des pixels de travail aux pixels d'origine.
15. Conversions : `From` quand elle existe.
    Un `as` qui perd de l'information suit un `round` ou un `clamp` visible. Ou bien il vit dans une fonction de conversion nommée.
16. Même entrée, même sortie quel que soit le nombre de fils.
    Aucune itération sur `HashMap` ou `HashSet` n'atteint une sortie. Aucune somme flottante n'est parallèle.
    La règle vaut pour ce qui sort du moteur, plages abîmées exclues.
    Les pixels que FFmpeg dissimule sur une image abîmée dépendent du nombre de fils de décodage ([`engine`](#engine)).
17. Une configuration se construit par une fonction qui valide et rend un `Result`. Les comptes non nuls sont des `NonZero*`.
18. Boucles chaudes : aucune allocation par pixel ni par image. Les tampons sont réutilisés.
    On y préfère les itérateurs, `zip` et `chunks_exact` aux indices.

Forme et tests :

19. Un module a une seule responsabilité. Ses tests sont dans `src/<module>/tests.rs`. Pas de `mod.rs`.
    Fonctions de 100 lignes et 7 arguments au plus.
20. `pub(crate)` par défaut. `Debug` sur tout type public. Pas de `pub use` dans `media/src/lib.rs`.
21. Zéro commentaire, sauf `// SAFETY:` et une ligne `///` sur un élément public dont le nom ne dit pas le contrat externe.
    Au plus 2 % des lignes.
22. Les tests sont sans commentaire et nommés par le pourquoi. `unwrap`, `expect` et `panic!` y sont permis.
    Les flottants se comparent avec une tolérance.
    Pas de faux FFmpeg : une vraie vidéo ou une vidéo fabriquée par openh264. Couverture d'au moins 95 %.
    Les tests lents vont dans `tests/slow_*.rs`, les réglages dans `tests/settings_*.rs` et les tests de la 092 dans `tests/reference_*.rs`.
    On prend proptest là où un oracle simple existe.
23. Une dépendance se déclare une fois dans `[workspace.dependencies]` puis `x.workspace = true`.
    Une dépendance hors de la liste d'une issue se demande avant. `cargo deny check` et `cargo shear` restent verts.

## Dépendances

- `--locked` est partout.
  `cargo deny check` échoue sur un avis de sécurité, une licence hors liste, une source inconnue, un paquet retiré ou une version sans borne.
  Il ne fait qu'avertir sur une version en double.
- `serde_json` a la feature `float_roundtrip` pour qu'un flottant de la référence se relise au bit près.
- **Pas d'`ab_glyph`** : il tire `ttf-parser`, que RUSTSEC-2026-0192 déclare non maintenu. L'avis n'est pas ignoré.
  `fontdue` et `rusttype` le tirent aussi. Les polices passent donc par `skrifa` et le tracé par `tiny-skia` sans sa feature `png`.
- `png` sert en direct dans `summary`. `draw` reste sans format de fichier.
- `clap` sert en builder : la commande se construit à l'exécution avec les textes de la langue, ce que `derive` ne fait pas.
- Les langues passent par `fluent-bundle`, `unic-langid` et `sys-locale`.
  `i18n-embed-fl` et ICU4X tireraient des crates dont la commande n'a pas besoin.

## Tests

Le niveau d'un test se choisit par le nom de son fichier. Ajouter un test ne demande de toucher ni la CI ni `mise.toml`.
`mise run check` joue `rust:static`, `rust:test` et `page:js` avant de rendre.
`mise run check:ci` joue `rust:coverage`, `rust:slow` et `rust:settings`.

| Niveau | Fichiers | Commande | Contenu |
|---|---|---|---|
| rapide | `src/<module>/tests.rs`, `tests/*.rs` | `mise run rust:test` (sans couverture), `mise run rust:coverage` (couverture) | vidéos fabriquées par openh264, binaire lancé pour de vrai |
| slow | `tests/slow_*.rs`, `#[ignore = "slow"]` | `mise run rust:slow` | l'extrait LFS : 1 230 images, 10000/333, 1440×1080, plage `tv` ; les tests de mémoire `slow_memory_flash`, `slow_memory_flash_wide` et `slow_memory_flash_long` (`dhat`, 960 et 1440 pixels de travail, 61 échantillons), hors de la couverture ; sous macOS, une image exFAT montée par `hdiutil` (`files/tests/slow_exfat.rs`, #236) |
| settings | `tests/settings_*.rs`, `#[ignore = "settings"]` | `mise run rust:settings` | les 16 cas du banc d'essai sur l'extrait et sa 720×540 (`settings_agreement`), sous Linux seulement en CI (#167) |
| reference | `tests/reference_*.rs`, `#[ignore = "reference"]` | `mise run rust:reference` | la 092 : 9 011 images, cadence moyenne 90110000/3000927, aux défauts et à 1440 pixels de travail (#151), en un seul binaire (`cli/tests/reference_092.rs`) |
| matériel | `tests/hardware_*.rs`, `#[ignore = "hardware"]` | étape d'essai de la CI macOS | VideoToolbox sans repli logiciel |
| local | `tests/local_*.rs`, `#[ignore = "local"]` | `BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=… cargo test --workspace --locked --profile ci --test 'local_*' -- --include-ignored` | les 26 vidéos de la référence, hors dépôt, jamais en CI ; récupérées du seau sur une autre machine par `mise run rust:data` ([emplacements](../tests/fixtures/reference/README.md#récupération-sur-une-autre-machine)) |

- La couverture se mesure sur les tests rapides en profil `ci`.
  Les lignes hors couverture sont des erreurs de FFmpeg qu'aucune vidéo de test ne provoque.
- Les tests de mémoire restent hors de la couverture.
  Sous l'instrumentation le premier prenait 83 s au lieu de 1 s sans couvrir une ligne de plus.
- Sous l'instrumentation, chaque voie du tri de `Network::order` incrémente un compteur atomique partagé par tous les fils.
  Le test du détecteur à 61 et 81 échantillons tourne donc sur une image de 144×24.
- Un test rapide peut relancer son propre binaire sur un test `#[ignore = "run by …"]` pour lire ce qu'il écrit sur stderr.
- Les fixtures se trouvent par `reference::fixture::fixture`. Cette fonction refuse un pointeur LFS non tiré avec le conseil `git lfs pull`.
- `cli/tests/reference_092.rs` passe la 092 aux défauts et à 1440 pixels de travail en parallèle, dans l'initialisation d'un `OnceLock` que relisent ses cinq tests.
  libtest lancerait les tests dans l'ordre de leur nom.
  Une erreur ou une panique y est rangée comme résultat, car un `OnceLock` qui panique se relancerait pour chaque test.
  Un test lancé seul par filtre fait quand même les deux passages.
  La 1440 est dans ce binaire pour partager le processus, d'où la dépendance de test `cli` → `compare`.

### Fixtures du Rust

- `tests/fixtures/rust/video_092_original_3m24-4m05/frames.jsonl` : les détections de l'extrait par le Rust.
  `slow_extract` les redemande au bit près sur les trois systèmes.
  Elles s'écrivent sur le Mac depuis la racine du dépôt par `verrou lourd cargo run --profile ci -p compare --bin compare -- extract --frames tests/fixtures/rust`.
  Un écart se signale.
- `tests/fixtures/rust/video_092_original_3m24-4m05/tracks.csv` : le tableau des pistes de l'extrait que C6 contrôle.
  Il se refait en anglais par le binaire `release` :
  `verrou lourd env -u LANGUAGE -u LC_ALL -u LC_MESSAGES LANG=C target/release/batdetect tests/fixtures/video_092_original_3m24-4m05.mp4 -o out/csv`.
  On copie ensuite `out/csv/video_092_original_3m24-4m05/video_092_original_3m24-4m05_tracks.csv` à sa place.
  Ses lignes finissent en CRLF comme tout tableau produit. `.gitattributes` le marque `-text`.
  Une PR qui change le suivi ou la détection de l'extrait doit refaire ce fichier.
- Cinq petites fixtures hors LFS ont été faites une fois par le ffmpeg 9.0.2 de Homebrew (libx264) :
  - `full_range.mp4` (1,9 Ko, `yuvj420p`, plage `pc`) : `ffmpeg -f lavfi -i color=gray:size=1440x1080:rate=30 -frames:v 2 -c:v libx264 -bf 0 -pix_fmt yuvj420p -color_range pc full_range.mp4`
  - `full_range_midway.mp4` (3,0 Ko, 2 images en plage non précisée puis 2 en `pc`) : `tv.h264` et `pc.h264` sortent de la commande précédente avec `-x264-params repeat-headers=1 -f h264`.
    L'un est en `-pix_fmt yuv420p -color_range tv` et l'autre en `-pix_fmt yuvj420p -color_range pc`.
    Ensuite : `cat tv.h264 pc.h264 > midway.h264 && ffmpeg -framerate 30 -i midway.h264 -c copy full_range_midway.mp4`
  - `two_gops.mp4` (5,9 Ko, 320×240, 20 images, images-clés en 0 et 10, perte au début par la couture) : `ffmpeg -f lavfi -i testsrc=size=320x240:rate=30 -frames:v 20 -c:v libx264 -bf 0 -g 10 -keyint_min 10 -sc_threshold 0 -crf 40 -pix_fmt yuv420p -color_range tv two_gops.mp4`
  - `spot.mp4` (3,1 Ko, 320×240, 40 images, plage `tv`) montre un carré clair qui avance de 5 pixels par image sur un fond gris.
    Une bande en haut est à Y = 0 et une en bas à Y = 254, hors de la plage `tv`.
    Commande : `ffmpeg -f lavfi -i "color=c=0x606060:s=320x240:r=30" -f lavfi -i "color=c=white:s=6x6:r=30" -filter_complex "[0]drawbox=x=0:y=0:w=320:h=24:color=black:t=fill,drawbox=x=0:y=216:w=320:h=24:color=white:t=fill[bg];[bg][1]overlay=x='40+5*n':y=110:eval=frame,format=yuv420p,lutyuv=y='if(lt(val\,20)\,4\,if(gt(val\,230)\,250\,val))'" -frames:v 40 -c:v libx264 -preset medium -crf 18 -g 20 -bf 0 -pix_fmt yuv420p -color_range tv -colorspace bt709 -color_primaries bt709 -color_trc bt709 spot.mp4`
  - `spot_full_range.mp4` (3,1 Ko) est le même flux remuxé sans réencodage et marqué plage pleine (ffprobe : `yuvj420p`, `color_range=pc`).
    Commande : `ffmpeg -i spot.mp4 -c copy -bsf:v h264_metadata=video_full_range_flag=1 spot_full_range.mp4`.
    `tests/full_range.rs` exige que leurs détections soient non vides et identiques au bit près.

### Caches de la CI

La table des tâches est dans [CONTRIBUTING.md](../CONTRIBUTING.md#ci).

- `rust` et `rust-slow` ne partagent aucune compilation.
  La couverture compile dans `target/llvm-cov-target`, les tests lents et le binaire dans `target/ci`.
- Le binaire contrôlé se construit par `--workspace --bin batdetect`.
  Avec `-p batdetect` les features de `syn` changent et les liaisons de FFmpeg se recompilent.
- Quand l'empreinte de FFmpeg change, chaque tâche le construit. Seules `rust` et `rust-windows` sauvent ce cache.
- Le cache de `target/` a une entrée par tâche.
  Sa clé porte la tâche, l'OS, l'architecture, `ImageOS`, **l'empreinte de FFmpeg**, `rust-toolchain.toml` puis `Cargo.lock`.
  L'empreinte y est parce que cargo ne suit pas les `.a` ([Construction](#construction)).
  En PR seulement, une clé de repli prend tout sauf `Cargo.lock`. Une PR qui change une dépendance ne recompile ainsi que ce qui a changé.
  Sur `main` il n'y a pas de repli : sans cette remise à zéro, le cache garderait les versions remplacées et grossirait.
  `rust-settings` ne sauve rien et restaure l'entrée de `rust-reference`.

### Tests figés

Six cas d'extraits et de zooms ont été jugés à l'aveugle le 15 octobre 2026 contre les sorties de la v0.1.1, puis figés.
Le verdict est « aussi bien » pour les six.

| Cas | Vidéo | Piste (début v0.1.1) | Fichier |
|---|---|---|---|
| bordure avec zoom | 125 | 0:28.41 | zoom |
| passage rapide (1 926 px/s) | 092 | 3:48.29 | extrait |
| piste courte (6 images) | 097 | 1:54.63 | extrait |
| piste lente | 092 | 3:29.61 | extrait |
| boîte interpolée (30 images) | 097 | 1:48.54 | extrait |
| zoom ×4 | 125 | 0:47.12 | zoom |

Les images résumé de la 097 et de la 089-091 ont aussi été jugées.
VideoToolbox n'est pas reproductible d'une machine à l'autre. On fige donc des empreintes d'images, jamais celle d'un mp4.

- **Extraits et zooms** : `render::drawn::drawn` (feature `testing`) donne les images dessinées que `render` pousserait à l'encodeur.
  Chaque fichier a une empreinte `of_sequence`.
  Les pistes viennent de `batdetect::process::tracked`, que `process_detected` appelle aussi.
  `cli/tests/slow_frozen.rs` tourne sur l'extrait dans la CI des trois systèmes : tiny-skia et swscale (`BITEXACT`) dessinent au pixel près partout.
  `cli/tests/local_frozen.rs` couvre les six cas sur leurs vidéos entières, jamais en CI.
- **Images résumé** : les pixels de chaque PNG écrit en français (`of_pixels`) et le fond de chaque vidéo, dans `cli/tests/local_frozen.rs`.
  Le test ne peut pas vivre dans `summary` : le fond est la médiane de la détection et `summary` n'a ni FFmpeg ni moteur.
  La 097 a une empreinte par système, car Linux y voit une image abîmée de plus que macOS vers 182 s.
- Un changement voulu du dessin, de la détection ou du suivi casse ces empreintes.
  On les refige en le disant, après un nouveau jugement à l'œil si le dessin a changé.

## Sorties

Pour chaque vidéo, l'outil écrit un tableau des pistes, les images résumé, les extraits et `params.json`, qui garde les réglages employés.
La liste des fichiers produits est dans [lire les résultats](lire-les-resultats.md#où-sont-les-résultats).

### `params.json`

`params.json` s'écrit en dernier, une fois les extraits finis. C'est la marque d'un dossier terminé que relit la reprise.
Il donne :

- les réglages employés, la version et le FFmpeg lié.
- les encodeurs retenus (`encoders`).
- l'état : `done`, ou `too_many_tracks` au-delà de `--max-tracks`.
- le nombre de pistes, la durée, le temps non analysé et le temps de calcul (`track_count`, `length_s`, `not_analysed_s`, `elapsed_s`, en secondes non arrondies).
- la taille de la vidéo en octets (`video_bytes`).
- la langue des noms (`language`, `fr` ou `en`).
- l'heure locale de la fin du calcul (`date`).
- les périodes ignorées (`ignored_s`) et illisibles (`damaged_s`, marge comprise).
- `damaged_frames` : le nombre d'images signalées par le décodeur ou qui suivent un trou dans les horodatages, 196 sur la 122.
  Ce n'est pas le nombre d'images écartées, 6 043 sur la 122.

`tracks` liste les pistes :

- numéro (`id`).
- début et durée non arrondis (`start_s`, `duration_s`).
- couleur de la trajectoire sur l'image résumé en RVB (`color`, `null` au-delà du garde-fou, qui n'a pas d'image résumé).
- image résumé qui la dessine (`summary`, par exemple `video_097_resume_3_sur_8.png`).
- chemins de l'extrait et du zoom écrits, relatifs au dossier de la vidéo (`null` quand il n'y en a pas).
- point de départ en pixels entiers de la vidéo d'origine (`start_xy`).
  Il aide la page à rattacher un verdict à sa piste et manque aux dossiers écrits avant #228.

Pour que la page redessine l'image résumé sans rien recalculer, `params.json` donne aussi :

- le fond, la scène sans trace (`background`, par exemple `video_097_fond.png`).
- la mise en page de chaque image résumé (`summaries`) :
  - `file`.
  - `window_s` : la tranche.
  - `part` : la partie, en `[n, total]` ou `null`.
  - `ignored_s` et `damaged_s` : les périodes ignorées et illisibles, coupées à la tranche.
  - `other_starts_s` : les débuts des pistes des autres parties, au centième.
  - `radius` : le rayon des pastilles, en pixels de la vidéo.
- pour chaque piste dessinée, sa trace (`trace`) :
  - `points` et `head` : la trajectoire et les trois points de la flèche, en chaîne `"x,y x,y …"` de pixels entiers de la vidéo, sans point répété.
  - `dot` : le centre de la pastille.
  - `anchor` : le départ du trait de rappel quand la pastille est écartée de la trajectoire, sinon `null`.

Une piste non dessinée a `trace` à `null`.
Un dossier écrit avant #259 n'a ni `background` ni `summaries`. La page montre alors l'image résumé.

### Tableau des pistes

Ses colonnes, dans cet ordre :

- `piste` : le numéro.
- `début` et `fin` : en `h:mm:ss`, tronqués à la seconde comme le nom de l'extrait.
- `durée (s)`.
- `points` : le nombre de détections.
- `corde (px)` : la distance de bout en bout.
- `chemin (px)` : le chemin parcouru.
- `vitesse (px/s)` : chemin ÷ durée.
- `surface max (px)` : la plus grande tache, en pixels d'origine.
- `amplitude (niveaux de gris)` : le plus fort écart au fond.
  Il est mesuré sur l'image filtrée et reste donc plus bas que l'écart brut pour une petite tache.
- `images comblées` : les images comblées par interpolation à l'affichage.
- `début (s)` : le début en secondes, au centième.

Un début dans le dernier centième d'une seconde s'écrit `0:00:04` d'un côté et `5,0` de l'autre : `début` tronque, `début (s)` arrondit.
`0:04` serait lu par LibreOffice comme 0 h 04 min : d'où `0:00:04`.
Le tableau suit la langue des messages.
En français, `_pistes.csv` commence par un BOM UTF-8 pour qu'Excel lise les accents.
Il sépare par un point-virgule et écrit la virgule décimale.
En anglais, `_tracks.csv` n'a pas de BOM, sépare par une virgule et écrit le point.
Ses en-têtes sont `track`, `start`, `end`, `duration (s)`, `points`, `chord (px)`, `path (px)`, `speed (px/s)`, `max area (px)`, `amplitude (grey levels)`, `filled frames`, `start (s)`.
Les fins de ligne sont en CRLF dans les deux langues.
Le tableau appartient à l'outil : un recalcul le remplace sans le lire.
Les verdicts n'y sont pas. Ils se notent dans la page et vivent à côté d'elle dans `verdicts.csv`, que l'outil lit sans jamais l'écrire.

### Temps écarté

Le temps illisible est signalé à part des mouvements.
Le bandeau de l'image résumé porte des lignes « illisible 1:12 - 1:15 », trois au plus, puis « + N autres, X illisibles en tout ».
La frise du temps le montre aussi.
Les périodes ignorées sont toujours signalées : bandeau et frise de l'image résumé, bilan de la soirée et `ignored_s`.

### Vidéo déjà faite

Une vidéo est sautée quand son dossier a un `params.json` terminé.
Il faut aussi que ni la vidéo, ni la version de batdetect, ni la langue des noms, ni les réglages n'aient changé.
La liste des encodeurs ne compte pas.
Rien n'est alors écrit ni effacé dans son dossier.
Le bilan reprend sa durée, ses pistes et son temps non analysé. Il affiche « — » en calcul et compte « N déjà faites ».
Une vidéo en échec est refaite à la relance.
Après un Ctrl-C, relancer la même commande reprend là où le lot s'est arrêté. `--force` refait toutes les vidéos.

### Ménage

Une fois la détection finie, chaque calcul efface d'abord `params.json` dans le dossier de la vidéo.
Un dossier interrompu ne passe ainsi jamais pour terminé.
Il efface ensuite les sorties des deux langues et celles des versions précédentes (`split/`, `<nom>.tracks.*`, `<nom>_boxes.mp4`).
Les sorties de la v0.1.1 restent où elles sont, dans le `out/` du dossier où la commande était lancée.

### Garde-fou

Au-delà de `max_tracks` pistes, ni les extraits, ni l'image résumé, ni son fond, ni la vidéo annotée ne sont produits.
`max_tracks` vaut 300 et 0 le désactive. Il se règle par `--max-tracks`.
Dans `params.json`, `background` est alors à `null`, `summaries` vide et `trace` à `null`.
Le tableau et `params.json` restent écrits. `params.json` garde le seuil (dans `RenderConfig`) et `"state": "too_many_tracks"`.
Sur une vidéo de bruit, l'image résumé se coupait en dizaines de parties et ne servait à rien.
300 est au-dessus de la vidéo la plus chargée sans explosion (148 pistes, 097).
Il est en dessous de la plus petite explosion connue (374 pistes, seuil 12 sans facteur de bruit).
Le seuil est fixe et suppose des clips d'environ 5 min comme ceux des jumelles. Pour une vidéo plus longue, on le monte avec `--max-tracks`.
Le bilan compte ces vidéos à part avec leur nombre de pistes. Une relance identique ne les refait pas.

### Place prise

`space` prévoit la place.
On l'a mesurée le 16 octobre 2026 sur les 26 vidéos avec openh264 et les réglages par défaut.
Les sorties vont de 0,002 à 6,9 fois la taille de la vidéo. Leur médiane est de 0,48.
Elles suivent le nombre et la durée des passages, qu'on ne connaît pas avant la détection.
Les soirées chargées (095 à 100, 78 à 149 pistes) dépassent 3 fois la vidéo. Les calmes restent sous 1 fois.
La vidéo annotée pèse de 0,64 fois la vidéo (120 sans piste) à 2,72 fois (090).
Un extrait pèse de 0,81 à 3,17 fois le débit de la vidéo d'entrée par seconde de fenêtre. Un zoom pèse de 0,29 à 2,63 fois ce débit.
Coefficients retenus (16 octobre 2026) :

- au lancement, 1 fois la vidéo : 18 soirées sur 26 tiennent dessous.
  Une soirée chargée le dépasse et c'est le contrôle par vidéo qui la protège.
- vidéo annotée : 2,8 fois la vidéo, le plus prudent mesuré arrondi au-dessus.
- par vidéo, avant les extraits : 3,2 fois le débit d'entrée par seconde d'extrait et 2,7 par seconde de zoom, les plus prudents mesurés arrondis au-dessus.

VideoToolbox écrit environ 0,77 fois ce qu'écrit openh264 : sur le Mac, ces coefficients sont plus prudents encore.

## Encodage

Sur le Mac, les extraits et la vidéo annotée passent par VideoToolbox matériel en q65 si un essai au démarrage réussit.
openh264 en QP 23 sert de repli ([`render`](#render)).
Ailleurs ils passent par openh264 seul. Les zooms passent toujours par openh264, chacun en travail à part.
Aucun de ces réglages n'est une option.

Mesure du 11 octobre 2026 sur le M1 Max, sur l'extrait. Le SSIM est celui du filtre `ssim` de FFmpeg contre l'original.

| Encodeur | Réglage | SSIM All (Y) | Taille | Encodage | CPU | Mémoire par encodeur |
|---|---|---|---|---|---|---|
| VideoToolbox matériel | q65, `allow_sw=0` | 0,9792 (0,9688) | 105,4 Mo | +0,4 s | ≈ 0 | ≈ 180 Mo |
| Logiciel d'Apple | `require_sw=1`, 20 Mb/s | 0,9809 (0,9714) | 105,5 Mo | 6,5 s | 54,8 s | ≈ 495 Mo |
| openh264 | profil high, QP 23 | 0,9799 (0,9698) | 136,9 Mo | 16,8 s | 16,3 s | ≈ 16 Mo |

- QP 23 atteint le SSIM de VideoToolbox q65. QP 24 donne 0,9761 et passe sous le plancher de 0,977.
  QP 23 est validé à l'œil sur la piste la moins contrastée de l'extrait.
- openh264 sert de repli plutôt que le logiciel d'Apple : 30 fois moins de mémoire, 3 fois moins de CPU et le même encodeur que sous Linux.
  Le prix est un fichier 30 % plus gros.
- **Couleur** : plage limitée et BT.709 pour la matrice, le transfert et les primaires, écrits dans le flux H.264.
  Les 27 vidéos connues sont toutes en BT.709.
- **Pièges de FFmpeg 7.1.1** :
  - `allow_sw=1` prend le matériel s'il répond.
    Seul `require_sw=1` force le logiciel d'Apple, qui refuse q65 (`-12900`) et ne marche qu'en débit fixé.
  - q65 s'écrit `global_quality = 65 × FF_QP2LAMBDA` plus le drapeau `QSCALE`. Ce réglage n'existe que sur arm64.
  - Sans réglage, openh264 encode en Constrained Baseline. Le profil high donne le même SSIM et un fichier 8 % plus petit.
  - `rc_mode=off` laisse le QP d'openh264 à 26. `rc_mode=quality` avec `qmin = qmax` applique le QP aux images I et P.
  - Les fils d'openh264 ne changent rien avec une tranche par image. Le parallélisme vient de plusieurs extraits à la fois.
  - `open_with` ignore les options inconnues sans erreur. Les tests relisent donc le profil et le PSNR.
- **Essai matériel** : `Codec::VideoToolbox` est le matériel seul.
  L'étape matérielle de la CI macOS encode les images 660 à 779 de l'extrait en q65 et les relit.
  Elle exige un PSNR de luminance supérieur à 37,9 dB (38,9 dB mesurés en q65, 34,9 dB sans).
  Le test lent d'openh264 exige 39,1 dB (39,53 à QP 23, 38,69 à QP 24).
  Cette étape bloque la CI. Si elle casse à cause d'une machine de GitHub, la question se rouvre.
  Sur `macos-latest`, le journal affiche « IOServiceMatchingfailed for: AppleM2ScalerParavirtDriver ». Le message est sans effet.
- **Sessions** : 17 sessions matérielles tiennent sur le M1 Max.
  À partir de 18 l'envoi d'images échoue (`-17691`) et le repli à l'ouverture ne couvre pas ce cas.
  Les extraits ouvrent donc au plus 8 sessions. `Codec::footprint` vaut 17 Mo pour openh264 et 180 Mo pour VideoToolbox.
- **Un encodeur par groupe** (`media::encode::fanout::Fanout`) : chaque image dessinée d'un groupe est encodée une seule fois, sans images B.
  Ses paquets vont dans chaque extrait ouvert avec des `pts` rebasés à 0.
  Une image-clé est forcée sur la première image de chaque extrait (`AVFrame.pict_type = I`).
  Le premier paquet d'un extrait doit être une image-clé (`NoKeyFrame`). Un extrait doit recevoir toutes ses images (`ShortClip`).
  Sur la 099 la commande passe ainsi de 58,68 s à 26,94 s avec le décodage à plusieurs fils.
- La limite est le débit du codeur matériel : environ 350 images 1440×1080 par seconde, toutes sessions confondues.
  Le M1 Max a deux moteurs, saturés dès deux sessions.
- openh264 pose une image-clé environ par seconde (`round(cadence)` images). VideoToolbox garde son réglage.
- **Zooms en openh264** : ils ne prennent aucune session VideoToolbox et occupent les cœurs libres.
  Sur la 099 (46 zooms) la commande prend 39,79 s contre 43,81 s tout en VideoToolbox.
  Elle prend 73 s de CPU en plus (mesure du 13 octobre 2026).

## Plage réelle des sources

Les jumelles étiquettent leurs vidéos en plage limitée (`tv`) mais leurs pixels débordent cette plage.
Pour l'affichage on les lit en plage pleine avec le noir calé sur 16. La détection lit Y brut.
Les fichiers produits restent en plage limitée BT.709.

Mesure du 11 octobre 2026 sur 20 images par vidéo, hors des plages abîmées.
Les 26 vidéos annoncent `tv` sur le conteneur et sur chaque image.

| Vidéos | Y hors 16..235 | Pire image | U/V hors 16..240 | Extrêmes |
|---|---|---|---|---|
| 089 à 103 | 0,92 à 1,96 % | 7,81 % (094) | 0,020 à 0,035 % | Y 0-255, U 68-146, V 98-255 |
| 120 à 130 | 0,41 à 0,62 % | 0,75 % | 0,031 à 0,033 % | Y 0-255 (0-252 pour la 120), U 75-147, V 113-255 |

- Les pixels hors plage sont le contenu thermique. Y < 16 est surtout en haut (arbres et ciel froids).
  Y > 235 est surtout en bas (sol chaud).
  V à 255 n'occupe qu'un petit carré, vraisemblablement une icône rouge de l'affichage des jumelles.
- Les fichiers produits sont resserrés sans écrêtage.
- **Noir calé sur 16** (jugé à l'œil sur les fonds de la 092, de la 122 et de la 124).
  Lu en plage pleine, le ciel froid sortait gris comme un voile.
  La formule est `Y' = max(0, Y − 16) × 255 / 239`, arrondie au plus proche, avant la conversion en plage pleine.
  La chrominance n'est pas touchée.
- Dans les extraits, l'image passe par les tables d'affichage (`Converter::display`) hors des zones dessinées.
  Dans les zones elle passe par `FullBlackAt16`, le RGBA et le retour en BT.709.

## Crates

La carte des crates et de leurs dépendances est dans [CLAUDE.md](../CLAUDE.md).
Cette section ne garde que les contrats et les pièges de chacune.

### `types`

- Un type n'entre dans `types` que s'il est produit par une crate et lu par au moins deux autres.
- `Timeline` donne deux temps. Le temps par les `pts` est celui de la commande. Le temps par la cadence est celui du CSV de la v0.1.1.
- `FrameDetections::push` est la seule façon de remplir la liste, tests compris.
  Une détection ne peut pas être rangée sous une autre image que la sienne.
- `span::merged` et `merged_by` sont la seule fusion de plages. Deux plages qui se recouvrent ou se touchent n'en font qu'une.
- `Stretch::Part` marque une piste gardée d'une piste recollée et `Stretch::Piece` un morceau.
  `parts()` ne rend que les pistes gardées : ce sont elles que juge le zoom.
- `WorkScale` fait seul le passage des pixels de travail aux pixels d'origine.
  Une coordonnée devient `(c + 0,5) × échelle − 0,5` et une surface × échelle².

### `media`

- `LibraryError::io_kind()` lit le genre d'erreur système derrière une erreur de FFmpeg.
  Sous Windows FFmpeg porte l'errno du CRT, qui a les mêmes valeurs.
  `decode::Error::NoDecoder` est séparé de `Open` pour qu'un `.mov` en ProRes ne passe pas pour un fichier illisible.
- `ffmpeg_next::format::input` ferait `to_str().unwrap()` sur le chemin.
  Le chemin passe donc en octets sous Unix jusqu'à `avformat_open_input`.
- `decode_errors` lit les `decode_error_flags` tels quels.
  Sans son cfg `ffmpeg_5_1`, le `DecodeError` de ffmpeg-next perd `CONCEALMENT_ACTIVE` et `DECODE_SLICES`.
  Une image est en erreur dès qu'un bit est posé, bit inconnu compris.
- Un paquet illisible est compté, pas fatal. Une erreur rend le décodeur inutilisable.
- À la fin du fichier, FFmpeg 7.1.1 rend l'erreur d'entrée-sortie du contexte à la place d'un `EOF` (`libavformat/demux.c:1530`).
  Elle sort en erreur `Read`.
- Un `linesize` négatif rendrait faux le `data()` de ffmpeg-next. Il est refusé (`FlippedFrame`) avant tout appel.
- Seuls `yuv420p` et `yuvj420p` sont acceptés. La cadence est `avg_frame_rate`, sinon `r_frame_rate`.
- `active_threads` dit ce que FFmpeg fait après l'ouverture.
  Un test exige plusieurs fils d'image, donc un FFmpeg construit sans fils fait échouer la CI.
- La couture des tests (`with_seam`) voit chaque paquet avant `send_packet`.
  Un paquet raccourci sans réécrire la longueur de son NAL est refusé en entier par le décodeur.
- `decode::placed` se place sur la dernière image-clé à ou avant un `pts` connu de l'index.
  Les images placées n'ont qu'un `pts`, pas de numéro.
- L'encodeur écrit sous un nom temporaire que `finish` renomme. Un échec ou une interruption ne laisse donc pas de vidéo tronquée.
  Le ménage de `files` doit reconnaître ce temporaire. Un test de `cli` l'exige.
- `open_stamped` garde les `pts` et la base de temps de la source au lieu de compter les images.
- Une écriture refusée (`Writing`) ne met pas en cause la destination, pour que la bascule vers openh264 joue.
  Une destination inutilisable rend tout de suite `Destination`.
- `Fallback` essaie VideoToolbox puis openh264 sur macOS. `trial` retire en tête de liste les encodeurs qui échouent.
  `create` ouvre le premier qui répond.
- **Ouverture bornée** (`bounded`) : sur la CI macOS, une ouverture VideoToolbox a attendu sans fin la réponse du service.
  L'ouverture se fait donc sur un fil à part, attendu au plus 10 s (`VIDEOTOOLBOX_START_LIMIT`).
  Au-delà elle rend `Unanswered` et on passe à openh264.
  Le silence se retient pour tout le processus. Un service muet coûte ainsi au plus 10 s par soirée.
  Un service lent qui répond avant 10 s coûte son attente à chaque ouverture.
  Le fil bloqué ne se tue pas. Il reste détaché jusqu'à la fin du programme.
  Une panique pendant l'ouverture repasse au fil appelant (`resume_unwind`).
  La recréation de session pendant `push` et la fermeture (`vtenc_close`) attendent sans borne.
  Aucun test ne reproduit le blocage du service.
  Un `atexit` de VideoToolbox qui attendrait la même connexion bloquerait après le bilan, Ctrl-C compris.
- **Conversion** : BT.709 dans les deux sens. La plage lue est un paramètre (`SourceRange`) et non l'étiquette de l'image.
  Le retour au YUV se fait en plage limitée, resserré sans écrêter : Y 0 devient 16 et Y 255 devient 235.
  Les tables d'affichage (`display`) évitent le passage par le RGBA.
  Celle de Y est tirée de swscale et égale l'aller-retour sur tout pixel gris.
  Les zones se convertissent par des contextes swscale à leur taille, dans un cache LRU de 32 tailles et 4 millions de pixels.
  Filtres : « point » à la montée et « area » à la descente.
  Les aplats de la palette, alignés sur 2×2, font l'aller-retour à 2 niveaux au plus.
- **Pour les tests** (feature `testing`) : `payload::wipe` écrase de `0xff` des parts égales de la boîte `mdat` d'un mp4 pour fabriquer une vidéo abîmée.
  `fingerprint` est la seule empreinte d'image des tests, un FNV-1a écrit à la main qui replie ses bits hauts.
  À la différence de `DefaultHasher`, il est le même sur chaque système et chaque version de Rust.
  Une empreinte peut donc s'enregistrer dans un test figé.

### `draw`

- Le texte est en Roboto 400 et les numéros en 600. Une taille est la hauteur des chiffres.
  Les chiffres ont tous la même largeur.
- Le texte est normalisé en NFC. Un caractère absent devient U+FFFD. Il n'y a ni crénage ni placement d'accent par GPOS.
- Un texte se prépare une fois (`shape`) puis se redessine sans allocation.
- La toile est toujours opaque. Ses octets sont du RGBA simple pour `media::convert`.
- Un bord de 1 px couvre le bord de la boîte. Chaque pixel de plus élargit vers l'extérieur.
  Les boîtes de 2 px et de 1 px gardent ainsi le même bord intérieur.
- Un test épingle les 12 couleurs de la palette à celles de la v0.1.1.
  Le numéro des pastilles s'écrit en noir (`INK`). Son contraste est d'au moins 6,94:1 et dépasse le 4,5:1 du WCAG AA.
  Le gris d'une piste « non » dans la page donne 6,49:1. Un test garde ce seuil si la palette change.

### `summary`

`summary` n'utilise pas FFmpeg : `cli` lui donne le fond déjà converti.
Couleurs, flèches, légende et tranches suivent les règles de la v0.1.1.

- `pages` rend aussi la géométrie de chaque piste pour la page, qui la redessine sans rien recalculer.
  Une pastille a un trait de rappel quand elle est à plus d'un rayon de la trajectoire (`Mark::leader`).
  C'est le seul critère, pour l'image comme pour la page.
- La tranche et le temps de légende d'une piste viennent du `pts` de sa première image.
- Le bandeau est un modèle sans texte (`HeaderLine`) que le trait `Labels` met en mots.
- Une tranche de plus de 20 pistes (`MAX_TRACKS_PER_IMAGE`) donne `ceil(n / 20)` images de paquets égaux : 21 pistes donnent 11 + 10.
  Les images gardent les numéros de la vidéo. Couleurs et pastilles s'attribuent paquet par paquet.
- Aucune pastille n'est hors cadre ni n'en recouvre une autre.
  Sur les 966 pistes de la référence, la v0.1.1 avait 266 recouvrements et ce placement 0.
- Le PNG s'écrit dans son partiel puis il est renommé.
  `png` n'expose la cause d'une erreur que par `cause()`, donc `Error::Write` garde l'`io::Error`.
- `tests/v011.rs` compare numéros, couleurs, tranches et en-tête à la v0.1.1 sur l'extrait et la 092.
  `tests/local_v011.rs` fait de même sur les 26 vidéos avec 22 écarts listés.

### `page`

`page` n'a aucune dépendance et ne lit pas le disque : `cli` lui donne un modèle et elle rend une chaîne.

- Les textes passent par `Labels`. Ceux dont le JS a besoin sont embarqués en JSON avec leurs trous écrits `{nom}`.
- Les chemins sont encodés en pourcents segment par segment, sans normalisation : le lien suit l'octet du disque.
  Le JSON embarqué ne peut pas fermer son `<script>`.
- Les noms arrivent en chaînes lossy.
  Pour une vidéo au nom non UTF-8 sous Linux, la page montre les pistes mais ni l'image résumé ni la vidéo annotée, et ses liens sont cassés.
- `<meta name="generator" content="batdetect">` dit qu'une page est à nous. `cli` n'écrit ni ne retire une page sans elle.
- Les scripts de `assets/` sont inclus dans un seul `<script>` : un `import` est bloqué en `file://` sous Chrome.
  La page tient en un fichier et se passe de réseau.
- `csv.js` décode en UTF-8 strict, sinon en Windows-1252. `store.js` est le seul endroit où se construit le mot d'un verdict.
- Les signes d'un verdict sont en SVG : les glyphes ✓ ✕ manquent dans la police du système sous Linux.
- Un verdict se rattache aux pistes de la même vidéo dont le début est à 0,2 s au plus, de façon gloutonne.
  Un verdict sans piste libre est gardé et montré.
- Le brouillon va dans `localStorage` sous `batdetect:verdicts:<chemin de la page>`.
  Un seul endroit écrit le texte et un seul l'envoie : #252 s'y branchera.
- `node --test` teste les fonctions pures du JS par la tâche `page:js` de `check`. Les vues se vérifient dans un navigateur.

### `reference`

- `location` refuse une variable d'emplacement absente ou vide.
- Un enregistrement a quatre fichiers : `video.json`, `frames.jsonl`, `tracks.jsonl` et `tracks.csv`.
  Relus puis réécrits, ils redonnent les mêmes valeurs.
- `video.json` version 1 est celui de la v0.1.1 avec un bloc `ffprobe`.
  La version 2 est celle de la v0.2.0, avec un bloc `decoder` et sans `tracks.csv`.
  Les fils sont dans `decoder` parce qu'ils changent ce que le décodeur dissimule.
  Le CSV manque parce que `compare` ne pourrait l'écrire sans cycle avec `cli`.
- Les points d'une piste ne sont pas forcément des détections : ceux des jumelles fusionnées sont combinés.
- `detection_matching` apparie deux détections de la même image à moins de 3 px, chacune une fois, la plus proche d'abord.
- `track_matching` apparie des pistes dont le début est à ±5 images. Une paire s'accorde si ses points diffèrent de 3 au plus.
  Il est glouton : deux pistes à moins de 10 images peuvent laisser une paire possible de côté. Un test le fige.

### `engine`

La logique de chaque étape est dans [detection.md](detection.md). Cette section garde les contrats du code.

- `Input::open` accepte toute taille et toute plage. La détection lit le plan Y tel quel avec la même table quelle que soit l'étiquette.
  OpenCV lirait autrement une vraie source `pc` : « au moins aussi bien que la v0.1.1 » ne s'y mesure pas.
  Aucune des 26 vidéos n'est concernée.
- Les tampons de `read_ahead` vivent jusqu'à ce que le consommateur ait fini.
  Sinon le pic de mémoire dépendait de la vitesse du décodage (#321).
- La garde exige décodées = annoncées − pertes. Seule une perte sans saut ni paquet illisible au bord fait refuser la vidéo (`FrameCount`).
- **Plages abîmées** : une plage part d'une image en erreur ou d'une image après un saut et va jusqu'à l'image-clé suivante.
  Elle est élargie de la demi-fenêtre du fond.
  Des images perdues avant la première ou après la dernière ouvrent une plage de la seule marge.
- **Canari** (`tests/slow_canary.rs`) : la couture coupe cinq images de l'extrait.
  Le Rust signale exactement {248, 555, 860, 861, 1222}. ffprobe 9.0.2 en signalait autant sur la même coupe.
- **Fils et pixels dissimulés** : une vidéo saine donne les mêmes pixels quel que soit le nombre de fils.
  Sur une tranche corrompue, les images en erreur et les plages ne changent pas.
  Les pixels dissimulés jusqu'à l'image-clé suivante peuvent changer d'une lecture à l'autre.
- **Écart entre systèmes** : sur lumiel, FFmpeg 7.1.1 signale 14 images en erreur de plus que ffprobe sur 8 vidéos (093, 097, 120, 121, 122, 123, 124, 128).
  Sur le Mac, seule la 2197 de la 124 est en plus. La cause n'est pas isolée.
  `local_damage` accepte l'un ou l'autre (`FLAGGED_ONLY_BY_SOME_SYSTEMS`).
- `detect_injected` est la couture du banc : un crochet générique sur la copie du plan Y. La commande passe un crochet vide et ne paie rien.
- **Borne de la fenêtre** : la fenêtre garde ses images à 2 octets par pixel de travail.
  Au-delà de `BACKGROUND_BUDGET` = 4 Gio, `half_window` refuse (`LongWindow`).
  Le refus se décide en `f64` avant tout calcul entier. Au plafond la détection tient autour de 5 Go, sous les 16 Go de la machine cible.
- `merge_radius` est borné à 12 pixels d'origine. Au rayon 25 en 1440, une image d'éclair plein cadre prenait 3,77 s.
  La borne tient une seconde par image à toute largeur.
- `DetectConfig::try_from` refuse ce que la v0.1.1 refuse, plus NaN et l'infini.
  Les dix réglages gardent leurs noms Python dans `video.json` et `params.json`.
- `reduction` : les rapports 2:1, 3:1, 3:2 et 4:3 ont des blocs fixés à la compilation pour que LLVM vectorise.
  Sans LTO la crate reste scalaire en aarch64 : la vectorisation ne se juge que sur le binaire.
- `blur` partage 4 096 parts entre les poids par les plus forts restes, par paires symétriques.
  Le noyau somme exactement 4 096, donc 255 × 4 096² tient sur un `u32`.
- `network` : la médiane de 11 échantillons (fenêtre par défaut) passe par `MEDIAN_OF_ELEVEN`, 28 comparateurs.
  Rien ne prouve que 28 soit le minimum.
- `threshold` garde les lignes du réseau sur la pile jusqu'à 64 échantillons (`STACKED_SAMPLES`).
  Sur le tas elles coûtaient 7 % de CPU, pour une raison non isolée.
- `components` rend les taches dans l'ordre de balayage de leur premier pixel.
  Le Rust n'imite pas l'ordre d'OpenCV, dont le suivi dépend en cas d'égalité.
- `detector` : une boucle de comparaisons écrite élément par élément se vectorisait mal une fois inlinée en `release`.
  Écrite sur des tableaux entiers, elle se vectorise.
- `background` écarte les images du fond qui tombent dans une plage abîmée, sauf s'il n'en reste aucune.
- Règle 16 : voir [Parallélisme](#parallélisme).
- **Mémoire** (`tests/slow_memory_flash*.rs`, `dhat`, 4 fils) : un éclair plein cadre coûte environ 24 octets par pixel de travail et par fil de plus qu'une vidéo calme.
  Le test exige plus de 16 et au plus 32 octets.
  Un éclair fort puis un éclair faible sur une autre image n'est pas testé et coûterait environ 8 octets de plus.

#### Gris de la détection

C'est l'exception à la règle 5.
Chaque Y passe par une table de 256 gris mesurée sur OpenCV (`cv2.VideoCapture` puis `COLOR_BGR2GRAY` sur une rampe Y = 0 à 255).
Le code la calcule par sa forme close `clamp((Y × 4769 − 81 520) >> 12, 0, 255)` dans `gray::computed`.
Elle égale la table sur les 256 valeurs.
Les seuils de la v0.1.1 sont réglés sur ce gris. Une formule exacte change trop de détections au seuil.
Taux d'appariement référence / Rust contre la v0.1.1, hors plages abîmées et instables :

| Variante | Extrait | 092 | 089 | Pire des 26 | Sous 98 % | Points de piste perdus |
|---|---|---|---|---|---|---|
| (a) table et arrondi à 8 bits | 100 / 99,75 | 100 / 99,87 | 98,19 / 98,46 | 98,19 | 0 | 19 |
| (b) formule, écrêtée, arrondie | 98,51 / 98,75 | 96,15 / 95,30 | 70,36 / 72,92 | 70,36 | 25 | 415 |
| (c) formule écrêtée, exacte | 97,51 / 98,25 | 95,38 / 96,00 | 75,58 / 76,76 | 75,58 | 22 | 340 |
| (d) formule exacte, sans écrêtage | 97,76 / 98,74 | 95,13 / 95,13 | 72,12 / 70,61 | 70,61 | 23 | 358 |

La table s'écarte du `BGR2GRAY` d'OpenCV quand la chrominance n'est pas neutre, dans la bande d'affichage des jumelles.
C'est pourquoi `sigma_0` et `window_2s` s'écartent de la v0.1.1 dans cette bande.
Retenter la formule avec des seuils réajustés est l'objet de #142.

### `cli`

Tout texte affiché vit dans `locales/{fr,en}.ftl`, lus par le seul binaire.
La bibliothèque reçoit ses libellés par les traits `Wording` et `Labels`.

- `doctor` et `completion` s'aiguillent sur le premier argument avant le parseur. Une vidéo nommée ainsi se donne en `./doctor`.
- `-h` et `--help` sont tous deux l'aide courte de clap : clap passe toujours le texte à la ligne en aide longue.
  Le niveau se choisit avant la lecture.
- clap coupe ses lignes à chaque changement de couleur et laissait parfois « ] » seul.
  La commande écrit donc le défaut sur sa propre ligne avec une espace insécable.
- Sous environ 95 colonnes, clap passe une section entière à la ligne. Les tests rendent l'aide à 100 colonnes.
- Nos refus de valeurs portent leur phrase dans `ContextKind::Custom` : le message de `Command::error` ne passe pas par le formateur de langue.
- **Admission** : rien n'est créé avant la détection.
  Toutes les entrées sont relevées avant la première détection et une seule fautive arrête le lot en code 2.
  Les raisons sont des tables fermées et sans texte (`jobs::Reason`, `access::Access`).
- L'inventaire ouvre chaque vidéo à faire en un seul fil. Un échec ensuite vient donc surtout de l'écriture.
  Les tests le provoquent par un fichier ordinaire posé à la place du dossier de la vidéo, ce qui échoue partout, Windows et root compris.
- Les homonymes de sortie se comparent à la casse près sur tous les systèmes.
  Sinon `a/Vol.mp4` et `b/vol.mp4` partageraient un dossier sur APFS ou NTFS.
- Le contrôle d'écriture crée une sonde de façon exclusive. `create_dir_all` réussit sur un dossier existant en lecture seule.
- La place libre se lit par `statfs` sous macOS, car les compteurs de `statvfs` n'y ont que 32 bits. Sous Windows elle n'est pas lue.
  Les besoins s'additionnent par volume (`MetadataExt::dev()`). Sous btrfs chaque sous-volume a son `dev()` et le besoin est sous-estimé.
  Les tests de place prennent une vraie vidéo allongée à 10 To par `set_len` en fichier épars. Ils ne tournent que sous Unix.
- La question des refus (`consent`) se teste dans `consent` sur une entrée fabriquée : un test de commande ne peut pas fabriquer de terminal.
- **Une vidéo** : `params.json`, la reprise et le ménage sont dans [Sorties](#sorties).
- La ligne d'étape est redessinée par `\r` et des espaces, sans séquence d'échappement.
  Elle est coupée à la largeur moins une colonne : une ligne repliée ne s'effacerait pas.
- « Réglages changés » nomme les champs que compare la reprise (`settings::compared`).
  Deux tests rangent toute option parmi celles qui font recalculer ou non.
- La page est écrite par `.partial` puis renommage, avant le lot et à la fin de chaque vidéo.
  Une page sans la marque de `page` n'est ni écrasée ni retirée.
  Le texte d'un refus est produit pour le terminal et pour la page. Un test de `sheet` vérifie qu'ils disent les mêmes lignes.
- **Langues** : la langue est la première variable non vide parmi `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` et `LANG`, sinon la langue du système (`sys-locale`).
  `LANG=C` reste en anglais. Les tests qui lancent le binaire fixent tous `LANG`.
  Le test des clés exige que chaque message se forme dans les deux langues et qu'aucune clé ne soit orpheline.
  Les marques d'isolement U+2068/U+2069 sont coupées : Roboto n'a pas leurs glyphes.
  Le français sépare les milliers par U+202F dans la console et U+00A0 sur le bandeau, car Roboto n'a pas U+202F.
  Le CSV, `params.json` et les noms de fichiers n'y passent pas.
- **Pannes** : `outcome::run_all` passe chaque vidéo dans `catch_unwind`, le seul du code livré. Le travail est `Fn + RefUnwindSafe`.
  Le crochet de panique est global parce que `resume_unwind` n'appelle pas le crochet.
  Une panique dans l'initialisation de FFmpeg (un `Once`) l'empoisonne.
  Le manque de mémoire et une panique dans un `rayon::spawn` ne sont pas rattrapés.
- **Version** : `crates/cli/build/engraved.rs` grave `BATDETECT_VERSION`, sinon `git describe --tags --match 'v[0-9]*' --dirty --always`, sinon `CARGO_PKG_VERSION`.
  Le script ne se relance que si `HEAD`, les tags, `crates/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/`, `packaging/` ou `tests/` changent.
  Un commit d'un autre worktree ne le relance donc pas. « -dirty » ne voit pas une modification de `docs/` ou de `mise.toml`.
- `video` de `params.json` est un texte lossy à lire, pas un chemin. La relance ne le compare jamais.

### `tracking`

- `exclude` ne peut pas échouer. Les réglages se lisent aux clés de la v0.1.1.
  Les clés venues depuis se lisent à 0 dans un fichier plus ancien.
- La médiane de la stabilité porte sur toute la vidéo.
  La stabilité dépend donc de sa longueur : la 089-091 concaténée ignore 27,9 s contre 28,9 s en morceaux.
- `Error::Track` ne vient d'aucune entrée. C'est un garde-fou contre un bogue de l'association ou de la fusion des jumelles.
- Un `piece_points` au moins égal à `min_hits` laisse la pile des morceaux vide sans erreur. `--min-hits 2` reste donc permis.
- `f64::hypot` peut différer d'un ulp de celui du Python sur un seuil exact. Le critère des tests est donc l'appariement avec tolérances.
- **Gardes du suivi** : le suivi courant tourne sur les détections et les plages de la [référence v0.2.0](#référence-v020) sans rien décoder.
  Il ne dépend donc ni du poste ni du décodeur.
  `committed_tracking` (19 cas du dépôt, en CI) et `local_tracking` (26 vidéos) figent une ligne par cas.
  `committed_exclusion` et `local_exclusion` comparent l'exclusion.
  Une PR de suivi met à jour ces lignes et donne la cause.
  Les 19 cas du dépôt n'ont aucune plage abîmée : en CI, seuls les tests unitaires gardent le suivi sur plages abîmées.
- `committed_metrics` (`bench`) fige l'effet des réglages de suivi gagnés depuis la v0.2.0 sur les deux bancs.

### `files`

- Les noms français n'ont pas d'accent. Une troisième table donne les noms de la v0.1.0, pour le [ménage](#ménage) seulement.
- Tout fichier s'écrit dans un partiel `.<nom>.<6 lettres ou chiffres>.partial` créé de façon exclusive, puis il est renommé.
  Deux écrivains du même fichier ont chacun le leur. `tempfile_in` n'est pas employé parce qu'il crée le fichier en `0600`.
- Le renommage du tableau des pistes n'écrase jamais un fichier présent.
  Sur un disque insensible à la casse, `vol_pistes.csv` annoté et la vidéo `VOL.mp4` ont le même chemin.
  Le ménage compare les octets et ne reconnaît pas ce fichier. Le renommage refuse de l'écraser.
  `tests/slow_exfat.rs` le vérifie sous macOS sur une image exFAT montée par `hdiutil` sans `sudo`.
- Une image sans `pts` reprend celui de la dernière qui en a un, plus un pas de cadence par image.
  Le temps n'est pas rapporté au premier `pts` : toutes les vidéos connues commencent à 0.
- `clock` écrit `h:mm:ss` tronqué à la seconde.
  C'est le seul format du début d'une piste sur la page : le binaire l'écrit et le script le relit.
- Contre la v0.1.1 sur la 092, les temps du CSV sont plus tôt d'un ou deux centièmes.
  La v0.1.1 divisait le numéro d'image par la cadence moyenne au lieu de lire les `pts`.
- `results` ignore un sous-dossier sans `params.json` ni sortie nommée d'après la vidéo, pour ne jamais lister un dossier que batdetect n'a pas écrit.
  `verdicts` prend le plus récent des `verdicts*.csv`, car le navigateur numérote les doubles.

### `render`

- Une liste de couleurs vide ou d'autant de couleurs que de pistes est acceptée (`Colored`).
  Toute autre longueur est refusée au lieu de retomber sur la boîte rouge.
- Chaque groupe d'extraits a un seul encodeur ([Encodage](#encodage)).
  Les zooms et la vidéo annotée (`--annotated`) sont des travaux à part. L'ensemble des travaux se partage la moitié des cœurs.
- Les images-clés et les images en erreur viennent de la lecture du moteur. `render` ne dépend donc pas d'`engine`.
- Pour chaque groupe d'extraits, la lecture se place par `pts` sur une image-clé saine avant le groupe (`source`).
- Une `Timeline` dont les `pts` connus ne croissent pas strictement est refusée avant tout décodage.
- La reprise se fait sur la dernière image-clé saine avant la plage : la dissimulation d'une image-clé en erreur puise dans les images d'avant.
  Une plage qui contient une image en erreur se décode sur un seul fil, car les pixels dissimulés ne se reproduisent qu'à un fil.
- Hypothèse : une image-clé saine remet le décodeur à zéro.
  C'est vrai d'une IDR, pas forcément d'une image à point de reprise que FFmpeg marque aussi image-clé.
  Seuls l'extrait et les 103, 123 et 124 la confirment.
- **Boîte bornée** : pour l'affichage seulement, chaque côté est borné à 4 fois la médiane du même côté sur les points détectés de la piste.
  Elle est née de la 092 piste 13 image 6945, où une tache étalée dessinait 363 × 259 px autour de rien.
  Le facteur 4 borne 33 des 33 865 points de la référence. Le facteur 3 en bornerait 129.
- Le zoom garde la taille de la vidéo et non le 720×540, car la pleine taille montre mieux les objets diffus.
- Seules les zones autour des dessins passent en RGBA. Un test vérifie que tout pixel dessiné tombe dans une zone.
- La vidéo annotée s'écrit aux `pts` de la source et reste en phase avec le CSV après une perte d'images.
- Chaque fil garde son `Picture`, car `Converter` n'est pas `Send`.
- Quand VideoToolbox casse hors erreur de destination, le groupe est refait en openh264 à la fin. Une ligne le dit.
  Une fausse ouverture (`testing::Fake`) l'exerce.
- `slow_clips` compare chaque image écrite à sa source par SSIM sur Y hors dessins.
  Les seuils sont 0,963 pour VideoToolbox et 0,964 pour openh264.

### `compare`

Outil de développement lancé depuis la racine du dépôt. Ses binaires `compare` et `record` ne sont pas livrés.

- `compare extract|092|092-1440|settings|local` compare à la [v0.2.0](#référence-v020) ou à la v0.1.1 par `--reference v0.1.1`.
  La règle est la même :
  - Le nombre d'images, les `pts`, les images-clés et contre la v0.2.0 les pertes aux bords doivent être ceux de la référence.
    Sinon la vidéo échoue.
  - Les erreurs de décodage ne sont pas exigées, car lumiel en voit 14 de plus que le Mac.
    Celles des deux côtés sont réunies et tracées en plages par la règle courante.
  - Les détections se comparent hors de ces plages.
    Les pistes passent par `exclude` puis `track_detections` aux défauts, puis par l'appariement.
  - Il n'y a pas de seuil. Le code de sortie ne vaut jamais 1 pour un écart à la référence : les gardes figent les leurs.
- `--same-as` dit si ce passage donne les mêmes détections qu'un passage `--frames` précédent, hors des plages abîmées.
  `cmp -r` entre deux dossiers `--frames` ne prouve rien, car les pixels dissimulés varient d'une lecture à l'autre.
- **`record`** enregistre la référence v0.2.0.
  Il refuse un binaire `-dirty`, construit hors de git, sans rustc ou après aucun tag de version.
  Un fichier non suivi sous `crates/`, `.cargo/`, `packaging/` ou `tests/` rend le binaire `-dirty`, car Cargo prend un `build.rs` non suivi.
  L'enregistrement se fait donc depuis un clone propre et toujours par `cargo run`, qui seul reconstruit un binaire dont une fixture a changé.
  `--out` est refusé s'il est dans `tests/fixtures/`, dans une référence ou dans le dossier de `--same-as`, ou s'il en contient un.
  Le script de version est seul dans `build/` : `cargo shear` prendrait son dossier pour la racine des sources.
- **Identité entre systèmes** : dans la chaîne de détection, seul `exp` dépend de la libm.
  Un test vérifie qu'un ulp d'écart ne change aucun poids du noyau pour les σ du banc.
  Le reste n'emploie que des opérations fixées bit à bit par IEEE 754.
  `area_to_source` multiplie par `échelle × échelle` et non par `powi(2)`, dont Rust ne fixe pas la précision.
- `tests/slow_brightness_jump.rs` porte les 3 tests de saut de luminosité de la v0.1.1. Le saut de 30 n'est qu'à un niveau du bord.
  Si openh264 encode autrement sur un autre système, ce test le montrera. L'écart se signale.

### `bench`

- `random` porte SplitMix64 et xoshiro256\*\*, épinglés sur les sorties publiées par leurs auteurs.
  Chaque cible a sa graine, tirée de `--seed` dans l'ordre des numéros.
- Les trajectoires passent par la libm. Une graine n'est donc reproductible que sur un même système.
  Les tests épinglent des propriétés, jamais un tirage.
- L'injection vise `gris(Y) + ⌊a·exp(−d²/2σ²)⌋` puis prend le Y dont le gris est le plus proche.
  La table `GRAY` saute un gris sur six environ.
  Un gris sauté se rend alternativement un niveau plus sombre et plus clair. Le contraste effectif est le changement de gris réalisé.
- `committed_metrics` réévalue chaque passage v0.2.0 du dépôt comme `--replay`. Ce qui vient de l'évaluation se fige par banc dans `FROZEN`.
  Une PR de suivi met à jour `FROZEN`, jamais la référence.

## Vérifier les réglages

Sans vérité connue, tout réglage est un pari.
On vérifie donc de deux façons : avec de fausses chauves-souris ajoutées à une vraie vidéo ([Banc de mesure](#banc-de-mesure)) et avec une vidéo dont chaque passage a été jugé à l'œil.
Cette vidéo est la 092 et ses 14 pistes sont gardées par `cli/tests/reference_092.rs` ([Gardes de détection](#gardes-de-détection)).
Selon le verdict de la naturaliste du 9 octobre 2026, 13 sont des chauves-souris.
La piste lente de 3:29 n'en est pas une ([résultats de la 092](resultats-092.md#verdict-de-lexperte-9-octobre-2026)).
Le banc ne dit pas si une piste réelle est une chauve-souris. Cela reste le jugement de la naturaliste.

## Réglages

Le banc d'essai du moteur hors des réglages par défaut compte 16 cas.
Chacun est une référence à d'autres réglages de détection, relue par `compare settings`.

`tests/fixtures/video_092_original_3m24-4m05_720x540.mp4` est l'extrait réduit de moitié (44 336 736 octets, sous LFS).
Elle a été faite une fois le 12 octobre 2026 par le ffmpeg 9.0.2 de Homebrew (libx264) et ne se refait pas.
Un autre encodeur ou un autre système ne donnerait pas les mêmes images et la référence ne vaudrait plus. CRF 16 garde le bruit du capteur.

```bash
ffmpeg -i tests/fixtures/video_092_original_3m24-4m05.mp4 -map 0:v:0 -an -sn -dn -map_metadata -1 \
  -vf scale=720:540:flags=area -c:v libx264 -preset slow -crf 16 -g 15 -keyint_min 15 \
  -sc_threshold 0 -bf 0 -threads 1 -pix_fmt yuv420p -color_range tv -colorspace bt709 \
  -color_primaries bt709 -color_trc bt709 -fps_mode passthrough -video_track_timescale 10000 \
  tests/fixtures/video_092_original_3m24-4m05_720x540.mp4
```

Les références v0.1.1 des 16 cas sont dans `tests/fixtures/reference/settings/<cas>/`.
Celles de la v0.2.0 sont dans `tests/fixtures/reference-v0.2.0/`.
Tous les cas sauf le dernier portent sur l'extrait.

| Cas | Réglages (`--detect`) |
| --- | --- |
| `threshold_25` | `threshold` 25 |
| `min_area_9` | `min_area` 9 |
| `max_area_400` | `max_area` 400 (écarte 6 taches de 400,5 à 488,25 px²) |
| `noise_factor_0` | `noise_factor` 0 |
| `window_2s` | `bg_window_s` 2,0 (61 images, 21 échantillons) |
| `step_1` | `bg_step` 1 (31 échantillons) |
| `osd` | `osd_regions` `[[0, 0, 1, 0.07]]` |
| `merge_radius_0` | `merge_radius` 0 |
| `width_480` | `work_width` 480 |
| `width_720` | `work_width` 720 |
| `width_1440` | `work_width` 1440 |
| `sigma_0` | `target_sigma` 0 |
| `sigma_3` | `target_sigma` 3 |
| `quick` | `work_width` 480, `threshold` 25, `target_sigma` 0, `noise_factor` 0 |
| `width_1440_osd` | `work_width` 1440, `osd_regions` `[[0, 0, 1, 0.07]]` |
| `size_720x540` | aucun, sur la 720×540 (travail en 720×540) |

- `max_area` vaut 400 et non 500.
  À 500 le cas redonnait les défauts, car la plus grande tache de l'extrait fait 488,25 px².
- `sigma_0` est le seul cas avec des plages instables. Sans flou, le bruit du capteur passe le seuil par endroits.
- `crates/compare/tests/settings_agreement.rs` exige les 16 cas comparés sans échec et fige leurs écarts.
- La 092 à 1440 pixels de travail (`tests/fixtures/reference/video_092_original_width_1440/`) se compare par `compare 092-1440` et dans `cli/tests/reference_092.rs`.

## Banc de mesure

Le banc incruste de fausses chauves-souris dans une vraie vidéo à des positions et des instants connus.
Il les incruste en pleine résolution, avant la réduction.
Ce sont des taches gaussiennes signées de contraste et de taille connus. Leur contraste varie au hasard de 0,6 à 1 à chaque image.
Elles suivent des courbes de Bézier ou des vols de chasse simulés.
Le banc compare ensuite ce que l'outil retrouve à ce qui a été injecté.
Il fait deux détections complètes : un passage de référence sur la vidéo telle quelle et un passage injecté avec les cibles.
Un banc de la 092 prend environ 31 s en `release`.
Sa place dans la méthode est dans [Vérifier les réglages](#vérifier-les-réglages).
Les valeurs qu'il a fixées sont dans [detection.md](detection.md#réglages).

```bash
cargo run --release -p bench -- in/video_092_original.mp4                    # 9 classes × 30 cibles
cargo run --release -p bench -- in/video_092_original.mp4 --motions hunt circle
cargo run --release -p bench -- --replay out/bench/video_092_original --max-gap 8 -o out/gap8
```

- **Tirer des cibles** (par défaut) : `--per-class`, `--amplitudes`, `--sigmas`, `--motions`, `--seed`.
  La vidéo se donne avant `--amplitudes`, qui prend plusieurs valeurs.
  Le défaut est le transit seul (`--motions pass`).
  On garde des bancs séparés pour la chasse : dans un banc mélangé, les cibles de chasse changent la détection des autres.
- **Rejouer des cibles** : `--targets FICHIER` reprend les cibles d'un `bats.jsonl.gz`.
  Une même graine ne redonne les mêmes cibles que sur un même système.
- **Réévaluer** : `--replay DOSSIER` relit un banc écrit et le réévalue avec d'autres réglages de suivi, de stabilité ou d'appariement.
  Il ne détecte rien et répond tout de suite.
- **Réglages** : ceux de la commande. `batdetect-bench --help` donne leurs défauts.
- **Sorties** : un dossier par vidéo dans `-o` (`out/bench` par défaut).
  Il contient `run.json`, les détections des deux passages (`reference.jsonl.gz`, `injected.jsonl.gz`) et les cibles avec leur contraste effectif (`bats.jsonl.gz`).
  `bench.json` donne les réglages, la version du code, le résumé par classe et le résultat par cible.
  Sa clé `code` est le `git describe --always --dirty` du dossier courant.
  `--replay` écrit `bench.json` dans `-o`. Une sortie qui écraserait l'entrée est refusée.
- La sortie standard donne le tableau par classe et le chemin du rapport.
  Le code de sortie vaut 1 sur un échec ou une garde en échec, 2 sur une erreur d'usage.

### Lire la sortie

- Une cible est **visible** à une image si son centre est à plus de 2σ des bords et si son contraste effectif atteint 8 niveaux.
  Visible moins de `min_hits` images, elle est écartée (« not visible »).
  Ne comptent pas non plus : une tache sombre sur un ciel saturé à 0, les images saturées de taches, les images illisibles et leur marge, une cible sous une `--osd-region`.
- Une détection couvre une cible si elle est à moins de `--match-radius` (24 px d'origine) de sa position vraie à la même image.
  Chaque piste va à la cible qu'elle couvre le plus, si au moins 50 % de ses points la couvrent.
- **Trouvée** : au moins `min_hits` images visibles couvertes. **Complétude** : images visibles couvertes ÷ images visibles.
- **Retard de début** et **fin anticipée** se comptent en images. **Fragments** : pistes attribuées à la cible.
  **Fausses pistes** : pistes du passage injecté sans cible ni équivalent dans le passage de référence.
- Chaque taux a un intervalle de Wilson à 95 %. Il dit si un écart entre deux réglages est réel.
  Avec 30 cibles par classe, il fait encore environ ±17 points à 50 %.
  Quelques points d'écart ne sont pas significatifs.
- **Limites** : on injecte après le décodage, sans artefacts de compression. Le biais va dans le sens optimiste.
  La cible est une gaussienne ronde.
  Le banc reproduit la saccade de période 3 : un correctif de la saccade y paraîtra efficace par construction.
  `min_hits` sert au suivi et à la mesure. Le changer change aussi le critère.
  Pour mesurer le coût d'un masque, on compare le nombre de cibles trouvées plutôt que la complétude.
  `--replay` avec un `--match-radius` plus grand que celui du tirage rompt l'écart entre cibles sur lequel repose l'appariement.
  Rien ne le refuse.

### Garde

`batdetect-bench VIDÉO --against DOSSIER` rejoue un banc enregistré par la v0.1.1 (`$BATDETECT_REFERENCE_DIR/bench/` `transit` ou `hunt`).
`local_guard` appelle la même fonction.

- Un banc fait à d'autres réglages est refusé et le refus nomme le réglage.
  La v0.1.1 n'avait ni recollage ni portée : il faut `--rejoin-hidden 0 --reach-margin 0`, et `without_rejoin().with_reach(None)` dans `local_guard`.
- Un banc d'une autre vidéo est refusé : le nom de fichier de `video` dans `bench.json` doit être celui de `VIDÉO`.
- La garde décide de la visibilité par le contraste enregistré par la v0.1.1, des deux côtés.
- Par classe, elle donne les cibles visibles des deux côtés et les trouvées de chaque côté.
  Elle donne la borne basse de Wilson de la v0.1.1 sur ces cibles.
  Elle liste ensuite les cibles perdues et gagnées.
  Sur la 092, chaque côté a 14 pistes de référence. Un autre nombre dirait une autre vidéo.
- Elle échoue (code 1) seulement si la proportion trouvée par le Rust tombe sous cette borne basse.
  Sinon la liste reste à juger.
- La sortie va dans `<-o>/<vidéo>_<dossier>/` avec `guard.txt`, qui porte le texte de la sortie standard.

## Parallélisme

Le calcul se partage entre tous les cœurs de la machine.
Chaque image ne dépend que de ses voisines et tout le calcul est en entiers jusqu'au seuil.
Le résultat est donc identique au bit près avec 1, 3 ou 4 fils et des lots de 1 à 64 images (règle 16). Un test le vérifie.

- **Décodage** : FFmpeg décode plusieurs images à la fois (`media/src/decode.rs`).
  Un fil de lecture garde 36 images d'avance : un lot de 32 plus 4 (`engine/src/read_ahead.rs`).
- **Préparation et détection** : les fils de calcul (`rayon`) font le gris, la réduction et le flou.
  Ils détectent ensuite un lot de 32 images à la fois (`engine/src/detector.rs`).
- **Taches mesurées en une passe** : l'aire et le pic de toutes les taches d'une image se comptent en un seul parcours des pixels retenus.
  À 1440 px le bruit fait des centaines de taches par image. C'est cette passe unique qui garde la détection rapide.
- **Rendu** : voir [`render`](#render).

## Temps actuels

Profil du 16 octobre 2026 sur un Apple M1 Max (10 cœurs, 64 Go, macOS 27.0). Binaire `release` du code livré de f364698, `LANG=fr_FR.UTF-8`.

| Vidéo | Réel | CPU | Pic | Détection | Extraits |
| --- | ---: | ---: | ---: | ---: | ---: |
| 092 entière (9 011 images) | 19,81 s | 143,6 s | 1,05 Go | 14,69 s | 4,60 s |
| extrait (1 230 images) | 4,48 s | 23,6 s | 0,70 Go | 2,25 s | 1,76 s |

- La détection occupe déjà presque les 10 cœurs. Elle ne gagnera que par un calcul plus court par image.
  Le fond et le seuil en prennent 22,7 % et la réduction avec le flou 20,3 %. Le reste du Rust tient sous 2 %.
- Sur lumiel (Ryzen 5 3600X sous Rocky Linux 10) la 092 prenait 28,7 s réelles et 238,9 s de CPU.
- Le 17 octobre, #214 et #213 ont raccourci la détection sans changer une piste.
  La détection seule de la 092 prend 11,13 s sur le M1 Max et 19,62 s sur lumiel.
  La commande entière n'a pas été remesurée depuis.
- `mise run check` prend 19,5 s sans recompilation et `rust:coverage` 519 s (M1 Max, 16 octobre 2026).

## Références

### Référence v0.1.1

Enregistrée par le Python, elle reste le témoin indépendant.
Son format, son emplacement et ses versions sont dans [son README](../tests/fixtures/reference/README.md).
Elle ne se réenregistre que depuis le tag `python-final` par `tools/reference_v011.py`, qui exige `src/` identique à `v0.1.1`.
Les attendus de l'image résumé viennent de `tools/summary_v011.py` au même tag.

Elle garde les entrées vues par ffprobe (`slow_extract`, `reference_092`, `local_command`, `slow_input`, `local_input`, `local_damage`).
Elle garde aussi l'aval nourri de ses pistes dans `summary`, `files` et `render`.
Elle garde enfin le format v1, les sorties (`local_outputs`) et le banc (`local_guard`).

Sur les 26 vidéos, la v0.2.0 retrouvait 963 des 966 pistes de la v0.1.1, dont 958 concordantes. Elle en perdait 3 et en ajoutait 9.
Ces écarts tombaient sur 9 endroits : 094 1:42, 097 0:20, 097 2:42, 098 0:05, 098 0:41, 099 de 0:14 à 0:19, 099 0:27, 100 1:36 et 126 1:31.

### Référence v0.2.0

Ce que la v0.2.0 détecte et suit, enregistré le 17 octobre 2026 sur lumiel par `record` depuis un clone propre (#250).
Format, emplacements, provenance, preuve et recette sont dans [son README](../tests/fixtures/reference-v0.2.0/README.md).
Elle ne se reproduit que hors des plages abîmées. Les gardes de `compare`, de la détection et du suivi portent sur elle.

### Gardes de détection

Chaque garde compare à la v0.2.0 par la règle de `compare` et fige ses écarts par vidéo et par cas (`Comparison::gaps`).
Ces écarts comptent les images qui diffèrent, les détections perdues et ajoutées, puis les pistes appariées, concordantes, perdues et ajoutées.
Une PR qui change la détection met à jour ces lignes et donne leur cause ([CONTRIBUTING.md](../CONTRIBUTING.md#ci)).

Les détections faibles (#21) ne font qu'ajouter des détections : les écarts sont des `Gaps::added_only`. Avant #21 ils valaient tous 0.
Le recollage (#24) et la portée (#326) changent des pistes. Leur effet commun se remesure sur l'arbre qui les a tous et ne s'additionne pas.
Une piste de la v0.2.0 réunie à une autre par le recollage compte pour perdue, car une piste ne s'apparie qu'à une seule.
Les endroits changés ont été jugés à l'œil.

- `compare/tests/slow_extract` : l'extrait et ses 4 pistes. Il garde aussi le bit près de `tests/fixtures/rust/` et `--same-as`.
- `compare/tests/settings_agreement` : les 16 cas.
  Sans flou (`sigma_0`) la portée retire des pistes de bruit et garde les 4 chauves-souris de l'extrait.
- `compare/tests/local_agreement` : une ligne par vidéo, 26 lignes et 972 pistes.
- `cli/tests/reference_092` : la 092 et la 092 à 1440 gardent leurs 14 pistes appariées et concordantes.
  La commande garde ses 15 extraits et son zoom.
  Les faits de la naturaliste sont les seuls affirmés sans référence ([resultats-092.md](resultats-092.md)).
  Les 13 chauves-souris confirmées le 9 octobre ont chacune leur piste à ±5 images de leur début.
  À côté d'elles, le test tolère au plus une piste à deux endroits : l'objet lent de 3:29 et la cible connue de 3:42.10, que la naturaliste n'a pas encore vue.
  Toute autre piste nouvelle fait échouer le test : c'est une question pour elle.
  Le nombre de pistes n'est affirmé que par les écarts figés, car un recollage peut passer de 14 à 13 pistes sans perdre un passage.
- `cli/tests/slow_command` : les 4 pistes de l'extrait sans écart.
- `cli/tests/local_command` : une ligne par vidéo. Chaque piste de la v0.2.0 doit être contenue dans une seule piste de la commande.
  Deux listes figées tiennent les exceptions.
  - `UNHELD` : les 11 pistes de la v0.2.0 dont aucune piste ne tient le premier point.
    Quatre viennent des détections faibles (090 #14 et #17, 096 #44, 097 #4).
    Sept viennent de la portée (090 #5 et #7, 094 #7, 099 #60 et #90, 100 #36 et #43).
    Parmi les endroits jugés à l'œil, seule la 090 #7 (2:12.33, deux points perdus) a été jugée pire.
  - `ZOOM_SWITCHES` : les 31 pistes dont le zoom automatique a changé, 30 gagnés et un perdu (098 #114).
    La cause de la 098 #113 n'est pas relevée.
- `cli/tests/local_clips` fige par vidéo le nombre de fenêtres d'extraits propres à la commande et à la v0.2.0.
- `process` et `tracked` n'écartent que les plages du décodeur courant.
  `reference_092`, `slow_command`, `local_command` et `local_clips` dépendent donc du poste sur les 8 vidéos où lumiel voit 14 images en erreur de plus que le Mac.
  Leurs lignes sont figées sur lumiel et restent à vérifier sur le Mac. `compare` réunit les deux lectures et n'en dépend pas.
