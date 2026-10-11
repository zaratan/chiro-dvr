# Migration vers Rust

Le lot 6 commence la réécriture de `batdetect` en Rust. Le but est de détecter au moins
aussi bien que la v0.1.1, pas de reproduire le Python bit à bit : les détections et les
pistes se comparent par appariement avec tolérances, et chaque écart se juge à l'œil.
Jusqu'à #106, le Python reste l'outil livré ; [02](02-methode.md) et
[10](10-logique-de-detection.md) décrivent le Python, ce document décrit le Rust.

## État (octobre 2026)

#95 a posé le workspace, FFmpeg, l'outillage et quatre crates. La commande
`batdetect-rs` ne fait encore que `--version`. Le moteur arrive avec #116 à #119, le
suivi avec #120 à #123.

## Construction

```bash
lockf /Users/zaratan/Projects/dvr-wt/mesure.lock mise run rust:ffmpeg   # construit FFmpeg une fois par poste
mise run rust:static      # fmt, clippy, règles de la charte, cargo deny, cargo shear
mise run rust:test        # tests rapides sous cargo-llvm-cov, couverture ≥ 95 %
lockf /Users/zaratan/Projects/dvr-wt/mesure.lock mise run rust:slow       # l'extrait LFS
lockf /Users/zaratan/Projects/dvr-wt/mesure.lock mise run rust:reference  # la 092 entière
```

`lockf` est le verrou de mesure des agents de ce poste ; ailleurs, ces commandes se
lancent sans lui.

- **Toolchain** : `rust-toolchain.toml` fixe Rust 1.99.0 (clippy, rustfmt,
  llvm-tools), édition 2024. C'est la seconde source de versions du dépôt, à côté de
  `mise.toml`. Une montée de version est un changement à part.
- **FFmpeg 7.1.1 exactement, openh264 2.6.0.** `packaging/ffmpeg-versions` donne les
  versions, les adresses et les sha256 :
  - la signature de l'archive FFmpeg a été vérifiée avec la clé publiée sur ffmpeg.org ;
  - la somme d'openh264 est celle de la formule Homebrew ;
  - nasm 2.16.03 n'est construit que sur x86_64, pour l'assembleur, et n'entre pas dans
    le binaire.

  7.1.1 est la version de la roue OpenCV qu'utilise la v0.1.1. Avec elle, le prototype
  lisait les mêmes images qu'OpenCV : 1 230 sur 1 230 sur l'extrait. Avec FFmpeg 9.0.2,
  il y avait jusqu'à 3 niveaux d'écart (rapport d'exploration, branche
  `exploration/rust`). La comparaison à la v0.1.1 part donc d'une même lecture.
  Épingler la version rend aussi reproductible d'un poste à l'autre la lecture des
  drapeaux d'erreur (`decode_error_flags`).
- **Ligne `configure`** : celle du prototype, en lecture seule (démultiplexeur mov,
  parsers et décodeurs h264 et hevc, protocole file, swscale). S'y ajoutent l'encodeur
  libopenh264, le muxeur mp4 et, sur macOS, VideoToolbox (décodage et encodeur
  `h264_videotoolbox`). `--disable-autodetect` écarte les bibliothèques du système.
