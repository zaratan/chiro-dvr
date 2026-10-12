# Migration vers Rust

Le lot 6 commence la réécriture de `batdetect` en Rust. Le but est de détecter au moins
aussi bien que la v0.1.1, pas de reproduire le Python bit à bit : les détections et les
pistes se comparent par appariement avec tolérances, et chaque écart se juge à l'œil.
Jusqu'à #106, le Python reste l'outil livré ; [02](02-methode.md) et
[10](10-logique-de-detection.md) décrivent le Python, ce document décrit le Rust.

## État (octobre 2026)

#95 a posé le workspace, FFmpeg, l'outillage et quatre crates. La commande
`batdetect-rs` ne fait encore que `--version`. Le moteur arrive avec #116 à #119 (#116 :
crate `engine`, lecture et refus à l'entrée ; #117 : images abîmées et canari ; #118 :
détection ; #119 : comparaison à la référence et identité entre systèmes), le suivi
avec #120 à #123.

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
- **Roboto en SIL Open Font License 1.1** (#101).
  - Embarquée dans `draw` : `crates/draw/fonts/Roboto.ttf`, police variable `wdth` et
    `wght`.
  - Licence à côté : `crates/draw/fonts/OFL.txt`.
  - Provenance : dépôt `google/fonts`, fichier `ofl/roboto/Roboto[wdth,wght].ttf`, commit
    `6183fc0` du 27 janvier 2026. sha256 `d7598e12…872a134`.
  - Pas de nom de police réservé. Elle entre dans `packaging/NOTICE` à #93.
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
   **Exception** (décision de l'utilisateur à #118) : le gris de la détection reprend la
   table mesurée sur OpenCV et l'arrondi à 8 bits après la réduction, parce que les
   seuils de la v0.1.1 sont réglés sur ce gris (mesures dans `engine`, « Gris de la
   détection »).

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
    La règle vaut pour ce qui sort du moteur, plages abîmées exclues : les pixels que
    FFmpeg dissimule sur une image abîmée dépendent du nombre de fils de décodage
    (mesure dans `engine`).
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
- Déclarées d'avance pour le lot : `rayon`, `dhat`. Tant qu'elles ne
  servent pas, elles ne sont pas dans `Cargo.lock` : leur version n'y est pas figée.
- `serde` sert `types` et `reference` depuis #124. `serde_json` a la feature
  `float_roundtrip` : un flottant de la référence se relit au bit près. `flate2`
  (moteur `miniz_oxide`, 100 % Rust) lit et écrit les `.gz` de la référence ; `zlib-rs`
  figure dans `Cargo.lock` sans être compilé.
- **`ab_glyph` est retiré.** Il tire `ttf-parser`, déclaré non maintenu par
  RUSTSEC-2026-0192 sans mise à jour sûre, et l'utilisateur refuse d'ignorer l'avis.
- **Pile de polices de #101** (accord de la tour, 11 octobre 2026) :
  - `skrifa` 0.48.0 (fontations de Google, MIT OR Apache-2.0), suggéré par l'avis :
    table des caractères, métriques, contours, axes variables ;
  - le tracé par `tiny-skia` 0.12.0 (BSD-3-Clause), sans sa feature `png` ;
  - `unicode-normalization` 0.1.25 (MIT OR Apache-2.0) pour la forme NFC.
  - `read-fonts` tire `yoke`, `zerofrom` et `synstructure`, sous Unicode-3.0, déjà
    admise.
  - Écartés : `fontdue` et `rusttype` (ils tirent `ttf-parser`), `swash` (un second
    rastériseur à côté de tiny-skia), `icu_normalizer` (une quinzaine de crates pour une
    seule normalisation).
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
| slow | `tests/slow_*.rs`, `#[ignore = "slow"]` | `mise run rust:slow` | l'extrait LFS : 1 230 images, 10000/333, 1440×1080, plage `tv` ; le test de mémoire `slow_memory_flash` (`dhat`), hors de la couverture |
| reference | `tests/reference_*.rs`, `#[ignore = "reference"]` | `mise run rust:reference` | la 092 : 9 011 images, cadence moyenne 90110000/3000927 |
| matériel | `tests/hardware_*.rs`, `#[ignore = "hardware"]` | étape d'essai de la CI macOS | VideoToolbox sans repli logiciel |
| local | `tests/local_*.rs`, `#[ignore = "local"]` | `BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=… cargo test --workspace --locked --profile ci --test 'local_*' -- --include-ignored` | les 26 vidéos de la référence, hors dépôt, jamais en CI |

- Le niveau d'un test se choisit par le nom de son fichier. Ajouter un test ne demande
  de toucher ni la CI ni `mise.toml`.
- Un test rapide peut relancer son propre binaire sur un test `#[ignore = "run by …"]`
  pour lire ce qu'il écrit sur stderr (aucun message du décodeur pendant la lecture).
- Couverture mesurée sur les tests rapides, en profil `ci` (seuil 95 %). CI de la PR
  #130 : 97,95 % des lignes sous Linux et 98,14 % sous macOS. Les lignes restantes sont
  des erreurs de FFmpeg qu'aucune vidéo de test ne provoque.
- Le test de mémoire est un test `slow` (#146) : sous l'instrumentation de la
  couverture, il prenait 83 s sous Linux et 55 s sous macOS (PR #145), 1 s en profil
  `ci` sur le M1 Max. Il ne couvrait aucune ligne de plus : 96,98 % des lignes sous
  macOS avec lui (CI de la PR #145) comme sans lui (M1 Max, même commit). Il tourne
  toujours sur les trois systèmes, par `rust:slow`.
- Les fixtures se trouvent par `reference::fixture::fixture`, qui refuse un pointeur
  LFS non tiré avec le conseil `git lfs pull`.
- `tests/fixtures/rust/video_092_original_3m24-4m05/frames.jsonl` : les détections de
  l'extrait par le Rust, écrites sur le Mac depuis la racine du dépôt par
  `lockf …/mesure.lock cargo run --profile ci -p compare -- extract --frames tests/fixtures/rust`
  (fichier git ordinaire, fin de ligne `\n` forcée par `.gitattributes`). Le test
  `slow_extract` de `compare` le redemande au bit près sur macOS, Linux et Windows ; un
  écart remonte à l'utilisateur (voir `compare`).
- Trois petites fixtures, hors LFS, faites une fois par le ffmpeg 9.0.2 de Homebrew
  (libx264) et commitées :
  - `full_range.mp4` (1,9 Ko, `yuvj420p`, plage `pc`, refusée à l'ouverture) :
    `ffmpeg -f lavfi -i color=gray:size=1440x1080:rate=30 -frames:v 2 -c:v libx264 -bf 0 -pix_fmt yuvj420p -color_range pc full_range.mp4`
  - `full_range_midway.mp4` (3,0 Ko, 2 images en plage non précisée puis 2 en `pc`,
    refusée à la troisième image) : `tv.h264` et `pc.h264` par la commande précédente
    avec `-x264-params repeat-headers=1 -f h264`, l'un en `-pix_fmt yuv420p -color_range tv`,
    l'autre en `-pix_fmt yuvj420p -color_range pc`, puis
    `cat tv.h264 pc.h264 > midway.h264 && ffmpeg -framerate 30 -i midway.h264 -c copy full_range_midway.mp4`
  - `two_gops.mp4` (5,9 Ko, 320×240, 20 images, images-clés en 0 et 10, perte au début
    par la couture) :
    `ffmpeg -f lavfi -i testsrc=size=320x240:rate=30 -frames:v 20 -c:v libx264 -bf 0 -g 10 -keyint_min 10 -sc_threshold 0 -crf 40 -pix_fmt yuv420p -color_range tv two_gops.mp4`
- **CI** : tâche `rust` sur macOS et Linux (statique, tests rapides, slow, contrôles du
  binaire) et tâche `rust-reference` sous Linux, en parallèle de `rust` depuis #146.
  Quand l'empreinte de FFmpeg change, `rust-reference` le construit donc aussi, sans le
  sauver : seule `rust` sauve le cache de FFmpeg. Le binaire contrôlé se construit par `--workspace --bin batdetect-rs` : avec
  `-p batdetect`, les features de `syn` changent (`draw` en ajoute par `skrifa`), et
  `bindgen`, `ffmpeg-sys-next` et `media` se recompilaient (11 s sous Linux, 22 s sous
  macOS, 60 s sous Windows à la PR #145). Le cache de FFmpeg est
  restauré, puis sauvé dès la construction, même si la suite échoue. L'action `setup`
  n'installe les outils cargo et la toolchain qu'avec `rust: true`. Premier passage
  (PR #130), sans cache de `target/` : `rust` 3 min 04 sur macOS et 3 min 46 sous Linux ;
  `rust-reference` 1 min 15.
- **Tâche `rust-windows`**, indépendante des autres. Elle construit FFmpeg, lance les
  tests rapides et `slow`, puis `cargo clippy --profile ci -- -D warnings`. En profil
  `ci`, clippy reprend les scripts de construction et les macros des tests ; en `dev`,
  il recompilait 46 unités, dont les liaisons de FFmpeg (82 s à la PR #145).
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
    pour clippy sous Unix, `ci` pour les tests, le binaire et clippy sous Windows),
    `target/llvm-cov-target` pour la couverture, et la rlib de ffmpeg-sys-next avec ses
    liaisons.
  - Il se sauve en fin de tâche réussie.
  - Sa clé porte, dans cet ordre, la tâche, l'OS, l'architecture, l'image du runner
    (`ImageOS`), **l'empreinte de FFmpeg**, `rust-toolchain.toml`, puis `Cargo.lock`.
    Cargo ne suit pas les `.a`. Un `target/` construit avec un autre FFmpeg garderait
    donc des binaires déjà liés à l'ancien, et les liaisons de bindgen tirées de ses
    en-têtes (voir « Reconstruction »).
  - En PR seulement, une clé de repli (`restore-keys`, #146) prend tout sauf
    `Cargo.lock` : une PR qui change une dépendance reprend le `target/` de `main` pour
    la même image, le même FFmpeg et la même toolchain, et ne recompile que ce qui a
    changé. Elle ne mélange jamais deux empreintes de FFmpeg.
  - Sur `main`, pas de repli : un nouveau `Cargo.lock` repart d'un `target/` vide. Un
    `target/` repris par le repli garde les artefacts des versions remplacées ; sans
    cette remise à zéro, il grossirait à chaque changement de dépendance. Tailles à la
    PR #145 : 393 Mo (`rust`, Linux), 366 Mo (macOS), 470 Mo (Windows), 101 Mo
    (`rust-reference`).

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

## Plage réelle des sources

Mesure du 11 octobre 2026 (#101) :

- 20 images par vidéo, réparties hors des plages abîmées de la référence, avec une
  marge de 15 images.
- Programme et table brute : `~/Projects/dvr-wt/rapports/mesure-101-plage/`
  (`plage.tsv`).
- Les 26 vidéos annoncent une plage limitée (`tv`), sur leur conteneur comme sur chaque
  image.

| Vidéos | Y hors 16..235 | Pire image | U/V hors 16..240 | Extrêmes |
|---|---|---|---|---|
| 089 à 103 | 0,92 à 1,96 % | 7,81 % (094) | 0,020 à 0,035 % | Y 0-255, U 68-146, V 98-255 |
| 120 à 130 | 0,41 à 0,62 % | 0,75 % | 0,031 à 0,033 % | Y 0-255 (0-252 pour la 120), U 75-147, V 113-255 |

- **Origine des pixels hors plage**, sur l'image 716 de l'extrait :
  - Y < 16 : surtout le haut de l'image, arbres et ciel froids ;
  - Y > 235 : surtout le bas, sol chaud ;
  - c'est le contenu thermique lui-même.
- **V à 255** : un seul petit carré, vraisemblablement une icône rouge de l'affichage des
  jumelles.
- **Décision de l'utilisateur** : les sources se lisent en plage pleine pour l'affichage,
  quelle que soit leur étiquette. Les fichiers produits restent en plage limitée BT.709
  (#111), resserrés sans écrêtage. L'application aux extraits revient à #112, celle à
  l'image résumé à #104.

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
  - `decode` : `Decoder::open(chemin, Threads)`, `stream` (taille, cadence, base de
    temps, plage, `pts` de départ `Option<Pts>`, nombre d'images annoncé par le conteneur
    `Option<NonZeroU64>`) et `next`, qui remplit un `YuvFrame` fourni par l'appelant
    (aucune allocation par image) et rend un `FrameInfo` : `FrameIndex`, `Option<Pts>`,
    image-clé (`is_key`), erreurs de décodage (`DecodeErrors`) et paquets illisibles
    depuis l'image précédente.
    - `DecodeErrors` (module `decode_errors`) : les `decode_error_flags` de l'image, lus
      tels quels et nommés par les constantes `FF_DECODE_ERROR_*` de FFmpeg, et non par
      le `DecodeError` de ffmpeg-next, qui perd `CONCEALMENT_ACTIVE` et `DECODE_SLICES`
      sans son cfg `ffmpeg_5_1`. Une image est en erreur si ses drapeaux ne sont pas
      vides, y compris un bit inconnu. Une tranche coupée donne 12 (dissimulation et
      tranches manquantes).
    - Fils : `Threads::Automatic` (choix de FFmpeg) ou `Threads::Exactly(n)`, en fils
      d'image. `active_threads` lit, après l'ouverture, ce que FFmpeg fait vraiment :
      `Single`, `Frame(n)` ou `Slice(n)`. Un test vérifie qu'un fil donne `Single`, que 4
      donnent `Frame(4)` et que l'automatique donne plusieurs fils d'image sur une machine
      à plusieurs cœurs : un FFmpeg construit sans fils ferait échouer la CI.
    - Couture : `with_seam` reçoit un `PacketSeam` qui voit chaque paquet du flux vidéo,
      dans l'ordre de lecture, avant `send_packet` (rang, image-clé, taille, `data_mut`,
      `shrink`). `data_mut` rend d'abord le paquet inscriptible
      (`av_packet_make_writable`, sans copie quand le tampon n'a qu'une référence) et
      rend `None` s'il ne peut pas l'être. Un paquet raccourci sans réécrire la longueur de son NAL est refusé en
      entier par le décodeur et compté illisible ; raccourci à 0 octet, il fait échouer
      `send_packet` (`EINVAL`).
    - Contrat : un paquet illisible est compté, pas fatal. Ceux qui suivent la dernière
      image se lisent par `unreadable_packets_after_last_frame`. `Err` rend le décodeur
      inutilisable ; après la fin, `next` rend toujours `None`.
    - Fin de fichier : FFmpeg 7.1.1 rend déjà l'erreur d'entrée-sortie du contexte
      (`pb->error`, sauf `EAGAIN`) à la place d'un `EOF` du démultiplexeur
      (`libavformat/demux.c:1530`) ; elle sort en erreur `Read`. Une lecture tronquée en
      fin de fichier sans erreur d'entrée-sortie donne moins d'images : la garde
      d'`engine` la refuse.
    - Image retournée : un `linesize` négatif rendrait faux le `data()` de ffmpeg-next
      (longueur `linesize as usize × hauteur`). Il est refusé (`FlippedFrame`) avant tout
      appel à `data()`. Le décodeur h264 n'en produit pas.
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
  - `convert` : `Converter` passe du YUV 4:2:0 au RGBA opaque, et retour (#101).
    - **Matrice** : BT.709 dans les deux sens (`sws_setColorspaceDetails`), décision du
      11 octobre 2026 (#111).
    - **Plage lue** : un paramètre explicite (`SourceRange`), pas l'étiquette de l'image.
      Nos jumelles se lisent en plage pleine (décision de l'utilisateur, 11 octobre
      2026, sur la mesure de [Plage réelle des sources](#plage-réelle-des-sources)).
    - **Plage écrite** : le retour au YUV se fait toujours en plage limitée. Une source
      pleine y est resserrée sans rien écrêter : Y 0 devient 16 et Y 255 devient 235.
    - Aucune allocation par image.
    - Filtres mesurés :
      - « point » à la montée et « area » à la descente, chrominance horizontale
        pleine ;
      - aplats de la palette alignés sur des blocs 2×2 : aller-retour à 2 niveaux au
        plus, contre 34 à 143 avec le bilinéaire.
    - Image 716 de l'extrait, sans dessin :
      - lue en plage limitée, comme le dit son étiquette : 345 704 pixels en noir pur
        et 9 719 en blanc pur ;
      - lue en plage pleine : 38 et 15, exactement des pixels à Y 0 et Y 255 de la
        source ;
      - réécrite en plage limitée, puis relue : écart moyen de 0,086, et 0,016 % des
        valeurs au-delà de 2 niveaux (973 sur 6,2 millions, non localisées).
  - `library` : version et licence du FFmpeg lié.
- **`draw`** (#101) : dessin pur sur une toile RGBA, sans FFmpeg ni entrée-sortie.
  - `text` : `Typeface::roboto`, la police embarquée.
    - Graisses 400 pour le texte et 600 pour les numéros (choix de l'utilisateur sur la
      planche, 11 octobre 2026).
    - Une taille est la hauteur des chiffres en pixels. Les chiffres ont tous la même
      largeur.
    - Texte normalisé en NFC. Un caractère absent devient U+FFFD.
    - Ni crénage ni placement d'accent par la table GPOS : il suffit pour des numéros et
      des légendes en caractères précomposés.
    - `shape` prépare un texte une fois (contours et positions), que
      `Canvas::shaped_text` redessine sans allocation, par exemple un `#N` à chaque
      image.
    - `TextStyle::new` refuse une taille qui n'est pas un nombre positif.
  - `canvas` : `Canvas`, toujours opaque, si bien que ses octets sont du RGBA simple
    pour `media::convert`. Primitives, toutes lissées (choix de l'utilisateur, même
    planche) :
    - rectangle aux bords nets :
      - 1 px couvre les pixels du bord de la boîte ;
      - chaque pixel de plus élargit le bord vers l'extérieur, si bien que la boîte de
        2 px (image détectée) et celle de 1 px (image interpolée) gardent le même bord
        intérieur ;
    - polyligne, ligne, segments chacun de son épaisseur ;
    - une épaisseur nulle ou non finie ne dessine rien ;
    - disque avec contour, polygone plein ou contouré ;
    - texte ;
    - assombrissement à 0,6, arrondi au plus proche, alors que la v0.1.1 tronque :
      écart d'un niveau au plus (règle 5 de la charte).
  - `color` : la palette des 12 couleurs et les couleurs des extraits et du panneau, en
    RGB. Un test épingle chacune au BGR inversé de `output/style.py`.
  - `style` : `style_for`, les tailles du résumé selon la largeur.
    - Mêmes proportions que le Python ; le rayon des pastilles est mesuré avec la
      nouvelle police.
    - L'arrondi va à l'entier le plus loin de zéro, selon la règle 5 de la charte (pas de
      `round_ties_even` pour copier `round()`). 1440 / 64 tombe pile sur 22,5 : en 1440,
      rayon 23 et chiffres de 21 px, contre 22 et 20 en Python, qui arrondit au pair.
    - `TRACK_LABEL` : le `#N` des extraits, chiffres de 17 px en graisse 600, figé par un
      test.
  - Choix faits sur une planche de rendus : Roboto contre Noto Sans et IBM Plex Sans,
    sur l'image 716 de l'extrait, comparée au rendu de la v0.1.1.
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
    `Detection::frame` et la cadence (`FrameRate::seconds`).
  - `track_matching` : début à ±5 images, le plus proche d'abord puis l'écart de points
    le plus faible ; une paire s'accorde si ses points diffèrent de 3 au plus. Glouton,
    comme le prototype : deux pistes à moins de 10 images l'une de l'autre peuvent
    laisser une paire possible de côté (test qui le fige). Les écarts
    à juger (piste manquée, en trop, points hors tolérance) portent la vidéo, l'image de
    début et son temps par la cadence (`FrameRate::seconds`), triés par image.
- **`engine`** → `media`, `types` : le moteur. `input` en est le seul point d'entrée.
  - `Input::open(chemin, Threads)` refuse, par une erreur typée qui nomme la vidéo :
    une taille autre que 1440×1080 (#98 lèvera ce refus), la plage pleine (`yuvj420p`
    ou `pc`, à l'ouverture et sur chaque image), 0 image annoncée, et, par `media`, un
    format autre que `yuv420p`. `Video` donne la taille, la cadence, la base de temps, le
    `pts` de départ et le nombre annoncé avant la lecture. Le refus de la plage pleine
    est à réexaminer avec #98 : la détection lit le plan Y tel quel, quelle que soit
    l'étiquette, et les sources sont lues comme pleines pour l'affichage (décision du
    11 octobre 2026).
  - `Input::with_seam` passe une couture de paquets au décodeur (canari).
  - `Input::read(visite)` décode toute la vidéo et appelle la visite pour chaque image
    (`FrameInfo` et `YuvFrame` complet). Une erreur de la visite arrête la lecture et
    revient telle quelle ; une erreur de lecture revient par `From<input::Error>`.
    `Reading` garde la table par image et les pertes (`Losses`).
  - Lecture anticipée (`read_ahead`) : un fil de `std::thread::scope` décode jusqu'à 4
    images d'avance (`READ_AHEAD`) dans 5 tampons faits une fois puis recyclés. Le fil
    s'arrête dès que le consommateur s'arrête ou échoue.
  - Pertes (`Losses`), qui deviennent des plages abîmées (`damage`) :
    - `at_jumps`, aux sauts de `pts` : le pas est la moyenne des intervalles réguliers
      selon la médiane des écarts de `pts` entre images connues (juste aussi pour un pas
      fractionnaire) ; un écart de k pas arrondis perd k − 1 images ;
    - `before_first`, avant la première image : écart entre son `pts` et le `pts` de
      départ du flux, en pas arrondis, moins les images déjà rendues sans `pts` ; 0 si
      le départ ou le pas est inconnu ;
    - `after_last`, après la dernière : les paquets illisibles qui la suivent (un paquet
      par image ; mesuré égal aux images perdues sur une fin abîmée, avec 1, 3 fils ou
      en automatique).
  - Garde du nombre d'images : décodées = annoncées − toutes les pertes. Seule une perte
    inexpliquée, qui ne laisse ni saut ni paquet illisible au bord, fait refuser la vidéo
    (`FrameCount`).
  - `damage` : `Damage::of(lecture, marge)` rend les images en erreur (`DecodeErrors`
    non vides), les images qui suivent un saut de `pts` et les plages abîmées
    (`Vec<Span>`), comme `damaged_spans` de la v0.1.1, avec deux différences :
    - une plage part d'une image en erreur ou d'une image après un saut, comme en
      v0.1.1 ;
    - elle part aussi de la première image s'il manque des images avant elle
      (`before_first`, l'équivalent d'un saut), et d'un départ fictif juste après la fin
      s'il en manque après la dernière (`after_last`) : la plage ne couvre alors que les
      `marge` dernières images, et aucune si la marge est nulle, car les images perdues
      n'entrent dans aucune fenêtre de fond. Ces départs aux bords sont un ajout : la
      v0.1.1 refusait ces vidéos (`check_frame_count`), le Rust les lit et les exclut
      (décision de l'utilisateur à #116) ;
    - le saut se juge avec le pas des pertes (`at_jumps`) : un écart de k images en
      vaut plus que k s'il dépasse k + ½ pas, au lieu de 1,5 × k × pas médian en
      v0.1.1. Même résultat quand tous les `pts` sont connus (k = 1, les 26 vidéos) ;
      à travers un `pts` inconnu (k ≥ 2), le Rust compte l'image cachée, la v0.1.1 non ;
    - elle va jusqu'à l'image qui précède l'image-clé suivante, ou jusqu'à la fin ;
    - elle est élargie de la marge de chaque côté, bornée à la vidéo, puis fondue avec
      la précédente si elle la touche. La marge est la demi-fenêtre du fond, passée par
      l'appelant (`DetectConfig` arrive avec #118).
  - Canari (`tests/slow_canary.rs`, sur chaque système de la CI) : la couture coupe le
    dernier tiers de la tranche des images 248 (P au milieu d'un GOP de 15), 555
    (image-clé), 860 et 861 (consécutives) et 1222 (dernier GOP) de l'extrait, en
    réécrivant la longueur du NAL, comme `cut_slices` du Python.
    - Le Rust signale exactement {248, 555, 860, 861, 1222}, avec 1 fil comme en
      automatique.
    - ffprobe 9.0.2 (Homebrew, `-threads 1 -show_log 16`, arguments de
      `src/batdetect/probe.py`) en signale autant sur une copie coupée de la même façon,
      mesurée une fois : `ffmpeg -c copy -bsf:v h264_mp4toannexb -f h264`, `cut_slices`
      sur ces images, puis `ffmpeg -framerate 10000/333 -i cut.h264 -c copy cut.mp4`.
      L'ensemble est écrit en constante ; le test vérifie que celui du Rust le contient.
  - Fils et pixels dissimulés (règle 16) : une vidéo saine donne les mêmes pixels quel
    que soit le nombre de fils. Sur une tranche corrompue, les images en erreur, les
    sauts et les plages ne changent pas, mais les pixels de l'image dissimulée et des
    suivantes jusqu'à l'image-clé peuvent changer. Mesures :
    - vidéo openh264 d'avant #111 : 1 et 2 fils, 3 fils, 4 fils et plus donnaient trois
      résultats, chacun reproductible ; avec l'encodage de #111 et une corruption au
      premier dixième, aucun écart ;
    - canari de l'extrait : 40 images diffèrent entre 1 fil et l'automatique, toutes
      dans les plages ; les 1 190 autres sont identiques au pixel près ;
    - 103, 122, 124 (`tests/local_threads.rs`) : 105, 1 189 et 1 472 images diffèrent,
      puis 105, 1 151 et 1 447 à la mesure suivante, toutes dans les plages abîmées
      (marge 15). Sur ces vidéos, les pixels dissimulés en fils d'image ne se
      reproduisent donc pas d'une lecture à l'autre ; les images en erreur, les sauts
      et les plages, si.

    Ces images sont hors de la règle 16, qui vaut pour ce qui sort du moteur hors des
    plages abîmées.
  - Mesures (lot 6, M1 Max, `--profile ci`) : l'extrait et la 092 donnent 1 230 et
    9 011 images, aux cadences `avg_frame_rate` de leur `video.json`, avec les `pts` et
    les images-clés de ffprobe ; la 092 se lit en 7,4 s (60 s de CPU, 177 Mo). Les 26
    vidéos de la référence donnent toutes `counts.frames` (103 : 8 998, 120 : 8 996,
    122 : 8 998), sans refus, en 229 s.
  - Dégâts contre la référence (#117, marge 15, `tests/local_damage.rs`) : l'extrait et
    la 092 n'ont aucune image abîmée. Sur les 26 vidéos, les images après un saut sont
    celles d'ffprobe, et les images en erreur le contiennent. 25 vidéos ont exactement
    ses images en erreur et ses plages (093 : 11 images, 419 abîmées ; 122 : 195 et
    6 043). Seule la 124 diffère : 310 images en erreur contre 309, 7 458 abîmées contre
    7 415 (+43).
    - L'image en plus, la 2197, porte le seul drapeau `CONCEALMENT_ACTIVE`, avec 1 fil
      comme en automatique.
    - ffprobe ne l'écrit qu'au niveau `info` (« concealing 392 DC, 392 AC, 392 MV
      errors in P frame »), que la v0.1.1 ne lit pas (`-show_log 16`). C'est la seule
      image des 26 vidéos dans ce cas.
    - Le Rust exclut donc 43 images de plus que la v0.1.1 sur la 124 (environ 1,4 s).
      Que ce soit souhaitable (image réellement dissimulée) ou trop prudent (passage
      possible perdu) reste à juger.
  - **Détection** (#118) : `detection::detect(chemin, Threads, &DetectConfig)`, ou
    `detect_input(Input, &DetectConfig)` pour une entrée déjà ouverte (le canari y passe
    sa couture). Réglages par défaut seulement, 1440×1080 seulement. Le pool `rayon`
    est celui de l'appelant (`ThreadPool::install`). `Detected` rend la lecture
    (`Reading`), les détections (`FrameDetections`, une liste par image décodée, vides
    comprises), les dégâts (`Damage`, marge = demi-fenêtre), la demi-fenêtre, les fils
    actifs du décodeur (`Input::active_threads`, pour `params.json`) et les images du
    fond.
    - `config` : `DetectConfig::default()` (seuil 12, surface 4 à 2 700, fenêtre 1,0 s,
      une image sur 3, fusion 6, largeur 960, bruit 8, sigma 1,5), champs en lecture.
      `half_window(cadence)` arrondit `fenêtre × cadence / 2` au plus proche, moitié
      vers le haut (la v0.1.1 arrondit au pair : une image d'écart quand ce produit vaut
      x,5, jamais à 30 i/s), et refuse moins de 3 images de fenêtre, moins de 3
      échantillons, ou plus de 16 (borne du réseau, levée par #98). La 092 : 15, et 11
      échantillons.
    - `geometry` : réduction 3:2 seulement (1440×1080 → 960×720), refus d'un travail de
      moins de 9 pixels de côté ou de plus de `u32::MAX` pixels. Seul `WorkScale` passe
      aux pixels d'origine.
    - `prepare` : Y → gris par la table, réduction 3:2 pondérée (4, 2, 2, 1 sur 9)
      arrondie à 8 bits, gaussienne de sigma 1 pixel de travail (noyau constant
      `[1, 18, 221, 991, 1634, …]` sur 4 096, testé contre la formule, bords en miroir
      sans répéter le pixel du bord), un seul arrondi final en 1/64 de niveau (`u16`,
      16 320 au plus ; passe verticale en `u32`, 255 × 4 096² tient). Le niveau d'une
      image est la somme de ses unités.
    - `network` : médiane par le réseau de Batcher coupé à n entrées et élagué aux
      comparaisons qui atteignent le milieu (32 pour 11 échantillons), sur des blocs de
      64 pixels ; test 0-1 exhaustif de 1 à 16 et proptest contre un tri. Si n est
      pair, la médiane est la demi-somme des deux du milieu.
    - `threshold` : résidu `f32` = image − fond − (niveau − niveau moyen), le gain en
      entiers jusqu'à une seule division. Candidats au-dessus du plancher (12
      niveaux) ; avec au moins 7 échantillons, seuil relevé à max(12, 8 × 1,4826 × MAD
      des écarts corrigés du gain), pour les seuls candidats.
    - `closing` : fermeture par le disque de rayon 4 (`round(6 / 1,5)`, demi-largeurs
      `[0, 3, 3, 4, 4, 4, 3, 3, 0]`, testées contre la formule), creuse : seuls les
      pixels touchés par la dilatation sont marqués, puis érodés, hors de l'image
      compté allumé. Un seul chemin, sans la réservation `allumés × disque` du
      prototype ; proptest contre une fermeture dense.
    - `components` : composantes 8-connexes par union-find, dans l'ordre de balayage de
      leur premier pixel (haut, puis gauche). La v0.1.1 suit la numérotation
      d'OpenCV, par blocs de 2×2 pixels : une tache qui commence une ligne plus bas
      peut sortir avant (dans sa référence, 12 280 des 29 650 images à plusieurs
      détections ont au moins une inversion d'une ligne). Le suivi en dépend en cas
      d'égalité ; le Rust ne l'imite pas (règle 5), et les 963 pistes sur 966
      ci-dessous l'incluent. Boîte et barycentre de
      la composante fermée ; surface = pixels allumés × 2,25 ; amplitude = plus grand
      |résidu| en niveaux de gris ; filtre 4 ≤ surface ≤ 2 700.
    - `window` : fenêtre glissante, tampons recyclés ; échantillons pris une image sur
      3 depuis le début de la fenêtre, coupée aux bords de la vidéo.
    - `detector` : lots de 32 images (copie du plan Y). Chaque lot est préparé, puis
      les cibles prêtes (demi-fenêtre suivante lue, ou fin) sont détectées, en groupes
      contigus zippés avec une réserve de tampons (un par fil `rayon`), rangés dans
      l'ordre. La lecture anticipée passe à 36 images pour la détection, `read` en
      garde 4.
    - `summary_frames` : les 25 images du fond de l'image résumé, aux rangs
      `(2i + 1) × N / 50` (N annoncé ; toutes si N < 25), gardées en YUV entier ; la
      plage d'affichage se décide dans #104. Un rang perdu en fin de vidéo donne moins
      d'images, comme la v0.1.1. Une image dans une plage abîmée est gardée : la
      médiane de 25 l'absorbe. Rang exact, sans le positionnement imprécis d'OpenCV.
    - Différences avec la v0.1.1, hors gris : gris puis réduction (la v0.1.1 réduit
      l'image en couleur, puis passe en gris), noyau quantifié sur 4 096 (la v0.1.1
      floute en `float32` avec le noyau exact), médiane et MAD sur des unités de 1/64
      de niveau, résidu et écarts en `f32` (`float64` en v0.1.1), niveau moyen exact,
      ordre des taches (ci-dessus).
    - Tests portés : 18 de `test_detect.py` (à l'échelle réelle, sources multiples de
      3 : tache sombre puis claire suivie à moins de 2 pixels, gain qui scintille
      d'une image à l'autre compensé, bornes de surface, vidéo plus courte que la
      fenêtre, vidéo vide, fragments fusionnés, 3 refus de fenêtre, nombreuses taches
      mesurées une à une, seuil relevé par pixel, seuil fixe aux bords, relèvement des
      seuls candidats, résidu sous le plancher, bruit d'un pixel lissé, amplitude sur
      le résidu filtré), `test_median.py` et `test_noise.py`. Vont à #98 : les 5 tests
      d'écran, sigma nul, facteur de bruit nul et configuration invalide (réglages par
      défaut seulement). Le saut de luminosité à seuil fixe de la v0.1.1 n'a pas
      d'équivalent aux réglages par défaut : un saut de 30 niveaux allume l'image au
      bord du saut, ce que la stabilité retire (tests de `compare`, #119).
    - Règle 16 : même sortie au bit près avec 1, 3 ou 4 fils et des lots de 1 à 64
      (test rapide) ; canari de l'extrait coupé (`slow`), 1 fil de décodage et 1 fil
      `rayon` contre l'automatique, mêmes plages et mêmes détections hors plages. Sur
      l'extrait, la 092 et les 26 vidéos, le parallèle redonne le séquentiel au bit près
      hors plages abîmées ; dans les plages, les pixels dissimulés varient d'une lecture
      à l'autre.
    - Mesures (outil `compare` et ses tests `slow_extract.rs`, `reference_agreement.rs`,
      `local_agreement.rs` depuis #119 ; appariement à 3 pixels, hors plages abîmées et
      instables de `video.json`) : extrait 100 % / 99,75 %, 092 100 % / 99,87 %, 26 vidéos toutes au
      moins à 98 % (la plus basse, 089 : 98,19 % / 98,46 %). Au niveau des pistes, 963
      des 966 appariées (958 concordantes), 3 perdues, 9 ajoutées.
    - Temps de la 092 (M1 Max, sous le verrou, décodeur et `rayon` en automatique) :
      11,4 s de détection en `release` (11,8 s réels, 102 s de CPU, 436 Mo dont environ
      58 pour les 25 images du fond), contre
      118,7 s pour la v0.1.1 et 11 s pour le prototype ; 16,9 s en `ci` (contrôles de
      débordement) ; 62,9 s en séquentiel (P1, `ci`). Avec la formule (d), 12,1 s : la table ne coûte rien. Une boucle de
      comparaisons écrite élément par élément se vectorisait mal une fois inlinée en
      `release` (détection 4 fois plus lente qu'en `ci`) ; écrite sur des tableaux
      entiers, elle se vectorise.
    - Mémoire (`tests/slow_memory_flash.rs`, `dhat`, 4 fils, 64 images) : un éclair plein
      cadre dans chaque groupe de cibles, au premier lot, coûte 57,8 Mo de plus qu'une
      vidéo calme, soit 20,9 octets par pixel de travail et par fil. Les deux pics
      tombent dans la même phase de lecture, d'où une mesure stable. Arithmétique :
      les pixels allumés (8 octets), touchés et fermés (4 octets chacun) d'un fil
      montent jusqu'à 2²⁰ éléments, 24,3 octets par pixel, gardés jusqu'à la fin
      (environ 170 Mo à 10 fils). Bornes du test : plus de 16, au plus 32. Les
      allocations de FFmpeg ne sont pas vues, les mêmes dans les deux vidéos.
  - **Gris de la détection** (#118, exception à la règle 5) : chaque Y passe par une
    table de 256 gris mesurée sur OpenCV (`GRAY` dans `prepare` : une rampe Y = 0 à
    255, U = V = 128, encodée sans perte en plage `tv`, lue par `cv2.VideoCapture`
    puis `COLOR_BGR2GRAY`), puis la réduction 3:2
    pondère ces gris et arrondit à 8 bits avant la gaussienne. La table suit
    `(Y − 16) × 255 / 219` de 1,2 à 2,3 niveaux en dessous, en dents de scie, à 0
    jusqu'à Y = 17 et à 255 dès Y = 237. Ablation (P1, M1 Max, un choix à la fois,
    taux d'appariement des détections référence / Rust, hors plages abîmées et
    instables de `video.json`) :

    | Variante | Extrait | 092 | 089 | Pire des 26 | Sous 98 % | Points de piste perdus |
    |---|---|---|---|---|---|---|
    | (a) table et arrondi à 8 bits | 100 / 99,75 | 100 / 99,87 | 98,19 / 98,46 | 98,19 | 0 | 19 |
    | (b) formule, écrêtée, arrondie | 98,51 / 98,75 | 96,15 / 95,30 | 70,36 / 72,92 | 70,36 | 25 | 415 |
    | (c) formule écrêtée, exacte | 97,51 / 98,25 | 95,38 / 96,00 | 75,58 / 76,76 | 75,58 | 22 | 340 |
    | (d) formule exacte, sans écrêtage | 97,76 / 98,74 | 95,13 / 95,13 | 72,12 / 70,61 | 70,61 | 23 | 358 |

    - Les écarts de (d) sont surtout du bruit au seuil : sur la 089, 489 des 508
      détections perdues sont des taches de 4,5 px² sous 20 niveaux, et autant
      d'ajoutées. Un décalage de quelques dixièmes de niveau suffit à les faire
      basculer.
    - Au niveau des pistes (détections → `exclude` avec les plages abîmées de la
      référence → `track_detections`, appariées aux 966 pistes par `match_tracks`) :
      (a) apparie 963 pistes (958 concordantes), en perd 3 et en ajoute 9 ; (d) en
      apparie 936 (912), en perd 30 et en ajoute 35. Parmi celles-ci, 13 pistes de (d)
      et 2 de (a) reviennent avec un début décalé de 6 à 26 images.
    - Pistes vraiment perdues par (d), sans jumelle à 1 s : 8 de 6 points (096 0:55,
      096 4:27, 097 2:37, 099 0:21, 099 3:24, 102 0:13, 121 2:29, 126 4:19), puis 100
      0:00 (7 points), 097 3:42 (8), 096 1:59 (9), 100 1:26 (9), 097 1:11 (10), 098
      4:27 (10), 103 4:11 (15), 100 2:00 (24) et 097 0:34 (102 points). Par (a) : 098
      0:41 (21 points).
    - Pistes vraiment ajoutées par (d) : 090 2:06, 092 2:24, 092 3:51, 094 1:14, 095
      3:20, 096 2:40, 096 3:03, 096 3:24, 097 2:43, 097 3:28, 098 1:06, 098 2:58, 098
      4:13, 099 0:19, 099 0:27, 099 2:17, 100 0:05, 100 1:36 (deux), 100 4:15, 103 3:40,
      125 0:15. Par (a) : 097 2:43, 098 0:05, 099 0:19, 099 0:27, 100 1:36 (deux), 126
      1:32 ; 097 2:43, 099 0:19 et 100 1:36 sont dans les deux.
    - La formule sera retentée après la 0.2.0 (issue en Backlog).
- **`cli`** : paquet `batdetect`, binaire `batdetect-rs`. En #95, `--version` affiche la
  version et la ligne de licence de FFmpeg. Tout texte vu par l'utilisateur vit dans
  `messages.rs`.
- **`tracking`** → `types` : exclusion des images abîmées et instables (#120), puis
  suivi (#121). `compare` enchaîne `exclude` puis `track_detections` ; `cli` le fera aussi.
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
  - Les 3 tests Python de saut de luminosité (`test_stability.py`) sont dans `compare`
    (#119) : ils enchaînent détection et stabilité.
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
- **`compare`** → `engine`, `media`, `tracking`, `reference`, `types` (#119) : outil de
  développement, binaire `compare`, non livré. `media` n'y sert qu'à choisir les fils du
  décodeur (`Threads::Automatic`).
  - `compare extract|092|local [--frames DOSSIER]` lance `engine::detection::detect` sur
    l'extrait, la 092 ou les 26 vidéos (`BATDETECT_REFERENCE_DIR`,
    `BATDETECT_VIDEOS_DIR`, 6 vidéos à la fois, résultats dans l'ordre des dossiers).
    Depuis la racine du dépôt, sous le verrou :
    `lockf …/mesure.lock cargo run --profile ci -p compare -- local`.
  - Sortie, dans l'ordre :
    - par vidéo, les détections : nombres de chaque côté, paires, taux dans les deux
      sens, détections perdues (dont celles qui sont un point d'une piste de la
      référence) et ajoutées ;
    - par vidéo, les pistes : appariées, concordantes, perdues, ajoutées, puis leur
      total ;
    - les écarts de détections groupés par vidéo et par temps (30 images entre
      deux écarts au plus), avec leur position moyenne et leur plus grande amplitude ;
    - les écarts de pistes (vidéo, temps, image de début) ;
    - le temps de détection de chaque vidéo, puis les vidéos sous 98 %. Avec `local`,
      ces temps sont pris pendant que 5 autres vidéos tournent : seul `compare 092`
      chronomètre la détection seule.
    Une vidéo en échec (vidéo absente, demi-fenêtre autre que celle de la référence…)
    est écrite sur la sortie d'erreur avec ses causes, sans effacer le rapport des
    autres. Code de sortie 1 si une vidéo échoue, si une vidéo est sous 98 % dans un
    sens, ou si aucune vidéo n'est comparée.
  - Seuils : 98 % dans les deux sens (`agreement::MIN_RATE`), appariement à moins de
    3 px (`reference::detection_matching`), plages abîmées et instables de la référence
    ignorées des deux côtés. Pistes : `exclude` avec les plages abîmées, le nombre
    d'images signalées et la cadence de la référence, puis `track_detections` aux
    réglages par défaut, puis `match_tracks`. Seules les détections viennent du Rust.
  - Mesure sur les 26 vidéos (#119, profil `ci`, M1 Max, sous le verrou) : mêmes chiffres
    qu'à #118, 410,8 s en tout et 3,2 Go de mémoire résidente. La lecture de la
    référence et l'appariement seuls prennent 0,9 s pour les 26 vidéos (test local de
    `reference`) : les allocations de `detection_matching` à chaque image ne comptent
    pas, et la fonction reste telle quelle.
  - `--frames` écrit `DOSSIER/<vidéo>/frames.jsonl` au format de la référence
    (`frames::frames_of`, puis `reference::frames::Frames::write`) : `pts`, image-clé et
    erreur de décodage vus par le décodeur de `media` (par ffprobe dans la référence),
    détections brutes avant exclusion. Sur l'extrait, `pts`, images-clés et erreurs sont
    égaux à ceux de la référence sur les 1 230 images.
  - **Identité entre systèmes.** Aucune opération flottante de la chaîne de détection
    (`engine`, et `WorkScale` dans `types`) ne dépend de la libm.
    - Ce qu'on y trouve : les quatre opérations, `abs`, `midpoint`, `min`, `max`,
      `round`, les conversions et le tri par `total_cmp`. Toutes sont exactes ou fixées
      bit à bit par IEEE 754. La seule exception, `max` entre +0 et −0, n'arrive pas :
      on compare toujours au plancher du seuil ou à un pic qui le dépasse.
    - `powi(2)` de `area_to_source` se calcule en une multiplication, et 1,5² = 2,25 est
      exact. Rust ne contracte jamais a × b + c en FMA.
    - Le décodage H.264 est exact au bit près par la norme, sauf dans les images
      dissimulées ; l'extrait n'en a aucune.
    - Hors de la chaîne, `hypot` (distance de l'appariement, association du suivi) et
      `atan2` (virage médian) viennent de la libm. Un écart d'un ulp ne ferait basculer
      qu'une paire à 3 px tout juste, ou un choix entre deux distances égales. Il ne
      touche pas `frames.jsonl`.
  - **Sauts de luminosité** (`tests/slow_brightness_jump.rs`, les 3 tests de
    `test_stability.py`) :
    - Le montage : une vidéo de 90 images 1440×1080 à 30 i/s (gris 200, bruit σ 3, saut
      à l'image 45), encodée par l'openh264 de `media`, passe par `detect`, `exclude`
      puis `track_detections`. Une vidéo plutôt qu'une entrée directe du détecteur, pour
      n'ouvrir aucune API nouvelle dans `engine`.
    - Niveau `slow` : 3,3 s en profil `ci`, mais 368,8 s sous l'instrumentation de la
      couverture, qui ralentit la détection.
    - Les résultats : un saut de 20, 30 ou 40 niveaux ne donne aucune piste. Un saut de
      30 donne une plage instable, [9, 80], autour du saut. Le test vérifie qu'elle
      laisse des images de chaque côté : la borne de 90 images du Python valait la
      longueur de la vidéo et ne pouvait pas échouer.
    - Un saut de 20 n'allume aucun pixel. La v0.1.1, à l'échelle 3 et sans filtre, y
      voyait quelques taches.
    - La bande où un saut donne des taches sans plage instable n'a ici qu'un niveau de
      large : 28 (22 détections). 29 en donne 1 617, avec la plage [9, 80]. Le test du
      saut modéré garde donc 20, et vérifie que rien n'est allumé ni ignoré.
    - Le saut de 30 n'est qu'à un niveau du bord. Si openh264 n'encode pas la même
      chose sur un autre système, ce test le montrera ; l'écart remonte alors à
      l'utilisateur.

## Référence v0.1.1

Format, emplacement et versions de la référence que le Rust doit égaler : README de
`tests/fixtures/reference/` (#94). La crate `reference` la lit ; le test local
`crates/reference/tests/local_reference.rs` relit les 26 vidéos (966 pistes) et les
apparie chacune à elle-même.
