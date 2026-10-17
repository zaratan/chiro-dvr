# La réécriture Rust : chaîne, charte et histoire

Le lot 6 a commencé la réécriture de `batdetect` en Rust. Le but : détecter au moins aussi
bien que la v0.1.1, pas reproduire le Python bit à bit ; les détections et les pistes se
comparent par appariement avec tolérances, et chaque écart se juge à l'œil. Depuis #199, le
Rust est seul (dernier état du Python au tag `python-final`).

Ce document tient trois choses ([qui tient quoi](README.md#qui-tient-quoi)) :

- **les sections tenues à jour**, de [Construction](#construction) à
  [Plage réelle des sources](#plage-réelle-des-sources), puis la méthode des
  [Réglages](#réglages) et la [Garde](#garde) du banc ;
- **la description des crates** ([Crates](#crates)), qui détaille la carte à jour de
  [CLAUDE.md](../CLAUDE.md) et suit les lots ;
- **l'histoire de la migration**, mesures datées comprises, qui ne se récrivent pas.

## Bilan (16 octobre 2026)

| Mesure | Rust | v0.1.1 | Quand | Détail |
| --- | --- | --- | --- | --- |
| Pistes des 26 vidéos de la référence (`compare local`) | 963 sur 966 appariées (958 concordantes), 3 perdues, 9 ajoutées | — | 15 octobre, f364698 | [Mesure de la bascule](#mesure-de-la-bascule-198-15-octobre-2026-m1-max-sous-le-verrou) |
| Pistes de la 092 (`rust:reference`) | 14, toutes concordantes | 14 | 15 octobre, f364698 | idem |
| Garde du banc, transit / chasse | 148 / 455 cibles trouvées | 148 / 456 | 15 octobre, f364698 | idem |
| La 092 par la commande, temps réel | 19,81 s | 135,4 s | 16 octobre, b38748a | [09](09-profilage.md) |
| Validation à l'aveugle | 6 cas sur 6 « aussi bien » | — | 15 octobre | [Validation à l'aveugle](#validation-à-laveugle-198-15-octobre-2026) |

- **Écarts voulus** : temps des pistes lus aux `pts` (0,01 à 0,02 s plus tôt sur la 092) ;
  contraste effectif des cibles du banc arrondi à l'entier ; graine du banc reproductible
  sur un même système seulement ; plus de cache du banc ni de `--workers` ; quatre réglages
  seulement en options de la commande.
- **Questions encore ouvertes** : dans [06](06-questions-ouvertes.md).
- **Jalons** : lot 6 (#95 à #123 : workspace, moteur, suivi, CSV) ; #98 (#149 à #151 : moteur
  générique, 16 cas de réglages) ; sorties (#104 image résumé, #127 frise, #157 parties,
  #110 et #112 extraits, #103 zooms) ; banc #100 (#107 à #109) ; commande (#55, #56, #63 à
  #65, #86, #91, #113 à #115) ; garde #198 ; retrait du Python et binaires renommés #199 ;
  docs #200 ; profil #201 ; `check` rapide #205.

## État (octobre 2026)

Le lot 6 est fini : workspace, FFmpeg et outillage (#95, #146), moteur (#116 : lecture
et refus à l'entrée ; #117 : images abîmées et canari ; #118 : détection ; #119 :
comparaison à la référence et identité entre systèmes), suivi et commande (#120 à #123 :
`batdetect-rs` détecte, suit et écrit le CSV des pistes et `params.json`). #144 a rendu
infaillibles les constructeurs de `types` qui ne font que filtrer ou renuméroter.

#98 rend le moteur générique en trois temps : #149 (banc d'essai à 16 réglages,
[Réglages](#réglages)), #150 (géométrie et filtres : toute taille, toute réduction,
noyau et disque calculés, plage pleine lue) et #151 (fenêtre et masques : médiane sans
borne, zones OSD, réglages unifiés). Depuis #151, aucun réglage valide n'est refusé, et
les 16 cas du banc sont comparés. L'image résumé
est arrivée avec #104, sa frise avec #127 ; les extraits avec #110 et #112, les zooms avec
#103.

#100 porte le banc de mesure en Rust (#107 à #109, crate `bench`, binaire
`batdetect-bench-rs`, renommé `batdetect-bench` à #199) : fausses chauves-souris injectées
par une couture du moteur, collecte, appariement, mesures, `bench.json` aux clés de la
v0.1.1, et une garde contre les deux bancs enregistrés par la v0.1.1
([Banc de mesure](#banc-de-mesure)). Le cache du Python n'est pas porté : `--targets` et
`--replay` le remplacent. [08](08-banc-de-mesure.md) décrit la commande.

#198 passe la garde de la bascule : parité remesurée sur la commande telle qu'elle est,
garde du banc rejouée, six cas d'extraits et de zooms jugés à l'aveugle et deux images
résumé jugées à découvert, puis figés ([Garde](#garde)). #199 retire le Python.

## Construction

```bash
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:ffmpeg   # construit FFmpeg une fois par poste
mise run rust:static      # fmt, clippy, règles de la charte, cargo deny, cargo shear
mise run rust:coverage    # tests rapides sous cargo-llvm-cov, couverture ≥ 95 %
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:slow       # l'extrait LFS
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:settings   # les 16 cas de réglages du banc d'essai
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:reference  # la 092 entière
```

`verrou lourd` est le verrou de mesure des agents de ce poste (`MESURES.md`, hors dépôt) ;
ailleurs, ces commandes se lancent sans lui.

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
  `--enable-pic` est explicite (#220) : sans lui, le `configure` n'active le PIC que si
  le compilateur en produit déjà par défaut, comme le clang de macOS et le gcc d'Ubuntu
  (PIE par défaut), pas le gcc de Rocky 10. Or Rust lie en PIE sous Linux, et l'édition de
  liens échoue sur des `.a` non PIC. Son ajout change l'empreinte : FFmpeg se reconstruit
  une fois par poste et par runner Linux ou macOS.
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
  - Ce que le binaire lie en dynamique (mesuré par `otool -L` le 16 octobre 2026) :
    - sur macOS : libc++, libSystem, libiconv (tirée par la bibliothèque standard de
      Rust), libobjc et six frameworks (Foundation, CoreFoundation, VideoToolbox,
      CoreMedia, CoreVideo, CoreServices). libobjc et Foundation viennent de ctrlc, par
      objc2 ;
    - sous Linux : libstdc++, libgcc_s, libc et libm.
  - `packaging/check.sh` contrôle cette liste sur le binaire livré (voir
    [Empaquetage](#empaquetage)).
- **Version liée vérifiable** : FFmpeg est construit avec `--extra-version=<empreinte>`.
  `batdetect --version` affiche donc `FFmpeg 7.1.1-<empreinte>`, et un test compare
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

### Empaquetage

```bash
mise run package          # binaire release, puis dist/batdetect-<cible>.tar.gz (zip sous Windows)
mise run package:check    # contrôles du livrable sur cette archive
```

- **Archives** (#223) : `batdetect-darwin-arm64.tar.gz`, `batdetect-linux-x86_64.tar.gz`
  et `batdetect-windows-x86_64.zip`.
  - Contenu, à la racine : le binaire, `LICENSE`, `NOTICE` et `licenses/`.
  - Sans numéro de version dans le nom, comme à la v0.1.1 : la formule Homebrew en
    dépend.
  - `packaging/package.sh` lit la cible dans la ligne `host:` de `rustc -vV`, pas dans
    `uname`, qui vaut `UCRT64_NT-…` sous MSYS2. Les scripts sont en bash : `mise` ne
    tourne pas sous Windows.
  - Sans `BATDETECT_VERSION`, il la pose à la version de `[workspace.package]` de
    `Cargo.toml`, et `--version` l'imprime. Ainsi `mise run package:check` contrôle en
    local la même version qu'en CI.
- **Contrôles** : `packaging/check.sh ARCHIVE [VERSION]` joue tous les contrôles, puis
  échoue à la fin si un seul a échoué.

  | Contrôle | Ce qu'il mesure |
  |---|---|
  | C1 | l'archive contient le binaire, `LICENSE`, `NOTICE` et `licenses/`, rien d'autre |
  | C2 | aucun lien dynamique hors d'une liste mesurée par système : `otool -L`, `readelf -d` (lignes NEEDED), `dumpbin -dependents` |
  | C3 | `minos` ≤ 14.0 sous macOS ; sous Linux, `GLIBC_` ≤ 2.35, `GLIBCXX_` ≤ 3.4.30, `CXXABI_` ≤ 1.3.13 et `GCC_` ≤ 12.0.0, la bibliothèque de l'Ubuntu 22.04 ; sous Windows, versions du système et du sous-système de l'en-tête PE ≤ 10.0 (`dumpbin -headers`), celle de Windows 10, que Windows 11 annonce aussi |
  | C4 | `--version` donne `batdetect <VERSION>`, puis le FFmpeg épinglé, avec l'empreinte du script de construction et la LGPL |
  | C5 | le bloc `Rust crates:` de `NOTICE` égale `cargo tree -p batdetect -e normal,no-proc-macro` sur les trois cibles de `packaging/about.toml` ([Licences](#licences)) |
  | C6 | le CSV de l'extrait, en anglais, est identique à l'octet à `tests/fixtures/rust/video_092_original_3m24-4m05/tracks.csv` |
- **Preuves négatives** (16 octobre 2026, Mac) : chaque contrôle échoue seul sur une
  archive altérée.
  - Fichier en trop et `NOTICE` retiré : C1.
  - Binaire remplacé par `/usr/bin/curl` : C2, C3, C4 et C6.
  - Construction sans `BATDETECT_VERSION` : C4.
  - Un octet changé dans le CSV : C6.
  - Sous-système 11.0 posé par `editbin` sur une copie de l'archive Windows (16 octobre 2026, CI de la PR #261) : `FAIL C3-floor: subsystem 11.00, os 6.00, above 10.00`, et C4 et C6, le chargeur refusant le binaire (`Exec format error`) ; l'archive intacte donne `ok C3-floor (subsystem 6.00, os 6.00)`.
- **Mesure** (16 octobre 2026, Mac, `release`) : l'archive macOS pèse 3 511 951 octets,
  dont un binaire de 8 130 992 octets. Le CSV des profils `ci` et `release` est
  identique à l'octet.

### Windows

Cible `x86_64-pc-windows-msvc`, construite seulement par la CI (tâches `rust-windows` et
`rust-windows-slow`) :
`mise` n'y tourne pas. Le binaire Windows se construit et s'empaquette dans `package.yml`
([Empaquetage](#empaquetage)) ; la construction sur un poste Windows est l'objet de #226.

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
  `.gitattributes` force `eol=lf` pour les scripts `.sh`, `ffmpeg-versions`,
  `tests/fixtures/reference/**`, et pour `LICENSE`, `packaging/NOTICE` et
  `packaging/licenses/**`. Les textes du zip sont ainsi les mêmes que ceux des archives
  macOS et Linux.
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

Le livrable est sous **MIT** pour le code de batdetect (`LICENSE`) et sous **LGPL 2.1 ou
ultérieure** pour FFmpeg. Il porte aussi openh264 (BSD-2), Roboto (OFL-1.1), la
bibliothèque standard de Rust et 83 crates tierces, chacun sous sa licence. Décision du
16 octobre 2026, #224.

`packaging/NOTICE` décrit le binaire composant par composant. Les textes de licence sont
dans `packaging/licenses/`, et `package.sh` copie l'un et l'autre dans chaque archive
([Empaquetage](#empaquetage)). Chaque texte vient de la source épinglée du composant :

| Fichier | Source |
|---|---|
| `FFmpeg-LGPL-2.1.txt` | `COPYING.LGPLv2.1` de `ffmpeg-7.1.1.tar.xz` |
| `openh264-BSD-2-Clause.txt` | `LICENSE` de l'archive openh264 2.6.0 |
| `Roboto-OFL-1.1.txt` | copie de `crates/draw/fonts/OFL.txt` |
| `Rust-MIT.txt`, `Rust-Apache-2.0.txt` | `LICENSE-MIT` et `LICENSE-APACHE` de `rust-lang/rust` au tag 1.99.0 |
| `crates.txt` | généré par `mise run notice` |

- **FFmpeg 7.1.1 en LGPL 2.1 ou ultérieure.**
  - Il est construit sans `--enable-gpl` ni `--enable-nonfree`, et `libavcodec` annonce
    « LGPL version 2.1 or later ». `--version` l'affiche, et `check.sh` le contrôle (C4).
  - Lié en statique, il impose de fournir de quoi relier. Chaque release joint donc
    `ffmpeg-7.1.1.tar.xz` (#223), et les sources de batdetect du tag sont publiques.
  - `NOTICE` dit comment relier : changer `FFMPEG_URL` et `FFMPEG_SHA256` dans
    `ffmpeg-versions`, lancer le script de construction, puis `cargo build`.
- **openh264 2.6.0 en BSD-2.**
  - Il est construit depuis les sources, et sa source est jointe à la release.
  - Il n'a donc **pas** la licence de brevets H.264 que Cisco n'accorde qu'à ses propres
    binaires.
  - Brevets acceptés le 16 octobre 2026. `NOTICE` le dit : les codecs H.264 et HEVC
    peuvent être couverts par des brevets, et aucune licence de brevet n'est fournie
    avec ce binaire.
- **Roboto en SIL Open Font License 1.1** (#101).
  - Embarquée dans `draw` : `crates/draw/fonts/Roboto.ttf`, police variable `wdth` et
    `wght`.
  - Licence à côté : `crates/draw/fonts/OFL.txt`, recopiée dans `packaging/licenses/`.
  - Provenance : dépôt `google/fonts`, fichier `ofl/roboto/Roboto[wdth,wght].ttf`, commit
    `6183fc0` du 27 janvier 2026. sha256 `d7598e12…872a134`.
  - Pas de nom de police réservé.
- **Bibliothèque standard de Rust** : MIT OU Apache-2.0. L'avis complet de la std (ses
  dépendances comprises) est `share/doc/rust/COPYRIGHT-library.html` de la toolchain :
  1,5 Mo de HTML. Il n'est pas joint à l'archive.
- **Crates livrées** : l'union de
  `cargo tree -p batdetect -e normal,no-proc-macro --target <triple>` sur les trois
  triples de `packaging/about.toml`, crates du workspace exclues. Mesurée le 16 octobre
  2026 : 83 crates, dont 78 sous macOS, 74 sous Linux et 75 sous Windows.
  - **Génération par `mise run notice`.**
    - Elle croise cette liste avec le JSON de `cargo-about`, épinglé dans `mise.toml`.
    - Elle réécrit le bloc `Rust crates:` de `NOTICE`, une ligne `- nom version,
      licence` par crate, et `licenses/crates.txt` : 41 textes distincts, chacun suivi
      des crates qui l'emploient.
    - **Avis de copyright.** Chaque texte garde celui que la crate publie. Plusieurs
      textes n'en ont aucun, comme leur amont :
      - un texte MIT commun à 17 crates (adler2 à zmij, dont rustc-hash et tinyvec) ;
      - le MIT de fdeflate ;
      - le WTFPL de ffmpeg-next.
      - Cinq crates ont un texte que `cargo-about` ne trouve pas seul. `about.toml` le
        lui donne par `clarify` :
        - dans la crate, pour type-map, windows-sys et windows-link ;
        - dans le dépôt amont, au commit publié, pour fluent-langneg et
          intl_pluralrules.
      - objc2, objc2-encode, block2 et dispatch2 ne publient aucun texte, seulement un
        `LICENSE.md` qui renvoie à MIT. Leur ligne de copyright devient « No copyright
        notice is published with these crates », suivie des auteurs de leur
        `Cargo.toml`.
      - La tâche échoue sur tout autre modèle de texte (`<year>`, `<copyright holders>`).
    - `cargo-about` ne sait pas écarter les proc-macros (13 crates de plus avec leurs
      dépendances) : la liste de `cargo tree` fait foi.
  - **Contrôle C5** de `check.sh` : le bloc de `NOTICE` de l'archive égale la liste de
    `cargo tree`, sans crate absente ni en trop. Une dépendance changée sans
    `mise run notice` fait donc échouer `package.yml`.
    - `cargo tree` lit les manifestes des trois cibles, que la machine n'a pas toujours
      téléchargés : C5 a besoin du réseau.
    - Preuve négative (16 octobre 2026) : une crate retirée du `NOTICE`, puis une crate
      en trop, font échouer C5 seul.
  - **Licence retenue** : l'ordre de `accepted` (`about.toml`) la choisit dans une
    expression « OU ». MIT l'emporte : 74 crates.
  - Les autres :
    - Unicode-3.0 : `tinystr`, `yoke`, `zerofrom` ;
    - BSD-3-Clause : `tiny-skia`, `tiny-skia-path` ;
    - BSD-2-Clause : `arrayref` ;
    - WTFPL : `ffmpeg-next`, `ffmpeg-sys-next` ;
    - Apache-2.0 : `self_cell`, sous `Apache-2.0 OR GPL-2.0-only`.

    Aucune crate livrée n'a de fichier NOTICE à reprendre (Apache-2.0 §4d).
- **Licences admises** : `deny.toml` n'admet que MIT, Apache-2.0 (avec ou sans
  LLVM-exception), BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, WTFPL et Zlib.
  - La liste a été mesurée avec toutes les dépendances déclarées, différées comprises.
  - D'autres licences n'apparaissent que comme alternatives dans une expression « OU » :
    0BSD (`adler2`), Unlicense (`memchr`) et LGPL-2.1-or-later (`r-efi`), à côté de MIT ;
    GPL-2.0-only (`self_cell`), à côté d'Apache-2.0 seulement.
  - `r-efi` ne vise qu'UEFI et n'est livré sur aucune des trois cibles.
- **`jiff` et ses crates `jiff-*`** (#66, #83 : heure locale des noms datés et de `params.json`) :
  Unlicense OU MIT, MIT retenu par `about.toml` ; sous Windows, `jiff-tzdb` embarque la base des
  fuseaux horaires de l'IANA, du domaine public.

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
  - au plus un appel à `catch_unwind` hors des tests (`*/tests.rs`, `*/testing.rs`,
    `*/tests/*`, lignes `use` exclues, toutes formes comprises), et seulement sous
    `crates/cli/src/` (#113) ; pas de `panic = "abort"` dans les profils ;
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
    `tests/slow_*.rs`, réglages dans `tests/settings_*.rs`, tests de la 092 dans
    `tests/reference_*.rs`. proptest là où un
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
- **PNG de l'image résumé** (#104, accord de la tour) : `png` 0.18.1 (MIT OR
  Apache-2.0), en direct dans `summary`, RGB 8 bits ; il tire `fdeflate` 0.3.7 (même
  licence). `draw` reste sans format de fichier, et la feature `png` de tiny-skia tirerait
  les mêmes crates. `png` dépend de `miniz_oxide` 0.8 quand `flate2` prend la 0.9 : le
  doublon existait par `dhat`, il entre avec `png` dans le binaire livré (avertissement
  de `cargo deny`).
- `clap` en builder, features `string` et `wrap_help` : la commande se construit à
  l'exécution avec les textes de la langue (#114), ce que `derive` ne fait pas.
- **Langues (#114, décision 7 du lot 8)** : `fluent-bundle` 0.16.0, `unic-langid` 0.9.6 et
  `sys-locale` 0.3.2 (MIT OR Apache-2.0). Ils ajoutent 13 crates à `Cargo.lock`
  (`fluent-syntax`, `fluent-langneg`, `intl-memoizer`, `intl_pluralrules`, `self_cell`,
  `type-map`, `tinystr`, `zerovec`, `displaydoc`, `unic-langid-impl`), sous MIT, Apache-2.0
  ou Unicode-3.0, déjà admises. Écartés : `i18n-embed-fl` (macro et chargement de fichiers
  dont la commande n'a pas besoin) et ICU4X (des dizaines de crates pour formater deux
  sortes de nombres). La macro `langid!` n'est pas prise : la langue se lit par
  `"fr".parse()`, et le test des pluriels prouve qu'elle est reconnue.
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
| rapide | `src/<module>/tests.rs`, `tests/*.rs` | `mise run rust:test` (sans couverture), `mise run rust:coverage` (couverture) | vidéos fabriquées par openh264, binaire lancé pour de vrai |
| slow | `tests/slow_*.rs`, `#[ignore = "slow"]` | `mise run rust:slow` | l'extrait LFS : 1 230 images, 10000/333, 1440×1080, plage `tv` ; les tests de mémoire `slow_memory_flash`, `slow_memory_flash_wide` et `slow_memory_flash_long` (`dhat`, 960 et 1440 pixels de travail, 61 échantillons), hors de la couverture ; sous macOS, une image exFAT montée par `hdiutil` (`files/tests/slow_exfat.rs`, #236) |
| settings | `tests/settings_*.rs`, `#[ignore = "settings"]` | `mise run rust:settings` | les 16 cas du banc d'essai sur l'extrait et sa 720×540 (`settings_agreement`), sous Linux seulement en CI (#167) |
| reference | `tests/reference_*.rs`, `#[ignore = "reference"]` | `mise run rust:reference` | la 092 : 9 011 images, cadence moyenne 90110000/3000927, aux défauts et à 1440 pixels de travail (#151), en un seul binaire depuis #184 (`cli/tests/reference_092.rs`) |
| matériel | `tests/hardware_*.rs`, `#[ignore = "hardware"]` | étape d'essai de la CI macOS | VideoToolbox sans repli logiciel |
| local | `tests/local_*.rs`, `#[ignore = "local"]` | `BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=… cargo test --workspace --locked --profile ci --test 'local_*' -- --include-ignored` | les 26 vidéos de la référence, hors dépôt, jamais en CI ; récupérées du seau sur une autre machine par `mise run rust:data` ([emplacements](../tests/fixtures/reference/README.md#récupération-sur-une-autre-machine)) |

- Le niveau d'un test se choisit par le nom de son fichier. Ajouter un test ne demande
  de toucher ni la CI ni `mise.toml`.
- Depuis #205, `mise run check` = `rust:static` + `rust:test`, joué avant de rendre ;
  `mise run check:ci` = `rust:coverage` (couverture, l'ancien `rust:test`) + `rust:slow` + `rust:settings`, que la CI
  joue par ses tâches inchangées. Durées mesurées le 16 octobre 2026 sur bb3a668 (M1 Max,
  au calme, `verrou chrono`) : `rust:static` 3,1 s ; `rust:test` 52,1 s, dont 22 s de
  compilation des 12 crates du workspace ; `rust:coverage` 519 s, dont 28 s de compilation
  instrumentée des mêmes crates et 253 s
  pour les tests unitaires de `batdetect` sous instrumentation. Le nouveau `check`, sans
  recompilation : 19,5 s réels, 57 s de CPU ; le hook pre-push de `lefthook.yml`, qui le
  lance, 19,8 s. L'ancien `check` (statique,
  couverture, `slow`, `settings`) prenait 12 à 21 min selon la charge dans la nuit du 15
  au 16 octobre (727 s au calme pour #202).
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
- Le test du détecteur à 61 et 81 échantillons de fond (#151) occupait presque seul le
  binaire unitaire d'`engine` sous la couverture : 209 s sous Linux et 116 s sous macOS
  pour ce binaire (CI de la PR #166), contre 0,7 s pour le test sans couverture. Mesuré
  sur le Mac (`sample`, désassemblage) : sous l'instrumentation, chaque voie du tri de `Network::order` incrémente un compteur
  atomique partagé par tous les fils : le tri n'est plus vectorisé et les fils se
  disputent le compteur. Depuis #167, il tourne sur une image de 144×24 au lieu de
  360×180 (24 blocs de 64 pixels au lieu de 450), avec les mêmes demi-fenêtres, fils et
  lots : 7 s sous la couverture sur le M1 Max, contre 146 s.
- Le niveau `reference` passe la 092 deux fois, en parallèle, depuis #184 : une fois
  aux défauts (détection par `engine::detection::detect`, puis
  `batdetect::process::process_detected` : suivi, extraits, zoom) et une fois à 1440
  pixels de travail (`compare`).
  - L'initialisation du binaire `cli/tests/reference_092.rs` les lance sur deux fils
    et range leurs résultats dans un `OnceLock`. Les cinq tests les relisent : la
    commande, la concordance aux défauts, la lecture (horodatages, images-clés, aucune
    plage abîmée, d'après ffprobe), le décodage (cadence, taille, plage, aucun paquet
    illisible) et la concordance à 1440.
  - Le parallèle se fait dans l'initialisation, pas par libtest : libtest lance les
    tests dans l'ordre de leur nom, 4 à la fois sur le runner, et celui de la 1440
    vient en dernier.
  - Une erreur est rangée avec ses causes, une panique avec son message : un `OnceLock`
    dont l'initialisation panique la relancerait pour chaque test en attente. La
    détection aux défauts, la commande qui la suit et la 1440 ont chacune leur
    résultat : un rendu qui échoue ne fait tomber que le test de la commande.
  - Un test lancé seul par filtre fait les deux passages (66 s sur le M1 Max, contre 8 s
    pour l'ancien test du décodage).
  - La concordance aux défauts vérifie d'abord que les réglages de l'enregistrement sont
    ceux de `DetectConfig::default()` : c'est la même détection que celle de la commande.
  - La 1440 ne touche pas `cli` : elle est dans ce binaire pour partager le processus,
    d'où la dépendance de test `cli` → `compare`.
  - Le décodage par le `Decoder` seul de `media` et la lecture du moteur ne se recoupent
    plus sur la 092, seulement sur l'extrait (`slow_decode`, `slow_input`).
  - Avant #184, quatre binaires passaient la 092 cinq fois : M1 Max, sous le verrou, 116 s
    de tests (32,7 + 65,8 + 9,3 + 8,1) et 759 s de CPU ; CI 413 s (119 + 243 + 26 + 25).
    Après : 66,0 s et 429 s de CPU, 70,6 s avec 4 fils de test ; pic de 1,65 Go.
- Les fixtures se trouvent par `reference::fixture::fixture`, qui refuse un pointeur
  LFS non tiré avec le conseil `git lfs pull`.
- Les tests `local_` de `bench` (#107 à #109) lisent les bancs de la v0.1.1
  (`BATDETECT_REFERENCE_DIR/bench/`) : `local_injection` et `local_metrics` sans vidéo, en
  moins d'une seconde ; `local_guard` rejoue les deux bancs sur la 092 du dépôt (LFS),
  1 min 54 en profil `ci`.
- `tests/fixtures/rust/video_092_original_3m24-4m05/frames.jsonl` : les détections de
  l'extrait par le Rust, écrites sur le Mac depuis la racine du dépôt par
  `verrou lourd cargo run --profile ci -p compare -- extract --frames tests/fixtures/rust`
  (fichier git ordinaire, fin de ligne `\n` forcée par `.gitattributes`). Le test
  `slow_extract` de `compare` le redemande au bit près sur macOS, Linux et Windows ; un
  écart remonte à l'utilisateur (voir `compare`).
- `tests/fixtures/rust/video_092_original_3m24-4m05/tracks.csv` : le tableau des pistes
  de l'extrait, contrôlé à l'octet par `check.sh` (C6) sur les trois systèmes.
  - Il se refait depuis la racine du dépôt, en anglais, par le binaire `release` :
    `verrou lourd env -u LANGUAGE -u LC_ALL -u LC_MESSAGES LANG=C target/release/batdetect tests/fixtures/video_092_original_3m24-4m05.mp4 -o out/csv`.
    On copie ensuite `out/csv/video_092_original_3m24-4m05/video_092_original_3m24-4m05_tracks.csv`
    à sa place.
  - Ses lignes finissent en CRLF, comme tout tableau produit. `.gitattributes` le marque
    `-text` : git garde donc ces octets tels quels.
  - Il est au format du tableau de #84 : en-têtes anglais, colonne `verdict` vide, début
    et fin en `h:mm:ss`. Refait par #224 depuis le format d'avant #84, avec les mêmes
    valeurs dans les 10 colonnes communes.
  - Ses valeurs diffèrent de la référence v0.1.1 sur trois cellules : le chemin de la
    piste 1 (912.3 au lieu de 912.7) et l'amplitude des pistes 1 et 3 (110.2 au lieu de
    110.3, 99.0 au lieu de 99.1).
- Cinq petites fixtures, hors LFS, faites une fois par le ffmpeg 9.0.2 de Homebrew
  (libx264) et commitées :
  - `full_range.mp4` (1,9 Ko, `yuvj420p`, plage `pc`, lue jusqu'au bout depuis #150) :
    `ffmpeg -f lavfi -i color=gray:size=1440x1080:rate=30 -frames:v 2 -c:v libx264 -bf 0 -pix_fmt yuvj420p -color_range pc full_range.mp4`
  - `full_range_midway.mp4` (3,0 Ko, 2 images en plage non précisée puis 2 en `pc`,
    lue jusqu'au bout depuis #150) : `tv.h264` et `pc.h264` par la commande précédente
    avec `-x264-params repeat-headers=1 -f h264`, l'un en `-pix_fmt yuv420p -color_range tv`,
    l'autre en `-pix_fmt yuvj420p -color_range pc`, puis
    `cat tv.h264 pc.h264 > midway.h264 && ffmpeg -framerate 30 -i midway.h264 -c copy full_range_midway.mp4`
  - `two_gops.mp4` (5,9 Ko, 320×240, 20 images, images-clés en 0 et 10, perte au début
    par la couture) :
    `ffmpeg -f lavfi -i testsrc=size=320x240:rate=30 -frames:v 20 -c:v libx264 -bf 0 -g 10 -keyint_min 10 -sc_threshold 0 -crf 40 -pix_fmt yuv420p -color_range tv two_gops.mp4`
  - `spot.mp4` (3,1 Ko, 320×240, 40 images, plage `tv`, un carré clair qui avance de 5
    pixels par image sur un fond gris, une bande en haut à Y = 0 et une en bas à Y = 254,
    hors de la plage `tv`) :
    `ffmpeg -f lavfi -i "color=c=0x606060:s=320x240:r=30" -f lavfi -i "color=c=white:s=6x6:r=30" -filter_complex "[0]drawbox=x=0:y=0:w=320:h=24:color=black:t=fill,drawbox=x=0:y=216:w=320:h=24:color=white:t=fill[bg];[bg][1]overlay=x='40+5*n':y=110:eval=frame,format=yuv420p,lutyuv=y='if(lt(val\,20)\,4\,if(gt(val\,230)\,250\,val))'" -frames:v 40 -c:v libx264 -preset medium -crf 18 -g 20 -bf 0 -pix_fmt yuv420p -color_range tv -colorspace bt709 -color_primaries bt709 -color_trc bt709 spot.mp4`
  - `spot_full_range.mp4` (3,1 Ko), le même flux remuxé sans réencodage, marqué plage
    pleine (ffprobe : `yuvj420p`, `color_range=pc`) :
    `ffmpeg -i spot.mp4 -c copy -bsf:v h264_metadata=video_full_range_flag=1 spot_full_range.mp4`.
    `tests/full_range.rs` vérifie que chaque image de l'un est lue en `Limited` ou
    `Unspecified` et chaque image de l'autre en `Full`, puis que leurs détections, non
    vides, sont identiques au bit près.
- **CI** : sur macOS et Linux, deux tâches parallèles depuis #179 : `rust` (statique,
  tests rapides sous couverture) et `rust-slow` (tests `slow`, contrôles du binaire, essai
  matériel sous macOS). Avant #179, une seule tâche `rust` les enchaînait : 916 s sous
  macOS, dont 302 s de couverture et 461 s de `rust:slow`, et 816 s sous Linux (CI de
  `main` après #175). La couverture compile dans `target/llvm-cov-target`, les tests lents
  et le binaire dans `target/ci` : les deux étapes ne partageaient aucune compilation.
  `rust` ne tire aucune vidéo LFS : aucun test rapide n'en lit. Tâche `rust-reference`
  sous Linux, en parallèle depuis #146.
  Tâche `rust-settings` sous Linux, en parallèle aussi depuis #167 : les 16 cas du banc
  d'essai (`rust:settings`). Ils ne tournent plus sous macOS ni Windows ; `slow_extract`
  garde l'identité au bit près sur les trois systèmes.
  Quand l'empreinte de FFmpeg change, `rust-slow`, `rust-reference` et `rust-settings` le
  construisent donc aussi, sans le sauver : seule `rust` sauve le cache de FFmpeg. Le binaire contrôlé se construit par `--workspace --bin batdetect` : avec
  `-p batdetect`, les features de `syn` changent (`draw` en ajoute par `skrifa`), et
  `bindgen`, `ffmpeg-sys-next` et `media` se recompilaient (11 s sous Linux, 22 s sous
  macOS, 60 s sous Windows à la PR #145). Le cache de FFmpeg est
  restauré, puis sauvé dès la construction, même si la suite échoue. L'action `setup`
  installe la toolchain, les outils cargo et le Python de `rust:rules` (sans entrée
  depuis #199, qui a retiré sa branche Python). Premier passage
  (PR #130), sans cache de `target/` : `rust` 3 min 04 sur macOS et 3 min 46 sous Linux ;
  `rust-reference` 1 min 15. Depuis #181, deux actions composites servent à chaque tâche :
  - `.github/actions/ffmpeg-key` pose `XDG_CACHE_HOME` et calcule les clés des caches de
    FFmpeg et de `target/`, par un seul script, `key.sh`, sous `bash`, ou sous MSYS2 sur
    Windows.
  - `.github/actions/windows` met en place MSYS2, MSVC et rustup, et renomme le
    `link.exe` de coreutils.
- **Tâches `rust-windows` et `rust-windows-slow`**, indépendantes des autres et
  parallèles depuis #179 (812 s pour la tâche unique, dont 347 s de tests rapides et 332 s
  de `slow`). Quand l'empreinte de FFmpeg change, chacune le construit ; seule
  `rust-windows` le sauve. `rust-windows`
  lance les tests rapides, contrôle le binaire, puis `cargo clippy --profile ci -- -D
  warnings` ; `rust-windows-slow` lance les tests `slow` (sans les réglages). En profil
  `ci`, clippy reprend les scripts de construction et les macros des tests ; en `dev`,
  il recompilait 46 unités, dont les liaisons de FFmpeg (82 s à la PR #145).
  - `dumpbin -dependents` n'admet que des DLL de Windows : `kernel32`, `ntdll`,
    `bcrypt`, `bcryptprimitives` et `api-ms-win-core-*`. C'est une liste blanche, et
    `kernel32` doit y être : le contrôle ne peut pas passer à vide.
  - Ni couverture, ni `cargo deny`, ni `mise` : ils tournent déjà sous Unix.
  - clippy vient en dernier, pour qu'un échec propre à Windows laisse lisibles les
    résultats des tests.
  - Seule `rust-windows-slow` tire l'extrait ; son cache LFS se reprend depuis Linux
    (`enableCrossOsArchive`).
  - En cas d'échec, `config.log` de FFmpeg est téléversé en artefact
    (`ffmpeg-windows-logs` ou `ffmpeg-windows-slow-logs`).
- **Cache de `target/`** (`actions/cache`, une entrée par tâche) :
  - Il contient tout `target/` de sa tâche : sous Unix, `debug` (clippy) et
    `target/llvm-cov-target` (couverture) pour `rust`, `ci` (tests lents, binaire) pour
    `rust-slow` ; sous Windows, `ci` dans les deux tâches. Chacune garde la rlib de
    ffmpeg-sys-next avec ses liaisons. La clé commence par le nom de la tâche : `rust` et
    `rust-slow`, `rust-windows` et `rust-windows-slow` ont des entrées distinctes, et le
    repli de l'une ne prend jamais l'entrée de l'autre.
    L'entrée de `rust` sur `main` garde `target/ci` jusqu'au prochain `Cargo.lock` : une
    clé trouvée telle quelle ne se resauve pas.
  - Il se sauve en fin de tâche réussie.
  - Sa clé porte, dans cet ordre, la tâche, l'OS, l'architecture, l'image du runner
    (`ImageOS`), **l'empreinte de FFmpeg**, `rust-toolchain.toml`, puis `Cargo.lock`
    (action `ffmpeg-key` ; la tâche est celle qui appelle, sauf `rust-settings`, qui
    passe `job: rust-reference`).
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
  - `rust-settings` ne sauve rien : elle restaure l'entrée de `rust-reference` (même
    clé), qui a déjà compilé les dépendances en profil `ci`. Le quota du cache
    (10 Go) ne gagne aucune entrée.

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

**Dans les extraits (#112)** :

- **Un encodeur par groupe** (`media::encode::fanout::Fanout`, nuit du 12 au 13 octobre
  2026) : chaque image dessinée d'un groupe est encodée une seule fois, sans images B, et
  ses paquets sont écrits dans chaque extrait ouvert, avec une image-clé forcée sur la
  première image de chaque extrait (`AVFrame.pict_type = I`, que VideoToolbox et openh264
  honorent) et des `pts` rebasés à 0. Un extrait se ferme (en-queue, renommage) dès que le
  paquet de sa dernière image arrive. Gardes : le premier paquet de chaque extrait doit
  porter le drapeau clé (`NoKeyFrame` sinon), et un extrait doit avoir reçu toutes ses
  images (`ShortClip`) ; un paquet sans `pts` est une erreur. Vérifié par `ffprobe` sur
  les 117 extraits de la 099 : tous commencent par une image-clé (`key_frame=1`), aucune
  image B ; chaque extrait se décode seul (tests).
- **Sessions** : une session VideoToolbox par groupe, au plus 8 à la fois ; avec un groupe
  par fil et la moitié des cœurs, ce plafond n'est atteint qu'au-delà de 16 cœurs. La règle des voies (0 à 7 en
  VideoToolbox) n'existe plus : toutes les fenêtres d'un groupe ont l'encodeur du groupe.
- **Mémoire** : un encodeur par groupe en cours (un par fil), plus les images en vol et le
  cache de zones de chaque fil (borné à 4 millions de pixels, LRU). Mesurée sur la 099 :
  1,04 Gio pour le processus avec le décodage à plusieurs fils, 0,81 Gio sans.
- **Image-clé d'openh264** : une par seconde environ (`round(cadence)` images), pour
  qu'un lecteur se place vite dans l'extrait. VideoToolbox garde son réglage.
- **Extrait refait** : voir `render` (reprise en openh264 quand VideoToolbox casse en
  cours d'écriture). Aucun échec réel n'a été vu sur les vidéos mesurées ; la reprise est
  exercée par une fausse ouverture dans les tests.
- **Droits** : chaque extrait est créé par `File::create` sous un nom temporaire, donc
  avec les droits habituels de l'utilisateur (umask), comme le CSV et le PNG, puis
  renommé. Avant #112, le temporaire de `tempfile` donnait 0600.
- **Mesure** (12 octobre 2026, M1 Max, `release`, sous le verrou, commande complète :
  détection, CSV, image résumé et extraits ; `rapports/mesure-112/run.sh` et
  `xpc_footprint.sh`, hors dépôt). Processus : `peak memory footprint` de
  `/usr/bin/time -l`. VideoToolbox : somme des `phys_footprint` des services
  `VTEncoderXPCService`, relevée chaque seconde, moins les 57 Mo des services déjà
  ouverts au repos. Valable pour ce Mac.

  | Vidéo | Pistes | Réel | CPU | Processus | VideoToolbox |
  |---|---|---|---|---|---|
  | 092 | 14 | 18,9 et 18,8 s | 128 s | 795 Mio | non relevé |
  | 125 | 47 | 29,3 s | 166 s | 888 Mio | ≈ 1 045 Mo |
  | 099 | 117, jusqu'à 9 fenêtres à la fois | 64,3 s | 205 s | 883 Mio | ≈ 1 050 Mo |
  | 097 | 149 | 74,8 s | 215 s | 791 Mio | ≈ 1 050 Mo |
  | 103 | 10 | 37,8 s | 229 s | 1 023 Mio | ≈ 1 090 Mo |
  | 123 | 8 | 16,4 s | 115 s | 852 Mio | ≈ 914 Mo |

  - Au pire, processus et service ensemble restent sous 2,2 Go (somme des deux
    maxima, qui ne tombent pas forcément au même instant).
  - La 092 prend 18,8 s, contre 11,7 à 12,0 s pour la commande avant l'image résumé et
    les extraits ; la commande complète de la v0.1.1 prenait 134 s.
  - Pendant la sous-phase 2b, avec la plage pleine et sans l'image résumé, la 092 prenait
    24 s ; un groupe à la fois (2a), 45 s.
- **Après les sessions rendues en route et la conversion par zones** (13 octobre 2026,
  même machine, trois passages par chaîne en alternance, autres processus sous 100 % de
  CPU) :

  | Vidéo | Avant (réel, extraits / CPU des extraits) | Après |
  |---|---|---|
  | 092 | 17,75 s, 5,53 s / 25,5 s | 17,55 s, 5,39 s / 14,9 s |
  | 125 | 28,00 s, 14,69 s / 62,0 s | 27,98 s, 14,37 s / 37,7 s |
  | 099 | 62,77 s, 49,71 s / 104,6 s | 58,68 s, 45,53 s / 70,1 s |

  - Le CPU des extraits baisse d'un tiers à deux cinquièmes, leur temps réel de 2 à 8 %.
  - **La limite est le débit du codeur matériel** : pendant les extraits, VideoToolbox
    encode de 322 à 378 images 1440×1080 par seconde, toutes sessions confondues, avant
    comme après. Le temps des extraits suit « images envoyées à VideoToolbox / environ
    350 » (099 : 16 016 images, 45,5 s).
  - Une image commune à plusieurs extraits d'un groupe était encodée une fois par extrait
    (099 : 16 016 envois pour 6 616 images distinctes).
- **Après l'encodage par groupe et le décodage à plusieurs fils** (nuit du 12 au 13 octobre
  2026, même machine, trois passages, autres processus sous 100 % de CPU) :

  | Vidéo | Avant (commit 9e68090) | Après | Mémoire au plus |
  |---|---|---|---|
  | 092 | 17,55 s | 16,58 s (16,93 / 16,56 / 16,25) | 0,95 Gio |
  | 125 | 27,98 s | 23,56 s (23,23 / 23,43 / 24,03) | 1,02 Gio |
  | 099 | 58,68 s | 26,94 s (26,85 / 27,16 / 26,80) | 1,06 Gio |

  - Banc VideoToolbox seul (1440×1080, encodeur seul) : 224 images par seconde avec une
    session, 366 à 392 avec 2, 4 ou 8 ; les options `prio_speed` et `realtime` n'aident
    pas. Le M1 Max a deux moteurs d'encodage, saturés dès deux sessions.
  - Sur la 099, l'encodage par groupe seul donne 29,67 s ; le décodage à plusieurs fils,
    27,35 s (premier passage de chaque essai).

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
- **Noir calé sur 16** (décision de l'utilisateur, 12 octobre 2026, à l'œil sur les fonds
  de la 092, de la 122 et de la 124, #104) :
  - Lu en plage pleine, le ciel froid, autour de Y = 16, sort gris et non noir. Le fond
    de l'image résumé a un plancher à 10 (1er centile, après l'assombrissement à 0,6),
    et aucun pixel à 2 ou moins sur la 092, contre 26,6 % en v0.1.1 : un voile.
  - Trois variantes comparées : plage pleine, plage limitée (comme la v0.1.1, blancs
    écrêtés au-dessus de Y = 235), et noir calé sur 16 sans écrêter les blancs.
  - Retenue : `Y' = max(0, Y − 16) × 255 / 239`, arrondi au plus proche, avant la
    conversion en plage pleine ; la chrominance ne change pas. Y = 16 et en dessous
    deviennent noirs, Y = 255 reste blanc, les zones chaudes gardent leurs nuances.
  - Fond de la 092 : plancher 0, médiane 75 (80 en plage pleine, 79 en v0.1.1),
    24,8 % de pixels à 2 ou moins (26,6 % en v0.1.1).
  - Ce réglage ne touche que l'affichage : la détection lit Y brut.
  - Il vaut aussi pour les extraits depuis #112 : hors des zones dessinées, l'image
    passe par les tables d'affichage (`Converter::display`) ; dans les zones, par
    `FullBlackAt16`, le RGBA et le retour en plage limitée BT.709 (voir `render`).
    Un rendu en plage pleine fait tomber le SSIM des pixels hors dessins à 0,917
    environ, au lieu de 0,966 (`slow_clips`, essai fait une fois, avant les zones).

## Crates

La carte à jour des crates est dans [CLAUDE.md](../CLAUDE.md) ; cette section la détaille et suit
les lots.

Douze crates depuis #100 (neuf au lot 6, `render` à #110, `bench` à #100), treize avec `page` (#71), `members = ["crates/*"]`. `unsafe` n'est permis que dans `media`.

- **`types`** : vocabulaire commun, sans entrée-sortie ni algorithme de détection. Un
  type n'y entre que s'il est produit par une crate et lu par au moins deux autres. Gelé
  pendant le lot 6 (#124), dégelé par #144 ; un ajout passe par une PR fusionnée avant
  les files qui en dépendent.
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
    `FrameDetections` : une liste par image, images vides comprises, ordre gardé ; chaque
    détection porte le rang de son image. `without(&[Span])` vide les images couvertes
    par des plages ; ce qui en sort garde ses rangs.
    `Measures` : les mesures d'une détection sans son image, sérialisables (les
    détections des fichiers de `reference`, #160) ; `Measures::of` les tire d'une
    détection. `FrameDetections::push` ajoute une image dont les détections prennent le
    rang qu'elles remplissent (#150 : le détecteur ne peut plus ranger une détection
    sous une autre image ; #160 : `reference` relit ainsi les détections de ses
    fichiers, sans erreur de rangement possible ; #172 : `push` sur `default()` est la
    seule façon de remplir la liste, tests compris).
  - `Span` : plage d'images, bornes comprises, refusée si elle finit avant de commencer.
    Constructeurs qui ne peuvent pas échouer (#144) : `single` (une image), `hull` (de la
    première à la dernière image de deux plages), `widened` (élargie de chaque côté,
    bornée à 0 et à `usize::MAX`) et `intersection` (`None` sans image commune). Les
    plages instables et abîmées se bâtissent ainsi, sans variante d'erreur.
  - `Track` : numéro et points, au moins un, images strictement croissantes. Mesures :
    corde, longueur du trajet (+0 pour un seul point, jamais −0), surface maximale,
    amplitude de crête, virage médian (médiane des changements de cap, positions
    répétées ignorées, 0 sans virage). `with_id` renumérote sans revalider.
  - `Language` (français, anglais), `SourceSize` et `WorkScale` : taille de travail
    (largeur demandée sans agrandir, hauteur arrondie, au moins 1), et seul passage des
    pixels de travail aux pixels d'origine (coordonnée `(c + 0,5) × échelle − 0,5`,
    longueur × échelle, surface × échelle²).
- **`media`** : frontière avec FFmpeg ; aucun type de `ffmpeg_next` n'en sort. Une
  erreur par module.
  - `frame` : `YuvFrame`, 4:2:0 planaire sans `stride`, avec sa plage de couleur.
  - `failure::LibraryError::io_kind()` (#64) : le genre d'erreur système derrière une erreur
    d'FFmpeg, lu sur l'errno d'`AVERROR(e)`. Une seule table sans `cfg` (ENOENT, EACCES,
    EPERM, ENOSPC, EROFS), parce que FFmpeg porte sous Windows l'errno du CRT, de mêmes
    valeurs, et non un code Win32 que `from_raw_os_error` lirait. Aucun type d'`ffmpeg_next`
    ne sort.
  - `decode::Error::NoDecoder` (#64) : le FFmpeg livré n'a pas de décodeur pour le flux
    (`DecoderNotFound`), séparé de `Open` pour qu'un `.mov` en ProRes ne passe pas pour un
    fichier illisible. Un autre échec du décodeur reste `Open`.
  - `decode` : `Decoder::open(chemin, Threads)`, `stream` (taille, cadence, base de
    temps, plage, `pts` de départ `Option<Pts>`, nombre d'images annoncé par le conteneur
    `Option<NonZeroU64>`) et `next`, qui remplit un `YuvFrame` fourni par l'appelant
    (aucune allocation par image) et rend un `FrameInfo` : `FrameIndex`, `Option<Pts>`,
    image-clé (`is_key`), erreurs de décodage (`DecodeErrors`) et paquets illisibles
    depuis l'image précédente.
    - Ouverture (#227) : le chemin passe en octets sous unix (`path::location`, comme l'encodeur), puis `avformat_open_input`, `avformat_find_stream_info` et `Input::wrap` (`opened`, un `unsafe` par opération), parce que `ffmpeg_next::format::input` ferait `to_str().unwrap()`. Une vidéo au nom non UTF-8 s'ouvre donc sous Linux, au départ (`engine::input`) comme pour les extraits (`render::source`). `UnusablePath` reste pour un NUL et, sous Windows, pour un nom qui n'est pas de l'Unicode valide.
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
    - Placement (`decode::placed`, #110) : `Decoder::seek_before(pts)` se place sur la
      dernière image-clé à ou avant ce `pts` que connaît l'index du conteneur
      (`avformat_seek_file`, borne haute = la cible, puis `flush` du décodeur), même
      après la fin du fichier. Il rend un `Placed`, dont les images (`PlacedFrame`)
      n'ont qu'un `pts` : après un placement, le compte des images rendues ne les numérote
      plus. `Decoder::at_start` lit depuis le début avec la même interface. La cible
      passe du `pts` aux microsecondes de FFmpeg en arrondissant vers le bas, pour ne
      jamais dépasser l'image voulue ; FFmpeg la ramène à la base de temps du flux à
      l'arrondi le plus proche, donc au plus sur elle.
  - `encode` : `Settings::new` (taille paire, cadence positive), puis
    `Encoder::create`, `push` et `finish`, en openh264 ou VideoToolbox, sans images B.
    `fanout::Fanout` (#112) : un encodeur, plusieurs mp4 (`open(chemin, images)` avant la
    première image de chaque extrait, image-clé forcée) ; voir « Dans les extraits ».
    `open_stamped(chemin, Stamps)` (#103) : la sortie garde les `pts` de la source au
    lieu de compter ses images à la cadence de l'encodeur. `Stamps` (un `pts` par image,
    base de temps de la source) est refusé vide, négatif, non croissant ou sur une base
    non positive. Le flux mp4 prend la base de la source (1/10000 sur les jumelles, gardée
    telle quelle par le muxeur) ; chaque paquet reprend le `pts` de son image, `dts`
    égal (pas d'images B, sinon `Reordered`), et dure jusqu'à l'image suivante, la
    dernière un pas de la cadence de l'encodeur (sur l'extrait, la durée égale celle de
    la source : 40,959 s). Un premier `pts` non nul est gardé. `partial_file` crée le
    fichier temporaire d'une vidéo sous un nom unique (un test de `cli` exige que le
    ménage de `files::names`, que `media` ne peut pas appeler, le reconnaisse) ;
    `StreamInfo::duration` la durée annoncée du flux.
    Erreurs de destination (`concerns_the_destination`) : chemin refusé, temporaire non
    créé, fichier non ouvert (`OpenFile`), renommage raté ; une écriture refusée
    (`Writing`) n'en est pas une, pour que la bascule vers openh264 joue.
    - Réglages fixes, détaillés dans [Encodage](#encodage) :
      - VideoToolbox en q65, matériel seul (`allow_sw=0`) ;
      - openh264 en profil high, `rc_mode=quality`, QP 23.
    - Le fichier s'écrit sous un nom temporaire ; seul `finish` lui donne son nom.
    - Une image en plage pleine est refusée (`FullRange`). L'encodeur écrit une plage
      limitée en BT.709, dans le flux H.264.
    - Un paquet sans durée reçoit une durée d'une image.
    - Images-clés : sans réglage, openh264 n'en écrit qu'une, la première.
      `Settings::with_key_interval(n)` en écrit une toutes les n images (`gop_size`) ;
      le défaut n'a pas changé (#110), sa valeur pour les sorties revient à #112.
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
      2026, sur la mesure de [Plage réelle des sources](#plage-réelle-des-sources)),
      noir calé sur 16 (`SourceRange::FullBlackAt16`, 12 octobre 2026) : une table de
      256 niveaux appliquée au plan Y pendant la copie qui précède la conversion.
    - **Plage écrite** : le retour au YUV se fait toujours en plage limitée. Une source
      pleine y est resserrée sans rien écrêter : Y 0 devient 16 et Y 255 devient 235.
    - Aucune allocation par image.
    - Tables d'affichage (`display`, #112) : une table par plan, sans passer par le
      RGBA. Celle de Y est tirée de swscale sur une rampe de gris (noir calé sur 16
      compris), donc égale à l'aller-retour sur tout pixel gris ; celle de U et V
      resserre la plage pleine en plage limitée (128 + (c − 128) × 224 / 255, arrondi).
      Sur les 1 230 images de l'extrait, contre l'aller-retour par RGBA
      (`slow_convert`) : 0,11 % (Y), 0,15 % (U) et 0,11 % (V) des valeurs diffèrent,
      0,02 %, 0,06 % et 0,04 % de plus de 2 niveaux ; au plus 34 (Y), 18 (U) et
      51 (V). Le pire écart est un pixel de V à 255 (l'icône rouge des jumelles) que
      l'aller-retour écrête en RGB et que la table garde.
    - Zones (`yuv_to_rgba_region`, `rgba_to_yuv_region`, `Region`, #112) : la même
      conversion sur un rectangle aux coordonnées paires, par des contextes swscale à
      sa taille (une taille nouvelle crée ses contextes ; cache LRU de 32 tailles et
      4 millions de pixels au plus ; `render` arrondit la taille des zones au multiple de
      16 pour que le cache serve). Dans le rectangle, les pixels
      RGBA sont ceux de la conversion entière, et le Y écrit aussi (testé).
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
  - `payload`, derrière la feature `testing` (tests seulement) : `wipe(chemin, parts,
    nombre)` écrase de `0xff` des parts égales de la boîte `mdat` d'un mp4. Les images
    qui y tombent sont perdues (saut de `pts`) ou dissimulées. Une plage de parts vide,
    hors de la boîte ou qui déborde est refusée. Les tests de `media`, d'`engine` et de
    `render` y fabriquent leurs vidéos abîmées.
  - `fingerprint`, derrière la même feature (#154) : `of(image)` hache les trois plans
    d'une image décodée, `of_pixels(largeur, hauteur, octets)` les pixels d'une image
    (le PNG d'une image résumé), `of_sequence` une suite d'empreintes (les images d'un
    extrait). C'est la seule empreinte d'image des tests de `media`, `engine`, `render`
    et `cli`. Depuis #198, c'est un FNV-1a sur des mots de 64 bits écrit à la main, dont
    chaque pas replie les bits hauts sur les bas (sans ce repli, deux mots changés dans
    leur seul octet haut s'annulaient une fois sur 256). Il est le même sur chaque
    système et chaque version de Rust (`DefaultHasher` avant) : une
    empreinte peut s'enregistrer dans un test figé (`a_stored_fingerprint_stays_valid_…`).
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
    pour `media::convert`. Primitives, lissées sauf les rectangles (choix de
    l'utilisateur, même planche) :
    - rectangle aux bords nets :
      - 1 px couvre les pixels du bord de la boîte ;
      - chaque pixel de plus élargit le bord vers l'extérieur, si bien que la boîte de
        2 px (image détectée) et celle de 1 px (image interpolée) gardent le même bord
        intérieur ;
    - rectangle plein aux bords nets, bornes comprises, coupé au cadre (masques et
      séparateur de l'image résumé, #104) ;
    - copie d'une toile dans une autre, au pixel près, coupée au cadre (#104) ;
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
- **`summary`** → `draw`, `types`, `files` : l'image résumé (#104), une page par tranche
  de 10 minutes, sans FFmpeg : `cli` lui donne le fond déjà converti.
  - Modules : `geometry` (chemins en `f64`, point à une longueur, rééchantillonnage,
    point le plus proche), `colors`, `arrows`, `placement`, `periods`, `header`,
    `marker`, `legend`, `layout` (les données d'une page), `compose` (le dessin),
    `image_file` (PNG). `assigned::track_colors(summary, largeur, hauteur)` (#13, feature `testing`, tests seulement) rend, dans l'ordre des pistes, la couleur de chacune sur l'image résumé de cette taille, par la même mise en page sans dessin : les tests figés de `cli` la passent au rendu des extraits, comme `process` y passe les couleurs de l'image dessinée.
  - `pages` rend `Summaries` (#259) : le fond (`backdrop`, la scène assombrie, zones masquées peintes), le même pour toutes les pages, et pour chaque page son image, sa tranche (`window`, entière même pour une partie), sa partie, ses périodes coupées à la tranche, les débuts des pistes des autres parties, le rayon des pastilles, et pour chaque piste dessinée un `Painted` : chemin, flèche, centre de la pastille et départ du trait de rappel, tirés des `Mark` de `layout`. `Mark::leader` est le seul critère du trait de rappel (pastille à plus d'un rayon de la trajectoire), pour `compose` comme pour la page. L'image complète ne change pas.
  - Mêmes règles que la v0.1.1 pour les couleurs (indice le moins employé autour), les
    flèches (25°, 3 % de la longueur), la légende (colonnes, coupe par « ... ») et les
    tranches. Les tris Python deviennent des clés entières ; les points ne sont pas
    arrondis avant le dessin (règle 5).
  - Temps par les `pts` (#35) : durée = début de l'image qui suivrait la dernière, tranche
    et temps de légende d'une piste = `frame_seconds` de sa première image, plages =
    `spans_in_seconds`, comme `params.json`.
  - Bandeau : `header` rend des `HeaderLine` (nom, pistes, fenêtre, hors analyse,
    illisible, « + N »), et un trait `Labels` les met en texte. Libellés v0.1.1 gardés,
    avec accents possibles (police Roboto, NFC).
    - Depuis #127, les plages illisibles sont datées comme les plages hors analyse :
      3 lignes « illisible a - b » (début tronqué, fin arrondie au-dessus), puis
      « + N autres, 3 min 22 s illisibles en tout » (« + N more, … unreadable in all »).
      La durée totale additionne les plages exactes, arrondie au-dessus, comme la v0.1.1,
      qui ne donnait qu'elle et le nombre de plages. Les plages hors analyse n'ont pas de
      total, comme en v0.1.1.
    - Dans le bandeau, les plages qui se touchent ou se recouvrent une fois arrondies à
      la seconde ne font qu'une ligne (121 : « 0:04 - 0:06 », « 0:05 - 0:08 »,
      « 0:07 - 0:09 » deviennent « 0:04 - 0:09 ») ; la frise et `params.json` gardent
      le détail exact.
    - Largeur du bandeau : seule la ligne du nom de la vidéo reste plafonnée à 3 colonnes
      de légende (coupée par « ... » au-delà, comme la v0.1.1) ; les autres lignes,
      bornées, l'élargissent toujours.
  - **Frise du temps** (#127, module `time_strip`) : sous l'image, sur sa largeur, dans
    chaque tranche.
    - Deux bandes, « hors analyse » (gris moyen) et « illisible » (gris clair du texte du
      bandeau), nommées à gauche par `Labels::band`. Une bande vide est un cadre fin sur
      le fond sombre du bandeau, pour ne pas se lire comme une bande pleine.
    - Plages par `spans_in_seconds`, comme `params.json` et le bandeau ; une plage fait
      au moins 2 px, si bien que les 87 plages de la 121 restent toutes visibles.
    - Graduée en minutes (pas de 1, 2, 5 ou 10 minutes selon la place, des diviseurs de
      la tranche de 10 minutes, si bien que son début est toujours gradué) ; un
      repère par piste, à sa couleur, au temps `pts` de sa première image.
    - Le bandeau reste à la hauteur de l'image ; la page grandit de la hauteur de la
      frise (135 px en 1440). Une vidéo sans période ignorée a ses deux bandes
      vides : mêmes données de mise en page qu'à #104. Choix de l'utilisateur sur la
      planche : repères de départ gardés.
    - La frise couvre toujours la fenêtre de la tranche, même sur une image coupée par
      #157 : les départs des pistes de l'image en couleur, ceux des autres pistes de la
      tranche en gris (choix de l'utilisateur sur la planche).
  - **Images coupées au nombre de pistes** (#157) : une tranche de plus de
    `MAX_TRACKS_PER_IMAGE` = 20 pistes (choix de l'utilisateur sur la planche de la 097
    et de la 099 à 15, 20 et 25) donne `ceil(n / 20)` images, en paquets égaux de
    pistes consécutives par départ (21 pistes : 11 + 10, et non 20 + 1).
    - Mêmes numéros (ceux de la vidéo entière), même fond, même mise en page ; couleurs
      et pastilles attribuées paquet par paquet (choix de l'utilisateur : des voisines
      d'autres images ne retirent plus de couleurs à celle-ci).
    - Bandeau : nom, « partie 3 / 8 » (« part 3 / 8 »), le nombre de pistes du paquet,
      sa plage de temps (du premier départ à la dernière fin du paquet, fin arrondie à la
      seconde supérieure comme les plages ignorées), puis les plages hors analyse et
      illisibles de toute la tranche.
    - Les pistes arrivent triées par départ depuis le suivi (`track_detections`) ; les
      paquets suivent cet ordre sans le refaire.
    - Une tranche de 20 pistes ou moins reste une seule image, inchangée.
    - Mesure (pistes de la Rust, commande complète) : 097, 149 pistes, 8 images ; 099,
      117 pistes, 6 images. Sur la référence, 10 vidéos sur 26 ont une tranche de plus de
      20 pistes.
  - Masques de l'affichage : une liste de rectangles peints en gris de panneau avant
    l'assombrissement. `cli` les tire des zones OSD de la configuration (#151) :
    `Region::columns` et `rows` sans marge, sur la taille de la source, soit
    `[⌊x0·w⌋, ⌈x1·w⌉)` comme `output/background.py` ; `PixelBox` inclut ses bornes,
    donc `right = fin − 1`. Vide tant que la commande n'a pas `--osd-region` (#113).
  - Écarts voulus : fond lu en plage pleine, noir calé sur 16
    ([Plage réelle des sources](#plage-réelle-des-sources)) ; police et rayon de
    `draw::style` (rayon 23 en 1440, 29 pour un numéro à trois chiffres contre 26 en
    v0.1.1).
  - **Placement des pastilles** (choix de l'utilisateur, 12 octobre 2026) : zéro
    pastille hors cadre et zéro recouvrement, chaque pastille au plus près de sa piste.
    - Candidats tous les demi-rayons le long de la piste ; clé (pastilles touchées,
      flèches touchées, croisements, écart au quart de la piste).
    - Une pastille qui ne touche qu'une tête de flèche reste sur sa piste. Si elle
      touche une autre pastille, elle s'écarte de sa piste par anneaux d'un demi-rayon
      (16 directions, 40 anneaux au plus) : le premier anneau sans autre pastille
      l'emporte, et dans cet anneau une place libre des flèches passe d'abord. Un trait de rappel, aux couleurs de la piste, la relie
      alors au point le plus proche de sa piste dès qu'elle ne la touche plus.
    - Puis jusqu'à 8 passes : chaque pastille encore en collision est replacée, si la
      nouvelle place touche moins que l'ancienne.
    - Mesure sur les pistes de la référence (26 vidéos, 966 pistes), centres à moins
      de 2 rayons : v0.1.1 266 recouvrements, portage fidèle 349, placement retenu 0 ;
      aucune pastille hors cadre. 121 pastilles s'écartent de leur piste de plus d'un
      rayon, presque toutes sur les vidéos denses ; la plus éloignée est à 304 px (096).
  - PNG écrit dans son partiel à côté (`files::names::partial_file`, #115, #245), puis
    renommé. Une erreur d'entrée-sortie de l'encodeur PNG remonte en `Error::Write` avec
    son `io::Error` (#64) : `png` n'expose sa cause que par `cause()`, et un disque plein
    se perdrait ; droits ordinaires du
    fichier créé (un fichier temporaire de `tempfile` resterait en `0600`).
  - Fond de la 092 mesuré une fois contre la v0.1.1 (luminance, hors traits, 81 % des
    pixels), avant le choix du noir calé sur 16 : SSIM 0,702 tel quel en plage pleine,
    0,984 ramené en plage limitée pour la mesure (écart moyen 1,4 niveau).
  - Tests : les 57 tests Python portés (sauf « en-tête ASCII », inversé en « nom
    décomposé dessiné comme le nom composé ») ; `tests/v011.rs` compare, sur l'extrait
    et la 092 de la référence, numéros, couleurs, tranches et en-tête aux attendus de la
    v0.1.1 ; `tests/local_v011.rs` fait de même sur les 26 vidéos, avec 22 écarts
    listés : 13 temps de légende et une ligne « hors analyse » de la 096 par les `pts`,
    la durée illisible totale de la 120 (1 min 14 s contre 1 min 13 s), et les plages
    hors analyse réunies à la seconde depuis #127 (089, 096, 098, 103). Pour les plages
    illisibles, la comparaison porte, sur chaque tranche, sur leur nombre et leur durée
    totale, tirés des plages elles-mêmes et non du bandeau. Une tranche coupée par #157
    est comparée pages réunies : numéros et en-tête ; ses couleurs, attribuées par
    paquet, ne le sont pas.
- **`page`** (#71) : la page de résultats d'une soirée, en HTML, sans aucune dépendance (ni crate du workspace, ni FFmpeg). Elle ne lit pas le disque : `cli::written` lui donne un `model::Page`, et elle rend une chaîne.
  - `model` : la page (titre, entrées dans l'ordre, entrées refusées, fichier de verdicts), une entrée par vidéo : `Video` (fond avec sa taille, images résumé avec leur taille et leur mise en page `Layout`, pistes avec couleur, image, extrait, zoom, point de départ `start_xy` et `Trace`), `Failed` (clé de la cause, place, détail, résultats d'avant s'il y en a), `Unfinished`, `Waiting`, `NotStarted` (vidéo du lot que le calcul arrêté n'a pas atteinte, #247). `VerdictsFile` : nom, date, nombre de fichiers `verdicts*.csv` trouvés et octets du plus récent (#228).
  - `labels` : `Label`, chaque texte de la page avec ses arguments, et le trait `Labels`, que `Texts` implémente dans le binaire (`sheet.rs`) avec les textes `page-*` des `.ftl` ; la raison et le conseil d'un échec sont les messages `failure` et `failure-todo` du bilan. `Label::Script(Script)` porte les textes du script de la page.
  - `script` (#228) : `Script`, les textes dont le JS a besoin, chacun avec sa clé, son message Fluent (`page-js-<clé>`) et ses trous (`{count}`, `{track}`…). `sheet` les rend avec chaque trou écrit `{nom}`, la page les embarque en JSON (`<script type="application/json" id="texts">`), et le JS remplit les trous. Les en-têtes et le dialecte du fichier de verdicts (séparateur, virgule décimale) sont aussi des textes de la langue.
  - `escape` : texte échappé (`& < > " '`) et chemins encodés en pourcents segment par segment, tout octet hors de `A-Z a-z 0-9 - . _ ~`, si bien qu'un `#`, un `%` ou un `?` dans un nom reste un nom de fichier. Rien n'est normalisé : le lien suit l'octet du disque. `json` : chaîne JSON qui ne peut pas fermer le `<script>` qui la contient (`<`, `>`, `&` en `\u`). `base64` : les octets du fichier de verdicts, sans dépendance.
    - Limite (#227) : les noms arrivent au modèle en chaînes lossy. Pour une vidéo au nom non UTF-8 (Linux), la section montre ses pistes, mais ni l'image résumé ni la vidéo annotée (`results` ne les retrouve pas), et les liens vers le tableau, les extraits et le dossier pointent vers `�` et sont cassés.
  - Image redessinée (#259) : quand la vidéo a un fond et l'image sa mise en page, le panneau de l'image porte `caption` (la ligne au-dessus : tranche, pistes, périodes en secondes entières fusionnées, trois au plus puis « + N autres »), le lien vers l'image complète, `scene` et `strip` ; sinon l'image complète, comme avant. Leurs textes passent par `Label::Scene(SceneText)`.
    - `scene` : un `<svg>` en pixels de la vidéo (`viewBox`), le fond en `<image>`, puis cinq calques dans l'ordre de `compose` : contours noirs, traits de couleur, flèches, traits de rappel, pastilles numérotées. Chaque forme porte le `data-id` de sa piste, pour la vignette et les traces cliquables (#269). Traits en épaisseur d'écran (`vector-effect: non-scaling-stroke`), pastilles et numéros à l'échelle de la mise en page, police du système. `role="img"` et `aria-label` : la liste reste l'équivalent au clavier et au lecteur d'écran.
    - `strip` : la frise en HTML, `aria-hidden`, à hauteur fixe. Bandes « hors analyse » et « illisible » seulement si elles ont une période ; débuts des pistes en traits de 3 × 16 px dans une zone de 24 px, couleur de la pastille, gris pour les autres parties ; minutes posées dans un élément large d'une minute (`.unit`, `container-type: inline-size`), si bien que des requêtes `@container` choisissent le pas, 1, 2, 5 ou 10 min, selon la place, sans JS.
    - `colour` : `rgb(r g b)`, partagé avec `tracks`.
    - Colonne de droite étroite (jugement à 1180 px : 384 px, colonne verdict de 82 px pour une puce « ● à voir » de 78 px, toutes coupées) : `.review` est un conteneur nommé (`container: review / inline-size`) et, sous 27rem, la colonne Zoom du tableau disparaît ; la tête du jugement dit déjà si la piste a un zoom. La colonne verdict passe à 140 px.
    - La vignette de la zone de jugement (`judge.js`) est un clone du SVG de la partie, avec la seule pastille de la piste active, agrandie, et des traits plus fins ; sans fond, l'image complète.
  - `document` (page entière, total, marque `<meta name="generator" content="batdetect">` et `is_ours`, lue dans le premier Kio), `bar` (la barre des verdicts collée en haut, les textes du script et le fichier de verdicts embarqué en base64 dans `<template id="verdicts-file">`, seulement s'il y a une vidéo), `section` (une vidéo : onglets de parties, images avec `width` et `height` pour que l'onglet ne fasse pas sauter la page, vignette, bloc des verdicts sans piste), `judge` (la zone de jugement, cachée tant qu'aucune piste n'est choisie), `tracks` (le tableau, une ligne par piste avec sa clé : `data-start`, `data-x`, `data-y`, `data-clip`, `data-zoom`), `cards` (échec, inachevée, en attente, pas encore traitée, entrées refusées).
  - `assets/` : `page.css` et six scripts classiques inclus dans l'ordre par `include_str!` dans un seul `<script>` (un `import` entre fichiers est bloqué en `file://` sous Chrome), chacun exposant un nom global : `csv.js` (`VerdictsCsv` : décodage UTF-8 strict sinon Windows-1252, lecture tolérante, écriture), `store.js` (`VerdictsStore` : l'état des verdicts, le rattachement, les décomptes, les verdicts sans piste et des vidéos absentes, l'écart au fichier lu et au dernier téléchargement), `bar.js`, `list.js`, `judge.js` (les vues) et `page.js` (onglets, tout déplier, brouillon, téléchargement, rechargement). La page tient en un fichier, sans réseau, avec `preload="none"` sur le lecteur de chaque vidéo.
  - Rattachement d'un verdict à sa piste (`store.js`) : candidates les pistes de la même vidéo (`data-video`, nom du dossier) dont le début est à 0,2 s au plus ; paires classées par distance des points de départ quand les deux sont connus, sinon par écart de début ; affectation gloutonne, une piste pour un verdict. Un verdict sans piste libre est gardé et montré, avec la piste la plus proche. Une vidéo sans extrait, sans zoom et sans image résumé n'a pas de zone de jugement (classe `no-viewer`, numéros sans bouton, liste réduite au numéro, au début et à la durée, sans colonnes Verdict ni Zoom, #247) : ses pistes ne sont pas « jugeables » (`judgeable`), restent hors de « à voir » et de « jugées », et un verdict qu'un fichier leur rattache n'est pas montré mais compte dans la barre et se réécrit tel quel.
  - Brouillon (`page.js`) : à chaque changement qui s'écarte du fichier lu, les lignes du fichier à écrire vont dans `localStorage` sous `batdetect:verdicts:<chemin de la page>`, avec l'empreinte (nom, date, taille) du fichier lu et l'heure du dernier téléchargement. À la réouverture, sur le même fichier, elles sont reprises et la page le dit ; sur un autre fichier, la page demande. `beforeunload` demande confirmation tant que des changements ne sont pas téléchargés. Un seul endroit écrit le texte (`VerdictsCsv.write(store.rows())`) et un seul l'envoie (le lien de téléchargement) : c'est là que #252 branchera l'enregistrement par un serveur local.
  - Tests du JS : `node --test` sur `csv.js` et `store.js` (`assets/tests/*.test.mjs`, qui chargent les scripts par `vm`), tâche `page:js` de `mise`, dans `check` et dans la tâche CI `page-js`, sous Linux seulement : ce sont des fonctions pures. Les vues se vérifient dans un navigateur.
- **`reference`** → `types` : la référence v0.1.1 en Rust. Gelée pendant le lot 6
  (#124), comme `types`.
  - `fixture` : chemins des fixtures et garde LFS. `location` : `committed(référence, nom)` pour
    l'extrait et la 092 du dépôt, `recording_directories(référence)` pour les dossiers `data/*`
    de la variable de la référence, triés par nom ; une variable absente ou vide est
    refusée. `recorded_video(nom)` trouve le `.mp4` d'un enregistrement par son nom
    (`video.name` de `video.json`) dans le dossier de `BATDETECT_VIDEOS_DIR` (sur le Mac
    de l'enregistrement : `~/dossier sans titre/img_0000/`), avec le même refus d'une
    variable absente ou vide.
  - Lecture et écriture des quatre fichiers, un module chacun : `video`
    (`VideoSummary`), `frames` (`Frames` : table par image et `FrameDetections`, de même
    longueur, vérifiée par `Frames::new`),
    `tracks`, `table` (CSV, fins de ligne CRLF, flottants à l'écriture la plus courte).
    `recording` les réunit et donne la `Timeline`. Un nom en `.gz` est compressé
    (gzip sans nom ni date) ; les sections sans type de `video.json` (`opencv`, réglages,
    versions) sont gardées telles quelles dans `recorded`. Relus puis réécrits, les
    fichiers redonnent les mêmes valeurs.
  - Deux versions du format de `video.json` (#274), pour la référence v0.2.0 du Rust (#250) :
    - La version 1 est celle de la v0.1.1 : un bloc `ffprobe` et un `tracks.csv`.
    - La version 2 remplace `ffprobe` par `decoder`, ce que voit `media` : `error_frames`, `gap_frames`, `losses` (`before_first`, `after_last`) et `threads`. Ces pertes au bord ouvrent une plage abîmée, et `frames.jsonl` ne les garde pas (#176). `threads` (`DecoderThreads`, comme `media::decode::ActiveThreads`) vaut `{"kind": "single"}`, ou `{"kind": "frame"|"slice", "count": n}` avec n ≥ 1 : le nombre de fils change ce que le décodeur dissimule dans les plages abîmées. Un nombre nul, un genre inconnu ou un nombre sur `single` sont refusés à la lecture. La version 2 n'a pas de `tracks.csv` : le CSV du Rust (13 colonnes, en-têtes Fluent de `cli`) ne s'écrirait pas depuis `compare`, qui enregistrera la référence, sans faire de cycle avec `cli`.
    - `VideoSummary::read` refuse toute autre version, avant de lire les champs. `read` et `write` refusent aussi un résumé dont les blocs ne sont pas ceux de sa version (`MissingSection`, `ForeignSection`), et `write` le fait avant d'écrire.
    - `Recording` donne `rows` en `Option`. Il refuse d'écrire des lignes sans nom de fichier, ou l'inverse (`TableWithoutFile`), avant d'écrire le moindre fichier.
    - `location::Reference` (`V011`, `V020`) choisit le dossier du dépôt (`tests/fixtures/reference/` ou `tests/fixtures/reference-v0.2.0/`) et la variable des données hors dépôt (`BATDETECT_REFERENCE_DIR` ou `BATDETECT_REFERENCE_V020_DIR`). `committed`, `committed_settings`, `recording_directories` et `recorded_bench` prennent la référence, comme `compare::case::Target::cases` et `Case::committed`. Toutes les gardes lisent encore `V011` ; elles passent à la v0.2.0 à #276 et #277.
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
  - `bench` (#107, #108) : les fichiers du banc de la v0.1.1, que le Rust réécrit au
    même format. `TargetLine` et `Patch` (`bats.jsonl.gz`, `motion` gardé en chaîne,
    aucun champ inconnu), les lignes `{frame, detections}` (`injected.jsonl.gz`, ordre des
    images vérifié, vers `FrameDetections`) et `BenchReport` (`bench.json`) : les réglages
    restent en JSON brut, les clés que la v0.1.1 n'écrit pas (`targets`, `seconds`) sont
    gardées dans `added`, les entiers du Python se lisent en `f64`. Une ligne enregistrée
    se réécrit octet pour octet. `location::recorded_bench(nom)` donne
    `$BATDETECT_REFERENCE_DIR/bench/<nom>/`, et `location::recorded_outputs(langue, nom)`
    donne `$BATDETECT_REFERENCE_DIR/outputs/<langue>/<nom>/`, les sorties de la v0.1.1
    (#115).
- **`engine`** → `media`, `types` : le moteur. `input` en est le seul point d'entrée.
  - `Input::open(chemin, Threads)` accepte toute taille et toute plage (#150). Il
    refuse, par une erreur typée qui nomme la vidéo, 0 image annoncée et, par `media`,
    un format autre que `yuv420p` ou `yuvj420p`. `Video` donne la taille, la cadence, la
    base de temps, le `pts` de départ et le nombre annoncé avant la lecture, pas la
    plage : la détection lit le plan Y tel quel, avec la même table quelle que soit
    l'étiquette (décision du 12 octobre 2026), et l'affichage force la plage pleine
    quelle que soit l'étiquette (décision du 11 octobre). Une vraie source `pc` est donc
    détectée sur les mêmes Y qu'une source `tv` ; « au moins aussi bien que la v0.1.1 »
    ne s'y mesure pas, OpenCV la lisant autrement. Aucune des 26 vidéos n'est concernée.
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
    (`Vec<Span>`), comme `damaged_spans` de la v0.1.1. `reported_frames` compte l'union des
    images en erreur et après un saut (`damaged_frames` de `params.json`, #123). Deux
    différences avec la v0.1.1 :
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
    sa couture). `detect_injected(Input, &DetectConfig, crochet)` (#107) appelle le
    crochet sur la copie du plan Y que garde le détecteur, une fois par image, dans
    l'ordre de lecture, avec son `FrameIndex` et une `LumaMut` (vue mutable du plan, à la
    taille d'origine). Les images du fond de l'image résumé reçoivent l'image lue, avant
    la copie : le crochet ne les touche pas. `detect_input` passe un crochet vide ; le
    crochet est générique, la commande ne paie ni appel indirect ni branche.
    `LumaMut::new(pixels, largeur)` refuse un plan qui n'est pas un nombre entier de
    lignes non vides. Réglages : ceux que `DetectConfig::try_from` accepte (`settings`) ;
    toute taille (#150). Le pool `rayon`
    est celui de l'appelant (`ThreadPool::install`). `Detected` rend la lecture
    (`Reading`), les détections (`FrameDetections`, une liste par image décodée, vides
    comprises), les dégâts (`Damage`, marge = demi-fenêtre), la demi-fenêtre, les fils
    actifs du décodeur (`Input::active_threads`, pour `params.json`), le nombre de fils de
    la détection (`detection_threads`, celui que le détecteur emploie, #123) et les images
    du fond.
    - `config` : `DetectConfig::default()` (seuil 12, surface 4 à 2 700, fenêtre 1,0 s,
      une image sur 3, fusion 6, largeur 960, bruit 8, sigma 1,5), champs en lecture.
      `DetectConfig::try_from(&DetectSettings)` la construit depuis les réglages bruts
      (#149) ; voir `settings`.
      `half_window(cadence, pixels de travail)` arrondit `fenêtre × cadence / 2` au plus
      proche, moitié vers le haut (la v0.1.1 arrondit au pair : une image d'écart quand ce
      produit vaut x,5, jamais à 30 i/s), et refuse moins de 3 images de fenêtre ou moins
      de 3 échantillons. Le nombre d'échantillons d'une cible intérieure,
      `⌊2·demi-fenêtre / pas⌋ + 1` (`window::most_sampled`, saturant), dimensionne les
      réseaux et les tampons. La 092 : 15, et 11 échantillons ; `window_2s` : 30 et 21 ;
      2 s au pas de 1 : 61.
      - Borne haute (#113) : la fenêtre garde toutes ses images, quel que soit le pas,
        à la taille d'un `window::Unit` (`u16`) par pixel de travail. `half_window` refuse
        (`LongWindow`), en `f64` et avant tout calcul entier (une fenêtre infinie aussi,
        même sans pixel), une fenêtre dont les images dépassent
        `BACKGROUND_BUDGET` = 4 Gio : 3 105 images à 960×720, 1 379 à 1440×1080. Une
        fenêtre absurde ou infinie (`--bg-window` du banc, `compare`) donne cette erreur
        au lieu d'un débordement ou d'un manque de mémoire. La commande n'expose pas la
        fenêtre ; le défaut en prend 31 images.
      - Mesure `dhat` (13 octobre, vidéo fabriquée de 3 200 images 1440×1080, travail
        960×720, pic du tas de la détection) : 31 images, 368 Mo ; 301, 876 Mo (884 Mo au
        pas de 1) ; 1 003, 1 857 Mo (1 992 Mo au pas de 1). Entre 301 et 1 003 images, 1,40
        Mo par image de fenêtre (2,02 octets par pixel de travail : le modèle du budget
        tient à 1 % près), sur environ 0,4 Go fixes ; le pas de 1 ajoute les réseaux de
        tri, 0,13 Go à 1 003 images. Au plafond, la détection tient donc autour de 5 Go au
        pas de 3, sous la mémoire de la machine de Manon (16 Go). La mesure s'est arrêtée
        avant les points de 2 000 et 3 105 images (verrou de mesure rendu).
    - `settings` (#149) : `DetectSettings`, les dix réglages de la v0.1.1 sous leurs noms
      Python (`threshold`, `min_area`, `max_area`, `bg_window_s`, `bg_step`,
      `osd_regions`, `merge_radius`, `work_width`, `noise_factor`, `target_sigma`),
      `Deserialize` (tous les champs exigés, aucun champ inconnu) et `Serialize` dans
      l'ordre de la dataclasse : le même type lit `video.json` et écrit `params.json`
      (#151, la copie privée de `cli` a disparu). `Default` se déduit de
      `DetectConfig::default()` par `From<&DetectConfig>`, une seule source des valeurs.
      Une zone OSD (`Region`) s'écrit par ses quatre bornes `x0`, `y0`, `x1`, `y1`, comme
      `asdict` ; `columns(largeur, marge)` et `rows(hauteur, marge)` la convertissent en
      pixels, `[⌊f0·L⌋ − marge, ⌈f1·L⌉ + marge)` borné à l'image, comme `osd_mask`.
      `contains(x, y)` (#107), bornes comprises, comme `Region.contains`, sert à la
      visibilité du banc.
      `DetectConfig::try_from` refuse ce que la v0.1.1 refuse (`Error::Setting` avec son
      `Setting` et la borne enfreinte, `Areas`, `Region` avec son rang), avec les mêmes
      écarts que le suivi : NaN et infini refusés. Écart voulu de #151 : `merge_radius`
      est borné à 12 pixels d'origine (`Bound::AtMost`), la v0.1.1 acceptait tout. Une
      image d'éclair plein cadre, sur un fil, en `release` : 0,34 s au rayon 6 et 0,99 s
      au rayon 12 au travail en 1440, 3,77 s au rayon 25 ; 0,81 s au rayon 25 au
      travail en 960. La borne tient une seconde par image à toute largeur, et recolle
      encore les fragments écartés de 18 à 24 px de docs/02. Plus aucun réglage valide
      n'est refusé : `Unsupported` a disparu.
    - `geometry` : toute réduction (#150), largeur et hauteur de travail de `WorkScale`
      (largeur demandée sans agrandir, hauteur arrondie), refus d'un travail de plus de
      `u32::MAX` pixels. Seul `WorkScale` passe aux pixels d'origine, avec l'échelle des
      largeurs pour les deux axes, comme la v0.1.1 : quand la hauteur de travail est
      arrondie, y s'écarte de moins de 0,5 pixel d'origine en bas de l'image.
    - `reduction` (#150) : Y → gris par la table `GRAY`, puis réduction par aire, par
      axe : le rapport source / travail réduit à p/q, le pixel de travail i couvre la
      source de i·p à (i+1)·p en unités de 1/q, et chaque pixel source pèse la longueur
      de son intersection. Les deux axes ont leur propre rapport (hauteur arrondie,
      comme `fx` et `fy` d'`INTER_AREA`). Un seul arrondi à 8 bits, `(total + T/2) / T`
      avec T = p_x × p_y, par l'inverse sur 64 bits (exact pour un total sur 32 bits,
      méthode de Lemire), par une vraie division au-delà. En 3:2, T = 9 et les poids 2, 1
      puis 1, 2 : les sommes et l'arrondi du premier moteur, testés par un proptest
      contre l'ancienne réduction et contre un calcul brut des aires. La passe
      horizontale va par blocs de q pixels de travail, le nombre de poids par pixel fixé
      à la compilation de 1 à 4.
      - Taille inchangée (1:1, #159) : quand la source a déjà la largeur et la hauteur
        de travail, la réduction n'est que la table `GRAY`, sans aire, sans arrondi et
        sans copie du gris de la source. Un fil, `release` : 0,41 ms par image à 1440
        contre 1,84. Sur `092-1440` : 158,5 s de CPU contre 166,1 (−4,6 %, trois paires
        A/B en alternance avec `main`), 613 Mo de mémoire résidente contre 628.
      - Essayés et écartés par #159, mesurés à 960 contre 1,17 ms par image : les
        champs de l'arrondi pris dans les fermetures (1,18 ms, LLVM le faisait déjà) ;
        des poids contigus avec totaux et inverse sur 32 bits (1,39 ms) ; l'inverse
        sur 32 bits seul (1,21 ms).
    - `blur` (#150) : noyau gaussien de σ = `target_sigma / échelle` pixels de travail,
      portée ⌈4σ⌉ (la même qu'OpenCV pour tous les σ du banc), poids
      `exp(−d²/2σ²)` (le poids central posé à 1, sans formule) partagés en 4 096 parts
      par les plus forts restes, par paires symétriques : somme exacte, symétrie, aucun
      poids négatif. σ = 0 donne l'identité, `[4096]`. σ = 1 redonne le noyau du
      premier moteur, `[1, 18, 221, 991, 1634, …]`. Dès un σ de travail d'environ 11,6
      (mesuré par la relecture de #150), le centre, qui ne reçoit que l'unité impaire,
      peut tomber d'une part sous son voisin ; jamais dans le banc (σ de travail de 2 au
      plus). Refus `TooSmall` si un côté de travail est
      plus étroit que le noyau (2 × portée + 1), avant toute allocation.
    - `prepare` : la réduction, puis le flou (flou horizontal à longueur de noyau fixée
      à la compilation de 1 à 17, une boucle générique au-delà ; bords en miroir sans
      répéter le pixel du bord), un seul arrondi final en 1/64 de niveau (`u16`, 16 320
      au plus ; passe verticale en `u32`, 255 × 4 096² tient puisque le noyau somme
      exactement 4 096). Le niveau d'une image est la somme de ses unités hors du masque :
      la somme de chaque ligne, moins celle de ses colonnes masquées.
      - Noyau plié (#159) : le noyau étant symétrique, la passe verticale et la passe
        horizontale à longueur fixée additionnent les deux pixels à même distance du
        centre avant de multiplier par leur poids commun. Chaque somme partielle reste
        sous le total, donc au plus 255 × 4 096² : rien ne déborde, ce que vérifie une
        image blanche de σ = 0 à 2,75 en profil `ci`. Flou seul, un fil, `release` :
        0,59 ms par image contre 0,77 à 960, 1,76 contre 2,50 à 1440. Sur la 092 :
        109,5 s de CPU contre 111,1 (−1,4 %, trois paires A/B).
    - `network` : médiane par le réseau de Batcher coupé à n entrées et élagué aux
      comparaisons qui atteignent le milieu (32 pour 11 échantillons), sur des blocs de
      64 pixels ; test 0-1 exhaustif de 1 à 20 (64 motifs par appel) et proptest contre
      un tri de 1 à 128. Si n est pair, la médiane est la demi-somme des deux du milieu.
      Un réseau par compte de 1 au maximum (#151), paires en `Box<[_]>` : 2,3 Ko à 11
      échantillons, 177 Ko à 61, 8,7 Mo à 301, 566 Mo à 1 801 (60 s au pas de 1), soit
      23 à 30 % de la fenêtre de 1 801 images, mesuré par la relecture de #151.
    - `threshold` : résidu `f32` = image − fond − (niveau − niveau moyen), le gain en
      entiers jusqu'à une seule division. Candidats au-dessus du plancher (12
      niveaux) ; avec au moins 7 échantillons et k > 0, seuil relevé à max(12, 8 × 1,4826
      × MAD des écarts corrigés du gain), pour les seuls candidats. Avec k = 0, le seuil
      relevé vaudrait le plancher : il n'est pas calculé (#151). Un candidat sous le
      masque est écarté avant le seuil relevé, comme `residual[~mask] = 0`.
      - Blocs masqués (#159) : un bloc de 64 pixels tout entier sous le masque est
        sauté avant la copie des échantillons et le réseau. Un fil, `release`, à 960 avec
        la bande `osd` : 2,19 ms par image contre 2,36. Sur l'extrait aux réglages
        `osd` : 14,66 s de CPU contre 14,94 (−1,9 %, trois paires A/B).
      - Tampons (#151) : les lignes du réseau sont sur la pile jusqu'à 64 échantillons
        (`STACKED_SAMPLES`, 8 Ko), sur le tas de chaque fil au-delà ; écarts et gains
        dans des tampons de chaque fil, dimensionnés une fois. Les mêmes lignes sur le
        tas coûtaient 7 % de temps CPU sur la 092 (117 s contre 110 s, mesuré en
        alternance avec `main`) ; par ablation, ni le test du masque, ni les écarts, ni
        la copie des deux valeurs du milieu, ni l'inlining du tri n'en sont la cause, et
        le mécanisme n'est pas isolé. Les deux chemins sont testés (16, 64, 65 et 80
        échantillons).
    - `mask` (#151) : zones OSD en pixels de travail, une liste d'intervalles de colonnes
      fusionnés par ligne (vide sans zone). Marge : la portée du noyau, ⌈4σ⌉ pixels de
      travail (la v0.1.1 : ⌈3σ⌉, `detect.py:171`), si bien que les unités préparées sous
      la bande ne dépendent pas de l'écran, au bit près. `WholeFrame` si aucun pixel ne
      reste (`detection::Error::Mask`), comme `detect.py:172`. Le niveau moyen exclut le
      masque : une bande qui scintille ne change pas le gain. Sur le banc, la marge de
      4σ ne perd aucune détection (`osd` et `width_1440_osd`).
    - `closing` : fermeture par le disque de rayon `round(merge_radius / échelle)`,
      moitié vers le haut (#150 ; la v0.1.1 arrondit au pair, un rayon d'écart quand ce
      quotient vaut x,5, par exemple 3,75 à l'échelle 1,5 ; 4 aux défauts, demi-largeurs `[0, 3, 3, 4, 4, 4, 3, 3,
      0]`), demi-largeur de chaque ligne `round(√(r² − dy²))`, comme l'ellipse
      d'OpenCV ; `sqrt` est exact par IEEE 754. Rayon 0 : la fermeture rend les pixels
      allumés. Le disque ne garde que les lignes et les demi-largeurs qui peuvent tomber
      dans l'image, ce qui borne sa taille sans rien changer. Creuse : seuls les
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
      la composante fermée ; surface = pixels allumés × échelle² ; amplitude = plus
      grand |résidu| en niveaux de gris ; filtre `min_area` ≤ surface ≤ `max_area`.
      `components` rend des `Measures`, sans image : le détecteur les range dans
      `FrameDetections` par `push`, dans l'ordre des images. Ce qui garde cet ordre est le
      test des fils et des lots (`detector`, 1, 3 ou 4 fils, lots de 1 à 64).
    - `window` : fenêtre glissante, tampons recyclés ; échantillons pris une image sur
      3 depuis le début de la fenêtre, coupée aux bords de la vidéo.
    - `detector` : lots de 32 images (copie du plan Y). Chaque lot est préparé, puis
      les cibles prêtes (demi-fenêtre suivante lue, ou fin) sont détectées, en groupes
      contigus zippés avec une réserve de tampons (un par fil `rayon`), rangés dans
      l'ordre. La lecture anticipée passe à 36 images pour la détection, `read` en
      garde 4.
    - `background` : le fond de l'image résumé (#104). Les 25 images aux rangs
      `(2i + 1) × N / 50` (N annoncé ; toutes si N < 25) sont gardées en YUV entier
      pendant la lecture. Rang exact, sans le positionnement imprécis d'OpenCV. Un rang
      perdu en fin de vidéo donne moins d'images, comme la v0.1.1.
      - À la fin de la lecture, les images qui tombent dans une plage abîmée sont
        écartées, sauf s'il n'en reste aucune (décision de l'utilisateur, 12 octobre
        2026). Leurs pixels masqués par FFmpeg dépendent du nombre de fils (règle 16), et
        elles sont la majorité sur 121 à 124 : 15 à 21 rangs sur 25 sur les plages v0.1.1.
        En Rust, 16 écartées sur la 122 (9 gardées), 21 sur la 124 (4 gardées).
      - Médiane plan par plan, échantillon par échantillon, par sélection
        (`select_nth_unstable`) sur un tampon réutilisé. Nombre pair : demi-somme des deux
        valeurs du milieu, arrondie au plus proche (règle 5, sans imiter
        `np.median().round()`). 0,29 s sur la 092 (profil `ci`).
      - `Detected::background()` rend l'image médiane, ou rien si aucune n'a été gardée ;
        les 25 images gardées sont libérées à la fin de la lecture.
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
      le résidu filtré), `test_median.py` et `test_noise.py`. Portés par #150 et #151 :
      sigma nul, les 5 tests d'écran (tache sous la bande jamais vue, zone arrondie
      vers l'extérieur, zones couvrant toute l'image refusées, aucun masque par défaut,
      chiffres de l'écran sans fuite sous la bande, ce dernier au bit près sur les
      unités préparées), facteur de bruit nul (le seuil reste au plancher ; le
      raccourci lui-même ne s'observe que par le temps) et configuration invalide. Le
      saut de luminosité à seuil fixe de la v0.1.1 n'a pas d'équivalent aux réglages par
      défaut : un saut de 30 niveaux allume l'image au bord du saut, ce que la stabilité
      retire (tests de `compare`, #119).
    - Règle 16 : même sortie au bit près avec 1, 3 ou 4 fils et des lots de 1 à 64
      (test rapide) ; canari de l'extrait coupé (`slow`), 1 fil de décodage et 1 fil
      `rayon` contre l'automatique, mêmes plages et mêmes détections hors plages. Sur
      l'extrait, la 092 et les 26 vidéos, le parallèle redonne le séquentiel au bit près
      hors plages abîmées ; dans les plages, les pixels dissimulés varient d'une lecture
      à l'autre.
    - Mesures (outil `compare` et ses tests `slow_extract.rs`, `reference_agreement.rs`,
      devenu `cli/tests/reference_092.rs` à #184,
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
    - Temps de la 092 après #150 (M1 Max, `release`, `compare 092`, en alternance avec
      `main` compilé de même, 12 octobre 2026, sous le verrou mais avec d'autres sessions
      actives) : 110,0 s de CPU contre 102,4 s pour `main`, soit +7,5 % ; temps réels de
      15,7 à 19,5 s contre 15,2 à 20,9 s, trop variables ce jour-là pour conclure. La
      première réduction générique coûtait +17,6 % ; la passe horizontale par blocs à
      nombre de poids fixe et l'arrondi par l'inverse l'ont ramenée là. Mémoire
      résidente : 452 Mo contre 437 Mo ; le gris de la source, 1,5 Mo par fil, en
      explique environ 15 à 10 fils (attendu, non isolé).
    - Temps de la 092 après #159 (M1 Max, `release`, `compare 092`, 13 octobre 2026, sous
      le verrou, machine calme, en alternance) : `main` avant #159, 110,7 s de CPU
      (moyenne de 6 passages, de 109,9 à 111,6), contre 102,6 s pour le moteur 3:2 de
      `main` juste avant #150 (3 passages, de 102,4 à 102,8), soit +7,9 %, proche des
      +7,5 % de #150, #151 et #100 compris. Après #159 : 109,1 s (3 passages, de 108,8
      à 109,3), soit +6,3 % sur le 3:2. Le gain vient du noyau plié, les deux autres
      pistes gardées ne jouant qu'en 1:1 ou sous un masque. Temps réels de 12,0 à 15,1 s,
      trop variables pour départager.
    - Mémoire (`tests/slow_memory_flash.rs`, `dhat`, 4 fils, 64 images) : un éclair plein
      cadre dans chaque groupe de cibles, au premier lot, coûte 57,8 Mo de plus qu'une
      vidéo calme, soit 20,9 octets par pixel de travail et par fil. Les deux pics
      tombent dans la même phase de lecture, d'où une mesure stable. Arithmétique :
      les pixels allumés (8 octets), touchés et fermés (4 octets chacun) d'un fil
      montent jusqu'à 2²⁰ éléments, 24,3 octets par pixel, gardés jusqu'à la fin
      (environ 170 Mo à 10 fils). Bornes du test : plus de 16, au plus 32. Les
      allocations de FFmpeg ne sont pas vues, les mêmes dans les deux vidéos.
      `WORK_PIXELS` vient de `WorkScale` et de la configuration (#150). Second cas,
      `tests/slow_memory_flash_wide.rs`, à 1440 pixels de travail (1 440 × 1 080) :
      134,2 Mo de plus, 21,58 octets par pixel et par fil (20,90 à 960, inchangé).
      Troisième cas (#151), `tests/slow_memory_flash_long.rs`, 2 s au pas de 1, soit
      61 échantillons : 53,0 Mo de plus, 19,18 octets par pixel et par fil. Les éclairs
      y tombent dans le dernier vidage (images 35, 43, 51 et 59), le seul où les 4 fils
      ont chacun une cible éclairée à cette demi-fenêtre de 30. Le pic de la vidéo
      calme, 245 Mo, porte la fenêtre (2 octets par pixel et par image gardée) ; sur
      64 images, il ne change pas avec la fenêtre.
  - **Gris de la détection** (#118, exception à la règle 5) : chaque Y passe par une
    table de 256 gris mesurée sur OpenCV (`GRAY` dans `gray`, privée, lue par
    `gray::gray(y)`, `const fn` publique depuis #107 : une rampe Y = 0 à
    255, U = V = 128, encodée sans perte en plage `tv`, lue par `cv2.VideoCapture`
    puis `COLOR_BGR2GRAY`), puis la réduction
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
- **`cli`** → `engine`, `tracking`, `files`, `summary`, `render`, `draw`, `media`, `types` : paquet `batdetect`, binaire
  `batdetect` (#123, `batdetect-rs` jusqu'à #199). Tout texte vu par l'utilisateur vit dans les deux catalogues
  Fluent `locales/{fr,en}.ftl`, lus par `texts.rs` (#114) ; `messages.rs` fait les messages
  à partir des vidéos et des erreurs, `arguments.rs` la ligne de commande. La bibliothèque (`failure`, `jobs`, `language`, `options`, `process`, `progress`, `settings`, `tally`, `writable`)
  sert aux tests.
  - `batdetect VIDÉO|DOSSIER… [-o DOSSIER]`. Sans `-o`/`--out-dir`, les résultats de
    chaque vidéo vont dans `<dossier de la vidéo>/batdetect/` (#86) ; l'aide le dit
    (« [par défaut : batdetect/ à côté de chaque vidéo] »). `--output` n'existe plus.
    Les anciennes sorties de la v0.1.1 (`out/`, relatif au dossier de lancement) restent où
    elles sont, sans migration.
  - Autres commandes (#58, #59) : `commands::other_command` (binaire) lit le premier argument avant `arguments::parse` ; `doctor` et `completion` partent vers leur module, tout le reste vers le parseur, sans changer `batdetect <vidéos>`. Une vidéo ou un dossier nommé ainsi se donne en `./doctor` ou `./completion`, ce que dit l'aide (`other-commands`, sous les options de `-h` et de `--help`).
    - `batdetect completion zsh` écrit sur la sortie standard le script `_batdetect` que produit `clap_complete` depuis le parseur complet (`arguments::full_command`, `Level::Full`), dans la langue du système. La ligne du positionnel est réécrite (`completion::inputs_spec`) : répétée (`*:`), elle appelle `_batdetect_inputs`, ajoutée au script (`completion::inputs_function`), qui propose `doctor` et `completion` avec leur description au premier mot seulement (`CURRENT == 2`), puis les vidéos par `_files -g "[^.]*.(#i)(mp4|mov|avi|mkv|m4v)"`, aux extensions de `jobs::video_extensions()`, et les dossiers, sans fichiers cachés ; `-o` complète des dossiers (`ValueHint::DirPath`, `_files -/`), `--zoom` ses trois valeurs. Quand la sortie standard est un terminal, la sortie d'erreur donne les lignes de `~/.zshrc`, avec et sans Homebrew ; la commande n'écrit jamais dans le profil. Un autre shell est refusé (code 2).
    - `batdetect doctor`, sans argument, vérifie le dossier courant (`checkup::checkup`, bibliothèque) et écrit sur la sortie standard une ligne par point (`doctor::report_of`) : version de batdetect et de FFmpeg ; essai réel de l'encodeur, le même que le traitement (`Fallback::for_this_platform().trial`, 1440 × 1080 à 30 i/s) ; place libre du volume (`disk::volume`, la même lecture que le contrôle de place au lancement : `statfs` sous macOS, `statvfs` sous Linux, depuis le plus proche dossier existant ; non mesurée sous Windows) ; droit d'écrire (`writable::writing_refused`, la sonde du contrôle d'écriture). ✗ et un « à faire » pour un encodeur absent (`NoEncoder` : réinstaller), un essai impossible faute de dossier temporaire (`Scratch`, `Destination` : place sur le disque du système, pas une réinstallation), moins de 20 Go libres (`checkup::ENOUGH_FREE_BYTES`, affichés arrondis par défaut au dixième) ou l'écriture refusée : code 1. ⚠ pour openh264 quand VideoToolbox ne répond pas, ou une place non mesurée : rien d'obligatoire, code 0.
  - Usages courants en tête de l'aide (#56), au format de `tldr` : « - description : », puis
    la commande en retrait. Six exemples : un dossier de soirée, plusieurs soirées, une
    vidéo, `-o`, `--zoom all`, `--annotated`. Le nom de la commande est
    `env!("CARGO_BIN_NAME")` (`arguments::PROGRAM`), si bien que le renommage de #199 n'a
    touché qu'un endroit du code livré. Textes dans `texts.rs` (`common_uses`, `all_options`,
    `files_produced`) et les `.ftl`, sans module à part.
    - Sans aucun argument : la phrase d'en-tête, les usages et « Toutes les options et les
      fichiers produits : batdetect --help », sur la sortie standard, code 0.
      `arg_required_else_help` est retiré. `-o` sans vidéo reste une erreur d'usage (code 2).
    - `-h` : la phrase, les usages (`before_help`), l'essentiel, et le même pied
      (`after_help`).
    - `--help` : la phrase, les usages, tout, puis « Fichiers produits » et « Mode d'emploi
      complet : https://github.com/zaratan/chiro-dvr#readme ».
      - Les noms des fichiers sont ceux de `files::names`, dans la langue de l'aide :
        `Names::numbered_clip` donne un nom d'extrait sans `Track`, `period_suffix` et
        `Names::part_suffix` les suffixes de l'image résumé, et `RESULTS_FOLDER`
        (`batdetect`, déplacé de `jobs`) le dossier de résultats. Aucun nom n'est écrit en
        dur dans les textes de l'aide.
    - `style` (binaire) porte `STYLES` et `HELP`, que `arguments`, `errors` et `texts`
      importent : plus de cycle entre ces modules.
      - Chaque nom est sur sa ligne, en jaune, et son rôle en retrait dessous : clap replie
        `before_help` et `after_help` à la largeur du terminal, et deux colonnes s'y
        casseraient.
    - Commandes des exemples en cyan gras, titres en bleu gras, comme le reste de l'aide.
  - Options décidées le 10 octobre (#113), en builder `clap`, deux niveaux d'aide :
    - `-h` : l'essentiel, `VIDÉO…`, `-o/--out-dir`, `--zoom`, `-h`, `--version` ;
    - `--help` : tout, avec l'utile (`--annotated`, `--max-tracks`, `--clip-margin`,
      `--osd-region` répétable) et, sous « Settings that change the detection » (« Réglages
      qui changent la détection »),
      `--threshold`, `--min-hits`, `--max-blobs` (0 coupe le filtre), `--work-width`.
    - Apparence, celle de l'aide d'argparse 3.14 (demande de l'utilisateur) : une ligne
      par option, le texte en colonne, coupé à la largeur du terminal (feature `wrap_help`,
      qui ajoute `terminal_size` ; cargo deny l'accepte). clap passe toujours le texte à la
      ligne suivante en aide longue : `-h` et `--help` sont donc tous deux l'aide courte de
      clap (`ArgAction::HelpShort`), et le niveau se choisit avant la lecture (`--help`
      présent avant `--` : tout est visible ; sinon l'utile et l'avancé sont cachés).
    - Couleurs (`Styles`) : titres et « Usage » en bleu gras, options en cyan gras, noms de
      valeurs en jaune, valeurs des défauts et valeurs possibles en vert, `error:` en rouge
      gras. clap les coupe hors terminal et avec `NO_COLOR`, les force avec
      `CLICOLOR_FORCE` ; aide et erreurs passent par `clap::Error::print`. Limites de clap
      acceptées : `-h` et `--help` de la même couleur, nom du programme en cyan.
    - Défauts jamais coupés, sur leur propre ligne (choix de l'utilisateur) : clap coupe ses
      lignes sur l'espace ASCII et à chaque changement de couleur, même sans espace
      (`StyledStr::wrap` passe chaque segment de couleur à `LineWrapper::wrap`, qui peut
      couper avant son premier mot). Son « [default: 20] », en trois segments, laissait donc
      parfois « ] » seul sur la ligne suivante. La commande écrit elle-même le défaut
      (`hide_default_value`), après un retour à la ligne : « [default: » et « ] » en texte
      normal, espace insécable, la valeur en vert. clap ne coupe jamais le premier mot d'une
      ligne, et la ligne est courte. Les valeurs possibles de `--zoom` suivent sur leur
      ligne, de la même forme.
    - La colonne du texte se cale par titre : clap calcule sa largeur pour chaque section
      séparément, sans réglage pour la partager entre « Options » et les réglages avancés.
    - Sous environ 95 colonnes, clap repasse le texte de toutes les options d'une section à
      la ligne suivante dès qu'une seule ne tient plus (règle des 40 % de `help_template`).
      Les tests rendent l'aide à 100 colonnes fixes.
    - `-V/--version` est une option de la commande, au ton des autres (« print the
      version »), comme `-h/--help`. Les deux écrivent les mêmes deux lignes, version
      puis FFmpeg et sa licence (#196) : la commande n'a qu'une `version`, la longue.
    - Le gabarit de la commande (`help_template`, #114) reprend celui de clap avec notre
      titre d'usage, et garde `{before-help}` et `{after-help}` : les exemples de #56 s'y
      logeront sans changer la forme des options.
    - Les défauts viennent des configurations, et l'aide les affiche ; les textes sont ceux de
      la v0.1.1, dans les deux langues (#114). `--osd-region` a aussi son défaut sur sa ligne
      (« [default: none] »). Les autres réglages restent au banc.
    - `options::Options` porte les quatre configurations (`DetectConfig`, `TrackConfig`,
      `StabilityConfig`, `RenderConfig`) et `annotated` ; `process` et `Settings::of` le
      reçoivent par référence.
    - Une valeur refusée (type, borne d'un constructeur, zone mal formée) est une erreur
      d'usage, code 2, qui nomme l'option et sa plage, sans ligne d'usage
      (`--threshold must be a finite number > 0, not -1`). Une valeur illisible
      (`--max-tracks many`) dit la même phrase avec la valeur telle qu'écrite (#55) :
      `parsing::Reader` lit chaque option et porte l'option, la plage et la valeur dans
      `parsing::Unreadable`, que `errors.rs` traduit. Les entiers ne disent que leur borne
      basse (`un entier ≥ 0`), pas le maximum du type. Une virgule décimale, sur
      `--threshold` et `--clip-margin` seulement, reçoit le conseil « écrire 11.5, avec un
      point ». Une valeur vide dit « une valeur est attendue ». `--osd-region` accepte une
      valeur qui commence par un tiret (`allow_hyphen_values`), refusée ensuite par la
      phrase de la zone.
    - Les refus que la ligne de commande ne peut pas atteindre (`TrackConfig`,
      `StabilityConfig`, une option sans valeur par défaut) nomment l'option et demandent
      de le signaler ; ceux d'un réglage interne (`min_area` contre `max_area`, `trail_s`)
      disent « les réglages par défaut sont refusés ». `render::config::Error::Seconds`
      porte le réglage refusé (`Seconds::ClipMargin`), et non son nom en texte.
    - `--osd-region` se lit par `FromStr for engine::settings::Region` (`x0,y0,x1,y1`), la
      même lecture que le banc ; `Display` l'écrit de la même façon.
    - Le rappel des réglages avancés modifiés ira dans le bilan (#65) ; `params.json` les
      enregistre déjà. `--force` part à #70.
  - Un dossier donne ses vidéos (`mp4`, `mov`, `avi`, `mkv`, `m4v`), par ordre de nom, au
    premier niveau seulement : un `batdetect/` voisin n'est pas repris.
    - Un nom qui commence par « . » est écarté (#204), comme la page écarte les dossiers
      cachés : ce sont surtout les `._x.mp4` (AppleDouble) que macOS dépose à côté de
      chaque fichier sur un volume exFAT ou FAT. Un dossier qui n'en contient que devient
      un dossier sans vidéo.
    - Une vidéo nommée, cachée ou non, reste jugée sur sa seule extension.
  - Refus en bloc avant tout calcul (`jobs::plan`, #55). Toutes les entrées sont relevées
    avant la première détection, et une seule fautive arrête tout le lot (code 2, aucun
    dossier créé). Chaque refus est une ligne « chemin : raison », sous « aucune vidéo
    n'est traitée : », sans ligne d'usage ni renvoi à l'aide. Sous chaque refus vient, en
    retrait, sa ligne « à faire » (#239), comme sous un échec du bilan. Ses textes sont des
    clés du même message `failure-todo`. Une lecture refusée reprend la clé de l'échec
    équivalent, `read-refused`. Les autres ont leur clé : `input-missing` (introuvable, ou
    un élément du chemin qui n'est pas un dossier : « le dossier ou la vidéo a peut-être
    été déplacé ou renommé », quand l'échec `missing` ne parle que de la vidéo),
    `unreachable` (inaccessible), `not-a-video-name`, `no-video`, `shared-folder`. Les raisons, typées par
    `jobs::Reason` :
    - chemin absent ;
    - fichier nommé dont l'extension n'est pas une des cinq ;
    - dossier sans vidéo, parmi d'autres entrées ;
    - chemin ou dossier illisible, la raison étant prise dans la table fermée
      `access::Access` (refusé, pas un dossier, introuvable, lecture seule, disque plein,
      sinon inaccessible) ;
    - plusieurs vidéos vers le même dossier de résultats : une ligne par dossier, qui
      nomme toutes ses vidéos, après les refus d'entrées puisque ce sont des sorties.
    - vidéo qui ne s'ouvre pas (`Reason::Unopened`, #62), vue par l'inventaire, après les précédentes : sa ligne est la phrase de l'échec équivalent et son « à faire » celui de cet échec (`failure::reading_cause`, la classification d'une lecture ratée), suivi du détail anglais quand ce « à faire » y renvoie (`unsupported`). Le bilan reprend les mêmes lignes, détail compris, puisque ce « à faire » y renvoie ; un échec à son tour, lui, y perd son détail.

    Si toutes les entrées sont des dossiers sans vidéo, le refus devient « aucune vidéo
    (mp4, mov, avi, mkv ou m4v) trouvée dans ce dossier : », suivi des dossiers, chacun
    avec sa ligne « à faire ».

    Quand des vidéos valides restent à côté des refus, la commande peut les traiter
    (`consent::Asking`, décision de l'utilisateur du 16 octobre) :
    - Si l'entrée standard est un terminal, la liste des refus passe sous « 2 entrées sont
      refusées : », puis la sortie d'erreur pose la question « Traiter quand même les 3
      vidéos valides ? (O/n) » (« (Y/n) » en anglais).
      - Entrée, o, O, y ou Y valent oui, dans les deux langues ; n ou N valent non.
      - Toute autre réponse fait reposer la question.
      - La fin de l'entrée ou une erreur de lecture vaut non.
      - Oui : les vidéos valides sont traitées et le code de sortie vaut 1. Non : « aucune
        vidéo n'est traitée », code 2.
    - `--skip-refused` vaut un oui sans question, avec ou sans terminal.
    - Sans terminal et sans l'option (script, redirection), le refus en bloc reste.
    - Si aucune entrée n'est valide, le refus en bloc reste, avec ou sans l'option.
    - Le bilan, toujours sur la sortie standard, compte « 2 entrées refusées » dans son
      titre et les liste sous « Entrées refusées, non traitées : », chacune avec sa ligne
      « à faire ».
    - Le chemin avec question se teste dans `consent`, sur une entrée fabriquée. Un test
      de commande ne peut pas fabriquer de terminal sans pseudo-terminal.
    Les causes d'une lecture ratée restent pour un fichier qui disparaît ou change entre le départ et son tour.
  - Inventaire au départ (`inventory::take_stock`, #62), après `jobs::plan` et avant la question des refus.
    - La reprise se décide là, une fois pour toutes (`reuse::earlier`, sauf `--force`), et non plus dans la boucle : une vidéo déjà faite n'est pas ouverte, sa durée est celle de son `params.json`.
    - Chaque autre vidéo est ouverte par `engine::input::Input::open`, en un seul fil, puis refermée aussitôt : sa durée est le nombre d'images annoncé divisé par la cadence.
    - Une vidéo qui ne s'ouvre pas rejoint les refus d'entrées, avec la même question, le même `--skip-refused` et le même code 2.
    - Coût, mesuré le 16 octobre 2026 sur lumiel (binaire release, fichiers en cache, `-o` vers un dossier en lecture seule pour s'arrêter juste après l'inventaire) : 0,39 à 0,44 s pour les 26 vidéos de `BATDETECT_VIDEOS_DIR`, dont 0,39 s de CPU, pic de mémoire 15,6 Mo.
    - Une vidéo fausse, tronquée ou au flux effacé échoue toujours à l'ouverture (essai sur la vidéo fabriquée) : un échec à son tour vient désormais surtout de l'écriture. Les tests de commande le provoquent par un fichier ordinaire posé à la place de `<racine>/<vidéo>/`, qui fait échouer la vidéo sur tous les systèmes, Windows et root compris (« erreur inattendue »).
  - Une vidéo donnée deux fois, sous n'importe quelle écriture (son dossier et son nom,
    `./`, un lien), n'est traitée qu'une fois, sous sa première écriture (#194). La clé est
    son chemin réel (`fs::canonicalize`), ou, pour un fichier absent, le dossier réel suivi
    du nom, si bien qu'un chemin introuvable donné deux fois n'est refusé qu'une fois.
  - Dossiers de sortie (`jobs::plan_jobs`, #86, #91). Les racines et les homonymes se
    comparent sur le dossier réel (#194). Une racine écrite de deux façons reprend sa
    première écriture, si bien que le bilan n'a qu'un titre par dossier. La racine d'une vidéo est `-o`, ou
    à défaut `<dossier de la vidéo>/batdetect/`. Chaque vidéo a son dossier, à son nom,
    dans sa racine.
    - Les homonymes se comptent par racine : sans `-o`, `nuit1/v.mp4` et `nuit2/v.mp4`
      donnent `nuit1/batdetect/v` et `nuit2/batdetect/v`.
    - Deux homonymes dans une même racine prennent `<dossier parent>_<nom>_<extension>`
      avec `-o`, et `<nom>_<extension>` sans `-o`, puisque le dossier est déjà celui de la
      vidéo.
    - Homonymes à la casse près (#91), sur tous les systèmes. La clé est le nom en NFC
      (`files::names::composed`, celle du ménage), puis `to_lowercase`. Un nom non UTF-8
      garde ses octets pour la clé, comme dans le ménage : par `to_string_lossy`, `\xff` et
      `\xfe` auraient la même clé et seraient refusés comme un seul dossier.
      - Avec `-o`, `a/Vol.mp4` et `b/vol.mp4` donnent `a_Vol_mp4` et `b_vol_mp4`. Sans
        cette clé, ils partageaient un dossier sur APFS ou NTFS, et le ménage de la seconde
        effaçait les extraits de la première.
      - Sous Linux, la règle va plus loin que le système de fichiers. Deux homonymes à la
        casse près reçoivent deux noms qui n'étaient pas nécessaires. Deux dossiers
        `Nuit/x.mp4` et `nuit/x.mp4` sont refusés comme un seul (code 2), avec ou sans `-o`.
    - Deux vidéos qui écriraient au même endroit, à la même clé près, sont refusées avant
      tout calcul (code 2), avec les autres entrées fautives (#55).
  - Contrôle d'écriture (`writable::unwritable_folders`, #86), avant tout calcul, pour
    chaque racine, celle de `-o` comprise. `create_dir_all` ne suffit pas : il réussit sur
    un dossier existant en lecture seule.
    - Une sonde est créée de façon exclusive (`.batdetect-probe-…`, par `tempfile`), puis
      retirée, dans le plus proche dossier existant de la racine (`.` pour une racine
      relative qui n'existe pas encore). Rien n'est créé avant la détection : une vidéo
      au chemin mal tapé ne laisse pas de `batdetect/` vide.
    - Un `batdetect/<vidéo>/` existant mais en lecture seule passe la sonde de sa racine,
      et sa vidéo échoue seule, après la détection.
    - Si une racine refuse, aucune vidéo n'est traitée (code 2) : « aucune vidéo n'est
      traitée, les résultats ne peuvent pas s'écrire dans ce dossier : », puis une ligne
      « dossier : raison » par dossier, sans ligne d'usage ni renvoi à l'aide (#55). La
      raison vient de la table fermée `access::Access` : écriture refusée, pas un dossier,
      introuvable, disque en lecture seule, disque plein, sinon inaccessible.
    - Chaque dossier refusé a sa ligne « à faire » (#82), comme une entrée refusée.
      Écriture refusée et disque plein reprennent les clés de l'échec (`forbidden`,
      `disk-full`). Disque en lecture seule : « copier les vidéos sur le Mac, ou choisir un
      autre dossier avec -o » (`read-only`, décision du 9 octobre). Pas un dossier ou
      introuvable : « choisir un autre dossier avec -o » (`other-folder`). Inaccessible :
      `unreachable`, comme pour une entrée.
    - Un `-o` qui est un fichier est refusé de même, avant la détection.
  - Contrôle de place au lancement (`space`, `disk`, `admission`, #61), après l'inventaire, sauf `--ignore-disk`.
    - Chaque vidéo que la reprise ne saute pas compte sa taille multipliée par le coefficient des sorties, et par celui de la vidéo annotée en plus sous `--annotated` (`space::launch_estimate` ; coefficients et mesure en docs/02, § Place prise).
    - Les besoins s'additionnent par volume du dossier de résultats (`-o`, ou le `batdetect/` voisin), pas par volume des vidéos : un disque externe pour les vidéos et le Mac pour les résultats se comptent à part. Le volume se reconnaît à `MetadataExt::dev()` du plus proche dossier existant. Sous btrfs, chaque sous-volume a son propre `dev()` : deux dossiers d'un même système de fichiers s'y comptent chacun contre toute la place libre, ce qui sous-estime le besoin (cas rare, sans effet sur le Mac).
    - La place libre est `f_bavail × f_bsize` de `rustix::fs::statfs` sous macOS (compteurs de blocs sur 64 bits, quand ceux de `statvfs` n'en ont que 32), et `f_bavail × f_frsize` de `rustix::fs::statvfs` sous Linux, exFAT monté sous `/Volumes/…` compris. Sous APFS, elle ne compte pas l'espace purgeable, que le Finder ajoute : le contrôle est donc plus prudent que le Finder. Sous Windows, la place n'est pas lue et rien n'est refusé.
    - Un volume est refusé si l'estimation plus la marge (10 % de l'estimation, au moins 1 Go, décision du 9 octobre) dépasse sa place libre. La ligne de refus nomme le premier dossier de résultats de ce volume, un chemin que la naturaliste reconnaît : « le disque qui contient … : 12,3 Go à prévoir, 4,1 Go libres », Go décimaux, à prévoir arrondi au-dessus et libre au-dessous. Son « à faire » cite `--annotated` seulement quand il est donné (`disk-short`, `disk-short-annotated`).
    - Le refus suit le circuit des refus d'entrées : liste, question dans un terminal, `--skip-refused` vaut un oui, code 2 sinon (décision de l'utilisateur du 16 octobre). Avant la question, il a son bloc à lui, « le disque des résultats semble trop petit : », après celui des entrées refusées ; hors terminal, il reste sous « aucune vidéo n'est traitée : » avec elles. Un oui lance tout le lot (`Stock::go_ahead_despite_the_disks`), et le refus de place ne compte pas parmi les entrées refusées du bilan, ni dans `.batdetect/refus.json` et sur la page (#267) : seul un lot lancé écrit ce fichier, et il ne l'est malgré un disque trop petit qu'après ce oui, qui retire le refus.
    - La sonde d'écriture (`writable::unwritable_folders`) passe dans `admission`, après l'inventaire et avant le contrôle de place : un dossier de résultats où rien ne s'écrit n'est refusé qu'une fois, pour l'écriture.
  - Contrôle de place par vidéo (`process::room_for_clips`, #61), après le suivi et l'image résumé, juste avant les extraits, même sous `--ignore-disk`.
    - L'estimation est presque exacte : secondes des fenêtres d'extrait (piste et marges, `render::windows::clip_windows`) et de zoom (pistes que `ZoomConfig::wants` retient), multipliées par le débit de la vidéo d'entrée et par leur coefficient, plus la vidéo annotée (`space::render_estimate`). Au-delà de `--max-tracks`, rien n'est compté, puisque rien n'est rendu.
    - Si elle dépasse la place libre du volume du dossier de la vidéo, sans marge, la vidéo échoue avec `Error::DiskSpace`, classée « disque plein » : son « à faire » dit de libérer de la place sur le disque du dossier. `params.json` n'est pas écrit, la vidéo sera refaite à la relance, et la suivante est tentée.
    - Les tests des deux contrôles prennent une vraie vidéo allongée par `set_len` à 10 To, en fichier épars : FFmpeg la lit comme l'originale, et son estimation dépasse tout disque, sans couture dans le code. Ils ne tournent que sous Unix.
  - Bilan de fin de soirée (#65), écrit une fois après `run_all`, sur la sortie standard.
    - Modèle sans texte dans la bibliothèque : `tally::Tally` (les `Row` de `run_all`),
      avec `roots()` (les dossiers de résultats dans l'ordre où ils apparaissent, et
      seulement ceux qui existent, #196 : une racine qui n'a que des vidéos en échec n'a
      de titre que si son dossier est sur le disque ; ses vidéos restent sous « vidéos en
      échec »),
      `rows_in`, `done`, `too_many`, `failed`, `kept` et `all_done`. Il servira à #69 et
      #71. Mise en forme dans le binaire : `board::write`.
    - Page de résultats (#71), à la racine de chaque dossier de résultats.
      - `written::write_page` relit la racine (`files::results`, et le fichier de verdicts par `files::verdicts`, passé tel quel à la page, #228), y ajoute les vidéos du lot
        pas encore traitées (« en attente »), ouvre celles du lot, et écrit `resultats.html`
        ou `results.html` par `.partial` puis renommage ; il retire la page de l'autre langue.
        Il n'écrit ni ne retire une page qui n'a pas la marque de `page`, et ne crée jamais la racine. `main` l'appelle avant le lot puis dans le rappel de fin de chaque vidéo ;
        une erreur va dans `Tally::unwritten`, dite à la fin du bilan (une page étrangère du
        même nom avec ce qu'il faut faire, sinon le chemin et le détail), et rend le code 1
        sans arrêter le lot. Au Ctrl-C, ou si la boucle du lot s'arrête sur une erreur, chaque page est réécrite avant la sortie avec `Run::Stopped`, à la position de la vidéo en cours (`Status::position`) : ce que le disque garde l'emporte ; sinon la vidéo en cours est « inachevée » et les suivantes du lot « pas encore traitée » (#247). Avant les pages, `written::create_reached_root` crée la racine de la vidéo en cours, et d'aucune autre, pour qu'un Ctrl-C pendant la première vidéo laisse sa page ; puis le `Ledger`, partagé avec le gestionnaire du Ctrl-C, écrit les échecs qui attendaient cette racine, et les refus s'y écrivent.
      - Les clés des vidéos sont les noms de dossier composés en NFC : un dossier créé en NFD
        et la même vidéo tapée en NFC font une seule entrée.
      - Un nom de dossier non UTF-8 garde ses octets pour la clé, comme dans le ménage et dans `jobs` (#227) : `vol_\xe9` et `vol_\xe8`, tous deux affichés `vol_�`, font deux entrées.
      - `ledger::Ledger::note`, dans le même rappel et avant la page : une vidéo en échec
        écrit `.batdetect/echecs/<dossier>.json` à la racine (clé stable de `Failure` :
        `Cause::key` ou `internal` pour un panic, place, détail, date UTC), une réussite ou
        « trop de pistes » l'efface. Un échec dont la racine n'existe pas encore attend
        qu'une autre vidéo du lot la crée ; sinon rien n'est créé pour lui.
      - Le détail technique n'est montré que pour les causes dont le conseil y renvoie
        (`failure::REFERS_TO_DETAIL`).
      - `refusals::Refusals::note` (#267), avant les pages et dans le même rappel : les entrées refusées au départ (`jobs::Refused`, de `plan` puis de `inventory`) s'écrivent dans `.batdetect/refus.json` de chaque racine du lancement, dès qu'elle existe, une fois par racine ; un lancement sans refus efface ce fichier. Chaque entrée garde son chemin, la clé de sa raison (`Reason::key`), celle de son conseil (`Reason::todo`), la cause d'une vidéo qui ne s'ouvre pas et son détail quand le conseil y renvoie (`failure::REFERS_TO_DETAIL`), les vidéos d'un dossier de résultats commun. Une interruption (Ctrl-C) ou un arrêt de la boucle les écrit aussi avant les pages. Ces deux clés sont la seule table entre raison et texte, partagée par le terminal (`messages`) et la page (`sheet`) ; le texte de la raison est encore produit deux fois, par `messages::input_refused` pour le terminal et `sheet::refused_input` pour la page : un test de `sheet` vérifie, pour chaque raison, chaque accès et chaque cause d'ouverture, dans les deux langues, que la page dit les mêmes lignes que le terminal, détail compris. La page les montre dans une carte avant les vidéos, liée depuis le total.
    - `notify` (#69) ouvre, après le bilan, la page de chaque racine dans la langue du
      lancement (`written::page_to_open`), et rien pour une racine sans page à nous.
    - Titre : « Bilan : 12 vidéos, 10 réussies, 1 avec trop de pistes, 1 en échec » (les
      états absents ne sont pas nommés). Puis un tableau par dossier de résultats (#86),
      les vidéos dans l'ordre du lot : vidéo, durée, pistes, non analysé, calcul.
      - La vidéo est nommée par son dossier de résultats, qui porte déjà la levée
        d'ambiguïté de #86 et #91.
      - Le nom est à gauche, les nombres et les durées à droite. Les largeurs se comptent
        en `char`, après composition NFC, sans crate. Une cellule inconnue vaut « — ».
      - Pistes : le nombre par le formateur de #114, « échec », ou « 412, trop ».
      - Durées en « 1 h 12 min 3 s » : la durée et le calcul sont tronqués à la seconde, le
        non-analysé arrondi au-dessus. `summary::header::duration` n'a pas d'heures et
        reste celui de l'image résumé.
    - Mesures : `process::extent(&Timeline, &Exclusion) -> Extent`, une fonction pure.
      - `length_s` va de la première image à la fin de la dernière, sans le premier `pts`.
      - `not_analysed_s` vaut `Exclusion::ignored_frame_count` (l'union des plages abîmées
        et instables) en secondes.
      - `Processed` et `Error::TooManyTracks` les portent. `TooManyTracks` porte aussi le
        CSV laissé : il naît dans `render_clips`, après le CSV.
    - Sous le tableau, des sections omises quand elles sont vides, dont :
      - « Trop de pistes (tableau et image résumé écrits, pas d'extraits) », avec le « à
        faire » de #64 ;
      - « 1 vidéo sur 12 en échec », avec la raison et le « à faire » de #64, sans le
        détail anglais, qui reste sur la sortie d'erreur. Le « à faire » qui renvoie au
        détail ci-dessous devient, dans le bilan, « avec le détail écrit pendant le
        traitement » (`tally-failure-todo`) ;
      - « Tableau laissé tel quel » (#115), pour une entrée au nom du tableau qui n'est
        pas un fichier, ou pour un fichier qui occupe encore le nom (autre casse) ;
      - « Réglages changés : --threshold 11, --max-blobs 0 ». Ce sont les cinq réglages de
        la détection (`--threshold`, `--min-hits`, `--max-blobs`, `--work-width`,
        `--osd-region` avec son nombre de zones), donnés par `Options::changed`, avec
        leur valeur telle qu'elle se tape.
    - Le rappel des réglages n'existait pas avant #65 : #113 n'écrit pas de ligne de départ.
    - Codes de sortie inchangés : 1 si une vidéo a échoué ou a trop de pistes. Un bilan
      qui ne peut pas s'écrire donne aussi 1. Si une écriture au fil de l'eau échoue,
      `run_all` s'arrête et il n'y a pas de bilan.
  - Vidéo et étape en cours (#63), sur la sortie d'erreur. La sortie standard ne porte que
    le bilan de fin de soirée (#65).
    - `progress::Progress` est le protocole d'étapes de la bibliothèque : `process` appelle
      `step` avant la détection (lecture comprise), le suivi (ménage, CSV, essai des
      encodeurs et `params.json` compris), l'image résumé et les extraits (zooms et vidéo
      annotée compris). `progress::Silent` sert aux tests. Pas d'avancement image par
      image : l'heure de fin (#68) se compte d'une vidéo à l'autre.
    - Une écriture ratée de l'étape fait échouer la vidéo avant sa détection
      (`Error::Progress`). Le rapport de cette vidéo échoue ensuite sur la même sortie
      d'erreur, et le lot s'arrête là, en code 1. Avant #63, le lot ne s'arrêtait qu'à
      la première ligne d'événement ou d'échec.
    - Avant le premier rang, une ligne annonce le lot sur la sortie d'erreur (#62) : « 12 vidéos, 1 h 2 min 5 s de vidéo, dont 3 déjà faites », la fin seulement s'il y en a ; la durée s'écrit comme la colonne « durée » du bilan (`clock`), vidéos déjà faites comprises.
    - Le binaire (`status.rs`) écrit « [3/12] video_092 — détection », puis « — suivi »,
      « — image résumé », « — extraits ». La vidéo est nommée sans extension, et son rang
      compte dans le lot.
    - La ligne d'une vidéo finie reste : « [3/12] video_092 — 14 pistes », « — trop de
      pistes » (le mot du bilan, #196) ou « — échec ».
    - Heure de fin estimée de la soirée (#68) : « [3/12] video_094 — détection · fin vers 22:41 » sur un terminal, à chaque étape ; hors terminal, à la fin de la ligne d'une vidéo finie, jamais sur une étape. Rien tant qu'aucune vidéo n'a été calculée, ni quand il ne reste rien à calculer (dernière ligne d'une vidéo finie hors terminal).
      - `eta::Evening`, pur : les secondes passées sur les vidéos calculées depuis le début du lot, divisées par leurs images, s'appliquent aux images restantes. Les images sont celles qu'annonce l'inventaire (#62, `Listed::frames_to_compute`) ; une vidéo que la reprise saute en a 0. Pendant une vidéo, le temps qu'elle a déjà pris se retire de sa part, sans descendre sous 0.
      - Seule une vidéo calculée jusqu'au bout donne du temps (`Status::computed`, après l'écriture de la page) : une vidéo en échec, souvent arrêtée tôt, tirerait l'estimation vers le bas. Le temps se mesure de l'entrée dans la vidéo (`Status::enter`) à ce moment, par `Instant`.
      - L'heure est locale (`jiff::Zoned::now`), arrondie à la minute la plus proche, sans jour : au-delà de 24 h de calcul restant, elle ne dit pas le jour.
      - La coupe à la largeur du terminal prend la fin de la ligne : sur une fenêtre trop étroite pour la ligne entière, l'heure de fin disparaît la première.
    - Ctrl-C (#196) : un gestionnaire `ctrlc`, posé par `main` une fois le lot admis,
      efface la ligne d'étape sur un terminal, écrit « lot interrompu à la vidéo 3/12 ;
      relancer la même commande reprend là où le lot s'est arrêté » (« refait le lot »
      sous `--force`, #70) sur la sortie d'erreur et sort en 130,
      même si la ligne ne s'écrit pas, en gardant la sortie d'erreur jusqu'à la sortie.
      `Status` retient la vidéo en cours. Rien
      d'autre ne se fait : un `.partial` laissé par l'interruption dans le dossier d'une
      vidéo est retiré par le ménage de la relance (#115), les autres restent
      (`names::partial_file`). La reprise d'un lot interrompu est #70.
    - Les lignes d'événement (extrait refait, cause d'un échec) s'écrivent
      au-dessus de cette dernière ligne.
    - Le mode se choisit par `std::io::IsTerminal` sur la sortie d'erreur.
      - Redirigée : une ligne par étape, rien n'est effacé.
      - Sur un terminal : la ligne d'étape est redessinée sur place, par `\r`, des espaces
        jusqu'à la longueur précédente, puis `\r`, sans séquence d'échappement (console
        Windows ancienne). La longueur compte des caractères. Sous Unix, la ligne est coupée à la largeur du terminal moins une colonne, lue à chaque étape par `tcgetwinsize` (`terminal.rs`) : une ligne qui passerait à la rangée suivante ne s'effacerait qu'à partir de sa dernière rangée. Sous Windows, la largeur n'est pas lue et une ligne plus large que le terminal garde ce défaut.
    - Le rang vient de `outcome::run_all`, qui passe `Rank { position, total }` au travail
      et au rapport de chaque vidéo.
  - Chaîne : `engine::detection::detect` (fils automatiques), puis la `Timeline` des
    `pts` lus, puis `exclude`. Les plages abîmées viennent du moteur, et les images
    signalées sont l'union des images en erreur et après un saut. Puis
    `track_detections`, puis le ménage, puis le CSV et `params.json` de `files`.
  - Ménage (#115) : `files::previous::remove_previous_outputs`, après la détection et le
    suivi, avant le CSV, comme la v0.1.1. Une détection ratée ne détruit donc rien.
    - Il retire les sorties des deux langues et des anciens noms : sur l'extrait, une
      passe en français, puis en anglais, puis en français dans le même `-o`, ne laisse
      que les fichiers de la dernière, sous les noms de la v0.1.1.
    - Le tableau des pistes appartient à l'outil (#228) : un fichier à son nom est retiré, quoi qu'il contienne. La mise de côté datée de #83 est retirée ; les tableaux qu'elle a datés gardent un nom que le ménage ne reconnaît pas, et restent.
    - Une entrée au nom du tableau qui n'est pas un fichier est laissée, et le bilan le
      dit (#65) sous « Tableau laissé tel quel » : « {vidéo} : {csv} n'a pas la forme
      d'un tableau des pistes ». Si elle porte le nom de la langue courante, ou si un
      fichier occupe encore ce nom après le ménage (`Error::Taken`, autre casse), le
      nouveau tableau n'est pas écrit par-dessus : `Processed::csv` vaut `None`, et la
      ligne finit par « ; le nouveau tableau n'est pas écrit ». La vidéo reste réussie.
  - Image résumé (#104) : `overview`, appelée après `params.json`, si bien qu'un échec du
    fond laisse le CSV et les réglages. Le fond passe par
    `Converter::new(…, SourceRange::FullBlackAt16)`, puis `summary::pages` ; chaque page
    prend son nom par `Names::summary_png`. Pas de fond : erreur `NoBackground`.
    - Le fond de `pages` est écrit avant les pages, sous `Names::background_png` (#259), un par vidéo quel que soit le nombre de tranches et de parties (0,87 Mo pour la 092). `sheets` recopie la mise en page de chaque page dans `params.json` (`summaries`, secondes au centième, fichier relatif au dossier de la vidéo) ; `listing` y met la trace de chaque piste dessinée (`trace`, pixels entiers de la vidéo, points consécutifs égaux fusionnés). Rien n'est recalculé après `summary` : `results` et la page relisent ces valeurs. Sur la 092, `params.json` passe à 13,5 Ko.
  - Extraits (#112) : `render_clips`, appelée après l'image résumé.
    - Le dossier des extraits (`extraits` ou `clips`, selon la langue), retiré par le
      ménage, n'est recréé qu'avec au moins un extrait à écrire, après l'essai d'encodeur
      réussi (#195) : 0 piste, trop de pistes ou aucun encodeur ne laissent aucun dossier.
    - Garde `max_tracks` (300) : au-delà, aucun extrait n'est écrit,
      et la vidéo échoue avec sa ligne (« 301 pistes, au-delà de 300 : extraits non
      écrits ») après le CSV, `params.json` et l'image résumé.
    - Encodeurs : un essai d'encodage (`Fallback::trial`) à la taille et à la cadence de
      la vidéo, avant `params.json`, donne la liste employée (VideoToolbox puis openh264
      sur Mac, openh264 ailleurs). Il est sauté au-delà de la garde. S'il échoue,
      `params.json` porte `encoders: []`, l'image résumé est écrite, et la vidéo échoue
      à l'étape des extraits, après le vidage du dossier.
    - `Source` reçoit les images-clés de la lecture du moteur (`Reading::key_frames`) et
      ses images en erreur ; chaque extrait prend son nom par `Names::clip`, à partir de
      `files::time::frame_seconds` de sa première image.
    - Un extrait refait en openh264 après un échec de VideoToolbox écrit une ligne sur la
      sortie d'erreur, avant la ligne de réussite : « {vidéo} : {extrait} réécrit avec
      openh264 après une erreur de VideoToolbox : {causes} ».
  - Langue du système : la première variable non vide parmi `LANGUAGE`, `LC_ALL`,
    `LC_MESSAGES` et `LANG`. Dans une liste (`LANGUAGE=de:fr`), c'est la première langue
    qui a des messages qui compte : `fr*` pour le français, `en*` pour l'anglais. Le Python
    ne connaissait que le catalogue français : `en:fr` y donnait du français, ici de
    l'anglais. Si aucune variable n'est posée ou toutes sont vides, `sys-locale` lit la
    langue de l'interface (macOS, Windows) ; à défaut, l'anglais. `LANG=C` reste en
    anglais : la variable est posée. Les tests qui lancent le binaire fixent tous `LANG`.
  - Langues (#114) : toute la commande parle la langue choisie, la ligne de commande comme
    les lignes de console et le bandeau de l'image résumé ; les noms de fichiers restent
    dans `files::names`.
    - Deux catalogues Fluent plats, `crates/cli/locales/fr.ftl` et `en.ftl`, inclus dans le
      binaire (`include_str!`) : ni termes, ni attributs, ni `NUMBER()`. Le français reprend
      mot pour mot les textes de la v0.1.1 (catalogue de #54) quand ils existent.
    - `texts::Message` énumère les messages avec leurs arguments typés, `texts::Fixed` ceux
      qui n'en ont pas. Le test des clés exige : chaque message se forme sans erreur dans les
      deux langues ; chaque argument sert dans au moins une langue (l'anglais n'accorde ni
      « réécrit » ni « illisible ») ; aucune clé orpheline ; les deux catalogues se lisent
      sans erreur ; aucune marque d'isolement U+2068/U+2069 (coupées par
      `set_use_isolating(false)` : Roboto n'a pas ces glyphes, et elles entreraient dans les
      chemins copiés depuis la console).
    - `texts::Texts` ne porte que la langue (`Copy`). Les deux bundles vivent dans deux
      `static OnceLock`, construits au premier usage sans `Result` : un bundle n'est pas
      `RefUnwindSafe`, et le travail de `run_all` ne capture ainsi que la langue.
    - La bibliothèque reste sans texte : `process(job, &dyn Wording, …)` reçoit le trait
      `Wording` (les libellés du bandeau et la langue des noms) ; ses tests passent des
      libellés de débogage en anglais.
    - Nombres : un formateur par langue (`numbers.rs`), branché sur le bundle. Français :
      virgule décimale, milliers séparés par U+202F dans la console et par U+00A0 dans le
      bandeau (Roboto n'a pas U+202F). Anglais : point, sans séparateur. `-0` s'écrit `0`.
      Le CSV, `params.json` et les noms de fichiers n'y passent pas. L'aide affiche
      « [par défaut : 1,5] », alors que l'option s'écrit `--clip-margin 1.5`, comme en
      v0.1.1.
    - Aide : la commande se construit avec les textes de la langue (aides, noms de valeurs,
      titres par `help_heading`, « Utilisation : » par le gabarit). Les titres français
      finissent par une espace insécable, puisque clap colle « : ». L'apparence de #113 est
      la même dans les deux langues. L'usage reste celui que clap génère avec nos noms de
      valeurs : « [OPTIONS] » s'y lit aussi en français, et `override_usage` n'a pas été
      pris.
      Chaque texte d'aide d'une option ou d'un argument commence par une majuscule, dans
      les deux langues (« Vidéos, ou dossiers… », « Print the version ») ; pas les noms de
      valeurs, les titres ni « [par défaut : …] ». Un test le garde (#115).
    - Erreurs de clap : un `ErrorFormatter` par langue (`errors.rs`), branché par
      `clap::Error::apply`, les rend depuis `ErrorKind` et `ContextKind` ; impression,
      couleurs et codes de sortie restent ceux de clap. Nos refus portent leur phrase dans
      `ContextKind::Custom`, et non par `Command::error`, dont le message tout fait ne passe
      pas par le formateur. En anglais, les erreurs de syntaxe essayées par le test (option
      inconnue, avec option ou valeur proche et conseil `--`, valeur hors choix ou vide,
      valeur donnée à un interrupteur, option répétée, argument manquant) sont rendues
      comme clap les écrit, couleurs comprises, au pied près. Un genre non prévu donne
      « les arguments sont refusés », sans le texte anglais du genre (#55).
    - Ligne d'usage et pied (#55). La ligne d'usage ne suit que les erreurs de syntaxe de
      clap. Les refus de valeurs ne l'ont pas et gardent le pied, « Pour en savoir plus,
      essayez 'batdetect --help'. », dont le nom vient de `CARGO_BIN_NAME`. Les refus
      d'entrées et de dossiers n'ont ni l'une ni l'autre (`errors::rejection`, marqués par
      `ErrorKind::Io`).
    - Causes : les `Display` des erreurs des bibliothèques restent en anglais dans le seul
      détail d'un échec (`failure-detail`, #64) et dans `clips-redone`. Aucun refus avant
      calcul n'interpole plus de texte anglais (#55).
    - Noms d'options : `engine::settings::Setting::flag()` (`threshold`, `work-width`, …)
      nomme l'option dans la commande comme au banc, pour `Arg::long` comme pour les refus.
      La phrase du refus reste en deux copies : anglaise au banc, Fluent dans la commande.
  - Raisons en clair (#64). Une vidéo en échec écrit trois lignes sur la sortie d'erreur,
    au-dessus de « [k/n] vidéo — échec » de #63. Les autres vidéos continuent ; le code de
    sortie vaut 1 si une vidéo a échoué.
    - Les trois lignes : « {vidéo} : {phrase} », « à faire : {conseil} », « détail :
      {chaîne anglaise des causes} ». « Trop de pistes » garde sa phrase de #113, a un « à
      faire » et pas de détail.
    - `failure::cause(&process::Error, &Options) -> Cause` classe sans texte.
      - Le `match` est exhaustif, sans `_`, sur `process::Error`, `detection::Error`,
        `input::Error` et `decode::Error`. `detection::Error::Input` est transparent :
        `input::Error` n'apparaît jamais dans `source()`, et seul le `match` y mène.
      - Côté écriture (`Folder`, `Previous`, `Files`, `Overview`, `Encoders` sauf `Scratch`,
        `Clips`), c'est la chaîne `source()` qui classe, sans `match` sur les énumérations
        de `files`, `render` ou `fallback`.
    - Causes :
      - à l'ouverture, par `io_kind()` : introuvable, lecture refusée, sinon « pas une vidéo
        lisible, ou copie incomplète ». Une copie tronquée depuis les jumelles perd le
        `moov`, écrit en fin de fichier, et FFmpeg ne la distingue pas d'un fichier qui n'est
        pas une vidéo ;
      - nom illisible par FFmpeg ;
      - vidéo abîmée (nombre d'images, lecture, décodage) ;
      - format non géré (décodeur absent, format de pixel…) ;
      - réglages, seulement si `--work-width` ou `--osd-region` a été donné, sinon format
        non géré ;
      - disque plein, écriture refusée et, sous Windows, fichier ouvert ailleurs (erreurs 32
        et 33 au ménage, #115). Ces trois-là ne se cherchent que du côté écriture, sur toute
        la chaîne `source()`, par `io::Error` ou `LibraryError::io_kind`. Le fichier ouvert
        se reconnaît sur `Previous`, aux variantes `Remove` et `Read`, les seules qui portent
        le chemin. Pour un dossier d'extraits, c'est le dossier qui est nommé, pas le fichier
        ouvert dedans ;
      - extraits non écrits ;
      - trop de pistes ;
      - erreur inattendue ;
      - erreur interne (panique).
    - Le doublon « failed: the detection failed: » disparaît : la phrase remplace le cadre
      « échec : », et « failed » ne reste qu'une fois, dans le détail.
  - Tenue aux pannes (#113) : `outcome::run_all(vidéos, travail, compte rendu)` passe
    chaque vidéo dans `catch_unwind`, le seul du code livré (règle 10), sur un travail
    `Fn + RefUnwindSafe` : le compilateur vérifie qu'il ne modifie rien de ce qu'il capture,
    appelle le compte rendu au fil de l'eau (le binaire n'y fait qu'écrire) et rend un
    `Row` par vidéo (#65) : le `Job`, le temps de calcul (`Instant` autour de
    `catch_unwind`, détection comprise) et un `State` : `Done`, `TooManyTracks` ou
    `Failed(Failure)` (`Failure::Error` ou `Failure::Panic` avec son message et son lieu).
    Une panique donne « erreur interne », avec le message et le lieu en détail, compte comme un échec (code 1), et la
    vidéo suivante est traitée.
    - Crochet (#64) : `outcome::quiet_panics()`, appelé une fois par `main`, garde le
      crochet par défaut. Pendant une vidéo (un `AtomicBool` mis par `run_all` autour de
      `catch_unwind`), il n'écrit rien et garde la première panique dans un `static
      Mutex`, repris même empoisonné. Il est global et non par fil : `resume_unwind`
      n'appelle pas le crochet. Hors d'une vidéo, il passe la main au crochet par défaut.
      Le lieu est celui que rustc connaît : `crates/…` sous macOS et Linux, `crates\…`
      sous Windows, un chemin absolu pour une dépendance.
    - Pourquoi la vidéo suivante peut repartir : le travail ne capture que des références
      vers des données qu'il ne modifie pas (options, langue, versions) ; les fils d'une
      vidéo sont rejoints ou arrêtés avant la relance (`read_ahead`, `serve_ahead`,
      `schedule`, rayon), ce que deux tests `should_panic` vérifient pour une panique du
      consommateur ; la vidéo suivante repart de zéro.
    - Ce qu'une panique peut laisser : les extraits passent par un temporaire effacé au
      déroulement, mais pas le CSV ni `params.json` : un CSV peut rester tronqué, comme un
      échec laisse ses fichiers déjà écrits. Une panique dans l'initialisation de FFmpeg
      (un `Once`) l'empoisonne, et les vidéos suivantes paniquent aussi. Le manque de
      mémoire, une panique pendant une panique et une panique dans un `rayon::spawn` ne sont
      pas rattrapés ; la borne de la fenêtre du fond (`engine`) écarte la première cause
      connue.
  - `params.json` : réglages rangés par configuration, avec les clés de la v0.1.1
    (`DetectConfig`, `TrackConfig`, `StabilityConfig`, par `tracking::settings` depuis
    #108). Un filtre coupé s'écrit 0, comme dans la v0.1.1. `threads` donne les fils réellement employés : `decoding` et
    `decoding_kind` (`single`, `frame`, `slice`) pour le décodeur, `detection` pour
    `rayon`. `version` est celle de `--version`, gravée par `crates/cli/build.rs`
    (#113) : la variable `BATDETECT_VERSION` si elle est posée, sinon `git describe --tags
    --match 'v[0-9]*' --dirty --always` sans le `v` (par exemple `0.1.1-81-g40591c2-dirty` ;
    le hachage court seul dans un clone sans tags), sinon `CARGO_PKG_VERSION`. Le script
    ne se relance que si `HEAD`, la branche qu'il nomme, `refs/tags`, `packed-refs`
    (chemins absolus, worktree compris, seulement ceux qui existent), `crates/`,
    `Cargo.toml`, `Cargo.lock` ou `rust-toolchain.toml` changent : un `git fetch` ou un
    commit d'un autre worktree ne le relancent pas ; « -dirty » ne voit pas une
    modification de `docs/` ou de `mise.toml`, et toute modification de source recompile
    `cli`, la crate feuille. `ffmpeg` est la version de FFmpeg liée.
    - `video` est écrit par `to_string_lossy` (#227) : un nom non UTF-8 y garde ses caractères lisibles, ses octets illisibles devenant `�`, au lieu de faire échouer l'écriture. C'est un texte à lire, pas un chemin à rouvrir : la relance compare la taille, la version, la langue et les réglages, jamais ce nom, et la page retrouve le tableau par son nom lossy (`results`). Des octets échappés dans la chaîne auraient changé la forme du champ pour tous les noms et demandé un décodage à chaque lecteur.
    - `RenderConfig` (#112, #103) : `box_pad`, `trail_s`, `clip_margin_s`, `max_tracks`,
      `zoom` (`auto`, `all` ou `none`), `zoom_below_area` (`100.0`),
      `zoom_below_amplitude` (`40.0`), `annotated`, et `encoders`, la liste des encodeurs
      retenue par l'essai (vide au-delà de la garde).
    - Écarts à la v0.1.1 : `mode` et `ffprobe` retirés ; dans `RenderConfig`, `crf`,
      `encoder` et `vt_quality` ne sont pas écrits, `encoders` les remplace pour
      l'encodage ; `version`, `ffmpeg` et
      `threads` ajoutés ; les nombres réels s'écrivent `12.0` et non `12`.
      `DetectConfig` s'écrit par `engine::settings::DetectSettings`, `osd_regions`
      compris, en objets `{x0, y0, x1, y1}` à sa place (#151).
  - Tests : `tests/command.rs` lance le binaire sur une vidéo fabriquée (openh264,
    1440×1080) ; `slow_command.rs` passe l'extrait (4 pistes appariées, chauve-souris
    de 3:58) ; `reference_092.rs` passe la 092 (14 pistes appariées ; `reference_command.rs`
    avant #184). Le test Python
    qui comptait 3:36 comme une chauve-souris n'est pas repris : c'est la piste lente de
    3:29 ([03](03-resultats-092.md)). Dans `command.rs`, la vidéo fabriquée passe en
    français, en anglais, puis en français dans le même `-o` (#115).
  - Sorties de l'extrait (`tests/local_outputs.rs`, `#[ignore = "local"]`,
    `BATDETECT_REFERENCE_DIR`, #115). Le binaire passe l'extrait en français, en
    anglais, puis en français, dans le même `-o`. Entre la première et la deuxième
    passe, le test ajoute des anciens noms et des parties en trop.
    - Après chaque passe, la liste des fichiers égale, au nom près, celle de la v0.1.1
      (`outputs/{fr,en}/` de la référence, hors dépôt) : CSV, image résumé,
      `params.json` et les 4 extraits.
    - Comme la v0.1.1, le Rust ne fait aucun zoom sur l'extrait (`--zoom auto`).
    - Environ 20 s (18,5 et 23,7 s le 14 octobre 2026, ce test ne mesure pas de temps).
  - Comparaison sur les 26 vidéos (`tests/local_command.rs`, `#[ignore = "local"]`,
    `BATDETECT_REFERENCE_DIR` et `BATDETECT_VIDEOS_DIR`). Elle passe `process` (la
    fonction qui écrit le CSV) et apparie ses pistes à `tracks.jsonl` par
    `match_tracks`.
    - Mesure du 12 octobre 2026 : 966 pistes de référence, 963 appariées
      (958 concordantes), 3 perdues, 9 ajoutées, les chiffres de #118 et #119.
    - Les 17 écarts tombent tous sur les 9 divergences connues (#97), figées dans le
      test :
      - 094 1:42 : perdue, revient 26 images plus tard ;
      - 097 0:20 : perdue, revient 7 images plus tard ;
      - 097 2:42 : 24 points au lieu de 86, et une piste ajoutée à 2:43 ;
      - 098 0:05 : ajoutée ;
      - 098 0:41 : perdue ;
      - 099 de 0:14 à 0:19 : trois écarts de points et une piste ajoutée ;
      - 099 0:27 : ajoutée ;
      - 100 1:36 : deux pistes ajoutées ;
      - 126 1:31 : 27 points au lieu de 32, et une piste ajoutée à 1:32.
    - Les plages abîmées du moteur, et non plus celles de la référence, ne changent
      rien.
  - Temps sur la 092 (M1 Max, `release`, sous le verrou, machine au calme) : 11,7 et
    12,0 s réels, 103,7 s de CPU, 434 Mo. Pour comparaison : 11,8 s réels pour la
    détection Rust seule (#118) ; 118,7 s pour la détection de la v0.1.1 et 134 s pour sa
    commande complète, rendu compris (README de la référence).
- **`tracking`** → `types` : exclusion des images abîmées et instables (#120), puis
  suivi (#121). `compare` et `cli` enchaînent `exclude` puis `track_detections`.
  - `exclusion` : seul point d'entrée. `exclude` reçoit les détections, les plages
    abîmées toutes faites, le nombre d'images signalées et la `FrameRate`. Les images
    abîmées sont retirées du calcul de stabilité, puis les détections des plages
    abîmées et instables sont vidées, images gardées. `Exclusion` rend aussi le nombre
    d'images ignorées et celui des images abîmées, chacune comptée une fois. `exclude`
    ne peut pas échouer (#144). Le contrôle du nombre d'images reste à #116.
  - `settings` (#108) : `TrackSettings` et `StabilitySettings`, l'écriture et la lecture
    des réglages de suivi et de stabilité aux clés de la v0.1.1, venues de `cli` pour que
    `params.json`, `bench.json` et `run.json` partagent une seule forme. Un filtre coupé
    s'écrit 0 (`twin_distance`, `max_blobs`).
  - `stability` : `StabilityConfig::new(max_blobs, pad_s)`, par défaut 20 et 1,0 s ;
    `max_blobs` est un `Option<NonZeroU32>`, `None` coupe le filtre (la commande
    traduira `--max-blobs 0`) ; `pad_s` fini, positif ou nul, sinon `Error::Setting`
    avec `Setting::PadS` (une seule forme pour la stabilité et le suivi, depuis #123). Limite de foule :
    `max_blobs + 6 × médiane` du nombre de détections des images lisibles, vides
    comprises, calculée en entiers. Une image qui l'atteint donne une plage de
    `pad_s × cadence` images de chaque côté, arrondi au plus proche, moitié vers le
    haut (la v0.1.1 arrondit au pair : une image d'écart quand ce produit vaut x,5,
    jamais avec les réglages par défaut). Le rembourrage est borné à la longueur de la
    vidéo. Seuls les bords sont bornés aux premières et dernières images lisibles : au
    milieu, une plage instable peut recouvrir une plage abîmée (093, 097). Deux plages
    séparées d'au plus le rembourrage se fondent.
  - `spans` (interne) : images couvertes par des plages, bornées à la vidéo, pour les
    comptes d'`Exclusion`. Les détections se vident par `FrameDetections::without`.
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
      `Setting`. `Error::Track` ne vient d'aucune entrée : c'est un garde-fou, qui
      signale un bogue de l'association ou de la fusion des jumelles (deux points sur
      une même image). La renumérotation finale passe par `Track::with_id`.
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
  `types::Language`, celle de la commande (#123, #114).
  - `names` : `Names::new(langue)` et ses méthodes. Français : `extraits`, `piste`,
    `_pistes`, `_resume`, `_annotee`, sans accent. Anglais : `clips`, `track`, `_tracks`,
    `_summary`, `_annotated`. `params.json` n'est pas traduit, `_zoom` est commun.
    - Le fond de la page (#259) : `_fond`, `_background`, reconnu par le ménage comme une image ; les noms de la première version n'en ont pas.
    - Le nom d'extrait se construit depuis la `Track`, avec le temps de début passé par
      l'appelant en secondes. Il s'écrit en minutes et secondes entières, tronquées comme
      dans la v0.1.1 : à 59,996 s, le nom dit `0m59s` et le CSV `1:00.00`.
    - Le numéro de piste prend la largeur du nombre de pistes, 2 chiffres au moins.
    - La période de l'image résumé est donnée en minutes (`_000m-010m`).
    - Une image coupée par #157 porte sa partie, numérotée à la largeur du total :
      `_3_sur_8`, `_03_sur_12` ; `_3_of_8` en anglais. Après la période s'il y en a une
      (`_000m-010m_2_sur_4`).
    - Les noms de vidéo sont des `OsStr`, gardés tels quels.
    - Une troisième table, celle des anciens noms (`split`, `.tracks.csv`, `.tracks.png`,
      `_boxes.mp4`, ceux de la v0.1.0), ne sert qu'à les reconnaître (#115).
  - `previous` (#115) : `remove_previous_outputs`, le ménage des sorties d'une passe
    précédente dans le dossier d'une vidéo, dans les trois tables. Comme la v0.1.1
    (`names.py`), il compare en NFC le nom de chaque entrée à celui de la vidéo.
    - Il retire :
      - les dossiers d'extraits ;
      - les CSV ;
      - les vidéos annotées ;
      - les images résumé, avec leur période (`_000m-010m`, 3 chiffres au moins) et
        leur partie (`_3_sur_8`, `_3_of_8`) ;
      - les partiels des vidéos annotées, des images, des CSV et de `params.json`, que laisse un arrêt brutal, sous les deux formes de `names::partial_file`.

      `params.json` est réécrit, pas retiré. Le reste du dossier n'est pas touché.
    - Une entrée à ce nom qui n'est pas un fichier ordinaire (dossier, lien) est laissée en
      place et rendue (`Kept::Left`, avec `ours` vrai s'il porte le nom de la langue
      courante) ; le nouveau tableau n'est alors pas écrit.
    - Un lien porte le nom d'un dossier d'extraits : il est retiré, sa cible non (sous
      Windows, un lien de dossier par `remove_dir`).
  - `names::partial_file` crée le partiel de tout fichier écrit : un fichier vide `.<nom>.<6 lettres ou chiffres>.partial` à côté du fichier final, au nom tiré au hasard et créé de façon exclusive (`tempfile::Builder::make_in` et `File::create_new`, #245). Deux écrivains du même fichier, comme le fil du Ctrl-C qui réécrit la page pendant que le fil principal l'écrit, ou deux `batdetect -o` sur le même dossier, ont donc chacun le leur : le dernier renommage gagne et chaque fichier final est entier, sauf pour le tableau des pistes, où le premier gagne (voir `table::write_tracks_csv`). Un chemin sans nom de fichier est refusé (`InvalidInput`). `table`, `params`, `failures`, `summary::image_file` et la page de `cli` l'emploient ; `media::path` a la même forme, liée par un test de `cli`. Le ménage reconnaît cette forme, quel que soit le mot de 6 lettres ou chiffres (`.v_pistes.csv.backup.partial` compris), et l'ancienne, `.<nom>.partial`, laissée par la 0.2.0. `tempfile_in` n'est pas employé : il crée le fichier en `0600`, que le fichier final garderait. Un partiel laissé par un arrêt brutal à la racine (page) ou dans `.batdetect/echecs/` n'est plus écrasé au passage suivant : il reste, caché.
  - `table::write_tracks_csv` écrit dans son partiel, puis le renomme sans jamais
    écraser un fichier présent (`exclusive::rename`, sinon `Error::Taken`).
    - Le renommage exclusif (`TempPath::persist_noclobber`) passe en premier. Sur toute
      autre erreur que « nom déjà pris », dont le refus de macOS sur exFAT (#236), le nom
      final est réservé (`exclusive::reserve`) puis le partiel renommé par-dessus. Si la réservation échoue, c'est son erreur qui remonte ; si
      le renommage échoue, la réservation vide est retirée, et si ce retrait échoue,
      c'est son erreur qui remonte.
    - Un arrêt brutal pendant l'écriture ne laisse qu'un partiel, que le ménage retire,
      et jamais un CSV tronqué qui passerait pour étranger.
    - La réservation ne sert qu'en repli : un arrêt brutal entre elle et le renommage
      laisserait un CSV vide, que le ménage retirerait au calcul suivant.
    - `tests/slow_exfat.rs`, sous macOS seulement : une image exFAT montée par `hdiutil`
      sans `sudo` ; `persist_noclobber` y est refusé, le tableau s'y écrit, refuse
      d'écraser un tableau annoté, est retiré par le ménage et se réécrit.
    - Sur un disque insensible à la casse (APFS, NTFS), un `vol_pistes.csv` annoté et la
      vidéo `VOL.mp4` ont le même chemin. Le ménage, qui compare les octets, ne
      reconnaît pas ce fichier ; le renommage refuse de l'écraser.
  - `time` : `clock` écrit `h:mm:ss`, tronqué à la seconde comme le nom de l'extrait.
    - Le temps d'une image vient de son `pts` (`Timeline::seconds_by_pts`). Une image
      sans `pts` reprend celui de la dernière image qui en a un, plus un pas de cadence
      par image ; la cadence seule ne sert qu'en l'absence de tout `pts`.
    - La fin d'une plage est le début de l'image suivante.
    - Le temps n'est pas rapporté au premier `pts` : les 26 vidéos de la référence
      commencent à `pts` 0. Un `pts` négatif reste à décider avec #116.
  - `table` : CSV des pistes pour un tableur (#84), 12 colonnes dans l'ordre de
    `table::COLUMNS` ([docs/02](02-methode.md#sorties)). Les en-têtes sont des textes
    Fluent, donnés par le binaire à travers le trait `table::Headings` ; `files` ne les
    connaît pas. La colonne `verdict` est retirée (#228) : les verdicts vivent dans `verdicts.csv`.
    - `dialect` porte la forme d'une langue : BOM, point-virgule et virgule décimale en
      français ; ni BOM, virgule et point en anglais. CRLF dans les deux.
    - Une case qui contient le séparateur ou un guillemet est mise entre guillemets ; les
      cases des nombres et des temps n'en ont jamais.
    - Les nombres sont arrondis, puis écrits sous leur forme la plus courte, avec `.0`
      pour un entier (`12.0`, `12,0`) ; `-0.0` devient `0.0`.
    - La surface max est entière, arrondie à mi-chemin vers le haut.
    - Durée = fin − début, au moins une image.
  - `params` : `Params<S>` écrit `video` (un chemin), `fps`, les réglages de l'appelant
    aplatis, `ignored_s` et `damaged_s` (au centième), `damaged_frames`, `version`
    (fixée à la compilation par l'appelant) et `ffmpeg` (version liée). Ni `mode` ni
    `ffprobe`.
    - Le nombre de fils réel arrive avec #123.
    - `files` n'impose pas les clés des réglages. Garder celles de la v0.1.1 (une clé
      par configuration, au nom de sa classe, dont les seuils de zoom dans
      `RenderConfig`) revient à l'appelant : #123 porte le test des clés, #103 celui des
      seuils de zoom (`cli/src/settings/tests.rs`).
    - Une clé `video` ou `fps` venue des réglages serait écrite en double.
  - Mesure (11 octobre 2026, relue par valeurs le 16 octobre 2026 avec #84), CSV écrit
    depuis les pistes de la référence (`files/tests/committed_table.rs`) :
    - extrait : mêmes valeurs que le `tracks.csv` de la v0.1.1, début et fin tronqués à
      la seconde ;
    - 092 : 14 lignes. Numéro, points, corde, chemin, amplitude, images comblées et durée
      sont égaux.
    - Les temps sont plus tôt d'un ou deux centièmes : débuts décalés sur 11 pistes sur
      14 (de 33,94 → 33,93 s à 237,18 → 237,16 s), fins sur 12 ; tronquée à la seconde,
      seule la fin de la piste 6 change (1:50.00 → 1:49.99, soit `0:01:49`). La v0.1.1 divisait le
      numéro d'image par la cadence moyenne (30,027 i/s), alors que les `pts` avancent
      de 333/10 000 s par image (30,03 i/s).
    - `speed_px_s` est plus haute de 0,1 à 0,2 px/s sur 10 pistes, jamais plus basse.
    - `max_area_px` : pistes 1, 5, 6 et 10, 274 → 275, 400 → 401, 490 → 491 et
      274 → 275. Les maxima tombent sur un demi (sur la 092, les surfaces sont des
      multiples de 2,25) : le Rust arrondit à mi-chemin vers le haut, la v0.1.1 au pair.
      Gardé (règle 5).
  - Lecture d'un dossier de résultats pour la page (#71) :
    - `results::read_results` : chaque sous-dossier non caché, dans l'ordre des noms.
      `params.json` lu (`Written`, avec `Deserialize` sur `Finish` et `TrackEntry`) :
      terminée. Un `params.json` à nous sans `state` (avant #66) : ancienne version. Sans
      `params.json`, avec une sortie nommée d'après la vidéo (`names_the_video`, nom du
      dossier pris pour celui de la vidéo ; un dossier `clips/` seul ne compte pas) :
      inachevée. Ni l'un ni l'autre : ignoré, pour ne jamais lister un dossier de
      l'utilisateur. Les échecs de
      `.batdetect/echecs/` s'y ajoutent, avec ou sans dossier.
    - Le fond (`background` de `params.json`) est donné avec sa taille, lue dans l'en-tête PNG ; absent du disque ou du fichier (avant #259), il manque, et `summaries` est vide (`#[serde(default)]`).
    - Les images résumé sont celles que nomment les pistes de `params.json`, dans leur ordre ;
      sans piste, l'image sans suffixe. Leur taille vient de l'en-tête PNG (`picture`).
    - Le tableau des pistes se cherche sous le nom tiré de `video` ; absent et si ce nom contient `�`, sous le fichier du dossier dont le nom lossy est le même (`on_disk`, #227 ; à égalité, le premier que liste le système). Une vidéo au nom non UTF-8 retrouve ainsi son tableau ; ses images résumé et sa vidéo annotée, cherchées sous le nom lossy, ne sont pas retrouvées.
    - `verdicts::read_verdicts_file` (#228) : le plus récent des fichiers `verdicts*.csv` de la racine (le navigateur numérote les doubles), son nom, sa date et le nombre de candidats ; les octets seulement, sans décodage ni lecture : c'est le JS de la page qui les lit. batdetect n'écrit jamais ce fichier.
    - `failures` : un fichier par vidéo en échec dans `.batdetect/echecs/`, écrit par
      `.partial` et renommage ; un fichier illisible ne cache que lui-même.
    - `refusals` (#267) : `.batdetect/refus.json`, les entrées refusées au dernier lancement, écrit par `.partial` et renommage ; illisible ou vide, il ne donne rien.
    - `names::results_page` : `resultats.html` / `results.html`.
- **`render`** → `media`, `draw`, `types` (#110, #112) : les extraits, du choix
  des images à l'écriture. `clips::render(source, pistes, couleurs, Outputs { clips, zooms },
  RenderConfig, encodeurs)` en est le point d'entrée ; `cli` l'appelle depuis #112, et
  choisit les zooms par `ZoomConfig::wants` (#103).
  - `drawn`, derrière la feature `testing` (tests seulement, #198) : `drawn(source,
    pistes, couleurs, Outputs, RenderConfig)` passe par le même rendu, sur un fil, mais chaque
    encodeur est remplacé par un enregistreur qui garde l'empreinte de chaque image
    poussée, fichier par fichier. Les tests figés de `cli` s'en servent.
  - `source` : `Source::new(chemin, Timeline, images-clés, images en erreur)`, puis
    `serve(plage, visite)`, qui rend chaque image de la plage une fois, dans l'ordre,
    avec son `FrameIndex`. Les images-clés et les images en erreur viennent du moteur
    (`FrameInfo::is_key`, `Damage::error_frames`) ; `render` ne dépend pas d'`engine`.
  - Les images se choisissent par `pts` : chaque `pts` rendu par le décodeur est cherché
    dans la `Timeline` (recherche inverse, interne à `render`). Une `Timeline` dont les
    `pts` connus ne croissent pas strictement est refusée. Une plage qui sort de la
    vidéo, ou dont une image n'a pas de `pts`, est refusée avant tout décodage.
  - Contrôle de la suite servie : un `pts` inconnu de la `Timeline`, une image qui revient
    après une plus récente, une image de la plage que le décodeur saute ou qui manque
    en fin de fichier sont des erreurs typées. Une erreur de la visite arrête la lecture
    et revient telle quelle, comme pour `engine::input`.
  - Image de reprise : la dernière image-clé à ou avant le début de la plage qui n'est
    pas elle-même en erreur. La dissimulation d'une image-clé en erreur puise dans les
    images d'avant : seule une image-clé saine remet le décodeur à zéro. Sans marge
    réglée : la marge de 30 du prototype supposait des GOP de 15 (ici de 8 à 16).
  - Fils de décodage : un seul (`Threads::Exactly(1)`) quand la plage lue, depuis l'image
    de reprise, contient une image en erreur, car les pixels dissimulés ne se reproduisent
    qu'à un fil (voir « Fils et pixels dissimulés ») ; sinon automatiques (nuit du 12 au
    13 octobre 2026), car une vidéo saine donne les mêmes pixels quel que soit le nombre
    de fils (`slow_source`, `local_clips`). L'appelant sert une
    plage par groupe de fenêtres chevauchantes, pour ne décoder qu'une fois une image
    partagée ; le parallélisme vient des groupes (#112). Il borne aussi les fenêtres à
    la dernière image : la v0.1.1 ne les bornait pas, `serve` refuse une plage qui
    sort de la vidéo.
  - Placement raté : si le placement échoue, ou si la première image rendue vient après
    l'image de reprise (conteneur sans index), la plage est relue depuis le début.
    `serve` rend d'où il a lu : `Origin::Key(image)`, ou `Origin::Start` avec sa raison
    (`Restart::NoCleanKey`, `Unplaced` avec l'erreur de FFmpeg, `Overshot` avec l'image
    de reprise et la première image rendue).
    - Le dépassement est testé : une image-clé forgée avant la première image du mp4
      est ramenée par FFmpeg à l'image 0 (`mov.c`), qui vient après elle.
    - Aucun mp4 ne fait échouer le placement : une cible avant le début est ramenée à
      la première image. `Unplaced` et l'erreur `OutOfOrder` (une image rendue après une
      plus récente, impossible tant que le décodeur rend dans l'ordre des `pts` et que
      la `Timeline` croît strictement) restent des gardes non couvertes.
  - Hypothèse : une image-clé saine remet le décodeur à zéro. C'est vrai d'une IDR ;
    FFmpeg marque aussi image-clé une image à point de reprise (SEI
    `recovery_frame_cnt` nul), qui n'est pas forcément indépendante. Seuls l'extrait et
    les 103, 123 et 124 le confirment. Le placement passe par le flux vidéo par défaut
    de FFmpeg, le décodeur par `best(Video)` : ils coïncident tant que le fichier n'a
    qu'un flux vidéo.
  - `Source` ne garde de la `Timeline` que le nombre d'images et la table `pts` ↔ image :
    sans durée de vie, `Send` et `Sync`. Un test sert trois plages depuis trois fils à
    la fois.
  - Mesures (M1 Max, `--profile ci`) : chaque plage est comparée à une lecture
    séquentielle à un fil (empreinte de chaque image) :
    - test rapide : vidéo openh264 1440×1080 de 48 images, une image-clé toutes les 8,
      dont la sixième part sur douze de `mdat` est écrasée : 40 images, 8 perdues d'un
      coup, l'image-clé 16 en erreur. Les 40 plages d'au plus 4 images et la plage entière sont
      égales ; celles qui commencent de 16 à 23 reprennent à l'image-clé 8. Reprendre à
      l'image-clé 16 en erreur fait échouer la plage 16–19 (essai fait une fois) ;
    - `slow_source` : 12 plages de 1 à 380 images tirées dans l'extrait (SplitMix64,
      graine fixe), égales, 24 s ;
    - `local_source` : la fenêtre 6580–6684 de la 103, seul extrait des 26 vidéos qui
      traverse une perte d'images, reprend à l'image-clé 6573 et rend ses 105 images
      égales (63 s) ; les 8 fenêtres d'extraits de la 123 et les 3 de la 124 qui touchent
      une plage abîmée (pistes de la référence, marge `round(1,5 × cadence)` = 45,
      bornée à la vidéo) sont égales (140 s pour les deux). Aucune de ces 12 fenêtres
      ne saute une image-clé en erreur : seule la vidéo fabriquée exerce cette règle.
  - `config` : `RenderConfig` (`box_pad` 12 px, `trail_s` 1 s, `clip_margin_s` 1,5 s,
    `max_tracks` 300, 0 pour sans limite), valeurs de la v0.1.1, validées à la
    construction (ni `NaN`, ni infini, ni négatif). Les secondes deviennent des images
    par `round(secondes × cadence)` (`f64::round`). `ZoomConfig` (#103) : `Zoom::Auto`,
    `All` ou `None`, et les seuils de la v0.1.1, `max_area < 100` px² ou
    `peak_amplitude < 40`, comparés strictement sur les mesures de la piste ; seuils finis
    et au-dessus de 0.
  - `windows` : la fenêtre d'une piste va de sa première image moins la marge à sa
    dernière plus la marge, bornée à la vidéo. Les fenêtres qui se chevauchent ou se
    touchent forment un groupe, décodé une fois sur son enveloppe et encodé une fois.
  - `overlay` et `picture` : sur chaque image, toutes les pistes présentes sont dessinées,
    une fois, et l'image dessinée est partagée par les extraits du groupe (comme
    `overlay.py` de la v0.1.1). Boîte de 2 px sur un point détecté, de 1 px sur un
    point interpolé, `#N` au-dessus, trace sur `trail_s`. Depuis #13, chaque piste a son encre (`ink`) : la couleur que `cli` lui passe, celle de l'image résumé, pour la boîte, le numéro et la trace, avec un liseré `INK` de 1 px de part et d'autre de la boîte et autour du numéro ; sans couleur, `BOX` et `TRAIL` sans liseré, le dessin d'avant. Les couleurs et les
    tailles viennent de `draw` ; les constantes propres aux extraits restent dans
    `overlay`. `Picture::draw` prend tout ce qui implémente `Marks` (ses zones, son
    dessin) : l'`Overlay` des extraits et la trace du zoom. Les zones viennent de
    `zones` (paires, arrondies à 16, bornées, fusionnées), les boîtes bornées et les
    points interpolés de `positions`.
  - Zoom (`zoom`, #103) : la vue lisible d'une petite cible, `<extrait>_zoom.mp4`.
    - Fenêtre : celle de l'extrait, mêmes images ; cadence déclarée divisée par 4
      (`FrameRate::slowed`, rationnel exact : 22527500/3000927 sur la 092, quand la
      v0.1.1 écrivait 908336/121001).
    - Cadre fixe pour la piste, formule de la v0.1.1 sur les **boîtes bornées** : leur
      enveloppe plus 40 px, en unités du PGCD de la largeur et de la hauteur (4×3 en
      1440×1080), au moins le quart de l'image (×4 au plus), centré puis borné à l'image.
      Placement par `f64::round` (la v0.1.1 arrondissait au pair : un pixel d'écart sur
      une égalité exacte).
    - Taille de sortie : celle de la vidéo. Une sortie en 720×540 pour les cadres qui y
      tiennent (86 % des zooms) a été essayée le 13 octobre : 099 en 36,3 s au lieu de
      41,8 s, zooms de 260 Mio au lieu de 413. Elle n'est pas retenue (décision de
      l'utilisateur) : la pleine taille montre mieux les objets diffus.
    - Agrandissement au plus proche, en YUV : pixel de sortie `x` ← `gauche + x ×
      largeur du cadre / largeur de sortie` (entiers), chroma de sortie `cx` ← chroma de
      la source de la luma `2·cx`. Les lignes répétées se recopient.
    - Trace seule, 1 px, de la couleur de la piste (#13) : centres des points détectés sur `trail_s` et position
      courante (boîte bornée, interpolée entre deux points), coupée dans la boîte
      courante élargie de 16 px (Liang-Barsky), placée à `(x − gauche + 0,5) × échelle`.
      Ni boîte ni `#N`. Affichage et conversion par zones comme les extraits.
    - Encodé en **openh264**, règle fixe : les zooms ne prennent aucune session
      VideoToolbox et occupent les cœurs libres. Un échec d'openh264 sur un zoom fait
      échouer la vidéo. Mesure au calme (13 octobre, 3 passages en alternance, `release`,
      zoom `auto`) contre une variante temporaire tout en VideoToolbox, zooms compris :

      | Vidéo | Zooms openh264 | Tout VideoToolbox | CPU (openh264 / VT) | Cœurs occupés pendant le rendu |
      | --- | --- | --- | --- | --- |
      | 092 (1 zoom) | 18,38 s | 19,09 s | 130 s / 129 s | 3,5 / 3,7 |
      | 099 (46 zooms) | 39,79 s | 43,81 s | 304 s / 231 s | 7,7 / 4,0 |

      Cœurs occupés : temps CPU du processus relevé chaque seconde, après la détection
      (les 14 premières secondes, environ 8 cœurs dans les deux cas). Sur la 099, openh264
      prend 73 s de CPU de plus, sur des cœurs qui restaient libres, et rend la commande
      9 % plus courte : VideoToolbox, partagé entre extraits et zooms, plafonnait.
  - Conversion par zones (#112, décision de l'utilisateur) : l'image entière passe par
    les tables d'affichage de `Converter` ; seules les zones autour des dessins
    passent en RGBA, reçoivent le dessin, et reviennent en YUV. La zone d'une marque
    couvre sa boîte, son `#N` (deux hauteurs de chiffre au-dessus de la ligne de base)
    et sa trace, plus 4 px ; elle est calée sur des coordonnées paires (4:2:0), sa taille
    arrondie au multiple de 16 (pour le cache de contextes), bornée
    à l'image, et les zones qui se touchent sont fusionnées. Un test vérifie, image par
    image, que tout pixel dessiné tombe dans une zone. Sur la 099, les tuiles de 16 px
    touchées par un dessin font 0,56 % de l'image en moyenne, 3,8 % au plus.
    `ACCURATE_RND` et `BITEXACT` sont gardés (ils coûtent environ 2,1 ms par image
    entière, presque tout dans le sens YUV → RGBA).
  - Vidéo annotée (`--annotated`, #103) : `<nom>_annotee.mp4` (`_annotated`), toutes les
    images qui ont un `pts`, avec le dessin des extraits, aux `pts` de la source (en
    phase avec le CSV après une perte d'images). Elle se lit par tronçons
    (`Source::pieces`) : une image en erreur isole le tronçon qui va de l'image-clé
    d'avant à l'image-clé d'après, lu sur un fil ; le reste se lit sur tous les fils ;
    une image sans `pts` coupe un tronçon et n'est pas écrite. Un seul encodeur pour
    toute la vidéo, VideoToolbox d'abord, une image-clé toutes les `round(cadence)`
    images avec les deux encodeurs. Elle ne partage rien avec les extraits. Relevé
    (13 octobre, un passage) : +19,7 s sur la 092 (18,4 → 38,1 s), +23,0 s sur la 103
    (37,7 → 60,7 s). Quand VideoToolbox casse, elle est refaite entière en openh264, sur
    un fil : tout en openh264, la 092 avec `--annotated` prend 133 s.
    Le ménage de `cli` (#115) efface la vidéo annotée des deux langues et son
    `.partial`, même sans `--annotated`, avant la garde `max_tracks` et l'essai
    d'encodeur ; au-delà de la garde, le message dit « clips and annotated video
    skipped » quand elle était demandée.
  - Ordonnancement (`clips`, `work`, `schedule`, `group`, `ahead`) :
    - les travaux sont la vidéo annotée, les groupes d'extraits et les zooms (#103),
      chacun relu sur sa plage ; la vidéo annotée part la première (la plus longue),
      puis les autres par image de début, le groupe avant le zoom qui commence avec
      lui ;
    - les travaux partent dans l'ordre, sur la moitié des cœurs
      (`available_parallelism / 2`) ; chaque fil garde son `Picture`, car `Converter`
      n'est pas `Send` ;
    - dans un groupe, un fil décode en avance (file de 4 images, cases recyclées), et
      l'image dessinée est poussée une fois dans l'encodeur du groupe ;
    - un groupe réserve avant de partir sa session VideoToolbox (une seule), et la garde
      jusqu'au bout ; l'encodeur de chaque fenêtre est celui de son groupe, le même d'un
      passage à l'autre (les sessions rendues en route, essayées le 13 octobre, n'ont plus
      d'objet) ;
    - une erreur ou une panique arrête les autres groupes à l'image suivante, et les
      groupes qui attendent ne partent pas. L'erreur rendue est celle du premier groupe,
      dans l'ordre, parmi ceux qui ont échoué avant d'être arrêtés ;
    - les extraits déjà finis restent dans le dossier quand la vidéo échoue ; la ligne
      d'erreur nomme la vidéo, et le lancement suivant vide le dossier.
  - Encodeurs (`encoders`) : VideoToolbox pour chaque groupe tant qu'il n'est pas écarté,
    sinon openh264. openh264 pose une image-clé par seconde environ
    (`with_key_interval(round(cadence))`). Un fichier s'écrit dans son partiel au nom unique,
    créé par `File::create_new` (droits habituels de l'utilisateur, umask compris), puis
    renommé ; il n'en reste aucun après une erreur.
  - Reprise : quand VideoToolbox casse en cours d'écriture (hors erreur de destination),
    le groupe entier est noté, VideoToolbox est écarté pour la suite de la vidéo, et le
    groupe est refait en openh264 une fois les groupes finis. `Rendered.redone` nomme ses
    extraits avec la cause ; un échec d'openh264 fait échouer la vidéo. Une erreur de destination
    (chemin refusé, temporaire non créé, fichier non ouvert ou non renommé) arrête la
    vidéo sans reprise ; l'erreur nomme l'extrait qu'elle concerne.
  - Boîte bornée (`overlay`, #112, affichage seulement : la détection, le suivi et le
    CSV ne changent pas). Chaque côté de la boîte dessinée est borné à 4 fois la
    médiane du même côté sur les points détectés de la piste (médiane de deux valeurs
    centrales : leur moyenne). Une boîte bornée est recentrée sur le barycentre de la
    détection, puis élargie de `box_pad` comme les autres. Les points interpolés
    viennent des boîtes déjà bornées.
    - Cas qui l'a motivée : la 092, piste 13, image 6945 (3:51). La tache, étalée, a
      une boîte de 339 × 235,5 px pour 353 px de surface, quand la piste a pour médiane
      31,5 × 23,25 : on dessinait 363 × 259 px autour de rien. Bornée, la boîte fait
      126 × 93 px avant `box_pad`. La v0.1.1 dessinait la même grande boîte.
    - Mesure sur les 966 pistes de la référence des 26 vidéos (33 865 points,
      `rapports/mesure-112/boxes.py`, hors dépôt) :

      | Facteur | Boîtes bornées | Pistes | Réduites de plus de 12 px |
      | --- | --- | --- | --- |
      | 3 | 129 (0,38 %) | 84 | 11 |
      | 4 (retenu) | 33 (0,10 %) | 27 | 7 |
      | 5 | 15 (0,04 %) | 12 | 6 |

      Avec 4, les 7 boîtes réduites de plus d'un `box_pad` sont : 092 piste 13 image 6945,
      094 piste 7 image 4444, 096 piste 79 images 4817 à 4819, 096 piste 113 image 6538 et
      100 piste 34 image 2896. Les 26 autres boîtes bornées perdent au plus 10,5 px
      (médianes de leurs pistes : de 3 à 15 px).
  - Tests des extraits (#112) :
    - rapides : fenêtres et groupes ; dessin ; ordonnanceur seul (au plus 8 sessions,
      ordre, arrêt, panique relancée) ; file d'images ; `Fanout` (extraits qui se
      chevauchent tirés d'un encodage, images communes identiques, image-clé et `pts` 0
      en tête, extrait vide ou inachevé refusé) ; une fausse ouverture
      (`testing::Fake`, un vrai `Fanout` en openh264) qui compte les encodeurs et les
      sessions et casse à la n-ième image ou au renommage. Avec elle : une session et un
      encodeur par groupe, un groupe refait en openh264 après un échec de VideoToolbox
      (y compris un extrait déjà renommé), pas de reprise après une erreur de
      destination, l'erreur qui nomme l'extrait fautif, un échec d'openh264 qui arrête
      la vidéo, sans `.partial`. Un fil et quatre donnent les mêmes fichiers à l'octet en
      openh264, zooms compris ;
    - zoom (#103) : seuils stricts, `all` et `none` ; cadre en unités, ×4 au plus, borné à
      l'image, insensible à une tache étalée ; agrandissement ×4, ×3
      depuis un coin impair (Y et chroma) et ×1 qui rend l'image ; trace coupée,
      immobile ou verticale, et tout pixel de trace dans une zone ; un zoom en openh264,
      sans session VideoToolbox, de la fenêtre de son extrait, à la cadence ÷ 4 ; un
      échec d'openh264 sur un zoom nomme le zoom ; `--zoom` et ses clés dans
      `params.json` (`cli`) ;
    - mêmes zooms que la v0.1.1 (`local_command`) : décision Rust sur les pistes Rust
      contre décision v0.1.1 sur les pistes appariées de la référence, sur les 26 vidéos.
      Une seule bascule, figée : la 097, piste 88, zoomée en Rust (aire maximale 94,5 px²
      contre 117), conséquence de la divergence connue de 4872 (24 points contre 86),
      pas d'un seuil. Les zooms enregistrés de la v0.1.1 (097, 125) sont exactement ceux
      que la règle recalcule sur ses pistes ;
    - vidéo annotée (#103) : tronçons (vidéo saine d'un seul tenant, image en erreur
      isolée entre deux images-clés, image sans `pts` laissée de côté) ; `Stamps`
      refusés ; `pts` relus exacts, saut et premier `pts` non nul compris, durée de la
      dernière image ; une session VideoToolbox, reprise entière en openh264 ;
      images-clés au plus à `round(cadence)` d'intervalle avec VideoToolbox et
      openh264 ; vol fabriqué qui perd 8 images : mêmes images et mêmes `pts` que la
      source (`cli`) ; `--annotated`, clé `annotated`, ménage sans `--annotated` et
      au-delà de la garde, message de garde ;
    - boîte qui suit la cible après un trou (`cli`, `process/tests.rs`) : vol fabriqué
      de 80 images, une image-clé toutes les 8, un dixième du `mdat` écrasé (8 images
      perdues, deux pistes de part et d'autre). Sur chaque point détecté, les quatre
      bords rouges lus dans l'image tombent à 2 px au plus de la boîte attendue, et le
      carré clair est dans la boîte de détection. Un dessin décalé d'une image le fait
      échouer (essai fait une fois) ;
    - `slow_clips` (l'extrait, pistes de la référence, extraits de 380, 108, 115 et
      117 images) : chaque image servie est celle de la lecture séquentielle, et chaque
      image écrite est comparée, dans l'ordre, à l'image source servie par `pts` et passée
      par la même conversion. Seule la vidéo annotée garde les `pts` de la source, et ses
      `pts` sont comparés à ceux de la source ; les extraits et les zooms comptent leurs
      images à la cadence de l'encodeur (le quart pour les zooms), depuis 0. SSIM sur Y par blocs 8×8 disjoints, hors blocs qui touchent un dessin
      (boîte, `#N` et trace, avec une marge). Mesuré sur Mac : VideoToolbox 0,9659 à
      0,9667, openh264 0,9671 à 0,9677 ; seuils 0,963 et 0,964 (le Y de #111, par le
      filtre `ssim` de FFmpeg contre l'original, valait 0,9688 et 0,9698). Les couleurs
      relevées sur la boîte et la trace s'écrivent dans la sortie du test, sans seuil.
      Depuis #180, une seule lecture séquentielle de l'extrait sert les trois tests ; les
      `pts` et le nombre d'images de la vidéo annotée se relèvent pendant sa comparaison,
      sans seconde lecture ; chaque bloc se compare dans un tableau fixe de 64 paires,
      sans allocation (même similarité au bit près). Le calcul de la similarité a son
      test rapide, `tests/similarity.rs`.
      Binaire entier sur le M1 Max, sous le verrou : 107 s et 115 s avant, 76 s et 59 s
      après (temps CPU : 289 s et 298 s, puis 217 s et 212 s). Vidéo annotée seule :
      95 s, puis 58 s.
    - `local_clips` (`cli`, du moteur à `Source`, fils automatiques, 300 s) : sur la 103,
      la 123 et la 124, les images-clés et les images en erreur viennent de la lecture du
      moteur, et chaque groupe qui touche une plage abîmée (1, 6 et 3 groupes) est servi
      avec les pixels d'une lecture séquentielle à un fil. Aucun n'est relu depuis le
      début (`Origin::Start` : 0, épinglé). Fenêtres contre la v0.1.1, figées dans le
      test : la 125 a ses 47 fenêtres égales ; la 097 en a 149 contre 148, et ses écarts
      sont les divergences connues des pistes (0:20, fenêtre qui commence 7 images plus
      tard ; 2:42, une fenêtre devenue deux) plus un début 4 images plus tôt à 1:39,
      dans la tolérance de l'appariement.
- **`compare`** → `engine`, `media`, `tracking`, `reference`, `types` (#119) : outil de
  développement, binaire `compare`, non livré. `media` n'y sert qu'à choisir les fils du
  décodeur (`Threads::Automatic`), et, dans `slow_same_as`, à couper l'extrait (`payload::wipe`, #176).
  - `compare extract|092|092-1440|settings|local [--frames DOSSIER] [--same-as DOSSIER]` lance
    `engine::detection::detect` sur l'extrait, la 092, la 092 à 1440 pixels de travail
    (#151, nommée par son dossier pour ne pas écraser la 092), les 16 cas de réglages
    ([Réglages](#réglages)) ou les 26 vidéos (`BATDETECT_REFERENCE_DIR`,
    `BATDETECT_VIDEOS_DIR`, 6 vidéos à la fois, résultats dans l'ordre des dossiers).
    Les réglages de détection sont ceux de l'enregistrement : `config.DetectConfig` du
    `video.json`, lu par `VideoSummary::detect_config` en `DetectSettings`, puis
    `DetectConfig::try_from` (#149). Pistes et stabilité restent aux défauts, comme à
    l'enregistrement.
    Depuis la racine du dépôt, sous le verrou :
    `verrou lourd cargo run --profile ci -p compare -- local`.
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
    sens, si une vidéo diffère du passage de `--same-as`, ou si aucune vidéo n'est comparée (`Outcome::holds`). Les refus de #149 et
    #150 ont disparu avec #151 : tout réglage valide se compare. Avec `settings`,
    `--frames` écrit dans un dossier par cas (`DOSSIER/<cas>/frames.jsonl`), les cas
    partageant leur vidéo.
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
  - `--same-as DOSSIER` (#176) dit si ce passage donne les mêmes détections qu'un passage précédent écrit par `--frames DOSSIER`, hors des plages abîmées. Les pixels que FFmpeg dissimule dans ces plages varient d'une lecture à l'autre : deux passages de `main` y différaient sur 17 vidéos et 2 351 images (#159), si bien que `cmp -r` entre deux dossiers `--frames` ne prouve rien.
    - Avant la détection, `DOSSIER/<vidéo>/frames.jsonl` est relu ; absent ou illisible, la vidéo échoue sans être détectée.
    - Après la détection, les entrées sont d'abord comparées : nombre d'images, puis `pts`, image-clé et erreur de décodage de chaque image. Si elles diffèrent, les détections ne sont pas comparées.
    - Sinon, les détections le sont image par image, à l'égalité exacte, hors des plages abîmées que le moteur vient de trouver (`Detected::damage`, marge comprise). La comparaison se fait en direct plutôt qu'entre deux dossiers parce que `frames.jsonl` ne garde pas les pertes avant la première image et après la dernière, qui ouvrent aussi une plage. Les plages sont donc celles du seul nouveau passage : des pertes au bord vues par l'ancien seul donnent « differs », du côté prudent ; vues par le nouveau seul, elles écartent des images que l'ancien ne jugeait pas abîmées. Les plages instables ne sont pas écartées : elles se déduisent des détections, que l'on compare brutes.
    - Sortie : une section à la fin du rapport, `same as DOSSIER, outside the damaged spans`, puis une ligne par vidéo : `identical`, `differs` (images hors plage, dix au plus puis `… and N more`) ou `inputs differ` (première image et champs qui diffèrent, ou nombres d'images de chaque côté), suivie du nombre de plages, de leurs images et des images qui diffèrent dedans ; enfin le bilan, `N videos: identical …, differing …, inputs differ …`.
    - Le test lent `slow_same_as` coupe un vingtième de l'extrait par `media::payload::wipe`, le détecte à un fil puis en fils automatiques : identique hors de sa plage (sur lumiel : images 601 à 644, avec 27 images différentes dedans, un nombre que le test ne demande pas, parce qu'il varie d'une lecture et d'une machine à l'autre) ; une détection déplacée hors plage est listée avec sa vidéo et son image. Il exige que les fils automatiques décodent par images (`ActiveThreads::Frame`) : sinon ses deux passages liraient la vidéo de la même façon. `slow_extract` relance l'extrait avec `--same-as` sur le dossier qu'il vient d'écrire, par le chemin de la commande.
    - Mesure (#176, lumiel, profil `ci`) : `compare local --frames A`, puis `compare local --frames B --same-as A`. Les 26 vidéos sont identiques hors plages ; 17 vidéos et 1 812 images diffèrent dans leurs plages, les 17 que `cmp -r` trouve entre A et B. Code de sortie 1 aux deux passages, par la seule 120 et la seule 123, sous 98 % côté Rust (0,9668 et 0,8039) : sur lumiel, le décodeur signale en erreur les images 4589 de la 120, 3986, 4902 et 5417 de la 123, que la référence (ffprobe, Mac) ne signale pas ; les groupes de détections ajoutées qui partent de ces images (10 à 13 images chacun) sont au bord bas de l'image (y = 1 078). Pistes : celles qu'attend `local_agreement`, 966, dont 963 appariées et 958 concordantes, 3 perdues, 9 ajoutées.
  - **Identité entre systèmes.** Une seule opération de la chaîne de détection
    (`engine`, et `WorkScale` dans `types`) dépend de la libm depuis #150 : `exp`, qui
    calcule le noyau du flou une fois par vidéo.
    - Pour chaque σ de travail du banc (0,5, 0,75, 1, 1,5 et 2 ; 0 n'appelle pas `exp`)
      et 3, un test vérifie que chaque part du noyau est à plus de 10⁻⁶ d'un entier et
      que la répartition des restes ne bascule pas pour 10⁻⁶ : un ulp d'écart ne change
      aucun poids. Le noyau de σ = 1, celui des défauts, redonne le premier moteur.
    - Hors de ces σ, rien n'est garanti. Échantillon de la relecture de #150 : sur
      99 901 σ de 0,01 à 10, aucun poids ne change quand chaque `exp` varie au hasard de
      −1 à +2 ulp (8 tirages par σ) ; ce n'est pas une preuve.
    - Ce qu'on y trouve : les quatre opérations, `abs`, `midpoint`, `min`, `max`,
      `round`, les conversions et le tri par `total_cmp`. Toutes sont exactes ou fixées
      bit à bit par IEEE 754. La seule exception, `max` entre +0 et −0, n'arrive pas :
      on compare toujours au plancher du seuil ou à un pic qui le dépasse.
    - `area_to_source` multiplie par `échelle × échelle`, et non par `powi(2)`, dont Rust
      ne fixe pas la précision. Rust ne contracte jamais a × b + c en FMA.
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

- **`bench`** → `engine`, `tracking`, `reference`, `media`, `types` (#107 à #109) : le
  banc de mesure, bibliothèque et binaire `batdetect-bench` (`batdetect-bench-rs` jusqu'à
  #199, pour ne pas heurter le banc Python). Aucune dépendance de plus au workspace.
  - `random` : SplitMix64 (une graine par cible, tirée de `--seed` dans l'ordre des
    numéros, comme `SeedSequence.spawn` : le tirage d'une cible ne dépend pas du nombre
    d'essais des précédentes) et xoshiro256\*\*, écrits dans le dépôt, épinglés sur les
    sorties publiées par leurs auteurs. `uniform(a, b)` rend une valeur de `[a, b]`
    (l'arrondi peut rendre `b`), `below(n)` tire par rejet, sans biais.
  - `synthetic` : `Motion` (`pass`, `hunt`, `circle`), `Class` (amplitude finie, σ > 0),
    `Target` (au moins une position, une amplitude par position), `Scene`.
    `trajectory` (saccades, Bézier quadratique, transit), `hunting` (virages, lignes
    droites, entrée par un bord, chasse et cercle), `sampling` (tirage loin des pistes de
    référence et des cibles déjà placées, 500 essais) sont un portage du calcul, pas de
    l'ordre des tirages de numpy (règle 5). Les constantes sont celles du Python.
  - Le tirage passe par `exp`, `atan2`, `powf`, `hypot`, `sin` et `cos` de la
    bibliothèque mathématique du système : un ulp d'écart peut changer un arrondi ou un
    rejet, puis tout ce qui suit. Une graine n'est reproductible que sur un même système ;
    aucun tirage n'est épinglé en CI, les tests épinglent des propriétés.
  - `injection` : sur le plan Y, avant la détection, par `detect_injected`. Fenêtre
    `[⌊x⌋ − ⌈3σ⌉, ⌊x⌋ + ⌈3σ⌉]` bornée à l'image ; pour chaque pixel, le gris visé est
    `gris(Y) + ⌊a·exp(−d²/2σ²)⌋` borné à [0, 255], puis Y devient le Y dont le gris est
    le plus proche (`nearest_luma`, table calculée à la compilation). La table `GRAY`
    saute un gris sur six environ ; un gris sauté se rend alternativement un niveau plus
    sombre et un niveau plus clair (décision de #107). Mesure de `local_injection` sur
    les 986 patchs enregistrés (471 046 pixels, fonds Y de 80 à 215) : partie entière
    de la gaussienne identique au Python sur 100 % des pixels, changement de gris à un
    niveau près du Python sur 100 %, biais moyen +0,0010 niveau (−0,1558 en allant
    toujours vers le Y le plus sombre, +0,1558 vers le plus clair). Le contraste
    effectif est le changement de gris réalisé, un entier (la v0.1.1 enregistrait le
    changement non arrondi). `Injector` range les cibles par image (un `Vec` par image)
    et rend chaque cible avec ses observations (`Injected`).
  - `recorded` : une ligne de `bats.jsonl.gz` vers une cible et ses observations, et
    retour ; `motion` inconnu, contraste hors du vol ou hors de l'ordre des images,
    cible nommée deux fois et dernière image impossible à numéroter refusés.
  - `config` (`MatchConfig` : rayon > 0, pureté dans ]0, 1], 24 px et 0,5 par défaut ;
    classes dans l'ordre du Python), `matching` (portage de `matching.py` : distance
    ≤ rayon, une piste va à la cible qu'elle couvre le plus si la pureté atteint 0,5, à
    égalité la première rencontrée), `metrics` (visibilité : centre à 2σ des bords, hors
    zone OSD, |contraste| ≥ 8, image hors des plages ignorées du passage injecté ;
    trouvée, complétude, retard, fin précoce, fragments, fausses pistes), `median`,
    `stats` (Wilson à z = 1,96, résumé par classe), sans arrondi (règle 5).
  - `run`, `folder`, `collect` : un passage (vidéo, détections de référence et
    injectées, cibles et observations, plages abîmées, réglages de détection,
    provenance des cibles). `collect` lance la détection de référence, tire les cibles
    loin des pistes de référence (ou prend celles de `--targets`, refusées si elles
    dépassent la vidéo), puis la détection injectée, et chronomètre les deux. `folder`
    écrit et relit un dossier : `run.json` (format propre au Rust : vidéo, taille,
    cadence exacte, plages, images signalées, demi-fenêtre, réglages de détection,
    provenance avec, pour un tirage, la graine, les classes et les réglages de suivi du
    placement), `reference.jsonl.gz`, `injected.jsonl.gz` et `bats.jsonl.gz` (formats de
    la v0.1.1).
  - `report` : `bench.json` aux clés de la v0.1.1, plus `targets` (tirées, graine ; ou
    rejouées, chemin) et `seconds` (détections, évaluation, total). `sampling` vaut
    `null` pour des cibles rejouées. `code` : `git describe --always --dirty` du dossier
    courant, ou `unknown`.
  - `guard`, `against` : la garde de la bascule ([Banc de mesure](#banc-de-mesure)).
  - Commande : `batdetect-bench VIDÉO [-o DOSSIER]` avec les options du Python
    (`--per-class`, `--amplitudes`, `--sigmas`, `--motions`, `--seed`,
    `--match-radius`, les dix de détection, `--max-blobs`, `--unstable-pad`, les six de
    suivi), sans `--mode` ni `--workers`, plus `--targets FICHIER`, `--replay DOSSIER` et
    `--against DOSSIER`. Sortie dans `<-o>/<nom de la vidéo>/` (défaut `out/bench`).
    - `--targets` rejoue positions et amplitudes ; l'évaluation prend le contraste
      mesuré par le Rust. Refusé avec une option de tirage ou `--against`.
    - `--replay` réévalue un dossier écrit par le banc, sans détection, avec d'autres
      réglages de suivi, de stabilité ou d'appariement ; la détection et les cibles
      viennent de `run.json`, leurs options sont refusées. Il écrit `bench.json`
      directement dans `-o`. Avec la vidéo telle qu'elle a été tapée, `run.json` garde le
      chemin relatif au dossier de lancement : relu d'ailleurs, `video` ne situe plus le
      fichier. La demi-fenêtre y est gardée pour information : les plages abîmées sont
      déjà élargies.
    - Une sortie qui écraserait l'entrée (`--replay` dans le dossier rejoué, `--targets`
      dont le fichier est dans le dossier de sortie) est refusée avant toute détection.
    - `--against` écrit dans `<-o>/<vidéo>_<dossier enregistré>/`, pour que transit et
      chasse ne s'écrasent pas ; voir [Banc de mesure](#banc-de-mesure).
    - Les nombres négatifs sont acceptés derrière une option ; une vidéo placée après
      `--amplitudes` serait lue comme une amplitude (comme argparse) : elle se met
      avant. Les erreurs de réglage nomment l'option (`--bg-window`, pas `bg_window_s`).
    - Code 0, 1 en cas d'échec ou de garde en échec, 2 pour une erreur d'usage.
  - Tests : vidéo de 96 × 72 et 60 images fabriquée par openh264 (fond bruité), avec une
    cible lente rejouée (−60, σ 3), trouvée ; les cibles tirées y seraient presque
    jamais trouvées, le tirage se teste par ses propriétés. `tests/command.rs` lance le
    binaire (cibles rejouées, tirées, rejeu, garde contre son propre passage). La vidéo
    et la cible lente viennent de `bench::noisy`, derrière la feature `testing` du banc
    (#154) : `bench` dépend de lui-même en dev-dependency avec cette feature, pour que
    `tests/command.rs` les voie aussi.
    `local_injection`, `local_metrics` (les détections et les cibles enregistrées
    redonnent `bench.json` cible par cible : entiers exacts, flottants à 1e−12 ; résumé
    à 5e−4 pour Wilson et la complétude, 5e−3 pour les fragments, 5e−2 pour le contraste)
    et `local_guard` lisent `BATDETECT_REFERENCE_DIR/bench/{transit,hunt}`.

## Réglages

#149 donne au moteur un banc d'essai hors des réglages par défaut : 16 références de la
v0.1.1 à d'autres réglages de détection, que `compare settings` relit. #150 et #151 ont
levé les refus un à un et s'y sont mesurés.

### Vidéo 720×540

`tests/fixtures/video_092_original_3m24-4m05_720x540.mp4` (44 336 736 octets, sous LFS) :
l'extrait réduit de moitié, faite une fois le 12 octobre 2026 par le ffmpeg 9.0.2 de
Homebrew (libx264), sous le verrou :

```bash
ffmpeg -i tests/fixtures/video_092_original_3m24-4m05.mp4 -map 0:v:0 -an -sn -dn -map_metadata -1 \
  -vf scale=720:540:flags=area -c:v libx264 -preset slow -crf 16 -g 15 -keyint_min 15 \
  -sc_threshold 0 -bf 0 -threads 1 -pix_fmt yuv420p -color_range tv -colorspace bt709 \
  -color_primaries bt709 -color_trc bt709 -fps_mode passthrough -video_track_timescale 10000 \
  tests/fixtures/video_092_original_3m24-4m05_720x540.mp4
```

- Mêmes 1 230 images, même cadence (10000/333), même base de temps (1/10 000), mêmes
  images-clés (toutes les 15 images), sans image B, plage `tv` et BT.709 comme l'extrait.
- Elle ne se refait pas : un autre encodeur, ou le même sur un autre système, ne donnerait
  pas les mêmes images, et la référence ne vaudrait plus. Les tâches CI `rust-slow` et
  `rust-settings` la tirent avec l'extrait ; `rust-windows-slow` ne tire que l'extrait
  (depuis #167 pour Windows, #179 pour la séparation).
- CRF 16 garde le bruit du capteur : 44 Mo, contre 60 Mo pour l'extrait en 1440×1080.

### Les 16 cas

`tests/fixtures/reference/settings/<cas>/`, enregistrés le 12 octobre 2026 par
`tools/reference_v011.py data <vidéo> <dossier> --gzip --detect '<JSON>'` (script au tag
`python-final`), au code du tag `v0.1.1`, un cas par appel sous le verrou ; 1,6 Mo en tout (`frames.jsonl.gz`,
`tracks.jsonl`, `tracks.csv`, `video.json`). Les mêmes versions que la référence de #94.
Sur l'extrait sauf le dernier. Temps de la v0.1.1 sous `/usr/bin/time -l`, pris à
l'intérieur du verrou (CPU = utilisateur + système).

| Cas | Réglages (`--detect`) | Détections | Pistes | Python réel / CPU | Rust |
| --- | --- | ---: | ---: | ---: | --- |
| `threshold_25` | `threshold` 25 | 337 | 4 | 20,1 s / 52,7 s | comparé |
| `min_area_9` | `min_area` 9 | 373 | 5 | 18,4 s / 52,7 s | comparé |
| `max_area_400` | `max_area` 400 (écarte 6 taches de 400,5 à 488,25 px²) | 396 | 4 | 17,8 s / 52,3 s | comparé |
| `noise_factor_0` | `noise_factor` 0 | 555 | 4 | 17,0 s / 51,0 s | comparé |
| `window_2s` | `bg_window_s` 2,0 (61 images, 21 échantillons) | 1 048 | 4 | 49,3 s / 81,7 s | comparé (#151), sous 98 % : affichage coloré |
| `step_1` | `bg_step` 1 (31 échantillons) | 385 | 5 | 93,7 s / 125,8 s | comparé (#151) |
| `osd` | `osd_regions` `[[0, 0, 1, 0.07]]` | 400 | 4 | 18,4 s / 52,2 s | comparé (#151) |
| `merge_radius_0` | `merge_radius` 0 | 462 | 4 | 16,5 s / 50,4 s | comparé (#150) |
| `width_480` | `work_width` 480 | 604 | 4 | 8,3 s / 26,1 s | comparé (#150) |
| `width_720` | `work_width` 720 | 528 | 5 | 16,8 s / 31,1 s | comparé (#150) |
| `width_1440` | `work_width` 1440 | 404 | 4 | 43,1 s / 73,8 s | comparé (#150) |
| `sigma_0` | `target_sigma` 0 | 9 655 (6 561 gardées) | 30 | 11,4 s / 45,3 s | comparé (#150), sous 98 % côté Rust : affichage coloré |
| `sigma_3` | `target_sigma` 3 | 352 | 4 | 18,6 s / 52,6 s | comparé (#150) |
| `quick` | `work_width` 480, `threshold` 25, `target_sigma` 0, `noise_factor` 0 | 388 | 4 | 8,1 s / 23,0 s | comparé (#150) |
| `width_1440_osd` | `work_width` 1440, `osd_regions` `[[0, 0, 1, 0.07]]` | 403 | 4 | 39,1 s / 72,2 s | comparé (#151) |
| `size_720x540` | aucun, sur la 720×540 (travail en 720×540) | 345 | 4 | 11,6 s / 21,7 s | comparé (#150) |

- `sigma_0` est le seul cas où la v0.1.1 trouve des plages instables (2) : sans flou, le
  bruit du capteur passe le seuil par endroits.
- Mémoire de la v0.1.1 : 0,19 à 0,53 Go (`width_1440`).
- Depuis #151, les 16 cas sont comparés et aucun n'est refusé.

### Méthode

- **Niveau 0, garde bloquante : aux défauts, rien ne bouge.** `slow_extract` au bit près
  sur les trois systèmes, `rust:reference` à 14 pistes, `compare local` à 963 pistes
  appariées, 3 perdues, 9 ajoutées, la 089 à 98,19 %. Mesuré à #149 sur le Mac : les
  trois identiques ; Linux et Windows par la CI de la PR. Mesuré de même à #150 : en
  plus, `compare 092 --frames` identique octet pour octet à `main`, et `compare local`
  identique vidéo par vidéo. De même à #151, avec en plus les cas du banc que la phase
  ne touche pas identiques octet pour octet à `main`.
- **Niveau 1 : la v0.1.1 elle-même à d'autres réglages**, les 16 cas ci-dessus.
  - Critère par cas que le moteur sait faire : au moins 98 % des détections appariées
    dans les deux sens (à moins de 3 px, même image, hors plages ignorées), pistes
    appariées (début ±5 images, points ±3), chaque écart listé (cas, temps) et jugé par
    l'utilisateur.
  - Un cas sous 98 % bloque tant que sa cause n'est pas isolée, par ablation comme à #118.
  - Test de réglages : `crates/compare/tests/settings_agreement.rs` exige les 16 cas comparés sans
    échec, 98 % dans les deux sens (sauf `sigma_0` et `window_2s`, figés à 98 % sous la
    bande d'affichage, voir #150 et #151) et toutes les pistes appariées et
    concordantes. `crates/reference/tests/committed_settings.rs` relit les 16
    cas en test rapide.
- **Niveau 2, sans Python** : invariances en tests rapides. Faits à #150 : même flux
  remuxé en plage pleine (`tests/full_range.rs`), noyau et disque contre la formule,
  réduction contre un calcul brut des aires et contre l'ancienne réduction 3:2,
  barycentre d'une tache symétrique gardé à 0,1 pixel à 3:2, 2:1, 3:1 et 36/25. À
  Faits à #151 : médiane 0-1 exhaustive jusqu'à 20 et contre un tri jusqu'à 128, même
  sortie au bit près avec 1, 3 et 4 fils à 61 échantillons, tache sous un masque jamais
  vue, unités préparées sous la bande indépendantes de l'écran, gain insensible à une
  bande qui scintille sous le masque.
- **Niveau 3, le banc (#109)** : les cibles des deux bancs de la v0.1.1 rejouées par le
  Rust, la visibilité décidée par le contraste enregistré par le Python. Par classe,
  échec automatique seulement si la proportion trouvée par le Rust tombe sous la borne
  basse de Wilson de la v0.1.1 ; sinon, la liste des cibles perdues et gagnées est
  jugée par l'utilisateur ([Banc de mesure](#banc-de-mesure)).

### Mesure de #149 (12 octobre 2026, M1 Max, profil `ci`, sous le verrou)

`compare settings` : 4 cas comparés, 12 refusés, 12,7 s réelles.

| Cas | Référence | Rust | Appariées | Taux référence / Rust | Pistes appariées (concordantes) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `max_area_400` | 396 | 397 | 396 | 100 % / 99,75 % | 4 (4) |
| `min_area_9` | 373 | 373 | 373 | 100 % / 100 % | 5 (5) |
| `noise_factor_0` | 555 | 555 | 555 | 100 % / 100 % | 4 (4) |
| `threshold_25` | 337 | 337 | 337 | 100 % / 100 % | 4 (4) |

- Seul écart : `max_area_400`, 00:18.98, image 570, une détection ajoutée en (517, 999),
  4,5 px², amplitude 12,6. C'est la détection ajoutée de l'extrait aux défauts (403
  contre 402) : elle ne vient pas du réglage. Les 6 taches au-delà de 400 px² tombent
  des deux côtés (397 = 403 − 6).
- `max_area` 500, la valeur de l'issue, ne mesurait rien : la plus grande tache de
  l'extrait fait 488,25 px², et la v0.1.1 y rendait les détections des défauts octet
  pour octet. Remplacé par 400, décision de l'utilisateur.
- Aucun écart de piste.

### Mesure de #150 (12 octobre 2026, M1 Max, profil `ci`, sous le verrou)

`compare settings` : 12 cas comparés, 4 refusés (#151), 35,3 s réelles, 289 s de CPU.
Les 76 pistes des 12 cas sont appariées et concordantes.

| Cas | Référence | Rust | Appariées | Taux référence / Rust |
| --- | ---: | ---: | ---: | ---: |
| `max_area_400` | 396 | 397 | 396 | 100 % / 99,75 % |
| `merge_radius_0` | 462 | 465 | 462 | 100 % / 99,35 % |
| `min_area_9` | 373 | 373 | 373 | 100 % / 100 % |
| `noise_factor_0` | 555 | 555 | 555 | 100 % / 100 % |
| `quick` | 388 | 388 | 388 | 100 % / 100 % |
| `sigma_0` | 6 561 | 7 574 | 6 544 | 99,74 % / 86,40 % |
| `sigma_3` | 352 | 352 | 352 | 100 % / 100 % |
| `size_720x540` | 345 | 345 | 345 | 100 % / 100 % |
| `threshold_25` | 337 | 337 | 337 | 100 % / 100 % |
| `width_1440` | 404 | 405 | 404 | 100 % / 99,75 % |
| `width_480` | 604 | 605 | 603 | 99,83 % / 99,67 % |
| `width_720` | 528 | 526 | 526 | 99,62 % / 100 % |

Écarts, jugés et acceptés par l'utilisateur (12 octobre 2026) :

- `max_area_400` et `merge_radius_0`, 00:18.98 : la détection ajoutée de l'extrait aux
  défauts.
- `merge_radius_0`, 00:00.30, images 9 et 10 : 2 ajoutées en (346, 40), amplitude 29,
  dans la bande d'affichage.
- `width_1440`, 00:04.50, image 135 : 1 ajoutée en (1438, 373), au bord droit.
- `width_480`, 00:20.75, image 623 : 1 perdue en (1249, 925), amplitude 12,0 ;
  00:32.67, images 981 à 987 : 2 ajoutées en (742, 552), amplitude 13,2.
- `width_720`, 00:19.91, image 598 : 1 perdue en (484, 716), amplitude 12,0 ;
  00:23.74, image 713 : 1 perdue en (1182, 496), amplitude 12,9, un point de piste ;
  la piste reste concordante.
- `sigma_0` : sous 98 % côté Rust, cause isolée.
  - Tous les écarts sont dans la bande d'affichage des jumelles, les 7 % du haut de
    l'image (la zone OSD du banc) : 2 305 détections de la référence et 3 320 du Rust
    y tombent, 2 291 appariées. Dessous : 99,93 % et 99,98 %.
  - Mesuré sur l'image 40 : dans cette bande, la chrominance est colorée (V jusqu'à
    255) et la table `GRAY` s'écarte du `BGR2GRAY` d'OpenCV jusqu'à 56 niveaux (972
    pixels à 3 niveaux ou plus) ; ailleurs, l'écart est nul (U = V = 128). Sans flou,
    rien n'atténue cet écart.
  - Décision de l'utilisateur : la table `GRAY` reste (exception de #118), l'écart est
    documenté, `settings_agreement` fige 98 % sous la bande d'affichage. L'idée de le revoir
    avec les masques de #151 ne tient pas : `sigma_0` n'a pas de zone OSD, les masques
    ne s'y appliquent pas (voir la mesure de #151).

### Mesure de #151 (12 octobre 2026, M1 Max, profil `ci`, sous le verrou)

`compare settings` : les 16 cas comparés, aucun refusé ; les 93 pistes sont appariées et
concordantes. Les 12 cas de #150 donnent des détections identiques octet pour octet à
`main` (`--frames`), comme `compare 092 --frames` ; `compare local` est identique vidéo
par vidéo (963 / 3 / 9).

| Cas | Référence | Rust | Appariées | Taux référence / Rust |
| --- | ---: | ---: | ---: | ---: |
| `osd` | 400 | 400 | 400 | 100 % / 100 % |
| `width_1440_osd` | 403 | 404 | 403 | 100 % / 99,75 % |
| `step_1` | 385 | 386 | 385 | 100 % / 99,74 % |
| `window_2s` | 1 048 | 990 | 828 | 79,01 % / 83,64 % |
| `sigma_0` | 6 561 | 7 574 | 6 544 | 99,74 % / 86,40 % (inchangé) |

Écarts, jugés par l'utilisateur (12 octobre 2026) :

- `width_1440_osd`, 00:04.50, image 135 : 1 ajoutée en (1438, 373), au bord droit, la
  même que `width_1440` ; la marge de 4σ autour de la zone ne perd aucune détection.
- `step_1`, 00:32.67, image 981 : 1 ajoutée en (860, 480), amplitude 12,4.
- `window_2s` : sous 98 %, cause isolée. Les 220 détections perdues et les 162 ajoutées
  sont toutes dans la bande d'affichage, autour de (339, 41), groupées en 11 passages
  de 00:02.50 à 00:36.43 (amplitudes de 40 à 120) ; dessous : 100 % et 100 %. Même
  cause que `sigma_0` : la table `GRAY` face à la chrominance colorée de l'affichage,
  ici sur les chiffres qui changent pendant la fenêtre de 2 s. Décision de
  l'utilisateur : même traitement que `sigma_0`, 98 % exigés sous la bande.
- `sigma_0` : inchangé ; sans zone OSD, les masques n'y peuvent rien. Sous la bande :
  99,93 % et 99,98 %.

Temps de la 092 aux défauts, `release`, en alternance avec `main` compilé de même :
110,75 et 110,59 s de CPU contre 110,70 et 109,95 s, soit l'égalité. Une première
version, aux lignes du réseau sur le tas, coûtait 117 s (voir `threshold`).

#### La 092 à 1440 pixels de travail

Référence `tests/fixtures/reference/video_092_original_width_1440/` (`--detect
'{"work_width": 1440}'`), `compare 092-1440`, test `reference` dans
`cli/tests/reference_092.rs` (`reference_agreement.rs` avant #184).

| | Détections | Pistes | Temps réel | CPU | Mémoire |
| --- | --- | --- | ---: | ---: | ---: |
| v0.1.1 | 751 | 14 | 277 s | 521 s | 0,54 Go |
| Rust (`release`, seul) | 753, 100 % / 99,73 % | 14 appariées, 14 concordantes | 28,4 s | 195 s | 0,63 Go |

Écarts : 2 ajoutées, à 02:24.97 (image 4353, en (328, 470), amplitude 13,2) et à
03:29.31 (image 6285, en (1438, 373), au bord droit, celle de `width_1440`). Mémoire
sous la borne de 2 Go de #98. Dans `rust:reference`, les deux tests de la 092 tournent
en même temps dans le même binaire : leurs temps de détection s'y mélangent.

## Banc de mesure

Le banc porté à #100 (crate `bench`). Sa commande et sa méthode sont dans
[08](08-banc-de-mesure.md) ; ici, ce que le Rust fait de plus ou autrement que la v0.1.1,
et la garde de la bascule.

### Garde

`batdetect-bench VIDÉO --against DOSSIER` (et le test `local_guard`, qui appelle la
même fonction `bench::against::against`) rejoue un banc enregistré par la v0.1.1 :

- `DOSSIER` contient `bench.json` et `bats.jsonl.gz` (`$BATDETECT_REFERENCE_DIR/bench/`
  `transit` ou `hunt`). Un banc fait à d'autres réglages de détection, de suivi, de
  stabilité ou d'appariement est refusé, le réglage nommé.
- Un banc d'une autre vidéo est refusé (#198) : le nom de fichier de `video` dans
  `bench.json`, sans son dossier, doit être celui de `VIDÉO`, vérifié avant toute
  détection ; le nombre d'images d'`injected.jsonl.gz` doit être celui que décode le
  Rust, vérifié après les deux détections. `injected.jsonl.gz` plutôt que
  `bench-run.json` : seul le premier existe aussi dans un banc écrit par le Rust.
- Mêmes cibles des deux côtés : positions et amplitudes de `bats.jsonl.gz`, placées par
  le numéro d'image du décodeur de `media` (identique à celui d'OpenCV sur la 092 ; une
  vidéo où ils diffèrent décalerait les cibles). Le Rust injecte et détecte.
- Deux évaluations du passage Rust. `bench.json` et le tableau prennent le contraste
  mesuré par le Rust, comme tout passage. La garde prend le contraste enregistré par le
  Python, qui décide donc de la visibilité des deux côtés ; seules les plages ignorées
  propres à chaque passage injecté peuvent encore la changer.
- Par classe, sur les cibles visibles des deux côtés : leur nombre, les trouvées de
  chaque côté, la borne basse de Wilson de la v0.1.1 recalculée sur ces cibles (elle
  diffère du `found_ci95` de son `bench.json` dès qu'une cible n'est visible que d'un
  côté), les cibles perdues et gagnées (numéro et image de début). À part : les cibles
  visibles d'un seul côté, celles dont la visibilité change avec le contraste mesuré par
  le Rust, les fausses pistes et les pistes de référence de chaque côté (14 et 14 sur la
  092 : un nombre différent dirait une autre vidéo).
- **Échec automatique** (code 1) seulement si la proportion trouvée par le Rust tombe
  sous cette borne basse. Sinon la liste est jugée par l'utilisateur. Une classe dont
  toutes les cibles quittent le dénominateur ne peut pas échouer : elle n'apparaît
  que dans la liste des cibles visibles d'un seul côté.
- Sortie : `<-o>/<vidéo>_<dossier>/`, le passage, `bench.json` et `guard.txt`, le même
  texte que la sortie standard.
- `--replay` d'un passage tiré avec un `--match-radius` plus grand que celui du tirage
  rompt l'écart de 3 rayons entre cibles sur lequel repose la règle d'égalité de
  `matching` ; rien ne le refuse.

### Mesure de #109 (13 octobre 2026, M1 Max, `release`, sous le verrou)

La 092 du dépôt (LFS), réglages par défaut. Garde : aucun échec, sur les deux bancs.

| Banc | Cibles visibles des deux côtés | Trouvées v0.1.1 | Trouvées Rust | Perdues | Gagnées | Visibilité changée par le contraste Rust |
|---|---|---|---|---|---|---|
| transit (9 classes × 30) | 253 | 148 | 148 | 0 | 0 | 0 |
| chasse (12 classes × 60) | 584 | 456 | 455 | 2 | 1 | 5 |

- Chasse : perdues la 19 (hunt −60 σ 1,5, image 3 221 ; 45 trouvées sur 53 pour une
  borne de 0,752) et la 543 (circle −30 σ 1,5, image 6 935) ; gagnée la 555 (circle −30
  σ 1,5, image 7 948). Visibilité changée par le contraste mesuré par le Rust : 84, 120,
  341, 602 et 665. Aucune cible visible d'un seul côté ; aucune fausse piste ; 14 pistes
  de référence de chaque côté.
- Avec son propre contraste, le Rust évalue 589 cibles de chasse (131 non visibles,
  contre 136) et en trouve 455 ; le transit est identique (253, 17, 148). Pistes du
  passage injecté : 163 et 514 (515 pour la v0.1.1).
- Temps d'un passage (deux détections de la 092 et l'évaluation), contre la v0.1.1
  (README de la référence, non relancée) :

  | Passage | Temps réel | CPU | Mémoire |
  |---|---|---|---|
  | Rust, banc tiré par défaut | 31,6 s | 222,5 s | 0,55 Go |
  | Rust, garde transit | 31,4 s | 221,1 s | 0,55 Go |
  | Rust, garde chasse | 32,4 s | 222,2 s | 0,60 Go |
  | v0.1.1, banc transit | 243 s | 689 s | |

  L'évaluation seule prend moins de 10 ms : un `--replay` est immédiat.

### Mesure de la bascule (#198, 15 octobre 2026, M1 Max, sous le verrou)

`main` @ f364698, plus la vérification de la vidéo de `--against`, sans effet sur la
détection, le suivi ou le rendu. Les quatre mesures de la garde de #106 :

- **`rust:reference`** : vert, 5 tests, 96 s réels. La 092 donne 14 pistes, toutes
  appariées et concordantes ; 780 détections de référence toutes appariées, une ajoutée
  à 03:43.80. À 1440 pixels : 14 pistes concordantes, deux détections ajoutées
  (02:24.97, 03:29.31), comme à #151.
- **`compare local`** (profil `ci`) : 966 pistes, 963 appariées (958 concordantes), 3
  perdues, 9 ajoutées ; la 089 à 98,19 % (référence) et 98,46 % (Rust) ; aucune vidéo
  sous 98 %. Les 17 écarts de pistes sont ceux de #123, à la même image : 094 1:42.21,
  1:43.08 ; 097 0:20.38, 0:20.62, 2:42.26, 2:43.39 ; 098 0:05.86, 0:41.46 ; 099 0:14.29,
  0:16.95, 0:17.92, 0:19.42, 0:27.91 ; 100 1:36.25, 1:36.38 ; 126 1:31.72, 1:32.45.
  Aucun écart par rapport au lot 7. 736 s réels, 4 598 s de CPU.
- **Garde du banc** (`batdetect-bench-rs` en `release`, la 092 du dépôt) : passe sur les
  deux bancs. Transit : 253 cibles visibles des deux côtés, 148 trouvées de chaque côté,
  0 perdue, 0 gagnée. Chasse : 584 cibles, 456 trouvées par la v0.1.1 et 455 par le Rust ;
  perdues la 19 (image 3 221) et la 543 (image 6 935), gagnée la 555 (image 7 948) ;
  visibilité changée par le contraste du Rust : 84, 120, 341, 602, 665. 14 pistes de
  référence et aucune fausse piste de chaque côté. C'est la liste de #109. Temps : 29,6 s
  réels et 218,6 s de CPU pour le transit, 0,54 Go ; 29,2 s et 219,4 s pour la chasse,
  0,60 Go.
- **La 092 par la commande** (`batdetect-rs` en `release`, détection, suivi, extraits,
  zoom et image résumé, `LANG=fr_FR.UTF-8`), machine calme selon `MESURES.md`, sans mise
  en veille : 19,0 s réels, 128,5 s de CPU, 1,09 Go de mémoire résidente au maximum
  (1,05 Go d'empreinte). 14 pistes, mêmes nombres de points que la v0.1.1. Les temps de
  début sont plus tôt de 0,01 à 0,02 s, moins d'une image. La v0.1.1 donnait 135,4 s
  réels, 436,6 s de CPU et 0,70 Go.

### Validation à l'aveugle (#198, 15 octobre 2026)

Jugée par l'utilisateur (décision du 10 octobre 2026 : sans Manon), sur les sorties de
f364698 en `release`, `LANG=fr_FR.UTF-8`, contre `$BATDETECT_REFERENCE_DIR/outputs/fr/`.
Deux fichiers par cas, A et B tirés au sort, ouverts l'un après l'autre en plein écran
dans QuickTime. Une question : « la cible se voit-elle aussi bien ? ». Le texte dessiné
(`#N`, Roboto contre la police d'OpenCV) trahit la provenance ; il n'était pas à juger.

| Cas | Vidéo | Piste (début v0.1.1) | Fichier | Verdict |
|---|---|---|---|---|
| bordure avec zoom | 125 | 0:28.41 | zoom | aussi bien |
| passage rapide (1 926 px/s) | 092 | 3:48.29 | extrait | aussi bien |
| piste courte (6 images) | 097 | 1:54.63 | extrait | aussi bien |
| piste lente | 092 | 3:29.61 | extrait | aussi bien |
| boîte interpolée (30 images) | 097 | 1:48.54 | extrait | aussi bien |
| zoom ×4 | 125 | 0:47.12 | zoom | aussi bien |

- L'utilisateur voit peu ou pas de différence entre A et B.
- La piste courte retenue d'abord, 097 2:42.26, est une divergence connue (24 points
  contre 86) : remplacée par 1:54.63, la même des deux côtés.
- Images résumé, à découvert, côte à côte : la 097 (148 pistes, plages abîmées et
  instables ; une image en v0.1.1, huit en Rust) et la 089-091 (plages instables dans
  la 089 et la 091 ; deux tranches de 10 min, chacune en deux images en Rust). Rien à
  redire.

### Tests figés (#198)

Les cas validés deviennent des empreintes, jamais celle d'un mp4 (VideoToolbox n'est
pas reproductible d'une machine à l'autre) :

- **Extraits et zooms** : les images dessinées que `render` pousserait à l'encodeur
  (`YuvFrame`, boîtes, traces et numéros compris), récupérées par `render::drawn::drawn`
  (feature `testing`), sans encodeur. Une empreinte `of_sequence` par fichier, avec son
  nombre d'images. Les pistes sont celles du Rust, pas celles de la référence : ce sont
  elles que l'utilisateur a jugées. Elles viennent de `batdetect::process::tracked`
  (timeline, exclusion, suivi), que `process_detected` appelle aussi : un changement de
  la commande entre la détection et le rendu se voit dans ces tests.
  - `cli/tests/slow_frozen.rs`, sur l'extrait LFS, dans la CI des trois systèmes : la
    piste lente (380 images) et le passage rapide (108 images). Sur l'extrait, elles
    portent les numéros `#1` et `#2` au lieu de `#11` et `#12` : seuls les numéros
    diffèrent des images jugées. tiny-skia (avec SIMD) et swscale (`BITEXACT`) n'avaient
    jamais été comparés au pixel près d'un système à l'autre : la CI de la PR #208 a
    confirmé les mêmes empreintes sous macOS, Linux et Windows.
  - `cli/tests/local_frozen.rs`, jamais en CI (vidéos hors dépôt) : les six cas sur
    leurs vidéos entières, exactement les images jugées.
- **Images résumé** : les pixels de chaque PNG écrit par `batdetect` en français
  (`of_pixels`), la 097 et la 089-091, dans `cli/tests/local_frozen.rs`. Ce test ne
  peut pas vivre dans `summary` : le fond est la médiane calculée par la détection, et
  `summary` n'a ni FFmpeg ni moteur.
- Un changement voulu du dessin, de la détection ou du suivi casse ces empreintes : on
  les refige en le disant, après un nouveau jugement à l'œil si le dessin a changé.

## Référence v0.1.1

Format, emplacement et versions de la référence que le Rust doit égaler : README de
`tests/fixtures/reference/` (#94). La crate `reference` la lit ; le test local
`crates/reference/tests/local_reference.rs` relit les 26 vidéos (966 pistes) et les
apparie chacune à elle-même. Les 16 cas de réglages de #149 sont dans
`tests/fixtures/reference/settings/` ([Réglages](#réglages)) ;
`crates/reference/tests/committed_settings.rs` les relit.

Attendus de l'image résumé (#104) : `tools/summary_v011.py` (au tag `python-final`) relit `tracks.jsonl` et
`video.json` d'un enregistrement, sans relancer la détection, avec la même garde que
`tools/reference_v011.py`. Tranches, couleurs et horloges viennent des fonctions v0.1.1
(`split_by_period`, `assign_colors`, `format_clock`, `format_duration`) ; l'en-tête est
recomposé en lignes structurées selon `header_lines`, sans les textes. Ceux de l'extrait
et de la 092 sont dans `crates/summary/tests/fixtures/` ; ceux des 26 vidéos dans
`summary/` du dossier de référence local, à côté de `data/`.