- **Script** : `packaging/ffmpeg-unix.sh`, en bash portable (bash 3.2 de macOS compris).
  - Il construit dans un dossier temporaire du cache, puis le déplace par `mv`. Si une
    construction concurrente a déjà posé la cible, `mv` refuse et la cible existante est
    gardée.
  - **Cache partagé entre worktrees** : il vit dans
    `${XDG_CACHE_HOME:-~/.cache}/batdetect/ffmpeg/<empreinte>-<os>-<arch>/`.
    L'empreinte est un sha256 du script et des seules clés `FFMPEG_`, `OPENH264_` et
    `NASM_` de `ffmpeg-versions` : le script Windows de #96 n'y touche pas.
  - Chaque worktree a un lien `.ffmpeg`, ignoré par git, vers sa cible. Les `.pc` sont
    réécrits en `${pcfiledir}/../..` : le cache marche à travers le lien et à n'importe
    quel chemin.
  - `--fingerprint` imprime l'empreinte (clé du cache CI) ; `--link` (`mise run
    rust:link`) pose le lien et échoue si le cache manque. Ainsi, `mise run check` ne
    construit jamais FFmpeg.
- **Liaison** :
  - `.cargo/config.toml` force `PKG_CONFIG_LIBDIR` vers `.ffmpeg/lib/pkgconfig`, et vide
    `PKG_CONFIG_PATH`, `CPATH` et `C_INCLUDE_PATH` : le FFmpeg 9 de Homebrew, ses `.pc`
    et ses en-têtes ne peuvent pas être pris. `cargo` seul suffit, sans variable
    d'environnement.
  - Nos `.pc` mettent toutes les dépendances dans `Libs:`, et le dossier ne contient que
    des `.a` : la liaison est statique.
  - Pas `FFMPEG_DIR`, dont la route dans ffmpeg-sys-next 9.0.1 oublie openh264 et la
    bibliothèque C++.
  - Pas la feature `static` de ffmpeg-next non plus : elle ajoute 16 frameworks macOS
    figés, dont QTKit (absent en arm64, l'éditeur de liens le signale) et AppKit, OpenGL,
    OpenCL, inutiles.
  - Ce que le binaire lie en dynamique :
    - sur macOS : libc++, libSystem, libiconv (tirée par la bibliothèque standard de
      Rust) et cinq frameworks (VideoToolbox, CoreFoundation, CoreMedia, CoreVideo,
      CoreServices) ;
    - sous Linux : libstdc++, libgcc_s, libc et libm.
  - La CI vérifie par `otool -L` et `ldd` qu'aucune `libav*`, `libsw*` ni openh264
    n'est liée en dynamique. La liste complète n'est pas contrôlée.
- **Version liée vérifiable** : FFmpeg est construit avec `--extra-version=<empreinte>`.
  `batdetect-rs --version` affiche donc `FFmpeg 7.1.1-<empreinte>`, et un test compare
  cette empreinte à la cible de `.ffmpeg`.
- **Reconstruction** : cargo ne suit pas les `.a`.
  - La rlib de ffmpeg-sys-next n'en contient aucune copie (mesuré le 11 octobre 2026
    sous macOS) : 5 516 320 octets, dont 5 515 160 de métadonnées (les liaisons de
    bindgen) et un objet de 336 octets sans symbole défini. `libavcodec.a` pèse, à elle
    seule, 13 Mo.
  - Le build script ne déclare que des `rustc-link-lib=avcodec`, sans `static=`, avec
    la recherche dans `.ffmpeg/lib` : les `.a` sont lus à l'édition de liens de chaque
    binaire.
  - Mais ni le build script ni cette édition de liens ne se refont quand les `.a`
    changent. Un FFmpeg changé laisserait donc des binaires liés à l'ancien et des
    liaisons tirées des anciens en-têtes.
  - Quand `.ffmpeg` change de cible, le script lance
    `cargo clean -p ffmpeg-sys-next` dans chaque profil, `target/llvm-cov-target` compris.
    Le nettoyage passe avant le changement du lien : s'il échoue, le passage suivant le
    refait. Après une montée de version de FFmpeg, faire un `cargo clean` complet, car les
    liaisons de bindgen changent. Un arrêt brutal (SIGKILL, délai de la CI dépassé) peut
    laisser un dossier `.build.*` dans le cache : il se supprime à la main.
- **Cible processeur** :
  - macOS arm64 garde le défaut (`apple-m1`) ;
  - Linux x86_64 compile le Rust en `x86-64-v2` (`.cargo/config.toml`). Sans SSE4.1, le
    min/max u16 du réseau de médiane ne serait pas vectorisé. FFmpeg choisit ses routines
    à l'exécution ;
  - ne jamais poser `RUSTFLAGS` : la variable remplacerait ce réglage.
- **macOS 14 au minimum** : `MACOSX_DEPLOYMENT_TARGET=14.0` vaut pour openh264, FFmpeg
  et rustc. Mesuré : `minos` 14.0 sur les 411 objets et sur le binaire.
- **Profils** :
  - `release` : `lto = "fat"`, `codegen-units = 1`, `panic = "unwind"` ;
  - `ci` : hérite de `release`, sans LTO, 16 unités, avec `overflow-checks` et
    `debug-assertions`. Il sert aux tests ; les temps se mesurent en `release`.
- **Mesures du 10 octobre 2026** :

  | Mesure | Mac M1 Max | Linux amd64 émulé (OrbStack) |
  |---|---|---|
  | Construction de FFmpeg et openh264 | 42 à 50 s | 224 s |
  | Taille du cache | 26 Mo | 43 Mo |

  Le binaire `release` pèse 690 576 octets, mais il ne fait que `--version` et la LTO
  retire presque tout FFmpeg : ce chiffre ne prédit pas la taille de l'outil complet.

### Windows

Cible `x86_64-pc-windows-msvc`, construite seulement par la CI (tâche `rust-windows`) :
`mise` n'y tourne pas, et la construction pour un poste Windows viendra avec #93.

- **Script** : `packaging/ffmpeg-windows.sh`, en bash sous MSYS2, avec MSVC dans le
  `PATH`.
  - Il lit le même `ffmpeg-versions`, plus deux clés `WINDOWS_NASM_*` pour le binaire
    officiel de nasm 2.16.03. Ce préfixe reste hors du filtre de l'empreinte Unix, qui ne
    change donc pas.
  - Son empreinte hache le script et les clés `FFMPEG_`, `OPENH264_` et `WINDOWS_`.
  - `pinned`, `sha256`, `fetch`, `relocate` et `clean_bindings` sont recopiées ou
    adaptées du script Unix : un fichier commun changerait `ffmpeg-unix.sh`, donc son
    empreinte et son cache.
    - Adaptées : `fetch` est coupée en `download` et `fetch` ; `relocate` traite le
      chemin sous ses deux formes, POSIX et mixte.
    - Un correctif du script Unix se reporte donc à la main.
  - Seule option : `--fingerprint`.
- **Même bibliothèque C partout** : `/MT` pour openh264 (`make OS=msvc`, avec son
  `CFLAGS_OPT` redonné sans `-Zi -FS`) et
  pour FFmpeg (`--toolchain=msvc --extra-cflags=-MT`), `+crt-static` pour le Rust
  (`.cargo/config.toml`, avec `target-cpu=x86-64-v2`). Le binaire ne demande donc ni
  `vcruntime140.dll` ni l'UCRT.
- **Liaison : deux pièges, que le script corrige après `make install`, et une
  conséquence** :
  - **Noms des archives.** FFmpeg nomme ses archives `libavcodec.a` même sous MSVC, alors
    que rustc passe `avcodec.lib` à `link.exe`. Le script renomme donc `libX.a` en
    `X.lib`.
  - **Bibliothèques système.** Le `configure` de FFmpeg écrit les bibliothèques système
    des `.pc` à la manière de MSVC (`bcrypt.lib`, `-libpath:`). pkg-config-rs ignore en
    silence un `bcrypt.lib` nu, et lit `-libpath:` comme `-l` suivi de `ibpath:`. Le
    script les réécrit en `-lbcrypt` et `-L`.
  - **Conséquence, sans correctif à faire : la liaison reste statique.** Sans la
    feature `static`, ffmpeg-sys-next appelle pkg-config-rs en `statik(false)`, qui
    demande ces bibliothèques sans type, donc en dylib. Sous MSVC,
    `link.exe` prend aussi bien une archive.
- **Autres pièges** :
  - Sous MSVC, ffmpeg-sys-next essaie vcpkg avant pkg-config, et `msvc-dev-cmd` (vcvars)
    définit `VCPKG_ROOT`. La tâche pose donc `VCPKGRS_DISABLE=1`.
  - Le `link.exe` de coreutils (MSYS2) est renommé avant toute construction. Le script
    refuse un `link` qui n'est pas celui de Microsoft.
  - MSYS2 réécrit en chemin tout argument qui commence par `/`. Les options de MSVC
    s'écrivent donc `-x`, et `cmd /c` passe par `MSYS2_ARG_CONV_EXCL='*'`.
  - Le cache se donne en chemin mixte (`D:/a/...`), puis le script le normalise par
    `cygpath`.
  - `LIBCLANG_PATH` pointe vers le LLVM de l'image pour bindgen.
- **Lien `.ffmpeg`** : une jonction NTFS (`mklink /J`), qui ne demande pas de droits.
  `fs::read_link` la lit, donc le test d'empreinte de la version liée vaut aussi sous
  Windows.
- **Gardes du script** :
  - LGPL 2.1+ et `HAVE_W32THREADS 1` dans `config.h`. Le multifil n'est gardé qu'à la
    compilation : aucun test ne vérifie que le décodage tourne sur plusieurs fils ;
  - pkg-config, interrogé sans `--static` comme le fait cargo, ne rend ni antislash, ni `.lib` nu, ni
    `-libpath:` ;
  - aucune archive `.a` ne reste ;
  - chaque `.lib` demande `LIBCMT` et jamais `MSVCRT` (`dumpbin -directives`).
- **Fins de ligne** : l'image Windows extrait le texte en CRLF (`core.autocrlf`).
  `.gitattributes` force `eol=lf` pour les scripts `.sh`, `ffmpeg-versions` et
  `tests/fixtures/reference/**`.
- **Construction suivie à l'essai.** Au plus 5 essais par méthode étaient permis :
  FFmpeg construit par nous, puis vcpkg, puis un BtbN lié en dynamique sur décision de
  l'utilisateur. La première méthode a abouti au 2ᵉ essai (PR #132) :

  | Essai | Résultat | Cause | Correctif |
  |---|---|---|---|
  | 1 | échec dans le script | `unzip -d` ne crée que le dernier dossier du chemin | `mkdir -p` avant `unzip` |
  | 2 | vert | — | — |
  | 3 | vert, à froid puis à chaud | retouches demandées : `dumpbin` en liste blanche, clippy, openh264 sans `-Zi` | — |

- **Mesures du 11 octobre 2026** (runner `windows-latest`, image `windows-2025-vs2026`
  20260925.250.1, MSVC 14.51) :
  - construction de FFmpeg et d'openh264 : 3 min 15 s ;
  - tâche entière, à froid : 5 min 26 s à l'essai 2 (sans clippy), 9 min 02 s à
    l'essai 3 (avec clippy) ;
  - à chaud (essai 3 relancé sur le même commit, caches LFS, `target/` et FFmpeg
    restaurés) : 1 min 53 s, sans aucun LNK4099 ;
  - `slow_decode` : 1 230 images en 2,63 s ;
  - dépendances du binaire : `kernel32`, `ntdll`, `bcrypt` et
    `api-ms-win-core-synch-l1-2-0`.

  À l'essai 2, chaque édition de liens affichait 65 avertissements LNK4099, un par
  objet d'openh264 compilé en `-Zi`, dont le `.pdb` n'est pas installé. Ils étaient sans
  effet sur le binaire, mais rustc les remonte en avertissements. Depuis l'essai 3,
  openh264 se compile sans `-Zi -FS`, c'est-à-dire sans informations de débogage.

## Licences

- **FFmpeg 7.1.1 en LGPL 2.1 ou ultérieure.**
  - Il est construit sans `--enable-gpl` ni `--enable-nonfree` ; `libavcodec` annonce
    « LGPL version 2.1 or later ».
  - Lié en statique, il impose les obligations de la LGPL à la distribution : fournir de
    quoi relier, ou passer en liaison dynamique. Cela se tranche à #93, qui réécrira
    aussi `packaging/NOTICE`.
  - Le binaire est donc MIT et LGPL.
- **openh264 2.6.0 en BSD-2.** Il est construit depuis les sources, donc **sans** la
  licence de brevets H.264 que Cisco n'accorde qu'à ses propres binaires. C'est une
  question pour l'utilisateur avant toute distribution, pas une décision prise ici.
- **Crates** : `deny.toml` n'admet que MIT, Apache-2.0 (avec ou sans LLVM-exception),
  BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, WTFPL (ffmpeg-next, ffmpeg-sys-next) et
  Zlib. La liste a été mesurée avec toutes les dépendances déclarées, différées
  comprises. 0BSD, LGPL-2.1 et Unlicense n'apparaissent que comme alternatives dans une
  expression « OU » qui contient aussi MIT (adler2, r-efi, memchr).

## Lints

La charte Rust du lot 6 (23 règles, section suivante) est résumée dans
[CLAUDE.md](../CLAUDE.md). Les lints se règlent dans `[workspace.lints]` du `Cargo.toml`
racine, jamais par attribut.

- **Activés** :
  - `clippy::pedantic` ;
  - des restrictions : `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`,
    `dbg_macro`, `print_stdout`, `print_stderr`, `exit`, `allow_attributes`,
    `let_underscore_must_use`, `unused_result_ok`, `map_err_ignore`,
    `iter_over_hash_type`, `mod_module_files`, `string_slice`, `clone_on_ref_ptr` ;
  - pour `unsafe` : `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block`,
    `unnecessary_safety_comment` ;
  - côté rustc : `unsafe_code = "forbid"`, `unreachable_pub`,
    `missing_debug_implementations`, `trivial_numeric_casts`, `rust_2018_idioms`,
    `non_ascii_idents`.
- **Sept lints de `pedantic` coupés**, dans la table seulement :
  - `cast_possible_truncation`, `cast_possible_wrap`, `cast_precision_loss` et
    `cast_sign_loss` : 30, 55, 48 et 31 occurrences sur les noyaux du prototype, dans
    des calculs de pixels où la règle 15 suffit (un `as` qui perd de l'information suit
    un `round` ou un `clamp` visible, ou vit dans une fonction de conversion nommée) ;
  - `missing_errors_doc` et `missing_panics_doc` : la règle du zéro commentaire ne
    demande pas de documentation ;
  - `must_use_candidate` : il signalerait presque chaque accesseur.
- **Refusés** : `indexing_slicing` (272 occurrences sur le prototype),
  `arithmetic_side_effects` (223), `as_conversions` (184), `missing_docs` (62),
  `nursery`, `cargo`.
- **`media` est la seule crate avec `unsafe`.** Elle porte sa propre table, copie exacte
  de celle du workspace sans `unsafe_code`. Cargo refuse `workspace = true` avec une
  surcharge locale, et un contrôle compare les deux tables.
- **Exception au niveau du manifeste** : `[workspace.metadata.cargo-shear] ignored`
  liste les dépendances déclarées d'avance qu'aucune crate n'utilise encore. L'utilisateur
  l'a acceptée comme les lints coupés de la table : ce n'est pas un `#[allow]` au sens de
  la règle 3. `cargo shear --deny-warnings` échoue dès qu'une entrée de la liste devient
  utilisée (`redundant_ignore`) : la liste se vide issue par issue.
- **Règles vérifiées par script** (`mise run rust:rules`, dans `rust:static`) :
  - aucun `#[allow]`, `#[expect]` ou `cfg_attr(…allow…)` ;
  - `[lints] workspace = true` partout sauf dans `media`, dont la table est égale à celle
    du workspace sans `unsafe_code` ;
  - aucun `ffmpeg_next` hors de `media` ;
  - les dépendances des crates ne viennent que du workspace ;
  - au plus un `catch_unwind`, pas de `panic = "abort"` ;
  - au plus 2 % de lignes de commentaire dans `crates/`, en comptant aussi les
    commentaires de fin de ligne et les `/* */` ;
  - aucun mot d'imitation (`round_ties_even`, `opencv`, `numpy`, `python`, `pairwise`).

## Charte

But : détecter au moins aussi bien que la v0.1.1, en Rust idiomatique, sans recopier ligne
à ligne ni le Python ni le prototype.

Outillage :

1. Toolchain de `rust-toolchain.toml`, édition 2024. `cargo fmt --all --check` ne
   signale rien.
2. `cargo clippy --workspace --all-targets --locked -- -D warnings` sans avertissement.
   Les lints se règlent dans `[workspace.lints]` du `Cargo.toml` racine, nulle part
   ailleurs.
3. Zéro `#[allow]`, `#[expect]` ou `cfg_attr(…allow…)`, même si clippy le suggère. Un
   lint gênant se résout en réécrivant le code ; s'il paraît faux, on demande.
