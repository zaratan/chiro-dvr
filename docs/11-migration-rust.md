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
- **Reconstruction** : cargo ne suit pas les `.a`, qu'il copie dans la rlib de
  ffmpeg-sys-next. Quand `.ffmpeg` change de cible, le script lance
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

13. L'index d'image (ordre de décodage) et le `pts` ont chacun leur type (`FrameIndex`,
    `Pts`). Pas de sentinelle : `Option<Pts>`.
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
- Déclarées d'avance pour le lot : `rayon`, `serde`, `serde_json`, `tiny-skia`, `dhat`.
  Tant qu'elles ne servent pas, elles ne sont pas dans `Cargo.lock` : leur version n'y
  est pas figée.
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
  (PR #130) : `rust` 3 min 04 sur macOS et 3 min 46 sous Linux ;
  `rust-reference` 1 min 15.

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

## Crates

Neuf crates au lot 6, `members = ["crates/*"]`. `unsafe` n'est permis que dans `media`.

- **`types`** : vocabulaire commun. En #95, `Rational` seulement : réduit à la
  construction, dénominateur positif, sans débordement (`OutOfRange` plutôt qu'une
  panique). V ajoute le reste (`FrameIndex`, `Pts`, `Span`…).
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
    - Le fichier s'écrit sous un nom temporaire ; seul `finish` lui donne son nom.
    - Une image en plage pleine est refusée (`FullRange`) : l'encodeur écrit en plage
      limitée, et la plage n'est pas encore un réglage.
    - Un paquet sans durée reçoit une durée d'une image.
    - openh264 n'écrit pas la plage d'une vidéo en plage limitée : relue, elle est
      `Unspecified`, ce qui veut dire limitée en H.264. #111 décidera s'il faut l'écrire.
  - `library` : version et licence du FFmpeg lié.
- **`reference`** : chemins des fixtures et garde LFS. V y ajoute le format de la
  référence v0.1.1 et l'appariement.
- **`cli`** : paquet `batdetect`, binaire `batdetect-rs`. En #95, `--version` affiche la
  version et la ligne de licence de FFmpeg. Tout texte vu par l'utilisateur vit dans
  `messages.rs`.
- **À venir** :
  - `engine` (#116, lecture, images abîmées, détection) ;
  - `tracking` (#120, plages, stabilité, suivi) ;
  - `files` (#122, noms, CSV, `params.json`) ;
  - `draw` (#101, dessin) ;
  - `compare` (#119, écarts à la référence).

## Référence v0.1.1

Format, emplacement et versions de la référence que le Rust doit égaler : README de
`tests/fixtures/reference/` (#94).