4. Chaque crate a `[lints] workspace = true`. Seule `media` a sa propre table : la copie
   exacte de celle du workspace, sans `unsafe_code`.

Pas d'imitation :

5. Ni Python, ni numpy, ni OpenCV ne s'imitent : pas de `round_ties_even` pour copier
   `round()`, pas d'ordre d'opérations recopié. Le calcul s'écrit simple et juste ;
   l'écart se mesure par appariement avec tolérances (`reference`), puis se juge.

Sûreté et erreurs :

6. `unsafe` seulement dans `media`. Un bloc contient une seule opération et porte une
   ligne `// SAFETY:` qui dit pourquoi le contrat tient.
7. Aucun type de `ffmpeg_next` ne sort de `media`. Une fonction `extern "C"` ne peut pas
   paniquer.
8. Chaque bibliothèque a son `enum` d'erreur (`thiserror`), qui garde la cause
   (`#[source]`). Ni `anyhow`, ni `Box<dyn Error>`, ni erreur `String`. Une erreur par
   module dans `media`.
9. Une entrée refusée (vidéo abîmée, format non géré, réglage hors bornes) rend une
   erreur typée, jamais une panique. `unwrap`, `expect`, `panic!`, `todo!` et `dbg!`
   sont refusés hors des tests.
10. Une panique signale un bug. Seule la commande la rattrape, une fois par vidéo. Les
    fils passent par `std::thread::scope` ou rayon, jamais détachés.
11. Aucun résultat jeté : ni `let _ =` sur un `Result`, ni `.ok();`.
12. Ni `println!` ni `eprintln!` : `writeln!` sur un flux verrouillé, erreur traitée.
    Tout texte vu par l'utilisateur vit dans un seul module de la commande.

Types et calcul :

13. L'index d'image (ordre d'affichage, celui où le décodeur rend les images) et le `pts`
    ont chacun leur type (`FrameIndex`, `Pts`). Pas de sentinelle : `Option<Pts>`.
14. Hors de la détection, tout est en pixels de la vidéo d'origine. Seul `WorkScale`
    passe des pixels de travail aux pixels d'origine.
15. Conversions : `From` quand elle existe. Un `as` qui perd de l'information suit un
    `round` ou un `clamp` visible, ou vit dans une fonction de conversion nommée.
16. Même entrée, même sortie, quel que soit le nombre de fils : pas d'itération sur
    `HashMap` ou `HashSet` qui atteint une sortie, pas de somme flottante parallèle.
17. Une configuration se construit par une fonction qui valide et rend un `Result`. Les
    comptes non nuls sont des `NonZero*`.
18. Boucles chaudes : aucune allocation par pixel ni par image, tampons réutilisés ;
    itérateurs, `zip` et `chunks_exact` plutôt que des indices.

Forme et tests :

19. Un module, une responsabilité, ses tests dans `src/<module>/tests.rs`. Pas de
    `mod.rs`. Fonctions de 100 lignes et 7 arguments au plus.
20. `pub(crate)` par défaut, `Debug` sur tout type public. Pas de `pub use` dans
    `media/src/lib.rs`.
21. Zéro commentaire, sauf `// SAFETY:` et une ligne `///` sur un élément public dont le
    nom ne dit pas le contrat externe. Au plus 2 % des lignes.
22. Tests sans commentaire, nommés par le pourquoi ; `unwrap`, `expect` et `panic!` y
    sont permis. Flottants comparés avec une tolérance. Pas de faux FFmpeg : vraie vidéo,
    ou vidéo fabriquée par openh264. Couverture d'au moins 95 %. Tests lents dans
    `tests/slow_*.rs`, tests de la 092 dans `tests/reference_*.rs`. proptest là où un
    oracle simple existe.
23. Une dépendance se déclare une fois dans `[workspace.dependencies]`, puis
    `x.workspace = true`. Une dépendance hors de la liste d'une issue se demande avant.
    `cargo deny check` et `cargo shear` restent verts.

## Dépendances

- Chaque dépendance se déclare une fois dans `[workspace.dependencies]`, puis
  `x.workspace = true`. `--locked` partout.
- Déclarées d'avance pour le lot : `rayon`, `tiny-skia`, `dhat`. Tant qu'elles ne
  servent pas, elles ne sont pas dans `Cargo.lock` : leur version n'y est pas figée.
- `serde` sert `types` et `reference` depuis #124. `serde_json` a la feature
  `float_roundtrip` : un flottant de la référence se relit au bit près. `flate2`
  (moteur `miniz_oxide`, 100 % Rust) lit et écrit les `.gz` de la référence ; `zlib-rs`
  figure dans `Cargo.lock` sans être compilé.
- **`ab_glyph` est retiré.** Il tire `ttf-parser`, déclaré non maintenu par
  RUSTSEC-2026-0192 sans mise à jour sûre. L'utilisateur refuse d'ignorer l'avis : #101
  choisira une pile de polices maintenue.
- `clap` n'a que la feature `string` ; `derive` attendra la commande complète (#123).
- `cargo deny check` :
  - échoue sur un avis de sécurité, un paquet retiré (`yanked`), une licence hors
    liste, une version sans borne (`*`) ou une source inconnue ;
  - ne fait qu'avertir sur une version en double ;
  - prévient que BSD-2-Clause et Zlib, admises pour les dépendances différées, ne sont
    pas rencontrées. Elles ne seront contrôlées qu'à l'arrivée de ces dépendances dans
    `Cargo.lock`.

## Tests

| Niveau | Fichiers | Commande | Contenu |
|---|---|---|---|
| rapide | `src/<module>/tests.rs`, `tests/*.rs` | `cargo test --workspace --locked` | vidéos fabriquées par openh264, binaire lancé pour de vrai |
| slow | `tests/slow_*.rs`, `#[ignore = "slow"]` | `mise run rust:slow` | l'extrait LFS : 1 230 images, 10000/333, 1440×1080, plage `tv` |
| reference | `tests/reference_*.rs`, `#[ignore = "reference"]` | `mise run rust:reference` | la 092 : 9 011 images, cadence moyenne 90110000/3000927 |
| matériel | `tests/hardware_*.rs`, `#[ignore = "hardware"]` | étape d'essai de la CI macOS | VideoToolbox sans repli logiciel |
| local | `tests/local_*.rs`, `#[ignore = "local"]` | `BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=… cargo test -p reference -p tracking --locked --profile ci --test 'local_*' -- --include-ignored` | les 26 vidéos de la référence, hors dépôt, jamais en CI |

- Le niveau d'un test se choisit par le nom de son fichier. Ajouter un test ne demande
  de toucher ni la CI ni `mise.toml`.
- Couverture mesurée sur les tests rapides, en profil `ci` (seuil 95 %). CI de la PR
  #130 : 97,95 % des lignes sous Linux et 98,14 % sous macOS. Les lignes restantes sont
  des erreurs de FFmpeg qu'aucune vidéo de test ne provoque.
- Les fixtures se trouvent par `reference::fixture::fixture`, qui refuse un pointeur
  LFS non tiré avec le conseil `git lfs pull`.
- **CI** : tâche `rust` sur macOS et Linux (statique, tests rapides, slow, contrôles du
  binaire) et tâche `rust-reference` sous Linux, après `rust`. Le cache de FFmpeg est
  restauré, puis sauvé dès la construction, même si la suite échoue. L'action `setup`
  n'installe les outils cargo et la toolchain qu'avec `rust: true`. Premier passage
  (PR #130), sans cache de `target/` : `rust` 3 min 04 sur macOS et 3 min 46 sous Linux ;
  `rust-reference` 1 min 15.
- **Tâche `rust-windows`**, indépendante des autres. Elle construit FFmpeg, lance les
  tests rapides et `slow`, puis `cargo clippy -- -D warnings`.
  - `dumpbin -dependents` n'admet que des DLL de Windows : `kernel32`, `ntdll`,
    `bcrypt`, `bcryptprimitives` et `api-ms-win-core-*`. C'est une liste blanche, et
    `kernel32` doit y être : le contrôle ne peut pas passer à vide.
  - Ni couverture, ni `cargo deny`, ni `mise` : ils tournent déjà sous Unix.
  - clippy vient en dernier, pour qu'un échec propre à Windows laisse lisibles les
    résultats des tests.
  - Le cache LFS de l'extrait se reprend depuis Linux (`enableCrossOsArchive`).
  - En cas d'échec, `config.log` de FFmpeg est téléversé en artefact.
- **Cache de `target/`** (`actions/cache`, une entrée par tâche) :
  - Il contient tout `target/` : les dépendances compilées dans chaque profil (`debug`
    pour clippy, `ci` pour les tests et le binaire), `target/llvm-cov-target` pour la
    couverture, et la rlib de ffmpeg-sys-next avec ses liaisons.
  - Il se sauve en fin de tâche réussie.
  - Sa clé porte la tâche, l'OS, l'architecture, l'image du runner (`ImageOS`), le
    `Cargo.lock`, `rust-toolchain.toml` et **l'empreinte de FFmpeg**. Cargo ne suit pas
    les `.a`. Un `target/` construit avec un autre FFmpeg garderait donc des binaires
    déjà liés à l'ancien, et les liaisons de bindgen tirées de ses en-têtes (voir
    « Reconstruction »).
  - Pour la même raison, la clé n'a pas de `restore-keys` : une clé partielle pourrait
    mélanger deux empreintes.

## Essai VideoToolbox

- **Sur le M1 Max** : `h264_videotoolbox` encode une seconde en 1440×1080, avec et sans
  repli logiciel (`allow_sw=1` et `allow_sw=0`).
- **Sur le runner `macos-latest`** (CI de la PR #130, commit `fca2c01`) : même
  résultat. Le repli logiciel passe dans les tests rapides. Sans repli, l'étape d'essai
  passe aussi (1 test, 0,33 s). Le journal n'affiche, avant le test, que
  « IOServiceMatchingfailed for: AppleM2ScalerParavirtDriver », sans effet sur le
  résultat.
- **Conclusion** : l'encodeur matériel répond sur le runner macOS de GitHub, donc #19
  se ferme par la CI. Le repli logiciel est testé à chaque passage, le matériel par une
  étape qui bloque la CI (décision de l'utilisateur, 11 octobre 2026). Si elle casse à
  cause d'une machine de GitHub plutôt que du code, la question se rouvre avec
  l'utilisateur.
- **Depuis #111**, l'étape matérielle encode 120 images de l'extrait (images 660 à 779)
  au réglage de production, q65 sans repli logiciel, puis les relit. Elle vérifie le
  nombre d'images, la taille, la cadence 10000/333 exacte, la plage limitée et un PSNR de
  luminance supérieur à 37,9 dB. Sur le M1 Max, on mesure 38,9 dB en q65 et 34,9 dB sans
  q65. #19 se ferme avec la PR de #111.
- **#111 retire le repli logiciel** (`allow_software`, décision de l'utilisateur,
  11 octobre 2026) : le logiciel d'Apple refuse q65. `Codec::VideoToolbox` est donc le
  matériel seul, et les tests rapides de VideoToolbox sur macOS passent eux aussi par le
  matériel. Le test lent d'openh264 relit le même passage, avec un seuil de 39,1 dB :
  - mesuré sur le M1 Max : 39,53 dB à QP 23 et 38,69 dB à QP 24 ;
  - le seuil, à mi-chemin, rejette un QP qui ne serait pas appliqué ;
  - l'openh264 x86 de la CI Linux reste à confirmer au premier passage.

## Encodage

Mesuré le 11 octobre 2026 sur le M1 Max, sous le verrou de mesure (#111) :

- **Programme** : `~/Projects/dvr-wt/rapports/mesure-111/`, hors du dépôt. Il contient le
  programme Rust jetable, lié au même FFmpeg, les scripts `one.sh` et `xpc.sh`, les
  journaux et `RESULTATS.md`.
- **SSIM** : filtre `ssim` du ffmpeg de Homebrew 9.0.2. Chaque sortie est comparée à
  l'extrait d'origine (1 230 images).
- **Temps** : médiane de 3 passages. Le décodage seul (6,98 s) est retranché.

| Encodeur | Réglage | SSIM All (Y) | Taille | Encodage | CPU | Mémoire par encodeur |
|---|---|---|---|---|---|---|
| VideoToolbox matériel | q65, `allow_sw=0` | 0,9792 (0,9688) | 105,4 Mo | +0,4 s | ≈ 0 | ≈ 180 Mo |
| Logiciel d'Apple | `require_sw=1`, 20 Mb/s | 0,9809 (0,9714) | 105,5 Mo | 6,5 s | 54,8 s | ≈ 495 Mo |
| openh264 | profil high, QP 23 | 0,9799 (0,9698) | 136,9 Mo | 16,8 s | 16,3 s | ≈ 16 Mo |

**Réglages retenus**, sans option pour l'utilisateur :

- **VideoToolbox matériel q65, puis openh264 QP 23** (décision de l'utilisateur,
  11 octobre 2026).
  - La cible d'openh264 est le SSIM de VideoToolbox q65 remesuré dans le moteur Rust
    (0,9792), avec 0,977 comme plancher.
  - Le 0,977 de [docs/09](09-profilage.md) passait par le BGR du Python et ne se compare
    pas tel quel.
  - QP 24 donne 0,9761. Il faut 28 Mb/s pour égaler QP 23 en débit, et le fichier pèse
    alors 143 Mo.
  - **Validé à l'œil** par l'utilisateur le 11 octobre 2026 : « Franchement c'est
    pareil ».
    - Passage comparé : la piste 0:23.48 de l'extrait, la moins contrastée (17 points,
      amplitude 68,7), images 660 à 767.
    - Fichiers : 9,3 Mo en VideoToolbox q65, 12,1 Mo en openh264 QP 23.
- **Pourquoi openh264 plutôt que le logiciel d'Apple** pour le repli :
  - 30 fois moins de mémoire et 3 fois moins de CPU ;
  - le même encodeur que sous Linux, donc testé partout ;
  - le prix : des fichiers 30 % plus gros.
- **Le logiciel d'Apple refuse q65** : l'ouverture échoue avec `-12900` (propriété
  `Quality` non gérée). Il ne marche donc qu'en débit fixé.
- **Couleur** : plage limitée, et BT.709 pour la matrice, le transfert et les primaires,
  écrits dans le flux H.264 par les deux encodeurs (décision du 11 octobre 2026).
  Hypothèse : toutes les sources sont en BT.709, ce qui est vérifié sur les 27 vidéos
  connues.

**Pièges de FFmpeg 7.1.1 relevés en mesurant** :

- **`allow_sw` ne choisit pas le logiciel d'Apple.**
  - `allow_sw=1` prend le matériel s'il répond.
  - Seul `require_sw=1` force le logiciel.
  - `allow_sw=0` exige le matériel.
- **q65 dans le processus** : `global_quality = 65 × FF_QP2LAMBDA` plus le drapeau
  `QSCALE`. Ce réglage n'existe que sur arm64.
- **Profil d'openh264** : sans réglage, il encode en Constrained Baseline (CAVLC). Le
  profil high (CABAC) donne le même SSIM et un fichier 8 % plus petit, mais il encode
  38 % plus lentement (16,8 s contre 12,2 s).
- **QP d'openh264** :
  - `rc_mode=off` laisse le QP à 26 quel que soit `qmin`/`qmax` ;
  - `rc_mode=quality` avec `qmin = qmax` applique le QP aux images I et P. Vérifié
    par `trace_headers`.
- **Fils d'openh264** : avec une tranche par image, ils ne changent rien (même fichier à
  l'octet, même temps). Le parallélisme vient de plusieurs extraits à la fois.
- **Options inconnues** : `open_with` les ignore sans erreur. Les tests relisent donc le
  profil et le PSNR.

**Encodeurs simultanés** (K encodeurs de 300 images chacun) :

- **Mémoire relevée** : le pic du processus, plus l'empreinte du service
  `VTEncoderXPCService` où tourne VideoToolbox.
  - openh264 : ≈ 16 Mo par encodeur.
  - Matériel : ≈ 130 Mo dans le service et ≈ 48 Mo dans le processus.
  - Logiciel d'Apple : ≈ 480 Mo dans le service et ≈ 15 Mo dans le processus.
  - Le matériel fait aussi monter la mémoire câblée du système, de +0,7 à +3,4 Go selon
    K. La mesure est trop bruitée pour l'attribuer : elle n'entre pas dans
    `Codec::footprint`.
  - `Codec::footprint` vaut 17 Mo pour openh264 (16,3 Mo mesurés, arrondis au-dessus) et
    180 Mo pour VideoToolbox.
- **Sessions matérielles** : 17 tiennent sur le M1 Max.
  - À partir de 18, toutes s'ouvrent, puis l'envoi d'images échoue (`-17691`).
  - Le repli à l'ouverture ne couvre donc pas ce cas : #112 borne les sessions ou refait
    l'extrait. Le seuil de la machine cible n'est pas mesuré.
- **Charge réelle** : la 125 (47 pistes, 39 zooms) n'a jamais plus de 8 fenêtres ouvertes
  à la fois.
- **Budget** : repère de 4 Gio pour l'ensemble des encodeurs ouverts. La borne se décide
  dans #112, avec la plus grosse empreinte de la liste de repli.

## Crates

Neuf crates au lot 6, `members = ["crates/*"]`. `unsafe` n'est permis que dans `media`.

- **`types`** : vocabulaire commun, sans entrée-sortie ni algorithme de détection. Un
  type n'y entre que s'il est produit par une crate et lu par au moins deux autres. Gelé
  pour le lot depuis #124 : un besoin imprévu passe par une petite PR fusionnée d'abord.
  - `Rational` : réduit à la construction, dénominateur positif, sans débordement
    (`OutOfRange` plutôt qu'une panique). S'écrit et se lit `num/den`, comme ffprobe.
  - `FrameIndex` (ordre où le décodeur rend les images, c'est-à-dire l'ordre
    d'affichage, à partir de 0) et `Pts` (ticks de la base de
    temps). En JSON, des nombres.
  - `FrameRate` : cadence positive ; son propre type, pour qu'elle ne s'échange pas avec
    la base de temps.
  - `Timeline` : `pts` de chaque image (`Option<Pts>`), base de temps positive et
    `FrameRate` (`avg_frame_rate`). Deux temps : `seconds_by_pts`, et `seconds_by_rate`
    (numéro ÷ cadence), celui des temps du CSV de la v0.1.1.
  - `Detection` : `frame`, barycentre `x`, `y`, boîte `left`, `top`, `width`, `height`,
    `area` et `amplitude`, en `f64` et en pixels de la vidéo d'origine.
    `FrameDetections` : une liste par image, images vides comprises, ordre gardé ; une
    détection rangée sous une autre image est refusée.
  - `Span` : plage d'images, bornes comprises, refusée si elle finit avant de commencer.
  - `Track` : numéro et points, au moins un, images strictement croissantes. Mesures :
    corde, longueur du trajet, surface maximale, amplitude de crête, virage médian
    (médiane des changements de cap, positions répétées ignorées, 0 sans virage).
  - `Language` (français, anglais), `SourceSize` et `WorkScale` : taille de travail
    (largeur demandée sans agrandir, hauteur arrondie, au moins 1), et seul passage des
    pixels de travail aux pixels d'origine (coordonnée `(c + 0,5) × échelle − 0,5`,
    longueur × échelle, surface × échelle²).
- **`media`** : frontière avec FFmpeg ; aucun type de `ffmpeg_next` n'en sort. Une
  erreur par module.
  - `frame` : `YuvFrame`, 4:2:0 planaire sans `stride`, avec sa plage de couleur.
  - `decode` : `Decoder::open`, `stream` (taille, cadence, base de temps, plage) et
    `next`, qui remplit un `YuvFrame` fourni par l'appelant (aucune allocation par
    image) et rend un `FrameInfo` non exhaustif.
    - Contrat : un paquet illisible est compté, pas fatal. Ceux qui suivent la dernière
      image se lisent par `unreadable_packets_after_last_frame`. `Err` rend le décodeur
      inutilisable ; après la fin, `next` rend toujours `None`.
    - Pour #117 : un `linesize` négatif (image retournée) rendrait faux le `data()` de
      ffmpeg-next. Le décodeur h264 n'en produit pas, mais rien ne le vérifie encore.
    - Cadence : `avg_frame_rate`, repli sur `r_frame_rate` si elle est inconnue.
    - Formats : `yuv420p` seul, `yuvj420p` lu comme plage pleine.
  - `encode` : `Settings::new` (taille paire, cadence positive), puis
    `Encoder::create`, `push` et `finish`, en openh264 ou VideoToolbox.
    - Réglages fixes, détaillés dans [Encodage](#encodage) :
      - VideoToolbox en q65, matériel seul (`allow_sw=0`) ;
      - openh264 en profil high, `rc_mode=quality`, QP 23.
    - Le fichier s'écrit sous un nom temporaire ; seul `finish` lui donne son nom.
    - Une image en plage pleine est refusée (`FullRange`). L'encodeur écrit une plage
      limitée en BT.709, dans le flux H.264.
    - Un paquet sans durée reçoit une durée d'une image.
    - La cadence est un rationnel exact : 10000/333 et 2500/333 se relisent à
      l'identique.
    - `Encoder::codec` dit quel encodeur écrit.
    - `Codec::footprint` donne la mémoire d'un encodeur ouvert en octets, mesurée en
      1440×1080, service hors du processus compris.
    - `encode::fallback::Fallback` est la liste ordonnée d'encodeurs, injectable :
      - `for_this_platform` donne `[VideoToolbox matériel, openh264]` sur macOS, et
        `[openh264]` ailleurs ;
      - `trial` encode quelques images grises jusqu'à `finish` avec chacun, et retire en
        tête de liste ceux qui échouent. C'est l'essai au démarrage ;
      - `create` ouvre le premier qui répond. C'est le repli à l'ouverture de chaque
        extrait. Un refus ne retire pas l'encodeur de la liste ;
      - si aucun ne répond, `NoEncoder` liste chaque encodeur avec sa cause ;
      - une destination inutilisable (dossier absent, chemin refusé) rend tout de suite
        `Destination`, sans essayer les encodeurs suivants.
    - **L'échec en cours d'écriture** (sessions matérielles saturées) n'est pas couvert
      par `Fallback` : il revient à #112.
  - `library` : version et licence du FFmpeg lié.
- **`reference`** → `types` : la référence v0.1.1 en Rust. Gelée pour le lot depuis
  #124, comme `types`.
  - `fixture` : chemins des fixtures et garde LFS. `location` : `committed(nom)` pour
    l'extrait et la 092 du dépôt, `recording_directories()` pour les dossiers `data/*`
    de `BATDETECT_REFERENCE_DIR`, triés par nom ; une variable absente ou vide est
    refusée. `recorded_video(nom)` trouve le `.mp4` d'un enregistrement par son nom
    (`video.name` de `video.json`) dans le dossier de `BATDETECT_VIDEOS_DIR` (sur le Mac
    de l'enregistrement : `~/dossier sans titre/img_0000/`), avec le même refus d'une
    variable absente ou vide.
  - Lecture et écriture des quatre fichiers, un module chacun : `video`
    (`VideoSummary`), `frames` (`Frames` : table par image et `FrameDetections`, de même
    longueur, vérifiée par `Frames::new`),
    `tracks`, `table` (CSV, fins de ligne CRLF, flottants à l'écriture la plus courte).
    `recording` les réunit et donne la `Timeline`. Un nom en `.gz` est compressé
    (gzip sans nom ni date) ; les sections sans type de `video.json` (décodeur, réglages,
    versions) sont gardées telles quelles dans `recorded`. Relus puis réécrits, les
    fichiers redonnent les mêmes valeurs.
  - Pas d'affirmation « points d'une piste ⊂ détections » : les points des pistes
    jumelles fusionnées sont combinés (17 sur 358 dans l'extrait, 63 sur 610 dans la
    092).
  - `detection_matching` : même image, distance strictement inférieure à 3 px, chaque
    détection appariée au plus une fois, la plus proche d'abord ; rend les paires et les
    détections seules de chaque côté. Une position non finie reste seule. Un à un,
    contrairement au prototype (plus proche voisin sans exclusivité) : écart mesuré par la
    revue de #124 sous 0,01 % (8 détections sur 542 261 pour la 103, décalées d'1 px).
    Les écarts de détections n'ont pas de type : `compare` en tire le temps par
    `Detection::frame` et la `Timeline`.
  - `track_matching` : début à ±5 images, le plus proche d'abord puis l'écart de points
    le plus faible ; une paire s'accorde si ses points diffèrent de 3 au plus. Glouton,
    comme le prototype : deux pistes à moins de 10 images l'une de l'autre peuvent
    laisser une paire possible de côté (test qui le fige). Les écarts
    à juger (piste manquée, en trop, points hors tolérance) portent la vidéo, l'image de
    début et son temps par la cadence (`FrameRate::seconds`), triés par image.
- **`cli`** : paquet `batdetect`, binaire `batdetect-rs`. En #95, `--version` affiche la
  version et la ligne de licence de FFmpeg. Tout texte vu par l'utilisateur vit dans
  `messages.rs`.
- **`tracking`** → `types` : exclusion des images abîmées et instables (#120), puis
  suivi (#121). `cli` et `compare` enchaîneront `exclude` puis `track_detections`.
  - `exclusion` : seul point d'entrée. `exclude` reçoit les détections, les plages
    abîmées toutes faites, le nombre d'images signalées et la `FrameRate`. Les images
    abîmées sont retirées du calcul de stabilité, puis les détections des plages
    abîmées et instables sont vidées, images gardées. `Exclusion` rend aussi le nombre
    d'images ignorées et celui des images abîmées, chacune comptée une fois. Le
    contrôle du nombre d'images reste à #116.
  - `stability` : `StabilityConfig::new(max_blobs, pad_s)`, par défaut 20 et 1,0 s ;
    `max_blobs` est un `Option<NonZeroU32>`, `None` coupe le filtre (la commande
    traduira `--max-blobs 0`) ; `pad_s` fini, positif ou nul. Limite de foule :
    `max_blobs + 6 × médiane` du nombre de détections des images lisibles, vides
    comprises, calculée en entiers. Une image qui l'atteint donne une plage de
    `pad_s × cadence` images de chaque côté, arrondi au plus proche, moitié vers le
    haut (la v0.1.1 arrondit au pair : une image d'écart quand ce produit vaut x,5,
    jamais avec les réglages par défaut). Le rembourrage est borné à la longueur de la
    vidéo. Seuls les bords sont bornés aux premières et dernières images lisibles : au
    milieu, une plage instable peut recouvrir une plage abîmée (093, 097). Deux plages
    séparées d'au plus le rembourrage se fondent.
  - `spans` (interne) : images couvertes par des plages, bornées à la vidéo, et
    détections vidées.
  - Les 3 tests Python de saut de luminosité (`test_stability.py`) attendent la
    détection (#118).
  - La médiane porte sur toute la vidéo : la stabilité dépend de sa longueur (la
    concaténation 089-091 ignore 27,9 s, contre 28,9 s en morceaux, mesuré par #94 ;
    la médiane comme cause est une hypothèse, non vérifiée).
  - Sur les 26 vidéos de la référence, plages instables et détections gardées sont
    égales à la v0.1.1 (test local `crates/tracking/tests/local_exclusion.rs`).
  - `track` : `track_detections(détections, TrackConfig)` rend des `Track` numérotées à
    partir de 1, triées par (image de début, `x` du premier point).
    - `TrackConfig::new(max_jump, max_gap, min_hits, min_travel, twin_distance,
      max_median_turn)`, par défaut 120, 6, 6, 45, `Some(36)` et 0,8. Les accesseurs
      portent les noms des clés de `params.json`. `max_gap` et `min_hits` sont des
      `NonZeroUsize`, `twin_distance` un `Option<f64>` (`None` coupe la fusion ; la
      commande traduira `0`). Un réglage hors bornes rend `Error::Setting` avec son
      `TrackSetting`.
    - Écarts voulus à la v0.1.1 : NaN refusé pour `max_jump`, `min_travel` et
      `twin_distance` ; infini refusé pour les quatre réglages `f64` ; `Some(0)` refusé
      pour `twin_distance`, le `0` de la v0.1.1 devenant `None`.
    - Image par image : les pistes restées plus de `max_gap` images sans détection
      finissent. Chaque piste prédit sa position par sa vitesse sur au moins 3 images
      (l'avant-dernier point à défaut). L'appariement est glouton sur
      (pistes qui ont une vitesse d'abord, distance ≤ `max_jump`, ordre de création,
      ordre des détections) ; une détection restée seule ouvre une piste.
    - Jumelles : pistes qui coexistent, partagent au moins une image et restent à
      ≤ `twin_distance` sur chaque image commune. Groupes par union-find sur les pistes
      triées par image de début ; les groupes sortent dans l'ordre de leur premier membre.
      Les points
      d'une même image se combinent : boîte englobante, surfaces additionnées,
      barycentre pondéré, amplitude maximale.
    - Filtres après fusion : `min_hits`, corde ≥ `min_travel`, virage médian ≤
      `max_median_turn` (mesures de `types::track::Track`).
    - Limite : plusieurs seuils tombent sur une distance exacte, et `f64::hypot` peut
      différer d'un ulp de celui de Python. Le critère des tests est l'appariement avec
      tolérances ; l'identité est mesurée en plus.
    - Mesure (11 octobre 2026), avec en entrée les détections brutes de la référence
      passées par `exclude` : sur les 26 vidéos (`local_tracking.rs`), 966 pistes sur
      966 appariées, 0 piste seule, 0 écart, et les 966 paires sont identiques au bit
      près sur les huit champs de chaque point et sur les numéros. Les fixtures
      (`committed_tracking.rs`) donnent 4 et 14 paires, identiques de même.
    - Critères des tests : sur l'extrait et la 092 (`committed_tracking.rs`, en CI),
      l'identité exacte fait échouer, en plus de l'appariement. Sur les 26 vidéos
      (`local_tracking.rs`), l'appariement avec tolérances fait échouer et l'identité
      est rapportée (décision de l'utilisateur, 11 octobre 2026). Une amélioration voulue
      du suivi mettra à jour l'attendu des fixtures, avec sa justification dans la même
      série de changements.
- **`files`** → `types` : fichiers écrits par la commande (#122). Reçoit un
  `types::Language` ; la langue du système viendra avec la commande (#123), puis
  Fluent (#114).
  - `names` : `Names::new(langue)` et ses méthodes. Français : `extraits`, `piste`,
    `_pistes`, `_resume`, `_annotee`, sans accent. Anglais : `clips`, `track`, `_tracks`,
    `_summary`, `_annotated`. `params.json` n'est pas traduit, `_zoom` est commun.
    - Le nom d'extrait se construit depuis la `Track`, avec le temps de début passé par
      l'appelant en secondes. Il s'écrit en minutes et secondes entières, tronquées comme
      dans la v0.1.1 : à 59,996 s, le nom dit `0m59s` et le CSV `1:00.00`.
    - Le numéro de piste prend la largeur du nombre de pistes, 2 chiffres au moins.
    - La période de l'image résumé est donnée en minutes (`_000m-010m`).
    - Les noms de vidéo sont des `OsStr`, gardés tels quels.
    - Ni ménage ni anciens noms (#115).
  - `time` : `clock` écrit `m:ss.cc` à partir de centièmes entiers.
    - Le temps d'une image vient de son `pts` (`Timeline::seconds_by_pts`). Une image
      sans `pts` reprend celui de la dernière image qui en a un, plus un pas de cadence
      par image ; la cadence seule ne sert qu'en l'absence de tout `pts`.
    - La fin d'une plage est le début de l'image suivante.
    - Le temps n'est pas rapporté au premier `pts` : les 26 vidéos de la référence
      commencent à `pts` 0. Un `pts` négatif reste à décider avec #116.
  - `table` : CSV des pistes, mêmes 12 colonnes et même sens que la v0.1.1 (en-têtes
    anglais, virgule, point décimal, CRLF, sans guillemets, même contenu dans les deux
    langues). La forme définitive reste à #84.
    - Les nombres sont arrondis, puis écrits sous leur forme la plus courte, avec `.0`
      pour un entier (`12.0`) ; `-0.0` devient `0.0`.
    - `max_area_px` est entier, arrondi à mi-chemin vers le haut.
    - Durée = fin − début, au moins une image.
  - `params` : `Params<S>` écrit `video` (un chemin), `fps`, les réglages de l'appelant
    aplatis, `ignored_s` et `damaged_s` (au centième), `damaged_frames`, `version`
    (fixée à la compilation par l'appelant) et `ffmpeg` (version liée). Ni `mode` ni
    `ffprobe`.
    - Le nombre de fils réel arrive avec #123.
    - `files` n'impose pas les clés des réglages. Garder celles de la v0.1.1 (une clé
      par configuration, au nom de sa classe, dont les seuils de zoom dans
      `RenderConfig`) revient à l'appelant : #123 et #101 en portent les tests.
    - Une clé `video` ou `fps` venue des réglages serait écrite en double.
  - Mesure (11 octobre 2026), CSV écrit depuis les pistes de la référence :
    - extrait : identique à l'octet au `tracks.csv` de la v0.1.1, aux fins de ligne près
      (le dépôt garde la fixture en LF) ;
    - 092 : 14 lignes. `id`, `hits`, `chord_px`, `path_px`, `peak_amplitude`,
      `filled_frames` et `duration_s` sont égaux.
    - Les temps sont plus tôt d'un ou deux centièmes : débuts décalés sur 11 pistes sur
      14 (de 0:33.94 → 0:33.93 à 3:57.18 → 3:57.16), fins sur 12. La v0.1.1 divisait le
      numéro d'image par la cadence moyenne (30,027 i/s), alors que les `pts` avancent
      de 333/10 000 s par image (30,03 i/s).
    - `speed_px_s` est plus haute de 0,1 à 0,2 px/s sur 10 pistes, jamais plus basse.
    - `max_area_px` : pistes 1, 5, 6 et 10, 274 → 275, 400 → 401, 490 → 491 et
      274 → 275. Les maxima tombent sur un demi (sur la 092, les surfaces sont des
      multiples de 2,25) : le Rust arrondit à mi-chemin vers le haut, la v0.1.1 au pair.
      Gardé (règle 5).
- **À venir** :
  - `engine` (#116, lecture, images abîmées, détection) ;
  - `draw` (#101, dessin) ;
  - `compare` (#119, écarts à la référence).

## Référence v0.1.1

Format, emplacement et versions de la référence que le Rust doit égaler : README de
`tests/fixtures/reference/` (#94). La crate `reference` la lit ; le test local
`crates/reference/tests/local_reference.rs` relit les 26 vidéos (966 pistes) et les
apparie chacune à elle-même.
