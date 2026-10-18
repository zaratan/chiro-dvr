# Le Rust : construction, charte et tests

Ce document s'adresse à qui modifie le code Rust de `batdetect` : chaîne de construction, licences, charte, tests, encodage, puis le détail de chaque crate ([qui tient quoi](README.md#qui-tient-quoi)).
La carte courte des crates est dans [CLAUDE.md](../CLAUDE.md), la logique de la détection dans [10](10-logique-de-detection.md) et la valeur de chaque réglage dans [02](02-methode.md).

Le Rust doit détecter au moins aussi bien que la v0.1.1, dernière version en Python (tag `python-final`), sans la reproduire bit à bit.
Les détections et les pistes se comparent par appariement, avec des tolérances, et chaque écart se juge à l'œil.
Les comparaisons à la v0.1.1 qui restent dans ce document expliquent une règle ou un test encore en vigueur ; le reste de l'histoire de la réécriture est dans git et le [CHANGELOG](../CHANGELOG.md).

## Construction

```bash
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:ffmpeg   # construit FFmpeg une fois par poste
mise run rust:static      # fmt, clippy, règles de la charte, cargo deny, cargo shear
mise run rust:coverage    # tests rapides sous cargo-llvm-cov, couverture ≥ 95 %
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:slow       # l'extrait LFS
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:settings   # les 16 cas de réglages du banc d'essai
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:reference  # la 092 entière
```

`verrou lourd` est le verrou de mesure des agents de ce poste (`MESURES.md`, hors dépôt) ; ailleurs, ces commandes se lancent sans lui.

- **Toolchain** : `rust-toolchain.toml` fixe Rust 1.99.0 (clippy, rustfmt, llvm-tools), édition 2024.
  C'est la seconde source de versions du dépôt, à côté de `mise.toml`, et une montée de version se fait dans un changement à part.
- **FFmpeg 7.1.1 exactement, openh264 2.6.0.** `packaging/ffmpeg-versions` donne les versions, les adresses et les sha256.
  La signature de l'archive FFmpeg a été vérifiée avec la clé publiée sur ffmpeg.org, et la somme d'openh264 est celle de la formule Homebrew.
  nasm 2.16.03 n'est construit que sur x86_64, pour l'assembleur, et n'entre pas dans le binaire.
  La 7.1.1 est la version de la roue OpenCV de la v0.1.1.
  Avec elle, le prototype lisait les mêmes images qu'OpenCV (1 230 sur 1 230 sur l'extrait), alors que FFmpeg 9.0.2 s'en écartait jusqu'à 3 niveaux (rapport d'exploration, branche `exploration/rust`).
  La comparaison à la référence v0.1.1 part donc d'une même lecture, et l'épinglage rend la lecture des drapeaux d'erreur (`decode_error_flags`) reproductible d'un poste à l'autre.
- **Ligne `configure`** : en lecture, le démultiplexeur mov, les parsers et décodeurs h264 et hevc, le protocole file et swscale ; en écriture, l'encodeur libopenh264, le muxeur mp4 et, sur macOS, VideoToolbox (décodage et encodeur `h264_videotoolbox`).
  `--disable-autodetect` écarte les bibliothèques du système.
  `--enable-pic` est explicite (#220) : sans lui, `configure` n'active le PIC que si le compilateur en produit par défaut.
  Le clang de macOS et le gcc d'Ubuntu le font (PIE par défaut), pas le gcc de Rocky 10, alors que Rust lie en PIE sous Linux et que l'édition de liens échoue sur des `.a` non PIC.
- **Script** : `packaging/ffmpeg-unix.sh`, en bash portable (bash 3.2 de macOS compris).
  - Il construit dans un dossier temporaire du cache, puis le déplace par `mv`.
    Si une construction concurrente a déjà posé la cible, `mv` refuse et la cible existante est gardée.
  - **Cache partagé entre worktrees** : `${XDG_CACHE_HOME:-~/.cache}/batdetect/ffmpeg/<empreinte>-<os>-<arch>/`.
    L'empreinte est un sha256 du script et des seules clés `FFMPEG_`, `OPENH264_` et `NASM_` de `ffmpeg-versions` : le script Windows n'y touche pas.
  - Chaque worktree a un lien `.ffmpeg`, ignoré par git, vers sa cible.
    Les `.pc` sont réécrits en `${pcfiledir}/../..` : le cache marche à travers le lien, à n'importe quel chemin.
  - `--fingerprint` imprime l'empreinte (clé du cache CI).
    `--link` (`mise run rust:link`) pose le lien et échoue si le cache manque : `mise run check` ne construit jamais FFmpeg.
- **Liaison** :
  - `.cargo/config.toml` force `PKG_CONFIG_LIBDIR` vers `.ffmpeg/lib/pkgconfig` et vide `PKG_CONFIG_PATH`, `CPATH` et `C_INCLUDE_PATH`, pour que le FFmpeg 9 de Homebrew, ses `.pc` et ses en-têtes ne puissent pas être pris.
    `cargo` seul suffit, sans variable d'environnement.
  - Nos `.pc` mettent toutes les dépendances dans `Libs:`, et le dossier ne contient que des `.a` : la liaison est statique.
  - Pas de `FFMPEG_DIR`, dont la route dans ffmpeg-sys-next 9.0.1 oublie openh264 et la bibliothèque C++.
    Pas de feature `static` de ffmpeg-next non plus : elle ajoute 16 frameworks macOS figés, dont QTKit (absent en arm64, l'éditeur de liens le signale), AppKit, OpenGL et OpenCL, inutiles.
  - Ce que le binaire lie en dynamique (`otool -L`, 16 octobre 2026) :
    - sur macOS, libc++, libSystem, libiconv (tirée par la bibliothèque standard de Rust), libobjc et six frameworks (Foundation, CoreFoundation, VideoToolbox, CoreMedia, CoreVideo, CoreServices) ; libobjc et Foundation viennent de ctrlc, par objc2 ;
    - sous Linux, libstdc++, libgcc_s, libc et libm.

    `packaging/check.sh` contrôle cette liste sur le binaire livré ([Empaquetage](#empaquetage)).
- **Version liée vérifiable** : FFmpeg est construit avec `--extra-version=<empreinte>`.
  `batdetect --version` affiche donc `FFmpeg 7.1.1-<empreinte>`, et un test compare cette empreinte à la cible de `.ffmpeg`.
- **Reconstruction** : cargo ne suit pas les `.a`.
  - La rlib de ffmpeg-sys-next n'en contient aucune copie (mesuré le 11 octobre 2026 sous macOS : 5 516 320 octets, dont 5 515 160 de métadonnées, les liaisons de bindgen, et un objet de 336 octets sans symbole défini), alors que `libavcodec.a` pèse à elle seule 13 Mo.
  - Le build script ne déclare que des `rustc-link-lib=avcodec`, sans `static=`, avec la recherche dans `.ffmpeg/lib` : les `.a` sont lus à l'édition de liens de chaque binaire.
    Ni ce script ni l'édition de liens ne se refont quand les `.a` changent, et un FFmpeg changé laisserait des binaires liés à l'ancien, avec des liaisons tirées des anciens en-têtes.
  - Quand `.ffmpeg` change de cible, le script lance donc `cargo clean -p ffmpeg-sys-next` dans chaque profil, `target/llvm-cov-target` compris.
    Le nettoyage passe avant le changement du lien : s'il échoue, le passage suivant le refait.
  - Après une montée de version de FFmpeg, on fait un `cargo clean` complet, car les liaisons de bindgen changent.
  - Un arrêt brutal (SIGKILL, délai de la CI dépassé) peut laisser un dossier `.build.*` dans le cache : il se supprime à la main.
- **Cible processeur** :
  - macOS arm64 garde le défaut (`apple-m1`) pour le Rust ;
  - Linux x86_64 compile le Rust en `x86-64-v2` (`.cargo/config.toml`), sans quoi le min/max u16 du réseau de médiane ne serait pas vectorisé (il demande SSE4.1) ;
  - FFmpeg choisit ses routines en assembleur à l'exécution, mais son code C et celui d'openh264 sont compilés pour une cible épinglée dans `ffmpeg-unix.sh`, donc dans l'empreinte (#310) : `-march=x86-64-v2` sous Linux, `-mcpu=apple-m1` sous macOS, par `--extra-cflags` pour FFmpeg et `CFLAGS_OPT="-O3 …"` pour openh264, dont le Makefile met `CFLAGS` dans ses compilations C, C++ et assembleur.
    Sans cible épinglée, ils suivent celle du compilateur du poste, `x86-64-v3` pour le gcc de Rocky 10 (`gcc -Q --help=target`) : le FFmpeg de lumiel s'arrêtait sur un `vmovsd` sous Intel SDE en Nehalem (#251), avec la même empreinte que celui de la CI ;
  - Windows n'épingle rien : MSVC compile en SSE2 par défaut sur x64, quel que soit le poste ;
  - ne jamais poser `RUSTFLAGS`, qui remplacerait ce réglage.
- **macOS 14 au minimum** : `MACOSX_DEPLOYMENT_TARGET=14.0` vaut pour openh264, FFmpeg et rustc.
  Mesuré : `minos` 14.0 sur les 411 objets et sur le binaire.
- **Profils** : `release` a `lto = "fat"`, `codegen-units = 1` et `panic = "unwind"`.
  `ci` hérite de `release`, sans LTO, en 16 unités, avec `overflow-checks` et `debug-assertions`.
  Les tests tournent en `ci`, les temps se mesurent en `release`.
- **Construction de FFmpeg et d'openh264** (10 octobre 2026) : 42 à 50 s et un cache de 26 Mo sur le Mac M1 Max, 224 s et 43 Mo sous Linux amd64 émulé (OrbStack).

### Empaquetage

```bash
mise run package          # binaire release, puis dist/batdetect-<cible>.tar.gz (zip sous Windows)
mise run package:check    # contrôles du livrable sur cette archive
```

- **Archives** (#223) : `batdetect-darwin-arm64.tar.gz`, `batdetect-linux-x86_64.tar.gz` et `batdetect-windows-x86_64.zip`.
  - Elles contiennent, à la racine, le binaire, `LICENSE`, `NOTICE` et `licenses/`.
  - Le nom n'a pas de numéro de version, parce que la formule Homebrew en dépend.
  - `packaging/package.sh` lit la cible dans la ligne `host:` de `rustc -vV`, pas dans `uname`, qui vaut `UCRT64_NT-…` sous MSYS2.
    Les scripts sont en bash, puisque `mise` ne tourne pas sous Windows.
  - Sans `BATDETECT_VERSION`, il prend la version de `[workspace.package]` dans `Cargo.toml`, que `--version` imprime : `mise run package:check` contrôle ainsi en local la même version qu'en CI.
- **Contrôles** : `packaging/check.sh ARCHIVE [VERSION]` joue tous les contrôles, puis échoue à la fin si un seul a échoué.

  | Contrôle | Ce qu'il mesure |
  |---|---|
  | C1 | l'archive contient le binaire, `LICENSE`, `NOTICE` et `licenses/`, rien d'autre |
  | C2 | aucun lien dynamique hors d'une liste mesurée par système : `otool -L`, `readelf -d` (lignes NEEDED), `dumpbin -dependents` |
  | C3 | `minos` ≤ 14.0 sous macOS ; sous Linux, `GLIBC_` ≤ 2.35, `GLIBCXX_` ≤ 3.4.30, `CXXABI_` ≤ 1.3.13 et `GCC_` ≤ 12.0.0, la bibliothèque de l'Ubuntu 22.04 ; sous Windows, versions du système et du sous-système de l'en-tête PE ≤ 10.0 (`dumpbin -headers`), celle de Windows 10, que Windows 11 annonce aussi ; C3-cpu, sous Linux et Windows : l'extrait tourne sous Intel SDE en mode Nehalem (`-nhm`, qui couvre x86-64-v2) et donne le CSV de C6 |
  | C4 | `--version` donne `batdetect <VERSION>`, puis le FFmpeg épinglé, avec l'empreinte du script de construction et la LGPL |
  | C5 | le bloc `Rust crates:` de `NOTICE` égale `cargo tree -p batdetect -e normal,no-proc-macro` sur les trois cibles de `packaging/about.toml` ([Licences](#licences)) |
  | C6 | le CSV de l'extrait, en anglais, est identique à l'octet à `tests/fixtures/rust/video_092_original_3m24-4m05/tracks.csv` |
- **Preuves négatives** : chaque contrôle échoue seul sur une archive altérée.
  - Le 16 octobre 2026 sur le Mac : un fichier en trop et `NOTICE` retiré font échouer C1 ; le binaire remplacé par `/usr/bin/curl`, C2, C3, C4 et C6 ; une construction sans `BATDETECT_VERSION`, C4 ; un octet changé dans le CSV, C6.
  - Un sous-système 11.0 posé par `editbin` sur une copie de l'archive Windows (CI de la PR #261) donne `FAIL C3-floor: subsystem 11.00, os 6.00, above 10.00`, puis l'échec de C4 et C6, le chargeur refusant le binaire (`Exec format error`).
    L'archive intacte donne `ok C3-floor (subsystem 6.00, os 6.00)`.
  - C3-cpu, le 17 octobre 2026 : un binaire construit en `target-cpu=x86-64-v3` donne `FAIL C3-cpu: batdetect failed on the extract: TID 0 SDE-ERROR: Executed instruction not valid for specified chip (NEHALEM): … vzeroupper`.
    On l'a vu sur lumiel, dans un conteneur `ubuntu:26.04` dont la glibc accepte Nehalem, puis en CI sur Ubuntu et Windows (PR #309) ; l'archive Linux de `main` donne `ok C3-cpu` dans `ubuntu:22.04`.
    `--version` passe sur ce binaire : le code fautif ne tourne qu'au traitement, d'où l'extrait.
- **Intel SDE** (C3-cpu, #251) : `packaging/sde.sh` télécharge le kit de `packaging/sde-versions` (10.13.1) par son URL directe, vérifie le sha256 épinglé, l'extrait dans `${XDG_CACHE_HOME:-~/.cache}/batdetect/sde/` et imprime le chemin de `sde64` ou de `sde.exe`.
  SDE présente le CPUID de Nehalem, dont FFmpeg et openh264 prennent alors les routines, et il arrête le programme sur toute instruction que Nehalem n'a pas.
  Il n'émule que ce qui manque à l'hôte : l'extrait prend 19 s sur lumiel, avec les pistes de C6 à l'octet.
  - Licence : Intel Simplified Software License (février 2020, `Licenses/LICENSE.pdf` du kit), usage et redistribution sans modification, sans clause sur la CI.
    L'utilisateur l'a acceptée le 17 octobre 2026, la page de téléchargement d'Intel demandant un clic.
  - Sous Ubuntu, Pin demande `kernel.yama.ptrace_scope=0` (`README.txt` du kit) : une étape de `package.yml` le pose par `sudo sysctl` avant `check.sh`.
  - Le kit (36 Mo) n'a pas de cache CI : `sde.sh` fait confiance à un kit déjà extrait sans recalculer son sha256, et un cache restauré contournerait l'épinglage.
    Une release dépend donc du miroir d'Intel, et son absence échoue en clair (`could not fetch Intel SDE`).
  - SDE ne tourne pas sous macOS arm64, ni utilement sous Rocky 10, dont la glibc exige x86-64-v3 (`sde64 -nhm -- /bin/true` y donne « CPU does not support x86-64-v3 ») : la preuve se fait dans les tâches `package / build` d'Ubuntu et de Windows.
    Sur un tel hôte, C3-cpu échoue sur une sonde (`sde64 -nhm -- /bin/true`) qui le dit, plutôt que sur l'extrait, et `mise run package:check` y passe au rouge.
- **Mesure** (16 octobre 2026, Mac, `release`) : l'archive macOS pèse 3 511 951 octets, pour un binaire de 8 130 992 octets.
  Le CSV des profils `ci` et `release` est identique à l'octet.

### Windows

La cible `x86_64-pc-windows-msvc` n'est construite que par la CI (tâches `rust-windows` et `rust-windows-slow`), parce que `mise` n'y tourne pas.
Le binaire Windows se construit et s'empaquette dans `package.yml` ([Empaquetage](#empaquetage)) ; la construction sur un poste Windows est l'objet de #226.

- **Script** : `packaging/ffmpeg-windows.sh`, en bash sous MSYS2, avec MSVC dans le `PATH`.
  - Il lit le même `ffmpeg-versions`, plus deux clés `WINDOWS_NASM_*` pour le binaire officiel de nasm 2.16.03.
    Ce préfixe reste hors du filtre de l'empreinte Unix, qui ne change donc pas.
  - Son empreinte hache le script et les clés `FFMPEG_`, `OPENH264_` et `WINDOWS_`.
  - `pinned`, `sha256`, `fetch`, `relocate` et `clean_bindings` sont recopiées du script Unix, parce qu'un fichier commun changerait `ffmpeg-unix.sh`, donc son empreinte et son cache.
    Deux sont adaptées : `fetch` est coupée en `download` et `fetch`, et `relocate` traite le chemin sous ses deux formes, POSIX et mixte.
    Un correctif du script Unix se reporte donc à la main.
  - Seule option : `--fingerprint`.
- **Même bibliothèque C partout** : `/MT` pour openh264 (`make OS=msvc`, avec son `CFLAGS_OPT` redonné sans `-Zi -FS`) et pour FFmpeg (`--toolchain=msvc --extra-cflags=-MT`), `+crt-static` pour le Rust (`.cargo/config.toml`, avec `target-cpu=x86-64-v2`).
  Le binaire ne demande donc ni `vcruntime140.dll` ni l'UCRT.
- **Liaison** : deux pièges que le script corrige après `make install`.
  - FFmpeg nomme ses archives `libavcodec.a` même sous MSVC, alors que rustc passe `avcodec.lib` à `link.exe` : le script renomme `libX.a` en `X.lib`.
  - Le `configure` de FFmpeg écrit les bibliothèques système des `.pc` à la manière de MSVC (`bcrypt.lib`, `-libpath:`).
    pkg-config-rs ignore en silence un `bcrypt.lib` nu, et lit `-libpath:` comme `-l` suivi de `ibpath:` : le script les réécrit en `-lbcrypt` et `-L`.

  La liaison reste statique sans rien corriger : sans la feature `static`, ffmpeg-sys-next appelle pkg-config-rs en `statik(false)`, qui demande ces bibliothèques sans type, donc en dylib, et sous MSVC `link.exe` prend aussi bien une archive.
- **Autres pièges** :
  - Sous MSVC, ffmpeg-sys-next essaie vcpkg avant pkg-config, et `msvc-dev-cmd` (vcvars) définit `VCPKG_ROOT` : la tâche pose donc `VCPKGRS_DISABLE=1`.
  - Le `link.exe` de coreutils (MSYS2) est renommé avant toute construction, et le script refuse un `link` qui n'est pas celui de Microsoft.
  - MSYS2 réécrit en chemin tout argument qui commence par `/`.
    Les options de MSVC s'écrivent donc `-x`, et `cmd /c` passe par `MSYS2_ARG_CONV_EXCL='*'`.
  - Le cache se donne en chemin mixte (`D:/a/...`), que le script normalise par `cygpath`.
  - `LIBCLANG_PATH` pointe vers le LLVM de l'image, pour bindgen.
  - openh264 se compile sans `-Zi -FS`, donc sans informations de débogage : avec `-Zi`, chaque édition de liens affichait 65 avertissements LNK4099, un par objet dont le `.pdb` n'est pas installé, sans effet sur le binaire mais remontés par rustc.
- **Lien `.ffmpeg`** : une jonction NTFS (`mklink /J`), qui ne demande pas de droits.
  `fs::read_link` la lit : le test d'empreinte de la version liée vaut aussi sous Windows.
- **Gardes du script** :
  - LGPL 2.1+ et `HAVE_W32THREADS 1` dans `config.h` ; le multifil n'est gardé qu'à la compilation, aucun test ne vérifie que le décodage tourne sur plusieurs fils ;
  - pkg-config, interrogé sans `--static` comme le fait cargo, ne rend ni antislash, ni `.lib` nu, ni `-libpath:` ;
  - aucune archive `.a` ne reste ;
  - chaque `.lib` demande `LIBCMT` et jamais `MSVCRT` (`dumpbin -directives`).
- **Fins de ligne** : l'image Windows extrait le texte en CRLF (`core.autocrlf`).
  `.gitattributes` force `eol=lf` pour les scripts `.sh`, `ffmpeg-versions`, `sde-versions`, `tests/fixtures/reference/**`, et pour `LICENSE`, `packaging/NOTICE` et `packaging/licenses/**`.
  Les textes du zip sont ainsi ceux des archives macOS et Linux.
- **Mesures** (11 octobre 2026, runner `windows-latest`, image `windows-2025-vs2026` 20260925.250.1, MSVC 14.51) : FFmpeg et openh264 se construisent en 3 min 15 s ; à chaud (caches LFS, `target/` et FFmpeg restaurés), la tâche prend 1 min 53 s.
  Le binaire dépend de `kernel32`, `ntdll`, `bcrypt` et `api-ms-win-core-synch-l1-2-0`.

## Licences

Le livrable est sous **MIT** pour le code de batdetect (`LICENSE`) et sous **LGPL 2.1 ou ultérieure** pour FFmpeg (décision du 16 octobre 2026, #224).
Il porte aussi openh264 (BSD-2), Roboto (OFL-1.1), la bibliothèque standard de Rust et 83 crates tierces, chacun sous sa licence.

`packaging/NOTICE` décrit le binaire composant par composant, et les textes de licence sont dans `packaging/licenses/`.
`package.sh` copie l'un et l'autre dans chaque archive ([Empaquetage](#empaquetage)).
Chaque texte vient de la source épinglée du composant :

| Fichier | Source |
|---|---|
| `FFmpeg-LGPL-2.1.txt` | `COPYING.LGPLv2.1` de `ffmpeg-7.1.1.tar.xz` |
| `openh264-BSD-2-Clause.txt` | `LICENSE` de l'archive openh264 2.6.0 |
| `Roboto-OFL-1.1.txt` | copie de `crates/draw/fonts/OFL.txt` |
| `Rust-MIT.txt`, `Rust-Apache-2.0.txt` | `LICENSE-MIT` et `LICENSE-APACHE` de `rust-lang/rust` au tag 1.99.0 |
| `crates.txt` | généré par `mise run notice` |

- **FFmpeg 7.1.1 en LGPL 2.1 ou ultérieure.**
  Il est construit sans `--enable-gpl` ni `--enable-nonfree`, et `libavcodec` annonce « LGPL version 2.1 or later », que `--version` affiche et que `check.sh` contrôle (C4).
  Lié en statique, il oblige à fournir de quoi relier : chaque release joint `ffmpeg-7.1.1.tar.xz` (#223), et les sources de batdetect du tag sont publiques.
  `NOTICE` dit comment relier : changer `FFMPEG_URL` et `FFMPEG_SHA256` dans `ffmpeg-versions`, lancer le script de construction, puis `cargo build`.
- **openh264 2.6.0 en BSD-2.**
  Il est construit depuis les sources, jointes à la release, et n'a donc **pas** la licence de brevets H.264 que Cisco n'accorde qu'à ses propres binaires.
  Brevets acceptés le 16 octobre 2026 ; `NOTICE` dit que les codecs H.264 et HEVC peuvent être couverts par des brevets et qu'aucune licence de brevet n'est fournie avec ce binaire.
- **Roboto en SIL Open Font License 1.1** (#101), embarquée dans `draw` : `crates/draw/fonts/Roboto.ttf`, police variable `wdth` et `wght`, avec sa licence à côté (`crates/draw/fonts/OFL.txt`, recopiée dans `packaging/licenses/`).
  Provenance : dépôt `google/fonts`, fichier `ofl/roboto/Roboto[wdth,wght].ttf`, commit `6183fc0` du 27 janvier 2026, sha256 `d7598e12…872a134`.
  Pas de nom de police réservé.
- **Bibliothèque standard de Rust** : MIT OU Apache-2.0.
  Son avis complet, dépendances comprises, est `share/doc/rust/COPYRIGHT-library.html` de la toolchain (1,5 Mo de HTML), qui n'est pas joint à l'archive.
- **Crates livrées** : l'union de `cargo tree -p batdetect -e normal,no-proc-macro --target <triple>` sur les trois triples de `packaging/about.toml`, crates du workspace exclues.
  Mesurée le 16 octobre 2026 : 83 crates, dont 78 sous macOS, 74 sous Linux et 75 sous Windows.
  - **`mise run notice`** croise cette liste avec le JSON de `cargo-about`, épinglé dans `mise.toml`.
    Il réécrit le bloc `Rust crates:` de `NOTICE`, une ligne `- nom version, licence` par crate, et `licenses/crates.txt` : 41 textes distincts, chacun suivi des crates qui l'emploient.
    `cargo-about` ne sait pas écarter les proc-macros (13 crates de plus avec leurs dépendances) : c'est la liste de `cargo tree` qui fait foi.
  - **Avis de copyright** : chaque texte garde celui que la crate publie.
    Plusieurs textes n'en ont aucun, comme leur amont : un texte MIT commun à 17 crates (adler2 à zmij, dont rustc-hash et tinyvec), le MIT de fdeflate, le WTFPL de ffmpeg-next.
    - Cinq crates ont un texte que `cargo-about` ne trouve pas seul, et `about.toml` le lui donne par `clarify` : dans la crate pour type-map, windows-sys et windows-link, dans le dépôt amont au commit publié pour fluent-langneg et intl_pluralrules.
    - objc2, objc2-encode, block2 et dispatch2 ne publient aucun texte, seulement un `LICENSE.md` qui renvoie à MIT.
      Leur ligne de copyright devient « No copyright notice is published with these crates », suivie des auteurs de leur `Cargo.toml`.
    - La tâche échoue sur tout autre modèle de texte (`<year>`, `<copyright holders>`).
  - **Contrôle C5** de `check.sh` : le bloc de `NOTICE` de l'archive égale la liste de `cargo tree`, sans crate absente ni en trop.
    Une dépendance changée sans `mise run notice` fait donc échouer `package.yml`.
    `cargo tree` lit les manifestes des trois cibles, que la machine n'a pas toujours téléchargés : C5 a besoin du réseau.
    Preuve négative (16 octobre 2026) : une crate retirée du `NOTICE`, puis une crate en trop, font échouer C5 seul.
  - **Licence retenue** : dans une expression « OU », l'ordre de `accepted` (`about.toml`) choisit.
    MIT l'emporte pour 74 crates ; les autres sont sous Unicode-3.0 (`tinystr`, `yoke`, `zerofrom`), BSD-3-Clause (`tiny-skia`, `tiny-skia-path`), BSD-2-Clause (`arrayref`), WTFPL (`ffmpeg-next`, `ffmpeg-sys-next`) et Apache-2.0 (`self_cell`, sous `Apache-2.0 OR GPL-2.0-only`).
    Aucune crate livrée n'a de fichier NOTICE à reprendre (Apache-2.0 §4d).
- **Licences admises** : `deny.toml` n'admet que MIT, Apache-2.0 (avec ou sans LLVM-exception), BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, WTFPL et Zlib, liste mesurée avec toutes les dépendances déclarées, différées comprises.
  D'autres licences n'apparaissent que comme alternatives dans une expression « OU » : 0BSD (`adler2`), Unlicense (`memchr`) et LGPL-2.1-or-later (`r-efi`), à côté de MIT ; GPL-2.0-only (`self_cell`), à côté d'Apache-2.0 seulement.
  `r-efi` ne vise qu'UEFI et n'est livré sur aucune des trois cibles.
- **`jiff` et ses crates `jiff-*`** (#66, #83 : heure locale des noms datés et de `params.json`) sont sous Unlicense OU MIT, MIT retenu par `about.toml`.
  Sous Windows, `jiff-tzdb` embarque la base des fuseaux horaires de l'IANA, du domaine public.

## Lints

La [charte](#charte) est résumée dans [CLAUDE.md](../CLAUDE.md).
Les lints se règlent dans `[workspace.lints]` du `Cargo.toml` racine, jamais par attribut.

- **Activés** :
  - `clippy::pedantic` ;
  - des restrictions : `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr`, `exit`, `allow_attributes`, `let_underscore_must_use`, `unused_result_ok`, `map_err_ignore`, `iter_over_hash_type`, `mod_module_files`, `string_slice`, `clone_on_ref_ptr` ;
  - pour `unsafe` : `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block`, `unnecessary_safety_comment` ;
  - côté rustc : `unsafe_code = "forbid"`, `unreachable_pub`, `missing_debug_implementations`, `trivial_numeric_casts`, `rust_2018_idioms`, `non_ascii_idents`.
- **Sept lints de `pedantic` coupés**, dans la table seulement :
  - `cast_possible_truncation`, `cast_possible_wrap`, `cast_precision_loss` et `cast_sign_loss`, qui tombaient 30, 55, 48 et 31 fois sur les noyaux du prototype, dans des calculs de pixels où la règle 15 suffit ;
  - `missing_errors_doc` et `missing_panics_doc`, parce que la règle du zéro commentaire ne demande pas de documentation ;
  - `must_use_candidate`, qui signalerait presque chaque accesseur.
- **Refusés** : `indexing_slicing` (272 occurrences sur le prototype), `arithmetic_side_effects` (223), `as_conversions` (184), `missing_docs` (62), `nursery`, `cargo`.
- **`media` est la seule crate avec `unsafe`.** Elle porte sa propre table, copie exacte de celle du workspace sans `unsafe_code`.
  Cargo refuse `workspace = true` avec une surcharge locale, et un contrôle compare les deux tables.
- **Exception au niveau du manifeste** : `[workspace.metadata.cargo-shear] ignored` liste les dépendances déclarées d'avance qu'aucune crate n'utilise encore.
  L'utilisateur l'a acceptée comme les lints coupés de la table : ce n'est pas un `#[allow]` au sens de la règle 3.
  `cargo shear --deny-warnings` échoue dès qu'une entrée de la liste devient utilisée (`redundant_ignore`), et la liste se vide ainsi issue par issue.
- **Règles vérifiées par script** (`mise run rust:rules`, dans `rust:static`) :
  - aucun `#[allow]`, `#[expect]` ou `cfg_attr(…allow…)` ;
  - `[lints] workspace = true` partout sauf dans `media`, dont la table est égale à celle du workspace sans `unsafe_code` ;
  - aucun `ffmpeg_next` hors de `media` ;
  - les dépendances des crates ne viennent que du workspace ;
  - au plus un appel à `catch_unwind` hors des tests (`*/tests.rs`, `*/testing.rs`, `*/tests/*`, lignes `use` exclues, toutes formes comprises), et seulement sous `crates/cli/src/` (#113) ; pas de `panic = "abort"` dans les profils ;
  - au plus 2 % de lignes de commentaire dans `crates/`, commentaires de fin de ligne et `/* */` compris ;
  - aucun mot d'imitation (`round_ties_even`, `opencv`, `numpy`, `python`, `pairwise`).

## Charte

But : détecter au moins aussi bien que la v0.1.1, en Rust idiomatique, sans recopier ligne à ligne ni le Python ni le prototype.

Outillage :

1. Toolchain de `rust-toolchain.toml`, édition 2024. `cargo fmt --all --check` ne signale rien.
2. `cargo clippy --workspace --all-targets --locked -- -D warnings` sans avertissement.
   Les lints se règlent dans `[workspace.lints]` du `Cargo.toml` racine, nulle part ailleurs.
3. Zéro `#[allow]`, `#[expect]` ou `cfg_attr(…allow…)`, même si clippy le suggère.
   Un lint gênant se résout en réécrivant le code ; s'il paraît faux, on demande.
4. Chaque crate a `[lints] workspace = true`. Seule `media` a sa propre table : la copie exacte de celle du workspace, sans `unsafe_code`.

Pas d'imitation :

5. Ni Python, ni numpy, ni OpenCV ne s'imitent : pas de `round_ties_even` pour copier `round()`, pas d'ordre d'opérations recopié.
   Le calcul s'écrit simple et juste ; l'écart se mesure par appariement avec tolérances (`reference`), puis se juge.
   **Exception** (décision de l'utilisateur à #118) : le gris de la détection reprend la table mesurée sur OpenCV et l'arrondi à 8 bits après la réduction, parce que les seuils de la v0.1.1 sont réglés sur ce gris (mesures dans `engine`, « Gris de la détection »).

Sûreté et erreurs :

6. `unsafe` seulement dans `media`. Un bloc contient une seule opération et porte une ligne `// SAFETY:` qui dit pourquoi le contrat tient.
7. Aucun type de `ffmpeg_next` ne sort de `media`. Une fonction `extern "C"` ne peut pas paniquer.
8. Chaque bibliothèque a son `enum` d'erreur (`thiserror`), qui garde la cause (`#[source]`).
   Ni `anyhow`, ni `Box<dyn Error>`, ni erreur `String`. Une erreur par module dans `media`.
9. Une entrée refusée (vidéo abîmée, format non géré, réglage hors bornes) rend une erreur typée, jamais une panique.
   `unwrap`, `expect`, `panic!`, `todo!` et `dbg!` sont refusés hors des tests.
10. Une panique signale un bug. Seule la commande la rattrape, une fois par vidéo.
    Les fils passent par `std::thread::scope` ou rayon, jamais détachés.
11. Aucun résultat jeté : ni `let _ =` sur un `Result`, ni `.ok();`.
12. Ni `println!` ni `eprintln!` : `writeln!` sur un flux verrouillé, erreur traitée.
    Tout texte vu par l'utilisateur vit dans un seul module de la commande.

Types et calcul :

13. L'index d'image (ordre d'affichage, celui où le décodeur rend les images) et le `pts` ont chacun leur type (`FrameIndex`, `Pts`).
    Pas de sentinelle : `Option<Pts>`.
14. Hors de la détection, tout est en pixels de la vidéo d'origine. Seul `WorkScale` passe des pixels de travail aux pixels d'origine.
15. Conversions : `From` quand elle existe.
    Un `as` qui perd de l'information suit un `round` ou un `clamp` visible, ou vit dans une fonction de conversion nommée.
16. Même entrée, même sortie, quel que soit le nombre de fils : pas d'itération sur `HashMap` ou `HashSet` qui atteint une sortie, pas de somme flottante parallèle.
    La règle vaut pour ce qui sort du moteur, plages abîmées exclues : les pixels que FFmpeg dissimule sur une image abîmée dépendent du nombre de fils de décodage (mesure dans `engine`).
17. Une configuration se construit par une fonction qui valide et rend un `Result`. Les comptes non nuls sont des `NonZero*`.
18. Boucles chaudes : aucune allocation par pixel ni par image, tampons réutilisés ; itérateurs, `zip` et `chunks_exact` plutôt que des indices.

Forme et tests :

19. Un module, une responsabilité, ses tests dans `src/<module>/tests.rs`. Pas de `mod.rs`. Fonctions de 100 lignes et 7 arguments au plus.
20. `pub(crate)` par défaut, `Debug` sur tout type public. Pas de `pub use` dans `media/src/lib.rs`.
21. Zéro commentaire, sauf `// SAFETY:` et une ligne `///` sur un élément public dont le nom ne dit pas le contrat externe.
    Au plus 2 % des lignes.
22. Tests sans commentaire, nommés par le pourquoi ; `unwrap`, `expect` et `panic!` y sont permis. Flottants comparés avec une tolérance.
    Pas de faux FFmpeg : vraie vidéo, ou vidéo fabriquée par openh264. Couverture d'au moins 95 %.
    Tests lents dans `tests/slow_*.rs`, réglages dans `tests/settings_*.rs`, tests de la 092 dans `tests/reference_*.rs`. proptest là où un oracle simple existe.
23. Une dépendance se déclare une fois dans `[workspace.dependencies]`, puis `x.workspace = true`.
    Une dépendance hors de la liste d'une issue se demande avant. `cargo deny check` et `cargo shear` restent verts.

## Dépendances

- Chaque dépendance se déclare une fois dans `[workspace.dependencies]`, puis `x.workspace = true`, et `--locked` est partout.
- Déclarées d'avance pour le lot : `rayon`, `dhat`.
  Tant qu'elles ne servent pas, elles ne sont pas dans `Cargo.lock`, et leur version n'y est pas figée.
- `serde` sert `types` et `reference`.
  `serde_json` a la feature `float_roundtrip`, pour qu'un flottant de la référence se relise au bit près.
  `flate2` (moteur `miniz_oxide`, 100 % Rust) lit et écrit les `.gz` de la référence ; `zlib-rs` figure dans `Cargo.lock` sans être compilé.
- **Pas d'`ab_glyph`** : il tire `ttf-parser`, déclaré non maintenu par RUSTSEC-2026-0192 sans mise à jour sûre, et l'utilisateur refuse d'ignorer l'avis.
- **Polices** (#101, 11 octobre 2026) :
  - `skrifa` 0.48.0 (fontations de Google, MIT OR Apache-2.0), suggéré par l'avis : table des caractères, métriques, contours, axes variables ;
  - le tracé par `tiny-skia` 0.12.0 (BSD-3-Clause), sans sa feature `png` ;
  - `unicode-normalization` 0.1.25 (MIT OR Apache-2.0) pour la forme NFC ;
  - `read-fonts` tire `yoke`, `zerofrom` et `synstructure`, sous Unicode-3.0, déjà admise.

  Écartés : `fontdue` et `rusttype` (ils tirent `ttf-parser`), `swash` (un second rastériseur à côté de tiny-skia) et `icu_normalizer` (une quinzaine de crates pour une seule normalisation).
- **PNG de l'image résumé** (#104) : `png` 0.18.1 (MIT OR Apache-2.0), en direct dans `summary`, RGB 8 bits ; il tire `fdeflate` 0.3.7 (même licence).
  `draw` reste sans format de fichier, et la feature `png` de tiny-skia tirerait les mêmes crates.
  `png` dépend de `miniz_oxide` 0.8 quand `flate2` prend la 0.9 : le doublon, qui existait déjà par `dhat`, entre avec `png` dans le binaire livré (avertissement de `cargo deny`).
- `clap` en builder, features `string` et `wrap_help` : la commande se construit à l'exécution avec les textes de la langue (#114), ce que `derive` ne fait pas.
- **Langues** (#114) : `fluent-bundle` 0.16.0, `unic-langid` 0.9.6 et `sys-locale` 0.3.2 (MIT OR Apache-2.0).
  Ils ajoutent 13 crates à `Cargo.lock` (`fluent-syntax`, `fluent-langneg`, `intl-memoizer`, `intl_pluralrules`, `self_cell`, `type-map`, `tinystr`, `zerovec`, `displaydoc`, `unic-langid-impl`), sous MIT, Apache-2.0 ou Unicode-3.0, déjà admises.
  Écartés : `i18n-embed-fl` (macro et chargement de fichiers dont la commande n'a pas besoin) et ICU4X (des dizaines de crates pour formater deux sortes de nombres).
  La macro `langid!` n'est pas prise : la langue se lit par `"fr".parse()`, et le test des pluriels prouve qu'elle est reconnue.
- `cargo deny check` échoue sur un avis de sécurité, un paquet retiré (`yanked`), une licence hors liste, une version sans borne (`*`) ou une source inconnue.
  Il ne fait qu'avertir sur une version en double.
  Il prévient aussi que BSD-2-Clause et Zlib, admises pour les dépendances différées, ne sont pas rencontrées : elles ne seront contrôlées qu'à l'arrivée de ces dépendances dans `Cargo.lock`.

## Tests

| Niveau | Fichiers | Commande | Contenu |
|---|---|---|---|
| rapide | `src/<module>/tests.rs`, `tests/*.rs` | `mise run rust:test` (sans couverture), `mise run rust:coverage` (couverture) | vidéos fabriquées par openh264, binaire lancé pour de vrai |
| slow | `tests/slow_*.rs`, `#[ignore = "slow"]` | `mise run rust:slow` | l'extrait LFS : 1 230 images, 10000/333, 1440×1080, plage `tv` ; les tests de mémoire `slow_memory_flash`, `slow_memory_flash_wide` et `slow_memory_flash_long` (`dhat`, 960 et 1440 pixels de travail, 61 échantillons), hors de la couverture ; sous macOS, une image exFAT montée par `hdiutil` (`files/tests/slow_exfat.rs`, #236) |
| settings | `tests/settings_*.rs`, `#[ignore = "settings"]` | `mise run rust:settings` | les 16 cas du banc d'essai sur l'extrait et sa 720×540 (`settings_agreement`), sous Linux seulement en CI (#167) |
| reference | `tests/reference_*.rs`, `#[ignore = "reference"]` | `mise run rust:reference` | la 092 : 9 011 images, cadence moyenne 90110000/3000927, aux défauts et à 1440 pixels de travail (#151), en un seul binaire (`cli/tests/reference_092.rs`) |
| matériel | `tests/hardware_*.rs`, `#[ignore = "hardware"]` | étape d'essai de la CI macOS | VideoToolbox sans repli logiciel |
| local | `tests/local_*.rs`, `#[ignore = "local"]` | `BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=… cargo test --workspace --locked --profile ci --test 'local_*' -- --include-ignored` | les 26 vidéos de la référence, hors dépôt, jamais en CI ; récupérées du seau sur une autre machine par `mise run rust:data` ([emplacements](../tests/fixtures/reference/README.md#récupération-sur-une-autre-machine)) |

- Le niveau d'un test se choisit par le nom de son fichier : ajouter un test ne demande de toucher ni la CI ni `mise.toml`.
- `mise run check` joue `rust:static` et `rust:test`, avant de rendre.
  `mise run check:ci` joue `rust:coverage`, `rust:slow` et `rust:settings`, que la CI répartit sur ses tâches.
  Durées du 16 octobre 2026 (bb3a668, M1 Max au calme, `verrou chrono`) : `rust:static` 3,1 s ; `rust:test` 52,1 s, dont 22 s de compilation des 12 crates du workspace ; `rust:coverage` 519 s, dont 28 s de compilation instrumentée et 253 s pour les tests unitaires de `batdetect` sous instrumentation.
  Sans recompilation, `check` prend 19,5 s réels et 57 s de CPU, et le hook pre-push de `lefthook.yml`, qui le lance, 19,8 s.
- Un test rapide peut relancer son propre binaire sur un test `#[ignore = "run by …"]` pour lire ce qu'il écrit sur stderr (aucun message du décodeur pendant la lecture, par exemple).
- La couverture se mesure sur les tests rapides, en profil `ci`, avec un seuil de 95 %.
  Les lignes qui restent hors couverture sont des erreurs de FFmpeg qu'aucune vidéo de test ne provoque.
- Les tests de mémoire sont des tests `slow`, hors de la couverture : sous l'instrumentation, le premier prenait 83 s sous Linux et 55 s sous macOS (PR #145), contre 1 s en profil `ci` sur le M1 Max, sans couvrir une ligne de plus.
  Ils tournent sur les trois systèmes, par `rust:slow`.
- Le test du détecteur à 61 et 81 échantillons de fond tourne sur une image de 144×24 (24 blocs de 64 pixels), avec les mêmes demi-fenêtres, fils et lots qu'à 360×180 : 7 s sous la couverture sur le M1 Max, contre 146 s.
  Sous l'instrumentation, chaque voie du tri de `Network::order` incrémente un compteur atomique partagé par tous les fils (mesuré sur le Mac par `sample` et désassemblage) : le tri n'est plus vectorisé et les fils se disputent le compteur.
- Le niveau `reference` passe la 092 deux fois, en parallèle : aux défauts (détection par `engine::detection::detect`, puis `batdetect::process::process_detected` : suivi, extraits, zoom) et à 1440 pixels de travail (`compare`).
  - L'initialisation du binaire `cli/tests/reference_092.rs` lance les deux passages sur deux fils et range leurs résultats dans un `OnceLock`.
    Les cinq tests les relisent : la commande, la concordance aux défauts, la lecture (horodatages, images-clés, aucune plage abîmée, d'après ffprobe), le décodage (cadence, taille, plage, aucun paquet illisible) et la concordance à 1440.
  - Le parallèle se fait dans l'initialisation et non par libtest, qui lance les tests dans l'ordre de leur nom, 4 à la fois sur le runner : celui de la 1440 viendrait en dernier.
  - Une erreur est rangée avec ses causes, une panique avec son message : un `OnceLock` dont l'initialisation panique la relancerait pour chaque test en attente.
    La détection aux défauts, la commande qui la suit et la 1440 ont chacune leur résultat, si bien qu'un rendu qui échoue ne fait tomber que le test de la commande.
  - Un test lancé seul par filtre fait quand même les deux passages : 66 s sur le M1 Max.
    Le binaire entier prend 66,0 s et 429 s de CPU, 70,6 s avec 4 fils de test, pour un pic de 1,65 Go.
  - La concordance aux défauts vérifie d'abord que les réglages de l'enregistrement sont ceux de `DetectConfig::default()` : c'est la même détection que celle de la commande.
  - La 1440 ne touche pas `cli` : elle est dans ce binaire pour partager le processus, d'où la dépendance de test `cli` → `compare`.
  - Le décodage par le `Decoder` seul de `media` et la lecture du moteur ne se recoupent que sur l'extrait (`slow_decode`, `slow_input`), pas sur la 092.
- Les fixtures se trouvent par `reference::fixture::fixture`, qui refuse un pointeur LFS non tiré avec le conseil `git lfs pull`.
- Les tests `local_` de `bench` lisent les bancs de la v0.1.1 (`BATDETECT_REFERENCE_DIR/bench/`) : `local_injection` sans vidéo, en moins d'une seconde ; `local_guard` rejoue les deux bancs sur la 092 du dépôt (LFS), en 1 min 54 en profil `ci`.
  `committed_metrics` (#277) lit les deux bancs de la v0.2.0, dans le dépôt, et tourne en CI sur les trois systèmes (0,07 s sur lumiel).
- `tests/fixtures/rust/video_092_original_3m24-4m05/frames.jsonl` : les détections de l'extrait par le Rust, écrites sur le Mac depuis la racine du dépôt par `verrou lourd cargo run --profile ci -p compare --bin compare -- extract --frames tests/fixtures/rust` (fichier git ordinaire, fin de ligne `\n` forcée par `.gitattributes`).
  Le test `slow_extract` de `compare` les redemande au bit près sur macOS, Linux et Windows ; un écart remonte à l'utilisateur (voir `compare`).
- `tests/fixtures/rust/video_092_original_3m24-4m05/tracks.csv` : le tableau des pistes de l'extrait, contrôlé à l'octet par `check.sh` (C6) sur les trois systèmes.
  - Il se refait depuis la racine du dépôt, en anglais, par le binaire `release` : `verrou lourd env -u LANGUAGE -u LC_ALL -u LC_MESSAGES LANG=C target/release/batdetect tests/fixtures/video_092_original_3m24-4m05.mp4 -o out/csv`.
    On copie ensuite `out/csv/video_092_original_3m24-4m05/video_092_original_3m24-4m05_tracks.csv` à sa place.
  - Ses lignes finissent en CRLF, comme tout tableau produit. `.gitattributes` le marque `-text`, pour que git garde ces octets tels quels.
  - Ses valeurs diffèrent de la référence v0.1.1 sur trois cellules : le chemin de la piste 1 (912.3 au lieu de 912.7) et l'amplitude des pistes 1 et 3 (110.2 au lieu de 110.3, 99.0 au lieu de 99.1).
- Cinq petites fixtures, hors LFS, faites une fois par le ffmpeg 9.0.2 de Homebrew (libx264) et commitées :
  - `full_range.mp4` (1,9 Ko, `yuvj420p`, plage `pc`) : `ffmpeg -f lavfi -i color=gray:size=1440x1080:rate=30 -frames:v 2 -c:v libx264 -bf 0 -pix_fmt yuvj420p -color_range pc full_range.mp4`
  - `full_range_midway.mp4` (3,0 Ko, 2 images en plage non précisée puis 2 en `pc`) : `tv.h264` et `pc.h264` par la commande précédente avec `-x264-params repeat-headers=1 -f h264`, l'un en `-pix_fmt yuv420p -color_range tv`, l'autre en `-pix_fmt yuvj420p -color_range pc`, puis `cat tv.h264 pc.h264 > midway.h264 && ffmpeg -framerate 30 -i midway.h264 -c copy full_range_midway.mp4`
  - `two_gops.mp4` (5,9 Ko, 320×240, 20 images, images-clés en 0 et 10, perte au début par la couture) : `ffmpeg -f lavfi -i testsrc=size=320x240:rate=30 -frames:v 20 -c:v libx264 -bf 0 -g 10 -keyint_min 10 -sc_threshold 0 -crf 40 -pix_fmt yuv420p -color_range tv two_gops.mp4`
  - `spot.mp4` (3,1 Ko, 320×240, 40 images, plage `tv`, un carré clair qui avance de 5 pixels par image sur un fond gris, une bande en haut à Y = 0 et une en bas à Y = 254, hors de la plage `tv`) : `ffmpeg -f lavfi -i "color=c=0x606060:s=320x240:r=30" -f lavfi -i "color=c=white:s=6x6:r=30" -filter_complex "[0]drawbox=x=0:y=0:w=320:h=24:color=black:t=fill,drawbox=x=0:y=216:w=320:h=24:color=white:t=fill[bg];[bg][1]overlay=x='40+5*n':y=110:eval=frame,format=yuv420p,lutyuv=y='if(lt(val\,20)\,4\,if(gt(val\,230)\,250\,val))'" -frames:v 40 -c:v libx264 -preset medium -crf 18 -g 20 -bf 0 -pix_fmt yuv420p -color_range tv -colorspace bt709 -color_primaries bt709 -color_trc bt709 spot.mp4`
  - `spot_full_range.mp4` (3,1 Ko), le même flux remuxé sans réencodage, marqué plage pleine (ffprobe : `yuvj420p`, `color_range=pc`) : `ffmpeg -i spot.mp4 -c copy -bsf:v h264_metadata=video_full_range_flag=1 spot_full_range.mp4`.
    `tests/full_range.rs` vérifie que chaque image de l'un est lue en `Limited` ou `Unspecified` et chaque image de l'autre en `Full`, puis que leurs détections, non vides, sont identiques au bit près.
- **CI** : sur macOS et Linux, deux tâches parallèles (#179), `rust` (statique, tests rapides sous couverture) et `rust-slow` (tests `slow`, contrôles du binaire, essai matériel sous macOS).
  Elles ne partagent aucune compilation : la couverture compile dans `target/llvm-cov-target`, les tests lents et le binaire dans `target/ci`.
  - `rust` ne tire aucune vidéo LFS, puisqu'aucun test rapide n'en lit.
  - Deux autres tâches tournent sous Linux, en parallèle : `rust-reference` (#146) et `rust-settings` (#167), les 16 cas du banc d'essai.
    Les réglages ne tournent ni sous macOS ni sous Windows ; `slow_extract` y garde l'identité au bit près sur les trois systèmes.
  - Quand l'empreinte de FFmpeg change, `rust-slow`, `rust-reference` et `rust-settings` le construisent aussi, sans le sauver : seule `rust` sauve le cache de FFmpeg.
    Ce cache est restauré, puis sauvé dès la construction, même si la suite échoue.
  - Le binaire contrôlé se construit par `--workspace --bin batdetect`.
    Avec `-p batdetect`, les features de `syn` changent (`draw` en ajoute par `skrifa`), et `bindgen`, `ffmpeg-sys-next` et `media` se recompilaient (11 s sous Linux, 22 s sous macOS, 60 s sous Windows à la PR #145).
  - L'action `setup` installe la toolchain, les outils cargo et le Python de `rust:rules`.
  - Deux actions composites servent à chaque tâche (#181).
    `.github/actions/ffmpeg-key` pose `XDG_CACHE_HOME` et calcule les clés des caches de FFmpeg et de `target/` par un seul script, `key.sh`, sous `bash`, ou sous MSYS2 sur Windows.
    `.github/actions/windows` met en place MSYS2, MSVC et rustup, et renomme le `link.exe` de coreutils.
- **Tâches `rust-windows` et `rust-windows-slow`**, indépendantes des autres et parallèles (#179).
  Quand l'empreinte de FFmpeg change, chacune le construit, et seule `rust-windows` le sauve.
  `rust-windows` lance les tests rapides, contrôle le binaire, puis `cargo clippy --profile ci -- -D warnings` ; `rust-windows-slow` lance les tests `slow` (sans les réglages).
  - En profil `ci`, clippy reprend les scripts de construction et les macros des tests, alors qu'en `dev` il recompilait 46 unités, dont les liaisons de FFmpeg (82 s à la PR #145).
  - `dumpbin -dependents` n'admet que des DLL de Windows : `kernel32`, `ntdll`, `bcrypt`, `bcryptprimitives` et `api-ms-win-core-*`.
    C'est une liste blanche, et `kernel32` doit y être, pour que le contrôle ne puisse pas passer à vide.
  - Ni couverture, ni `cargo deny`, ni `mise` : ils tournent déjà sous Unix.
  - clippy vient en dernier, pour qu'un échec propre à Windows laisse lisibles les résultats des tests.
  - Seule `rust-windows-slow` tire l'extrait ; son cache LFS se reprend depuis Linux (`enableCrossOsArchive`).
  - En cas d'échec, `config.log` de FFmpeg est téléversé en artefact (`ffmpeg-windows-logs` ou `ffmpeg-windows-slow-logs`).
- **Cache de `target/`** (`actions/cache`, une entrée par tâche) :
  - Il contient tout `target/` de sa tâche : sous Unix, `debug` (clippy) et `target/llvm-cov-target` (couverture) pour `rust`, `ci` (tests lents, binaire) pour `rust-slow` ; sous Windows, `ci` dans les deux tâches.
    Chacune garde la rlib de ffmpeg-sys-next avec ses liaisons.
  - La clé commence par le nom de la tâche : `rust` et `rust-slow`, `rust-windows` et `rust-windows-slow` ont des entrées distinctes, et le repli de l'une ne prend jamais l'entrée de l'autre.
    L'entrée de `rust` sur `main` garde `target/ci` jusqu'au prochain `Cargo.lock`, parce qu'une clé trouvée telle quelle ne se resauve pas.
  - Il se sauve en fin de tâche réussie.
  - Sa clé porte, dans cet ordre, la tâche, l'OS, l'architecture, l'image du runner (`ImageOS`), **l'empreinte de FFmpeg**, `rust-toolchain.toml`, puis `Cargo.lock` (action `ffmpeg-key` ; la tâche est celle qui appelle, sauf `rust-settings`, qui passe `job: rust-reference`).
    Comme cargo ne suit pas les `.a`, un `target/` construit avec un autre FFmpeg garderait des binaires déjà liés à l'ancien et les liaisons de bindgen tirées de ses en-têtes (voir « Reconstruction »).
  - En PR seulement, une clé de repli (`restore-keys`, #146) prend tout sauf `Cargo.lock` : une PR qui change une dépendance reprend le `target/` de `main` pour la même image, le même FFmpeg et la même toolchain, et ne recompile que ce qui a changé.
    Elle ne mélange jamais deux empreintes de FFmpeg.
  - Sur `main`, pas de repli : un nouveau `Cargo.lock` repart d'un `target/` vide.
    Un `target/` repris par le repli garde les artefacts des versions remplacées et grossirait sans cette remise à zéro à chaque changement de dépendance.
    Tailles à la PR #145 : 393 Mo (`rust`, Linux), 366 Mo (macOS), 470 Mo (Windows), 101 Mo (`rust-reference`).
  - `rust-settings` ne sauve rien : elle restaure l'entrée de `rust-reference` (même clé), qui a déjà compilé les dépendances en profil `ci`, et le quota du cache (10 Go) ne gagne aucune entrée.

### Tests figés

Six cas d'extraits et de zooms ont été jugés à l'aveugle par l'utilisateur le 15 octobre 2026 (sur f364698, en `release`, `LANG=fr_FR.UTF-8`) contre les sorties de la v0.1.1, puis figés.
Deux fichiers par cas, A et B tirés au sort, ouverts l'un après l'autre en plein écran dans QuickTime, et une seule question : « la cible se voit-elle aussi bien ? ».
Le texte dessiné (`#N`, Roboto contre la police d'OpenCV) trahit la provenance et n'était pas à juger.
Les six verdicts sont « aussi bien », et l'utilisateur voit peu ou pas de différence entre A et B.

| Cas | Vidéo | Piste (début v0.1.1) | Fichier |
|---|---|---|---|
| bordure avec zoom | 125 | 0:28.41 | zoom |
| passage rapide (1 926 px/s) | 092 | 3:48.29 | extrait |
| piste courte (6 images) | 097 | 1:54.63 | extrait |
| piste lente | 092 | 3:29.61 | extrait |
| boîte interpolée (30 images) | 097 | 1:48.54 | extrait |
| zoom ×4 | 125 | 0:47.12 | zoom |

Deux images résumé ont été jugées à découvert, côte à côte, sans rien à redire : la 097 (plages abîmées et instables) et la 089-091 (plages instables dans la 089 et la 091).

Les cas validés deviennent des empreintes, jamais celle d'un mp4, parce que VideoToolbox n'est pas reproductible d'une machine à l'autre :

- **Extraits et zooms** : les images dessinées que `render` pousserait à l'encodeur (`YuvFrame`, boîtes, traces et numéros compris), récupérées sans encodeur par `render::drawn::drawn` (feature `testing`).
  Une empreinte `of_sequence` par fichier, avec son nombre d'images.
  Les pistes sont celles du Rust, pas celles de la référence, puisque ce sont elles que l'utilisateur a jugées.
  Elles viennent de `batdetect::process::tracked` (timeline, exclusion, suivi), que `process_detected` appelle aussi : un changement de la commande entre la détection et le rendu se voit dans ces tests.
  - `cli/tests/slow_frozen.rs`, sur l'extrait LFS, dans la CI des trois systèmes : la piste lente (380 images) et le passage rapide (108 images).
    Sur l'extrait, elles portent les numéros `#1` et `#2` au lieu de `#11` et `#12`, et seuls les numéros diffèrent des images jugées.
    La CI de la PR #208 a montré les mêmes empreintes sous macOS, Linux et Windows : tiny-skia (avec SIMD) et swscale (`BITEXACT`) dessinent au pixel près d'un système à l'autre.
  - `cli/tests/local_frozen.rs`, jamais en CI (vidéos hors dépôt) : les six cas sur leurs vidéos entières.
    Trois ont été refigés après une planche avant/après jugée par l'utilisateur le 17 octobre 2026 : avec les détections faibles (#21), le zoom de la 125 à 0:47.12 et les deux extraits de la 097 (1:48.52 et 1:54.62) gardent leurs pistes et leurs débuts, et gagnent des détections faibles à la place de positions estimées.
    Avec le recollage (#24), 9 pistes de moins commencent avant ces deux extraits de la 097, dont les numéros passent de #62 à #53 et de #67 à #58 ; deux voisines changent de couleur.
    Les points, les boîtes et les traces ne changent pas.
- **Images résumé** : les pixels de chaque PNG écrit par `batdetect` en français (`of_pixels`), la 097 et la 089-091, dans `cli/tests/local_frozen.rs`, avec le fond de chaque vidéo (`_fond.png`).
  Ce test ne peut pas vivre dans `summary`, parce que le fond est la médiane calculée par la détection et que `summary` n'a ni FFmpeg ni moteur.
  Refigées le 18 octobre 2026, après #21, #311 et #326 : la 097 tient en sept images, la tranche 0-10 min de la 089-091 en une.
  Les images de la 097 ont une empreinte par système : vers 182 s, Linux voit une image abîmée de plus que macOS, et la plage non analysée finit à 182,15 s au lieu de 181,68 s ; ses pistes et son fond sont identiques.
  La 089-091 est dans le seau (README de la référence).
- Un changement voulu du dessin, de la détection ou du suivi casse ces empreintes : on les refige en le disant, après un nouveau jugement à l'œil si le dessin a changé.

## Encodage

Sur le Mac, les extraits et la vidéo annotée passent par VideoToolbox matériel en q65, avec openh264 en QP 23 pour repli ; ailleurs, par openh264 seul.
Les zooms passent toujours par openh264. Aucun de ces réglages n'est une option (décision de l'utilisateur, 11 octobre 2026).

### Choix des encodeurs

Mesuré le 11 octobre 2026 sur le M1 Max, sous le verrou de mesure (#111), par un programme Rust jetable lié au même FFmpeg (`~/Projects/dvr-wt/rapports/mesure-111/`, hors dépôt, avec les scripts `one.sh` et `xpc.sh`, les journaux et `RESULTATS.md`).
Le SSIM vient du filtre `ssim` du ffmpeg 9.0.2 de Homebrew, chaque sortie comparée à l'extrait d'origine (1 230 images).
Les temps sont la médiane de 3 passages, décodage seul (6,98 s) retranché.

| Encodeur | Réglage | SSIM All (Y) | Taille | Encodage | CPU | Mémoire par encodeur |
|---|---|---|---|---|---|---|
| VideoToolbox matériel | q65, `allow_sw=0` | 0,9792 (0,9688) | 105,4 Mo | +0,4 s | ≈ 0 | ≈ 180 Mo |
| Logiciel d'Apple | `require_sw=1`, 20 Mb/s | 0,9809 (0,9714) | 105,5 Mo | 6,5 s | 54,8 s | ≈ 495 Mo |
| openh264 | profil high, QP 23 | 0,9799 (0,9698) | 136,9 Mo | 16,8 s | 16,3 s | ≈ 16 Mo |

- La cible d'openh264 est le SSIM de VideoToolbox q65 remesuré dans le moteur Rust (0,9792), avec 0,977 comme plancher.
- QP 24 donne 0,9761. Pour égaler QP 23 en débit, il faut 28 Mb/s, et le fichier pèse alors 143 Mo.
- L'utilisateur a validé QP 23 à l'œil le 11 octobre 2026 (« Franchement c'est pareil ») sur la piste 0:23.48 de l'extrait, la moins contrastée (17 points, amplitude 68,7, images 660 à 767) : 9,3 Mo en VideoToolbox q65, 12,1 Mo en openh264 QP 23.
- openh264 sert de repli plutôt que le logiciel d'Apple : il prend 30 fois moins de mémoire et 3 fois moins de CPU, et c'est le même encodeur que sous Linux, donc testé partout.
  Le prix : des fichiers 30 % plus gros.
- Le logiciel d'Apple refuse q65 : l'ouverture échoue avec `-12900` (propriété `Quality` non gérée), et il ne marche donc qu'en débit fixé.
- **Couleur** : plage limitée, et BT.709 pour la matrice, le transfert et les primaires, écrits dans le flux H.264 par les deux encodeurs (décision du 11 octobre 2026).
  On suppose toutes les sources en BT.709, ce qui est vérifié sur les 27 vidéos connues.

### Pièges de FFmpeg 7.1.1

- **`allow_sw` ne choisit pas le logiciel d'Apple** : `allow_sw=1` prend le matériel s'il répond, `allow_sw=0` exige le matériel, et seul `require_sw=1` force le logiciel.
- **q65 dans le processus** : `global_quality = 65 × FF_QP2LAMBDA` plus le drapeau `QSCALE`. Ce réglage n'existe que sur arm64.
- **Profil d'openh264** : sans réglage, il encode en Constrained Baseline (CAVLC).
  Le profil high (CABAC) donne le même SSIM et un fichier 8 % plus petit, mais il encode 38 % plus lentement (16,8 s contre 12,2 s).
- **QP d'openh264** : `rc_mode=off` laisse le QP à 26 quel que soit `qmin`/`qmax`, alors que `rc_mode=quality` avec `qmin = qmax` applique le QP aux images I et P (vérifié par `trace_headers`).
- **Fils d'openh264** : avec une tranche par image, ils ne changent rien (même fichier à l'octet, même temps).
  Le parallélisme vient de plusieurs extraits encodés à la fois.
- **Options inconnues** : `open_with` les ignore sans erreur : les tests relisent donc le profil et le PSNR.

### Essai VideoToolbox

`Codec::VideoToolbox` est le matériel seul (#111, décision de l'utilisateur du 11 octobre 2026), puisque le logiciel d'Apple refuse q65.
Les tests rapides de VideoToolbox sur macOS passent donc aussi par le matériel.

- L'étape matérielle de la CI macOS encode 120 images de l'extrait (images 660 à 779) au réglage de production, q65 sans repli logiciel, puis les relit.
  Elle vérifie le nombre d'images, la taille, la cadence 10000/333 exacte, la plage limitée et un PSNR de luminance supérieur à 37,9 dB ; sur le M1 Max, on mesure 38,9 dB en q65 et 34,9 dB sans q65.
- Cette étape bloque la CI (décision de l'utilisateur, 11 octobre 2026).
  Si elle casse à cause d'une machine de GitHub plutôt que du code, la question se rouvre avec l'utilisateur.
- Sur le runner `macos-latest`, le journal affiche « IOServiceMatchingfailed for: AppleM2ScalerParavirtDriver » avant le test, sans effet sur le résultat.
- Le test lent d'openh264 relit le même passage, avec un seuil de 39,1 dB.
  Sur le M1 Max, on mesure 39,53 dB à QP 23 et 38,69 dB à QP 24 : le seuil, à mi-chemin, rejette un QP qui ne serait pas appliqué.
  L'openh264 x86 de la CI Linux reste à confirmer au premier passage.

### Plusieurs encodeurs à la fois

Mesuré le 11 octobre 2026 avec K encodeurs de 300 images chacun.

- **Mémoire** : le pic du processus, plus l'empreinte du service `VTEncoderXPCService` où tourne VideoToolbox.
  openh264 prend environ 16 Mo par encodeur ; le matériel, environ 130 Mo dans le service et 48 Mo dans le processus ; le logiciel d'Apple, environ 480 Mo dans le service et 15 Mo dans le processus.
  `Codec::footprint` vaut 17 Mo pour openh264 (16,3 Mo mesurés, arrondis au-dessus) et 180 Mo pour VideoToolbox.
  Le matériel fait aussi monter la mémoire câblée du système, de +0,7 à +3,4 Go selon K, mais la mesure est trop bruitée pour l'attribuer, et elle n'entre pas dans `Codec::footprint`.
- **Sessions matérielles** : 17 tiennent sur le M1 Max.
  À partir de 18, toutes s'ouvrent, puis l'envoi d'images échoue (`-17691`), un cas que le repli à l'ouverture ne couvre pas : les extraits ouvrent donc au plus 8 sessions.
  Le seuil de la machine cible n'est pas mesuré.
- **Charge réelle** : la 125 (47 pistes, 39 zooms) n'a jamais plus de 8 fenêtres ouvertes à la fois.
- **Budget** : repère de 4 Gio pour l'ensemble des encodeurs ouverts.

### Dans les extraits

- **Un encodeur par groupe** (`media::encode::fanout::Fanout`, nuit du 12 au 13 octobre 2026).
  Chaque image dessinée d'un groupe est encodée une seule fois, sans images B, et ses paquets sont écrits dans chaque extrait ouvert, avec une image-clé forcée sur la première image de chaque extrait (`AVFrame.pict_type = I`, que VideoToolbox et openh264 honorent) et des `pts` rebasés à 0.
  Un extrait se ferme (en-queue, renommage) dès que le paquet de sa dernière image arrive.
  - Gardes : le premier paquet de chaque extrait doit porter le drapeau clé (`NoKeyFrame` sinon), un extrait doit avoir reçu toutes ses images (`ShortClip`), et un paquet sans `pts` est une erreur.
  - Vérifié par `ffprobe` sur les 117 extraits de la 099 : tous commencent par une image-clé (`key_frame=1`), sans image B, et chaque extrait se décode seul (tests).
  - Sans lui, une image commune à plusieurs extraits d'un groupe était encodée une fois par extrait : 16 016 envois pour 6 616 images distinctes sur la 099.
    Avec lui et le décodage à plusieurs fils, la commande passe de 58,68 s à 26,94 s sur la 099 (M1 Max, trois passages).
- **La limite est le débit du codeur matériel.**
  Pendant les extraits, VideoToolbox encode de 322 à 378 images 1440×1080 par seconde, toutes sessions confondues, et le temps des extraits suit « images envoyées à VideoToolbox / environ 350 ».
  Seul (encodeur seul, 1440×1080), il fait 224 images par seconde avec une session et 366 à 392 avec 2, 4 ou 8 : le M1 Max a deux moteurs d'encodage, saturés dès deux sessions, et les options `prio_speed` et `realtime` n'aident pas.
- **Sessions** : une session VideoToolbox par groupe, au plus 8 à la fois.
  Avec un groupe par fil sur la moitié des cœurs, ce plafond n'est atteint qu'au-delà de 16 cœurs.
- **Mémoire** : un encodeur par groupe en cours (un par fil), plus les images en vol et le cache de zones de chaque fil (borné à 4 millions de pixels, LRU).
  Mesurée sur la 099 : 1,04 Gio pour le processus avec le décodage à plusieurs fils, 0,81 Gio sans.
  Avec le service VideoToolbox (somme des `phys_footprint` des `VTEncoderXPCService` relevée chaque seconde, moins les 57 Mo des services ouverts au repos), processus et service restaient sous 2,2 Go ensemble sur la commande complète du 12 octobre 2026 (somme des deux maxima, qui ne tombent pas forcément au même instant).
- **Image-clé d'openh264** : une par seconde environ (`round(cadence)` images), pour qu'un lecteur se place vite dans l'extrait.
  VideoToolbox garde son réglage.
- **Extrait refait** : quand VideoToolbox casse en cours d'écriture, le groupe est repris en openh264 (voir `render`).
  Aucun échec réel n'a été vu sur les vidéos mesurées ; la reprise est exercée par une fausse ouverture dans les tests.
- **Droits** : chaque extrait est créé par `File::create` sous un nom temporaire, donc avec les droits habituels de l'utilisateur (umask), comme le CSV et le PNG, puis renommé.

### Zooms en openh264

Les zooms ne prennent aucune session VideoToolbox et occupent les cœurs libres.
Mesure au calme (13 octobre 2026, 3 passages en alternance, `release`, zoom `auto`) contre une variante temporaire tout en VideoToolbox, zooms compris :

| Vidéo | Zooms openh264 | Tout VideoToolbox | CPU (openh264 / VT) | Cœurs occupés pendant le rendu |
| --- | --- | --- | --- | --- |
| 092 (1 zoom) | 18,38 s | 19,09 s | 130 s / 129 s | 3,5 / 3,7 |
| 099 (46 zooms) | 39,79 s | 43,81 s | 304 s / 231 s | 7,7 / 4,0 |

Les cœurs occupés sont le temps CPU du processus relevé chaque seconde, après la détection (les 14 premières secondes, environ 8 cœurs dans les deux cas).
Sur la 099, openh264 prend 73 s de CPU de plus, sur des cœurs qui restaient libres, et rend la commande 9 % plus courte : VideoToolbox, partagé entre extraits et zooms, plafonnait.

## Plage réelle des sources

Mesure du 11 octobre 2026 (#101) sur 20 images par vidéo, réparties hors des plages abîmées de la référence avec une marge de 15 images (programme et table brute : `~/Projects/dvr-wt/rapports/mesure-101-plage/`, `plage.tsv`).
Les 26 vidéos annoncent une plage limitée (`tv`), sur leur conteneur comme sur chaque image.

| Vidéos | Y hors 16..235 | Pire image | U/V hors 16..240 | Extrêmes |
|---|---|---|---|---|
| 089 à 103 | 0,92 à 1,96 % | 7,81 % (094) | 0,020 à 0,035 % | Y 0-255, U 68-146, V 98-255 |
| 120 à 130 | 0,41 à 0,62 % | 0,75 % | 0,031 à 0,033 % | Y 0-255 (0-252 pour la 120), U 75-147, V 113-255 |

- Sur l'image 716 de l'extrait, les pixels hors plage sont le contenu thermique lui-même : Y < 16 surtout en haut de l'image (arbres et ciel froids), Y > 235 surtout en bas (sol chaud).
  V à 255 n'occupe qu'un petit carré, vraisemblablement une icône rouge de l'affichage des jumelles.
- **Décision de l'utilisateur** : pour l'affichage, les sources se lisent en plage pleine, quelle que soit leur étiquette.
  Les fichiers produits restent en plage limitée BT.709 (#111), resserrés sans écrêtage.
- **Noir calé sur 16** (décision de l'utilisateur du 12 octobre 2026, à l'œil sur les fonds de la 092, de la 122 et de la 124, #104).
  - Lu en plage pleine, le ciel froid, autour de Y = 16, sort gris et non noir : le fond de l'image résumé de la 092 a un plancher à 10 (1er centile, après l'assombrissement à 0,6) et aucun pixel à 2 ou moins, contre 26,6 % en v0.1.1, comme un voile.
  - Trois variantes ont été comparées : plage pleine, plage limitée (comme la v0.1.1, blancs écrêtés au-dessus de Y = 235), et noir calé sur 16 sans écrêter les blancs.
  - Retenue : `Y' = max(0, Y − 16) × 255 / 239`, arrondi au plus proche, avant la conversion en plage pleine, sans toucher la chrominance.
    Y = 16 et en dessous deviennent noirs, Y = 255 reste blanc, et les zones chaudes gardent leurs nuances.
  - Fond de la 092 : plancher 0, médiane 75 (80 en plage pleine, 79 en v0.1.1), 24,8 % de pixels à 2 ou moins (26,6 % en v0.1.1).
  - Ce réglage ne touche que l'affichage : la détection lit Y brut.
  - Dans les extraits, l'image passe par les tables d'affichage (`Converter::display`) hors des zones dessinées, et par `FullBlackAt16`, le RGBA et le retour en plage limitée BT.709 dans les zones (voir `render`).
    Un rendu en plage pleine ferait tomber le SSIM des pixels hors dessins à 0,917 environ, au lieu de 0,966 (`slow_clips`, essai fait une fois, avant les zones).

## Crates

La carte courte des crates et de leurs dépendances est dans [CLAUDE.md](../CLAUDE.md) ; cette section dit, crate par crate, ce que le code ne dit pas.
Le workspace compte treize crates (`members = ["crates/*"]`), et `unsafe` n'est permis que dans `media`.

- **`types`** : vocabulaire commun, sans entrée-sortie ni algorithme de détection.
  Un type n'y entre que s'il est produit par une crate et lu par au moins deux autres, et un ajout passe par une PR fusionnée avant les files qui en dépendent.
  - `Rational` est réduit à la construction, avec un dénominateur positif et sans débordement (`OutOfRange` plutôt qu'une panique).
    Il s'écrit et se lit `num/den`, comme dans ffprobe.
  - `FrameIndex` compte les images dans l'ordre où le décodeur les rend, c'est-à-dire l'ordre d'affichage, à partir de 0 ; `Pts` compte des ticks de la base de temps.
    En JSON, ce sont des nombres.
  - `FrameRate` est une cadence positive, avec son propre type pour qu'elle ne s'échange pas avec la base de temps.
  - `Timeline` porte le `pts` de chaque image (`Option<Pts>`), une base de temps positive et la `FrameRate` (`avg_frame_rate`).
    Elle donne deux temps : `seconds_by_pts`, et `seconds_by_rate` (numéro ÷ cadence), celui des temps du CSV de la v0.1.1.
  - `Detection` est en `f64` et en pixels de la vidéo d'origine ; `Measures` en est la partie sans image, sérialisable.
  - `FrameDetections` garde une liste par image, images vides comprises.
    `push` est la seule façon de la remplir, tests compris, et chaque détection prend le rang de l'image qu'elle remplit : ni le détecteur ni la lecture de `reference` ne peuvent ranger une détection sous une autre image.
  - `Span` est une plage d'images, bornes comprises, refusée si elle finit avant de commencer.
    Ses constructeurs dérivés (`single`, `hull`, `widened`, `intersection`) ne peuvent pas échouer : les plages instables et abîmées se bâtissent sans variante d'erreur.
  - `span::merged` et `span::merged_by` (#302) sont la seule fusion des plages qui se recouvrent ou se touchent (une plage qui commence l'image après la fin d'une autre la rejoint), triées par première image.
    `merged_by` garde les éléments de chaque groupe (`Run`), à égalité de début dans l'ordre reçu.
    Les plages abîmées d'`engine` (`Damage`), les groupes d'extraits de `render` (`windows::groups`) et la réunion des plages de `compare` passent par elle.
  - `Track` : un numéro et des points, au moins un, aux images strictement croissantes.
    Une piste recollée garde ses bouts : `Track::rejoined` reçoit des `Stretch` non vides, `Part` pour une piste gardée et `Piece` pour un morceau qui relie ou prolonge (#311).
    `parts()` rend les seules pistes gardées, ou la piste entière si elle n'est pas recollée, et `with_id` renumérote sans revalider en gardant les bouts (#24).
  - `Language` (français, anglais), `SourceSize` et `WorkScale`.
    `WorkScale` donne la taille de travail (largeur demandée sans agrandir, hauteur arrondie, au moins 1) et fait seul le passage des pixels de travail aux pixels d'origine : coordonnée `(c + 0,5) × échelle − 0,5`, longueur × échelle, surface × échelle².
- **`media`** : la frontière avec FFmpeg, dont aucun type de `ffmpeg_next` ne sort. Une erreur par module.
  - `frame` : `YuvFrame`, 4:2:0 planaire sans `stride`, avec sa plage de couleur.
  - `failure::LibraryError::io_kind()` (#64) lit le genre d'erreur système derrière une erreur d'FFmpeg, sur l'errno d'`AVERROR(e)`.
    La table est unique et sans `cfg` (ENOENT, EACCES, EPERM, ENOSPC, EROFS), parce que FFmpeg porte sous Windows l'errno du CRT, de mêmes valeurs, et non un code Win32 que `from_raw_os_error` lirait.
  - `decode::Error::NoDecoder` (#64) : le FFmpeg livré n'a pas de décodeur pour le flux (`DecoderNotFound`).
    Il est séparé de `Open` pour qu'un `.mov` en ProRes ne passe pas pour un fichier illisible ; tout autre échec du décodeur reste `Open`.
  - `decode` : `Decoder::next` remplit un `YuvFrame` fourni par l'appelant, sans allocation par image.
    - Ouverture (#227) : `ffmpeg_next::format::input` ferait `to_str().unwrap()` sur le chemin.
      Le chemin passe donc en octets sous unix (`path::location`, comme l'encodeur), puis par `avformat_open_input`, `avformat_find_stream_info` et `Input::wrap` (`opened`, un `unsafe` par opération).
      Une vidéo au nom non UTF-8 s'ouvre ainsi sous Linux, au départ (`engine::input`) comme pour les extraits (`render::source`).
      `UnusablePath` reste pour un NUL et, sous Windows, pour un nom qui n'est pas de l'Unicode valide.
    - `DecodeErrors` (module `decode_errors`) lit les `decode_error_flags` de l'image tels quels, nommés par les constantes `FF_DECODE_ERROR_*` de FFmpeg.
      On n'utilise pas le `DecodeError` de ffmpeg-next, qui perd `CONCEALMENT_ACTIVE` et `DECODE_SLICES` sans son cfg `ffmpeg_5_1`.
      Une image est en erreur si ses drapeaux ne sont pas vides, bit inconnu compris ; une tranche coupée donne 12 (dissimulation et tranches manquantes).
    - Fils : `Threads::Automatic` (choix de FFmpeg) ou `Threads::Exactly(n)`, en fils d'image.
      `active_threads` lit, après l'ouverture, ce que FFmpeg fait vraiment : `Single`, `Frame(n)` ou `Slice(n)`.
      Un test vérifie qu'un fil donne `Single`, que 4 donnent `Frame(4)` et que l'automatique donne plusieurs fils d'image sur une machine à plusieurs cœurs : un FFmpeg construit sans fils ferait échouer la CI.
    - Couture : `with_seam` reçoit un `PacketSeam` qui voit chaque paquet du flux vidéo, dans l'ordre de lecture, avant `send_packet` (rang, image-clé, taille, `data_mut`, `shrink`).
      `data_mut` rend d'abord le paquet inscriptible (`av_packet_make_writable`, sans copie quand le tampon n'a qu'une référence), ou `None` s'il ne peut pas l'être.
      Un paquet raccourci sans réécrire la longueur de son NAL est refusé en entier par le décodeur et compté illisible ; raccourci à 0 octet, il fait échouer `send_packet` (`EINVAL`).
    - Contrat : un paquet illisible est compté, pas fatal, et ceux qui suivent la dernière image se lisent par `unreadable_packets_after_last_frame`.
      `Err` rend le décodeur inutilisable ; après la fin, `next` rend toujours `None`.
    - Fin de fichier : FFmpeg 7.1.1 rend l'erreur d'entrée-sortie du contexte (`pb->error`, sauf `EAGAIN`) à la place d'un `EOF` du démultiplexeur (`libavformat/demux.c:1530`), et elle sort en erreur `Read`.
      Une lecture tronquée en fin de fichier sans erreur d'entrée-sortie donne moins d'images, et la garde d'`engine` la refuse.
    - Image retournée : un `linesize` négatif rendrait faux le `data()` de ffmpeg-next (longueur `linesize as usize × hauteur`).
      Il est refusé (`FlippedFrame`) avant tout appel à `data()` ; le décodeur h264 n'en produit pas.
    - Cadence : `avg_frame_rate`, ou `r_frame_rate` si elle est inconnue.
    - Formats : `yuv420p` seul, `yuvj420p` lu comme plage pleine.
    - Placement (`decode::placed`, #110) : `Decoder::seek_before(pts)` se place sur la dernière image-clé à ou avant ce `pts` que connaît l'index du conteneur (`avformat_seek_file`, borne haute = la cible, puis `flush` du décodeur), même après la fin du fichier.
      Il rend un `Placed`, dont les images (`PlacedFrame`) n'ont qu'un `pts`, puisqu'après un placement le compte des images rendues ne les numérote plus.
      `Decoder::at_start` lit depuis le début avec la même interface.
      La cible passe du `pts` aux microsecondes de FFmpeg en arrondissant vers le bas, pour ne jamais dépasser l'image voulue ; FFmpeg la ramène à la base de temps du flux à l'arrondi le plus proche, donc au plus sur elle.
  - `encode` : `Settings::new` (taille paire, cadence positive), puis `Encoder::create`, `push` et `finish`, en openh264 ou VideoToolbox, sans images B.
    - Réglages fixes, détaillés dans [Encodage](#encodage) : VideoToolbox en q65, matériel seul (`allow_sw=0`) ; openh264 en profil high, `rc_mode=quality`, QP 23.
    - Le fichier s'écrit sous un nom temporaire (`partial_file`), et seul `finish` lui donne son nom ; un test de `cli` exige que le ménage de `files::names`, que `media` ne peut pas appeler, reconnaisse ce temporaire.
    - Une image en plage pleine est refusée (`FullRange`) : l'encodeur écrit une plage limitée en BT.709, dans le flux H.264.
    - Sans réglage, openh264 n'écrit qu'une image-clé, la première ; `Settings::with_key_interval(n)` en écrit une toutes les n images.
    - `Codec::footprint` donne la mémoire d'un encodeur ouvert, mesurée en 1440×1080, service hors du processus compris ([Encodage](#plusieurs-encodeurs-à-la-fois)).
    - `fanout::Fanout` (#112) : un encodeur, plusieurs mp4, voir « Dans les extraits ».
    - `open_stamped(chemin, Stamps)` (#103) : la sortie garde les `pts` de la source au lieu de compter ses images à la cadence de l'encodeur.
      `Stamps` (un `pts` par image, base de temps de la source) est refusé vide, négatif, non croissant ou sur une base non positive.
      Le flux mp4 prend la base de la source (1/10000 sur les jumelles, gardée telle quelle par le muxeur).
      Chaque paquet reprend le `pts` de son image, avec un `dts` égal (pas d'images B, sinon `Reordered`), et dure jusqu'à l'image suivante ; la dernière dure un pas de la cadence de l'encodeur, si bien que sur l'extrait la durée égale celle de la source (40,959 s).
      Un premier `pts` non nul est gardé.
    - Erreurs de destination (`concerns_the_destination`) : chemin refusé, temporaire non créé, fichier non ouvert (`OpenFile`), renommage raté.
      Une écriture refusée (`Writing`) n'en est pas une, pour que la bascule vers openh264 joue.
    - `encode::fallback::Fallback` est la liste ordonnée d'encodeurs, injectable : VideoToolbox matériel puis openh264 sur macOS, openh264 ailleurs.
      `trial`, l'essai au démarrage, encode quelques images grises jusqu'à `finish` et retire en tête de liste les encodeurs qui échouent ; `create`, le repli à l'ouverture de chaque extrait, ouvre le premier qui répond, sans retirer ceux qui refusent.
      Une destination inutilisable (dossier absent, chemin refusé) rend tout de suite `Destination`, sans essayer les encodeurs suivants.
      L'échec en cours d'écriture (sessions matérielles saturées) n'est pas couvert par `Fallback` : `render` le reprend.
    - `encode::bounded` (#313) : la création d'une session VideoToolbox (`avcodec_open2`, qui appelle `VTCompressionSessionCreate`) se fait sur un fil à part, attendu au plus 10 s (`VIDEOTOOLBOX_START_LIMIT`, décision de l'utilisateur).
      Sur la CI macOS, une session a attendu sans fin la réponse du service (`xpc_connection_send_message_with_reply_sync`, PR #309), alors que le test matériel entier (session, encodage d'un passage, relecture) y prend de 5 à 7 s, et l'essai de l'encodeur 0,25 s sur le M1 Max.
      - Au-delà du délai, l'ouverture rend `Unanswered`, qui ne concerne pas la destination : l'essai (`Fallback::trial`) et l'ouverture d'un extrait (`render::encoders`) passent à openh264 comme pour un encodeur qui refuse.
      - Le silence se garde pour tout le processus, donc pour tout le lancement (`bounded::VIDEOTOOLBOX`, statique) : toute ouverture VideoToolbox suivante rend `Unanswered` tout de suite, sans redemander au service.
        L'essai des vidéos suivantes ne garde qu'openh264, que `params.json` nomme (`encoders`) ; la vidéo pendant laquelle le silence arrive garde la liste de son essai, comme pour un extrait réécrit.
        Un service muet coûte au plus 10 s pour la soirée, moins de 20 s si plusieurs groupes ouvraient une session au même moment (jusqu'à 8, chacun attend ses 10 s).
        Un service lent qui répond avant 10 s n'est jamais déclaré muet : il coûte son attente à chaque ouverture, essai de chaque vidéo et chaque groupe, sans plafond.
      - Le fil bloqué ne se tue pas : il est détaché et reste dans son appel jusqu'à la fin du programme, que la sortie de `main` ou `process::exit` (Ctrl-C) termine sans l'attendre.
        Il coûte une pile réservée (2 Mio de mémoire virtuelle, peu de mémoire réelle) et ce que VideoToolbox avait déjà alloué, au plus un par session en attente au moment du silence.
        Une réponse tardive est jetée, et la session est fermée par la libération du contexte.
      - Une panique pendant l'ouverture repasse au fil appelant (`resume_unwind`), pour ne pas passer pour un silence.
      - La borne ne couvre que l'ouverture (`avcodec_open2`).
        Deux autres appels passent par le service sans borne : sur `kVTInvalidSessionErr`, FFmpeg recrée la session pendant `push` (`vtenc_send_frame` → `vtenc_configure_encoder`), et la fermeture (`vtenc_close`) attend les images en cours (`CompleteFrames`).
      - Les tests prouvent, sur tous les systèmes, l'attente bornée d'un appel qui ne revient jamais (un canal jamais servi, `bounded/tests.rs`), la mémoire du silence et le refus immédiat qui suit, la réponse tardive jetée, la panique repassée, un encodeur openh264 ouvert sur le fil de la borne qui encode ensuite sur l'appelant, et le passage de l'essai au suivant (`fallback/tests.rs`).
        `tests/slow_videotoolbox_silent.rs`, dans son propre processus, rend muet le vrai `VIDEOTOOLBOX` (10 s), puis vérifie qu'une session VideoToolbox rend `Unanswered` aussitôt et que l'essai ne garde qu'openh264.
      - Aucun test ne reproduit le blocage du service lui-même, ni sur la CI ni sur le Mac ; sur macOS, les tests vérifient que VideoToolbox marche encore à travers la borne.
        La fin du programme avec un fil encore bloqué dans le service ne se jugera que sur macOS, au prochain blocage : un gestionnaire `atexit` de VideoToolbox qui attendrait la même connexion bloquerait après le bilan, Ctrl-C compris.
  - `convert` : `Converter` passe du YUV 4:2:0 au RGBA opaque, et retour (#101).
    - **Matrice** : BT.709 dans les deux sens (`sws_setColorspaceDetails`), décision du 11 octobre 2026 (#111).
    - **Plage lue** : un paramètre explicite (`SourceRange`), pas l'étiquette de l'image.
      Les jumelles se lisent en plage pleine ([Plage réelle des sources](#plage-réelle-des-sources)), noir calé sur 16 (`SourceRange::FullBlackAt16`) : une table de 256 niveaux appliquée au plan Y pendant la copie qui précède la conversion.
    - **Plage écrite** : le retour au YUV se fait toujours en plage limitée.
      Une source pleine y est resserrée sans rien écrêter : Y 0 devient 16 et Y 255 devient 235.
    - Aucune allocation par image.
    - Tables d'affichage (`display`, #112) : une table par plan, sans passer par le RGBA.
      Celle de Y est tirée de swscale sur une rampe de gris (noir calé sur 16 compris), donc égale à l'aller-retour sur tout pixel gris ; celle de U et V resserre la plage pleine en plage limitée (128 + (c − 128) × 224 / 255, arrondi).
      Sur les 1 230 images de l'extrait, contre l'aller-retour par RGBA (`slow_convert`), 0,11 % (Y), 0,15 % (U) et 0,11 % (V) des valeurs diffèrent, dont 0,02 %, 0,06 % et 0,04 % de plus de 2 niveaux ; l'écart va au plus à 34 (Y), 18 (U) et 51 (V).
      Le pire est un pixel de V à 255 (l'icône rouge des jumelles) que l'aller-retour écrête en RGB et que la table garde.
    - Zones (`yuv_to_rgba_region`, `rgba_to_yuv_region`, `Region`, #112) : la même conversion sur un rectangle aux coordonnées paires, par des contextes swscale à sa taille.
      Une taille nouvelle crée ses contextes, dans un cache LRU de 32 tailles et 4 millions de pixels au plus ; `render` arrondit la taille des zones au multiple de 16 pour que le cache serve.
      Dans le rectangle, les pixels RGBA sont ceux de la conversion entière, et le Y écrit aussi (testé).
    - Filtres mesurés : « point » à la montée et « area » à la descente, chrominance horizontale pleine.
      Les aplats de la palette, alignés sur des blocs 2×2, font l'aller-retour à 2 niveaux au plus, contre 34 à 143 avec le bilinéaire.
    - Image 716 de l'extrait, sans dessin : lue en plage limitée, comme le dit son étiquette, elle a 345 704 pixels en noir pur et 9 719 en blanc pur ; lue en plage pleine, 38 et 15, exactement les pixels à Y 0 et Y 255 de la source.
      Réécrite en plage limitée puis relue, elle s'écarte en moyenne de 0,086, avec 0,016 % des valeurs au-delà de 2 niveaux (973 sur 6,2 millions, non localisées).
  - `payload`, derrière la feature `testing` (tests seulement) : `wipe(chemin, parts, nombre)` écrase de `0xff` des parts égales de la boîte `mdat` d'un mp4.
    Les images qui y tombent sont perdues (saut de `pts`) ou dissimulées.
    Une plage de parts vide, hors de la boîte ou qui déborde est refusée.
    Les tests de `media`, d'`engine` et de `render` y fabriquent leurs vidéos abîmées.
  - `fingerprint`, derrière la même feature (#154) : `of(image)` hache les trois plans d'une image décodée, `of_pixels(largeur, hauteur, octets)` les pixels d'une image (le PNG d'une image résumé), `of_sequence` une suite d'empreintes (les images d'un extrait).
    C'est la seule empreinte d'image des tests de `media`, `engine`, `render` et `cli`.
    C'est un FNV-1a sur des mots de 64 bits écrit à la main, dont chaque pas replie les bits hauts sur les bas (sans ce repli, deux mots changés dans leur seul octet haut s'annulaient une fois sur 256).
    Il est le même sur chaque système et chaque version de Rust, à la différence de `DefaultHasher`, si bien qu'une empreinte peut s'enregistrer dans un test figé (`a_stored_fingerprint_stays_valid_…`).
- **`draw`** (#101) : dessin pur sur une toile RGBA, sans FFmpeg ni entrée-sortie.
  Police et primitives ont été choisies sur une planche de rendus (Roboto contre Noto Sans et IBM Plex Sans, sur l'image 716 de l'extrait, à côté du rendu de la v0.1.1).
  - `text` : `Typeface::roboto`, la police embarquée.
    - Graisse 400 pour le texte et 600 pour les numéros (choix de l'utilisateur sur la planche, 11 octobre 2026).
    - Une taille est la hauteur des chiffres en pixels, et les chiffres ont tous la même largeur.
    - Le texte est normalisé en NFC, et un caractère absent devient U+FFFD.
    - Ni crénage ni placement d'accent par la table GPOS : il suffit pour des numéros et des légendes en caractères précomposés.
    - `shape` prépare un texte une fois, que `Canvas::shaped_text` redessine sans allocation, par exemple un `#N` à chaque image.
  - `canvas` : `Canvas`, toujours opaque, si bien que ses octets sont du RGBA simple pour `media::convert`.
    Les primitives sont lissées, sauf les rectangles (choix de l'utilisateur, même planche) :
    - rectangle aux bords nets : 1 px couvre les pixels du bord de la boîte, et chaque pixel de plus élargit le bord vers l'extérieur, si bien que la boîte de 2 px (image détectée) et celle de 1 px (image interpolée) gardent le même bord intérieur ;
    - une épaisseur nulle ou non finie ne dessine rien ;
    - assombrissement à 0,6, arrondi au plus proche, alors que la v0.1.1 tronque : un niveau d'écart au plus (règle 5 de la charte).
  - `color` : la palette des 12 couleurs et les couleurs des extraits et du panneau, en RGB.
    Un test épingle chacune au BGR inversé de `output/style.py` de la v0.1.1.
    Le numéro des pastilles s'écrit en noir (`INK`) sur chaque couleur, sur l'image résumé comme dans la page : de 6,94:1 (rouge, 255 92 92) à 17,62:1 (gris clair), au-dessus des 4,5:1 du texte en WCAG AA, que le blanc n'atteint sur aucune (3,03:1 au plus).
    Le gris d'une piste « non » dans la page (`--rejected`) donne 6,49:1. Un test garde ce seuil si la palette change (#299).
  - `style` : `style_for` donne les tailles du résumé selon la largeur, dans les proportions de la v0.1.1, avec un rayon des pastilles mesuré sur la nouvelle police.
    L'arrondi va à l'entier le plus loin de zéro (règle 5 de la charte) : 1440 / 64 tombe pile sur 22,5, d'où un rayon 23 et des chiffres de 21 px en 1440, contre 22 et 20 en v0.1.1, qui arrondissait au pair.
    `TRACK_LABEL` est le `#N` des extraits, chiffres de 17 px en graisse 600, figé par un test.
- **`summary`** → `draw`, `types`, `files` : l'image résumé (#104), une page par tranche de 10 minutes, sans FFmpeg : `cli` lui donne le fond déjà converti.
  - `assigned::track_colors(summary, largeur, hauteur)` (#13, feature `testing`, tests seulement) rend la couleur de chaque piste sur l'image résumé de cette taille, dans l'ordre des pistes, par la même mise en page sans dessin.
    Les tests figés de `cli` la passent au rendu des extraits, comme `process` y passe les couleurs de l'image dessinée.
  - `pages` rend `Summaries` (#259) : le fond (`backdrop`, la scène assombrie, zones masquées peintes), le même pour toutes les pages, et pour chaque page son image, sa tranche (`window`, entière même pour une partie), sa partie, ses périodes coupées à la tranche, les débuts des pistes des autres parties et le rayon des pastilles.
    Chaque piste dessinée y a un `Painted` : chemin, flèche, centre de la pastille et départ du trait de rappel, tirés des `Mark` de `layout`.
    `Mark::leader` est le seul critère du trait de rappel (pastille à plus d'un rayon de la trajectoire), pour `compose` comme pour la page.
  - Les règles sont celles de la v0.1.1 pour les couleurs (indice le moins employé autour), les flèches (25°, 3 % de la longueur), la légende (colonnes, coupe par « ... ») et les tranches.
    Les tris deviennent des clés entières, et les points ne sont pas arrondis avant le dessin (règle 5).
  - Temps par les `pts` (#35) : la durée va jusqu'au début de l'image qui suivrait la dernière ; la tranche et le temps de légende d'une piste viennent du `frame_seconds` de sa première image ; les plages, de `spans_in_seconds`, comme `params.json`.
  - Bandeau : `header` rend des `HeaderLine` (nom, pistes, fenêtre, hors analyse, illisible, « + N »), qu'un trait `Labels` met en texte, avec les libellés de la v0.1.1 et des accents possibles (police Roboto, NFC).
    - Les plages illisibles sont datées comme les plages hors analyse (#127) : 3 lignes « illisible a - b » (début tronqué, fin arrondie au-dessus), puis « + N autres, 3 min 22 s illisibles en tout » (« + N more, … unreadable in all »).
      La durée totale additionne les plages exactes et s'arrondit au-dessus ; les plages hors analyse n'ont pas de total.
    - Les plages qui se touchent ou se recouvrent une fois arrondies à la seconde ne font qu'une ligne (121 : « 0:04 - 0:06 », « 0:05 - 0:08 », « 0:07 - 0:09 » deviennent « 0:04 - 0:09 ») ; la frise et `params.json` gardent le détail exact.
    - Seule la ligne du nom de la vidéo reste plafonnée à 3 colonnes de légende (coupée par « ... » au-delà) ; les autres lignes, bornées, élargissent toujours le bandeau.
  - **Frise du temps** (#127, module `time_strip`) : sous l'image, sur sa largeur, dans chaque tranche.
    - Deux bandes, « hors analyse » (gris moyen) et « illisible » (gris clair du texte du bandeau), nommées à gauche par `Labels::band`.
      Une bande vide est un cadre fin sur le fond sombre du bandeau, pour ne pas se lire comme une bande pleine.
    - Les plages viennent de `spans_in_seconds`, comme `params.json` et le bandeau, et font au moins 2 px : les 87 plages de la 121 restent toutes visibles.
    - Graduée en minutes, au pas de 1, 2, 5 ou 10 minutes selon la place, des diviseurs de la tranche de 10 minutes : son début tombe toujours sur une graduation.
      Un repère par piste, à sa couleur, au temps `pts` de sa première image (choix de l'utilisateur sur la planche).
    - Le bandeau reste à la hauteur de l'image, et la page grandit de la hauteur de la frise (135 px en 1440).
    - La frise couvre toujours la fenêtre de la tranche, même sur une image coupée : les départs des pistes de l'image en couleur, ceux des autres pistes de la tranche en gris (choix de l'utilisateur sur la planche).
  - **Images coupées au nombre de pistes** (#157) : une tranche de plus de `MAX_TRACKS_PER_IMAGE` = 20 pistes donne `ceil(n / 20)` images, en paquets égaux de pistes consécutives par départ (21 pistes : 11 + 10, et non 20 + 1).
    L'utilisateur a choisi 20 sur une planche de la 097 et de la 099 à 15, 20 et 25.
    - Mêmes numéros (ceux de la vidéo entière), même fond, même mise en page.
      Couleurs et pastilles s'attribuent paquet par paquet, pour que des voisines d'autres images ne retirent pas de couleurs à celle-ci (choix de l'utilisateur).
    - Bandeau : nom, « partie 3 / 8 » (« part 3 / 8 »), le nombre de pistes du paquet, sa plage de temps (du premier départ à la dernière fin du paquet, fin arrondie à la seconde supérieure comme les plages ignorées), puis les plages hors analyse et illisibles de toute la tranche.
    - Les pistes arrivent triées par départ depuis le suivi (`track_detections`), et les paquets suivent cet ordre sans le refaire.
    - Une tranche de 20 pistes ou moins reste une seule image.
    - Sur la référence, 10 vidéos sur 26 ont une tranche de plus de 20 pistes.
  - Masques de l'affichage : une liste de rectangles peints en gris de panneau avant l'assombrissement.
    `cli` les tire des zones OSD de la configuration : `Region::columns` et `rows` sans marge, sur la taille de la source, soit `[⌊x0·w⌋, ⌈x1·w⌉)` comme dans la v0.1.1 ; `PixelBox` inclut ses bornes, donc `right = fin − 1`.
    La liste reste vide tant que la commande n'a pas `--osd-region`.
  - Écarts voulus à la v0.1.1 : fond lu en plage pleine, noir calé sur 16 ([Plage réelle des sources](#plage-réelle-des-sources)) ; police et rayon de `draw::style` (rayon 23 en 1440, 29 pour un numéro à trois chiffres contre 26 en v0.1.1).
  - **Placement des pastilles** (choix de l'utilisateur, 12 octobre 2026) : aucune pastille hors cadre, aucun recouvrement, et chaque pastille au plus près de sa piste.
    - Les candidats se prennent tous les demi-rayons le long de la piste, classés par la clé (pastilles touchées, flèches touchées, croisements, écart au quart de la piste).
    - Une pastille qui ne touche qu'une tête de flèche reste sur sa piste.
      Si elle touche une autre pastille, elle s'écarte de sa piste par anneaux d'un demi-rayon (16 directions, 40 anneaux au plus) : le premier anneau sans autre pastille l'emporte, et dans cet anneau une place libre des flèches passe d'abord.
      Dès qu'elle ne touche plus sa piste, un trait de rappel aux couleurs de la piste la relie au point le plus proche.
    - Suivent jusqu'à 8 passes : chaque pastille encore en collision est replacée si la nouvelle place touche moins que l'ancienne.
    - Mesure sur les pistes de la référence (26 vidéos, 966 pistes), centres à moins de 2 rayons : 266 recouvrements en v0.1.1, 349 pour un portage fidèle, 0 pour le placement retenu, et aucune pastille hors cadre.
      121 pastilles s'écartent de leur piste de plus d'un rayon, presque toutes sur les vidéos denses ; la plus éloignée est à 304 px (096).
  - Le PNG s'écrit dans son partiel à côté (`files::names::partial_file`, #115, #245), puis il est renommé, avec les droits ordinaires d'un fichier créé (un fichier temporaire de `tempfile` resterait en `0600`).
    Une erreur d'entrée-sortie de l'encodeur PNG remonte en `Error::Write` avec son `io::Error` (#64) : `png` n'expose sa cause que par `cause()`, et un disque plein se perdrait.
  - Fond de la 092 mesuré une fois contre la v0.1.1 (luminance, hors traits, 81 % des pixels), avant le choix du noir calé sur 16 : SSIM 0,702 tel quel en plage pleine, 0,984 une fois ramené en plage limitée pour la mesure (écart moyen 1,4 niveau).
  - Tests : `tests/v011.rs` compare, sur l'extrait et la 092 de la référence, numéros, couleurs, tranches et en-tête aux attendus de la v0.1.1 ([Référence v0.1.1](#référence-v011)).
    `tests/local_v011.rs` fait de même sur les 26 vidéos, avec 22 écarts listés : 13 temps de légende et une ligne « hors analyse » de la 096 par les `pts`, la durée illisible totale de la 120 (1 min 14 s contre 1 min 13 s), et les plages hors analyse réunies à la seconde (089, 096, 098, 103).
    Pour les plages illisibles, la comparaison porte, sur chaque tranche, sur leur nombre et leur durée totale, tirés des plages elles-mêmes et non du bandeau.
    Une tranche coupée en parties est comparée pages réunies, sur ses numéros et son en-tête ; ses couleurs, attribuées par paquet, ne le sont pas.
- **`page`** (#71) : la page de résultats d'une soirée, en HTML, sans aucune dépendance (ni crate du workspace, ni FFmpeg).
  Elle ne lit pas le disque : `cli::written` lui donne un `model::Page`, et elle rend une chaîne.
  Les verdicts se saisissent dans la page, qui les télécharge en `verdicts.csv` à côté d'elle (#228) ; batdetect lit ce fichier en octets (`files::verdicts`) sans jamais l'écrire.
  - Les textes passent par le trait `Labels`, que `Texts` implémente dans le binaire (`sheet.rs`) avec les textes `page-*` des `.ftl` ; la raison et le conseil d'un échec sont les messages `failure` et `failure-todo` du bilan.
    Ceux dont le JS a besoin (`script`, #228) ont chacun une clé, un message Fluent (`page-js-<clé>`) et des trous (`{count}`, `{track}`…) : `sheet` les rend avec chaque trou écrit `{nom}`, la page les embarque en JSON (`<script type="application/json" id="texts">`), et le JS remplit les trous.
    Les en-têtes et le dialecte du fichier de verdicts (séparateur, virgule décimale) sont aussi des textes de la langue.
  - `escape` : le texte est échappé (`& < > " '`), et les chemins encodés en pourcents segment par segment, pour tout octet hors de `A-Z a-z 0-9 - . _ ~`, si bien qu'un `#`, un `%` ou un `?` dans un nom reste un nom de fichier.
    Rien n'est normalisé : le lien suit l'octet du disque.
    `json` écrit une chaîne JSON qui ne peut pas fermer le `<script>` qui la contient (`<`, `>`, `&` en `\u`), et `base64` les octets du fichier de verdicts, sans dépendance.
    - Limite (#227) : les noms arrivent au modèle en chaînes lossy.
      Pour une vidéo au nom non UTF-8 (Linux), la section montre ses pistes, mais ni l'image résumé ni la vidéo annotée (`results` ne les retrouve pas), et les liens vers le tableau, les extraits et le dossier pointent vers `�` et sont cassés.
  - Image redessinée (#259) : quand la vidéo a un fond et l'image sa mise en page, le panneau de l'image porte `caption` (la ligne au-dessus : tranche, pistes, périodes en secondes entières fusionnées, trois au plus puis « + N autres »), le lien vers l'image complète, `scene` et `strip`.
    Sinon, le panneau montre l'image complète. Leurs textes passent par `Label::Scene(SceneText)`.
    - `scene` : un `<svg>` en pixels de la vidéo (`viewBox`), le fond en `<image>`, puis cinq calques dans l'ordre de `compose` : contours noirs, traits de couleur, flèches, traits de rappel, pastilles numérotées.
      Chaque forme porte le `data-id` de sa piste, pour la vignette et les traces cliquables (#269).
      Les traits gardent leur épaisseur à l'écran (`vector-effect: non-scaling-stroke`), pastilles et numéros suivent l'échelle de la mise en page, en police du système.
      `role="img"` et `aria-label` : la liste reste l'équivalent au clavier et au lecteur d'écran.
    - `strip` : la frise en HTML, `aria-hidden`, à hauteur fixe.
      Les bandes « hors analyse » et « illisible » n'y sont que si elles ont une période.
      Les débuts des pistes sont des traits de 3 × 16 px dans une zone de 24 px, à la couleur de la pastille, en gris pour les autres parties.
      Les minutes sont posées dans un élément large d'une minute (`.unit`, `container-type: inline-size`), si bien que des requêtes `@container` choisissent le pas, 1, 2, 5 ou 10 min, selon la place, sans JS.
    - Colonne de droite étroite : à 1180 px, le jugement avait 384 px et la colonne verdict 82 px pour une puce « ● à voir » de 78 px, toutes coupées.
      `.review` est donc un conteneur nommé (`container: review / inline-size`), et sous 27rem la colonne Zoom du tableau disparaît, puisque la tête du jugement dit déjà si la piste a un zoom ; la colonne verdict passe à 140 px.
    - La vignette de la zone de jugement (`judge.js`) est un clone du SVG de la partie, avec la seule pastille de la piste active, agrandie, et des traits plus fins ; sans fond, c'est l'image complète.
    - Traces cliquables (#269, `overlay.js`) : un clic dans la grande image ou la vignette se ramène aux coordonnées de la vidéo (`getScreenCTM`).
      `hits.js` mesure sa distance à chaque piste (trait, flèche, trait de rappel, pastille visibles) et retient celles à moins de 12 px d'écran, la plus proche d'abord.
      Si la piste jugée en est, le clic prend la suivante de cette liste, sinon la plus proche : un second clic au même endroit passe à l'autre trace, sans état caché, et un clic sur la seule trace de la piste jugée ne la relance pas.
      Le survol allume la piste que le clic prendrait (classes `hovering` et `hot`), et un clic avec Cmd, Ctrl, Maj ou Alt est laissé au lien de la vignette.
      Au clavier, les pastilles de la grande image sont des boutons à un seul arrêt de tabulation (flèches, Début, Fin, Entrée), et le SVG devient `role="group"` (son `<image>` de fond `aria-hidden`).
      Leur libellé (piste, verdict en clair, début en mots : `VerdictsOverlay.named`, #337) suit le magasin par `subscribe`, et « Image résumé » rend le focus à la pastille de la piste jugée (`onLeave`).
      Le clone de la vignette perd ces attributs par `VerdictsOverlay.bare`.
      Les formes de chaque SVG sont lues une fois (`WeakMap`), sans tenir compte du filtre ; dans la vignette, seules la pastille et le trait de rappel de la piste active comptent.
      Tout est posé par le script : sans lui, l'image reste une image.
      La vignette, dans un lien, n'a pas d'arrêt clavier, la liste et ← → y suffisant, et une vidéo sans zone de jugement n'a pas de calque.
    - Temps lus par un lecteur d'écran (#338) : chaque temps qu'un lecteur d'écran lit a son double en mots (`Label::Aloud`, rendu par `summary::header::duration` comme les durées de la page, « 3 min 27 s », « 1 h 2 min 5 s »).
      Un intervalle se dit « de 12 s à 31 s » (`SceneText::SpanAloud`), parce que le tiret n'est souvent pas lu.
      Le texte affiché porte `aria-hidden` et son double, `.sr`, le suit : dans la cellule du début (relue par le script pour les pastilles et le lecteur), dans la ligne au-dessus de l'image (rendue deux fois par `caption`) et dans les verdicts sans piste (`VerdictsTime.aloud`).
      La frise reste entièrement `aria-hidden`.
      Au-delà de 1024 px, la liste et les verdicts sans piste sont `position: relative`, pour que ces doubles défilent avec leur ligne.
      Les `.sr` sont hors sélection (`user-select: none`) : copier une ligne ne colle que ce qui est affiché.
    - Filtres et marques (#268) : `filter.js` (`VerdictsFilter`) pose deux barres « Toutes · À voir · Oui · Non · Doute · Autre » avec leur nombre (`store.counts(vidéo)`), l'une en tête de `.pictures`, l'autre en tête de `.review`, montrée seulement pendant le jugement.
      « Autre » n'y est que si une piste de la vidéo a un autre mot, et le choix revient à « Toutes » quand il n'en reste plus.
      Les boutons font 44 px, avec `aria-pressed` ; un seul choix par vidéo, la piste jugée toujours montrée (`VerdictsFilter.passes`, pur), et ‹ › et Enchaîner (`step`) ne passent que par les pistes montrées.
      `overlay.js` fait peindre par `marks.js` chaque élément `data-id` du SVG et de la vignette : classe `verdict-<genre>` (`store.kindOf`), `filtered` (masqué, ni cliquable ni atteignable au clavier), et un petit signe dessiné en SVG près de la pastille de la grande image.
      Le signe est une coche verte, une croix rouge ou un « ? » ambre (rien pour un autre mot), bordé d'un contour noir de 1 px (traits doublés d'un halo, `paint-order` pour le texte) ; il est dessiné parce que les glyphes ✓ ✕ manquent dans la police du système sous Linux.
      Les couleurs `--mark-*` sont fixes, choisies pour qu'au moins le signe ou son contour passe 3:1 sur la scène (ciel noir, roches).
      La vignette n'a ni signe ni gris : la piste regardée y ressort, et les autres sont voilées (opacité 0,45).
      Les « non » prennent `--rejected`, un gris fixe puisque la scène ne suit pas le thème ; dans la frise, leur départ pâlit au lieu de griser, pour ne pas ressembler aux départs des autres parties.
      Dans la frise, chaque départ porte `data-id` (`page::strip`), et un clic mène à la piste la plus proche du pointeur dans 12 px (`VerdictsHits`), car deux départs proches se recouvrent ; deux départs à la même image donnent toujours le plus petit numéro, l'autre s'ouvrant par l'image ou la liste.
      Le survol d'un départ allume sa trace et souligne le départ.
      Le juge appelle `onCurrent` quand la piste jugée change (avant de faire défiler la liste, puis après la vignette) pour repeindre la liste et le calque ; « Image résumé » rend le focus à la pastille de la piste jugée, ou à la suivante montrée si le filtre la cache.
  - La marque `<meta name="generator" content="batdetect">` dit qu'une page est à nous ; `is_ours` la cherche dans le premier Kio.
    Le fichier de verdicts est embarqué en base64 dans `<template id="verdicts-file">`, seulement s'il y a une vidéo.
    Les images ont `width` et `height`, pour qu'un changement d'onglet ne fasse pas sauter la page.
  - `assets/` : `page.css` et onze scripts classiques, inclus dans l'ordre par `include_str!` dans un seul `<script>`, parce qu'un `import` entre fichiers est bloqué en `file://` sous Chrome.
    Chacun expose un nom global (`VerdictsCsv`, `VerdictsStore`, `VerdictsTime`…).
    `csv.js` décode en UTF-8 strict, sinon en Windows-1252, et lit le fichier de façon tolérante.
    `store.js` est le seul endroit où se construit le mot d'un verdict (`word`, #305).
    La page tient en un fichier, sans réseau, avec `preload="none"` sur le lecteur de chaque vidéo.
  - Rattachement d'un verdict à sa piste (`store.js`) : les candidates sont les pistes de la même vidéo (`data-video`, nom du dossier) dont le début est à 0,2 s au plus.
    Les paires se classent par distance des points de départ quand les deux sont connus, sinon par écart de début, et l'affectation est gloutonne, une piste pour un verdict.
    Un verdict sans piste libre est gardé et montré, avec un bouton vers la piste la plus proche ; sur une vidéo sans zone de jugement, la piste la plus proche est nommée en texte, à regarder dans la vidéo (#300).
    Hors jugement, quand la vidéo a une image (`:has(.pictures .panel)`), la colonne de la liste prend la hauteur de l'image par `contain: size`.
    « Verdicts sans piste » y garde sa place au-dessus de la liste, jusqu'à la moitié de la colonne, au-delà de laquelle il défile sous son titre collé en haut ; la liste défile dessous (#299).
    Une vidéo sans extrait, sans zoom et sans image résumé n'a pas de zone de jugement (classe `no-viewer`, #247).
    Ses numéros n'ont ni bouton ni couleur de piste : ils s'écrivent à la couleur du texte du thème sur un fond `--line` sans bordure, pour ne pas ressembler à un bouton (`.dot.plain`, #299), et sa liste se réduit au numéro, au début et à la durée, sans colonnes Verdict ni Zoom.
    Ses pistes ne sont pas « jugeables » (`judgeable`) et restent hors de « à voir » et de « jugées » ; un verdict qu'un fichier leur rattache n'est pas montré, mais compte dans la barre et se réécrit tel quel.
  - Brouillon (`page.js`) : à chaque changement qui s'écarte du fichier lu, les lignes du fichier à écrire vont dans `localStorage` sous `batdetect:verdicts:<chemin de la page>`, avec l'empreinte (nom, date, taille) du fichier lu et l'heure du dernier téléchargement.
    À la réouverture, sur le même fichier, elles sont reprises et la page le dit ; sur un autre fichier, la page demande.
    `beforeunload` demande confirmation tant que des changements ne sont pas téléchargés.
    Un seul endroit écrit le texte (`VerdictsCsv.write(store.rows())`) et un seul l'envoie (le lien de téléchargement) : c'est là que #252 branchera l'enregistrement par un serveur local.
  - Tests du JS : `node --test` sur `csv.js`, `store.js`, `hits.js`, `filter.js`, `time.js`, le nom d'une pastille (`VerdictsOverlay.named`, verdict en clair, #337) et les règles d'opacité de la vignette dans `page.css` (#336), par `assets/tests/*.test.mjs`, qui chargent les scripts par `vm`.
    Ils tournent par la tâche `page:js` de `mise`, dans `check`, et dans la tâche CI `page-js`, sous Linux seulement : ce sont des fonctions pures, et une lecture du texte de `page.css`.
    Les vues se vérifient dans un navigateur.
- **`reference`** → `types` : la lecture et l'écriture des références ([v0.1.1](#référence-v011) et [v0.2.0](#référence-v020)), et l'appariement.
  - `fixture` : chemins des fixtures et garde LFS.
  - `location::Reference` (`V011`, `V020`) choisit le dossier du dépôt (`tests/fixtures/reference/` ou `tests/fixtures/reference-v0.2.0/`) et la variable des données hors dépôt (`BATDETECT_REFERENCE_DIR` ou `BATDETECT_REFERENCE_V020_DIR`).
    `committed(référence, nom)` donne l'extrait et la 092 du dépôt, `committed_settings` les cas de réglages, `recording_directories(référence)` les dossiers `data/*` de la variable, triés par nom ; `compare::case::Target::cases` et `Case::committed` prennent la référence de la même façon.
    Une variable absente ou vide est refusée.
    `recorded_video(nom)` trouve le `.mp4` d'un enregistrement par son nom (`video.name` de `video.json`) dans le dossier de `BATDETECT_VIDEOS_DIR` (sur le Mac de l'enregistrement : `~/dossier sans titre/img_0000/`), avec le même refus.
    Les bancs de la v0.1.1 restent hors dépôt (`recorded_bench(nom)`, `$BATDETECT_REFERENCE_DIR/bench/<nom>/`) ; ceux de la v0.2.0, 2,7 Mo, sont dans git (`committed_bench(nom)`, `tests/fixtures/reference-v0.2.0/bench/<nom>/`).
    `recorded_outputs(langue, nom)` donne `$BATDETECT_REFERENCE_DIR/outputs/<langue>/<nom>/`, les sorties de la v0.1.1 (#115).
  - Un enregistrement a quatre fichiers (`video.json`, `frames.jsonl`, `tracks.jsonl`, `tracks.csv`), un module chacun.
    Un nom en `.gz` est compressé (gzip sans nom ni date), et les sections sans type de `video.json` (`opencv`, réglages, versions) sont gardées telles quelles.
    Relus puis réécrits, les fichiers redonnent les mêmes valeurs.
  - Deux versions du format de `video.json` (#274) :
    - la version 1, celle de la v0.1.1, a un bloc `ffprobe` et un `tracks.csv` ;
    - la version 2, celle de la v0.2.0, remplace `ffprobe` par `decoder`, ce que voit `media` : `error_frames`, `gap_frames`, `losses` (`before_first`, `after_last`) et `threads`.
      Ces pertes au bord ouvrent une plage abîmée, et `frames.jsonl` ne les garde pas (#176).
      `threads` (`DecoderThreads`, comme `media::decode::ActiveThreads`) vaut `{"kind": "single"}`, ou `{"kind": "frame"|"slice", "count": n}` avec n ≥ 1, parce que le nombre de fils change ce que le décodeur dissimule dans les plages abîmées.
      Un nombre nul, un genre inconnu ou un nombre sur `single` sont refusés à la lecture.
      La version 2 n'a pas de `tracks.csv` : `compare`, qui l'enregistre, ne pourrait écrire le CSV du Rust (13 colonnes, en-têtes Fluent de `cli`) sans faire de cycle avec `cli`.
    - `VideoSummary::read` refuse toute autre version, avant de lire les champs.
      `read` et `write` refusent aussi un résumé dont les blocs ne sont pas ceux de sa version (`MissingSection`, `ForeignSection`), et `write` le fait avant d'écrire.
  - Les points d'une piste ne sont pas forcément des détections : les points des pistes jumelles fusionnées sont combinés (17 sur 358 dans l'extrait, 63 sur 610 dans la 092).
  - `detection_matching` : même image, distance strictement inférieure à 3 px, chaque détection appariée au plus une fois, la plus proche d'abord.
    Il rend les paires et les détections seules de chaque côté, et une position non finie reste seule.
    L'appariement est un à un, contrairement au plus proche voisin sans exclusivité du prototype : l'écart, mesuré à la revue de #124, reste sous 0,01 % (8 détections sur 542 261 pour la 103, décalées d'1 px).
  - `track_matching` : début à ±5 images, le plus proche d'abord puis l'écart de points le plus faible ; une paire s'accorde si ses points diffèrent de 3 au plus.
    Il est glouton, si bien que deux pistes à moins de 10 images l'une de l'autre peuvent laisser une paire possible de côté (un test le fige).
    Les écarts à juger (piste manquée, en trop, points hors tolérance) portent la vidéo, l'image de début et son temps par la cadence (`FrameRate::seconds`), triés par image.
  - `bench` (#107, #108) : les fichiers du banc de la v0.1.1, que le Rust réécrit au même format.
    Dans `bench.json`, les réglages restent en JSON brut, les clés que la v0.1.1 n'écrit pas (`targets`, `seconds`) sont gardées dans `added`, et les entiers du Python se lisent en `f64`.
    Une ligne enregistrée se réécrit octet pour octet.
- **`engine`** → `media`, `types` : le moteur, dont `input` est le seul point d'entrée.
  - `Input::open(chemin, Threads)` accepte toute taille et toute plage.
    Il refuse, par une erreur typée qui nomme la vidéo, 0 image annoncée et, par `media`, un format autre que `yuv420p` ou `yuvj420p`.
    `Video` donne la taille, la cadence, la base de temps, le `pts` de départ et le nombre annoncé avant la lecture, mais pas la plage, qui ne sert à personne :
    la détection lit le plan Y tel quel, avec la même table quelle que soit l'étiquette (décision du 12 octobre 2026), et l'affichage force la plage pleine quelle que soit l'étiquette (décision du 11 octobre).
    Une vraie source `pc` est donc détectée sur les mêmes Y qu'une source `tv`, et « au moins aussi bien que la v0.1.1 » ne s'y mesure pas, OpenCV la lisant autrement ; aucune des 26 vidéos n'est concernée.
  - Lecture anticipée (`read_ahead`) : un fil de `std::thread::scope` décode jusqu'à 4 images d'avance (`READ_AHEAD`) dans 5 tampons faits une fois puis recyclés, et s'arrête dès que le consommateur s'arrête ou échoue.
    Les tampons vivent tous jusqu'à ce que le consommateur ait fini, même quand le fil a déjà atteint la fin du fichier ou échoué : le fil rend sa file de tampons et celui qu'il tenait, libérés après le consommateur (#321), pour que le pic de mémoire ne dépende pas de la vitesse du décodage (voir le test de mémoire plus bas).
    Sur une vraie vidéo, le pic ne bouge pas : au-delà d'environ 1 800 images (50 × 36), les 25 images du fond sont gardées avant que le fil puisse approcher de la fin, et le vidage final (`Detector::finish`) vient après la libération des tampons (déduit, non mesuré).
  - Pertes (`Losses`), qui deviennent des plages abîmées (`damage`) :
    - `at_jumps`, aux sauts de `pts` : le pas est la moyenne des intervalles réguliers selon la médiane des écarts de `pts` entre images connues (juste aussi pour un pas fractionnaire), et un écart de k pas arrondis perd k − 1 images ;
    - `before_first`, avant la première image : l'écart entre son `pts` et le `pts` de départ du flux, en pas arrondis, moins les images déjà rendues sans `pts` ; 0 si le départ ou le pas est inconnu ;
    - `after_last`, après la dernière : les paquets illisibles qui la suivent (un paquet par image ; mesuré égal aux images perdues sur une fin abîmée, avec 1, 3 fils ou en automatique).
  - Garde du nombre d'images : décodées = annoncées − toutes les pertes.
    Seule une perte inexpliquée, qui ne laisse ni saut ni paquet illisible au bord, fait refuser la vidéo (`FrameCount`).
  - `damage` : `Damage::of(lecture, marge)` rend les images en erreur (`DecodeErrors` non vides), les images qui suivent un saut de `pts` et les plages abîmées (`Vec<Span>`).
    `reported_frames` compte l'union des images en erreur et après un saut (`damaged_frames` de `params.json`).
    - Une plage part d'une image en erreur ou d'une image après un saut, comme `damaged_spans` de la v0.1.1.
    - Elle part aussi de la première image s'il manque des images avant elle (`before_first`, l'équivalent d'un saut), et d'un départ fictif juste après la fin s'il en manque après la dernière (`after_last`).
      La plage ne couvre alors que les `marge` dernières images, et aucune si la marge est nulle, car les images perdues n'entrent dans aucune fenêtre de fond.
      La v0.1.1 refusait ces vidéos (`check_frame_count`) ; le Rust les lit et exclut ces plages (décision de l'utilisateur à #116).
    - Le saut se juge avec le pas des pertes (`at_jumps`) : un écart de k images en vaut plus que k s'il dépasse k + ½ pas, au lieu de 1,5 × k × pas médian en v0.1.1.
      Le résultat est le même quand tous les `pts` sont connus (k = 1, les 26 vidéos) ; à travers un `pts` inconnu (k ≥ 2), le Rust compte l'image cachée, la v0.1.1 non.
    - La plage va jusqu'à l'image qui précède l'image-clé suivante, ou jusqu'à la fin.
    - Elle est élargie de la marge de chaque côté, bornée à la vidéo, puis fondue avec la précédente si elle la touche.
      La marge est la demi-fenêtre du fond, passée par l'appelant.
    - `Damage::reported(rapports, images-clés, nombre d'images, marge)` (#275) trace ces plages à partir de ce qu'une lecture rapporte (`Reports` : images en erreur, images après un saut, pertes aux bords), triés et sans doublon.
      `Damage::of` passe par lui, et `compare` s'en sert pour réunir ce que rapportent la référence et la lecture courante.
  - Canari (`tests/slow_canary.rs`, sur chaque système de la CI) : la couture coupe le dernier tiers de la tranche des images 248 (P au milieu d'un GOP de 15), 555 (image-clé), 860 et 861 (consécutives) et 1222 (dernier GOP) de l'extrait, en réécrivant la longueur du NAL.
    - Le Rust signale exactement {248, 555, 860, 861, 1222}, avec 1 fil comme en automatique.
    - ffprobe 9.0.2 (Homebrew, `-threads 1 -show_log 16`, les arguments de la v0.1.1) en signale autant sur une copie coupée de la même façon, mesurée une fois : `ffmpeg -c copy -bsf:v h264_mp4toannexb -f h264`, coupe des mêmes tranches, puis `ffmpeg -framerate 10000/333 -i cut.h264 -c copy cut.mp4`.
      L'ensemble est écrit en constante, et le test vérifie que celui du Rust le contient.
  - Fils et pixels dissimulés (règle 16) : une vidéo saine donne les mêmes pixels quel que soit le nombre de fils.
    Sur une tranche corrompue, les images en erreur, les sauts et les plages ne changent pas, mais les pixels de l'image dissimulée et des suivantes jusqu'à l'image-clé peuvent changer.
    - Sur le canari de l'extrait, 40 images diffèrent entre 1 fil et l'automatique, toutes dans les plages ; les 1 190 autres sont identiques au pixel près.
    - Sur la 103, la 122 et la 124 (`tests/local_threads.rs`), 105, 1 189 et 1 472 images diffèrent, puis 105, 1 151 et 1 447 à la mesure suivante, toutes dans les plages abîmées (marge 15).
      Les pixels dissimulés en fils d'image ne se reproduisent donc pas d'une lecture à l'autre ; les images en erreur, les sauts et les plages, si.

    Ces images sont hors de la règle 16, qui vaut pour ce qui sort du moteur hors des plages abîmées.
  - Lecture des 26 vidéos (M1 Max, `--profile ci`) : l'extrait et la 092 donnent 1 230 et 9 011 images, aux cadences `avg_frame_rate` de leur `video.json`, avec les `pts` et les images-clés de ffprobe.
    Les 26 vidéos donnent toutes `counts.frames` (103 : 8 998, 120 : 8 996, 122 : 8 998), sans refus.
  - Dégâts contre la référence v0.1.1 (marge 15, `tests/local_damage.rs`) : l'extrait et la 092 n'ont aucune image abîmée.
    Sur les 26 vidéos, les images après un saut sont celles d'ffprobe, et les images en erreur les contiennent.
    25 vidéos ont exactement ses images en erreur et ses plages (093 : 11 images, 419 abîmées ; 122 : 195 et 6 043).
    - Seule la 124 diffère, sur le Mac : 310 images en erreur contre 309, et 7 458 abîmées contre 7 415 (+43).
      L'image en plus, la 2197, porte le seul drapeau `CONCEALMENT_ACTIVE`, avec 1 fil comme en automatique.
      ffprobe ne l'écrit qu'au niveau `info` (« concealing 392 DC, 392 AC, 392 MV errors in P frame »), que la v0.1.1 ne lit pas (`-show_log 16`), et c'est la seule image des 26 vidéos dans ce cas.
      Le Rust exclut donc 43 images de plus que la v0.1.1 sur la 124 (environ 1,4 s) ; reste à juger si c'est souhaitable (image réellement dissimulée) ou trop prudent (passage possible perdu).
    - Sur lumiel (#250, 17 octobre 2026), le même FFmpeg 7.1.1 signale 14 images de plus que ffprobe sur 8 vidéos, la 2197 de la 124 non comptée : 093 (1307, 7395 ; 449 images abîmées), 097 (5444 ; 535), 120 (4589 ; 2 215), 121 (8844 ; 4 912, plages inchangées), 122 (2264 ; 6 073), 123 (330, 1250, 2267, 3986, 4902, 5417 ; 6 627), 124 (3643 ; 7 458, comme sur le Mac) et 128 (2830 ; 1 006).
      Trois lectures de chaque vidéo, deux en fils automatiques et une à un fil, donnent les mêmes images et les mêmes plages.
      La cause de l'écart entre les deux systèmes n'est pas isolée.
      `local_damage` accepte donc, par vidéo, soit aucune de ces images, soit exactement toutes, avec leur nombre d'images abîmées (`FLAGGED_ONLY_BY_SOME_SYSTEMS`), et passe sur les deux systèmes.
  - **Détection** (#118) : `detection::detect(chemin, Threads, &DetectConfig)`, ou `detect_input(Input, &DetectConfig)` pour une entrée déjà ouverte (le canari y passe sa couture).
    - `detect_injected(Input, &DetectConfig, crochet)` (#107) appelle le crochet sur la copie du plan Y que garde le détecteur, une fois par image, dans l'ordre de lecture, avec son `FrameIndex` et une `LumaMut` (vue mutable du plan, à la taille d'origine).
      Les images du fond de l'image résumé reçoivent l'image lue, avant la copie, si bien que le crochet ne les touche pas.
      `detect_input` passe un crochet vide ; le crochet est générique, et la commande ne paie ni appel indirect ni branche.
      `LumaMut::new(pixels, largeur)` refuse un plan qui n'est pas un nombre entier de lignes non vides.
    - Le pool `rayon` est celui de l'appelant (`ThreadPool::install`).
    - `config` : les valeurs de `DetectConfig::default()` sont justifiées dans [02](02-methode.md).
      `half_window(cadence, pixels de travail)` arrondit `fenêtre × cadence / 2` au plus proche, moitié vers le haut (la v0.1.1 arrondit au pair : une image d'écart quand ce produit vaut x,5, jamais à 30 i/s), et refuse moins de 3 images de fenêtre ou moins de 3 échantillons.
      Le nombre d'échantillons d'une cible intérieure, `⌊2·demi-fenêtre / pas⌋ + 1` (`window::most_sampled`, saturant), dimensionne les réseaux et les tampons.
      Sur la 092 : 15, et 11 échantillons ; `window_2s` : 30 et 21 ; 2 s au pas de 1 : 61.
      - Borne haute (#113) : la fenêtre garde toutes ses images, quel que soit le pas, à la taille d'un `window::Unit` (`u16`) par pixel de travail.
        `half_window` refuse (`LongWindow`), en `f64` et avant tout calcul entier, une fenêtre dont les images dépassent `BACKGROUND_BUDGET` = 4 Gio : 3 105 images à 960×720, 1 379 à 1440×1080 ; une fenêtre infinie est refusée aussi, même sans pixel.
        Une fenêtre absurde ou infinie (`--bg-window` du banc, `compare`) donne ainsi cette erreur au lieu d'un débordement ou d'un manque de mémoire.
        La commande n'expose pas la fenêtre, et le défaut en prend 31 images.
      - Mesure `dhat` (13 octobre 2026, vidéo fabriquée de 3 200 images 1440×1080, travail 960×720, pic du tas de la détection) : 31 images, 368 Mo ; 301, 876 Mo (884 Mo au pas de 1) ; 1 003, 1 857 Mo (1 992 Mo au pas de 1).
        Entre 301 et 1 003 images, chaque image de fenêtre coûte 1,40 Mo (2,02 octets par pixel de travail : le modèle du budget tient à 1 % près), sur environ 0,4 Go fixes ; le pas de 1 ajoute les réseaux de tri, 0,13 Go à 1 003 images.
        Au plafond, la détection tient donc autour de 5 Go au pas de 3, sous la mémoire de la machine de Manon (16 Go).
        La mesure s'est arrêtée avant les points de 2 000 et 3 105 images (verrou de mesure rendu).
    - `settings` (#149) : `DetectSettings`, les dix réglages de la v0.1.1 sous leurs noms Python (`threshold`, `min_area`, `max_area`, `bg_window_s`, `bg_step`, `osd_regions`, `merge_radius`, `work_width`, `noise_factor`, `target_sigma`).
      Le même type lit `video.json` et écrit `params.json`, dans l'ordre de la dataclasse de la v0.1.1, et `Default` se déduit de `DetectConfig::default()` : une seule source des valeurs.
      Une zone OSD (`Region`) se convertit en pixels par `[⌊f0·L⌋ − marge, ⌈f1·L⌉ + marge)`, borné à l'image.
      `DetectConfig::try_from` refuse ce que la v0.1.1 refuse (`Error::Setting` avec son `Setting` et la borne enfreinte, `Areas`, `Region` avec son rang), avec les mêmes écarts que le suivi : NaN et infini refusés.
      Écart voulu (#151) : `merge_radius` est borné à 12 pixels d'origine (`Bound::AtMost`), là où la v0.1.1 acceptait tout.
      Une image d'éclair plein cadre, sur un fil, en `release`, prend 0,34 s au rayon 6 et 0,99 s au rayon 12 au travail en 1440, 3,77 s au rayon 25 ; 0,81 s au rayon 25 au travail en 960.
      La borne tient une seconde par image à toute largeur, et recolle encore les fragments écartés de 18 à 24 px de [02](02-methode.md).
      Aucun réglage valide n'est refusé.
    - `geometry` : toute réduction, largeur et hauteur de travail de `WorkScale` (largeur demandée sans agrandir, hauteur arrondie), refus d'un travail de plus de `u32::MAX` pixels.
      Seul `WorkScale` passe aux pixels d'origine, avec l'échelle des largeurs pour les deux axes, comme la v0.1.1 : quand la hauteur de travail est arrondie, y s'écarte de moins de 0,5 pixel d'origine en bas de l'image.
    - `reduction` (#150) : Y passe au gris par la forme close de la table `GRAY` (#214), puis la réduction se fait par aire, axe par axe.
      Le rapport source / travail est réduit à p/q : le pixel de travail i couvre la source de i·p à (i+1)·p en unités de 1/q, et chaque pixel source pèse la longueur de son intersection.
      Les deux axes ont leur propre rapport (hauteur arrondie, comme `fx` et `fy` d'`INTER_AREA`).
      Un seul arrondi à 8 bits, `(total + T/2) / T` avec T = p_x × p_y, par un inverse sur 32 bits dans les blocs fixes ci-dessous, sinon par l'inverse sur 64 bits (exact pour un total sur 32 bits, méthode de Lemire), par une vraie division au-delà.
      En 3:2, T = 9, avec les poids 2, 1 puis 1, 2 ; un proptest compare la réduction à la réduction 3:2 d'origine et à un calcul brut des aires.
      La passe horizontale va par blocs de q pixels de travail, le nombre de poids par pixel étant fixé à la compilation, de 1 à 4.
      - Blocs fixes (#214) : pour les rapports 2:1, 3:1, 3:2 et 4:3 (largeurs de travail 720 et 480, le défaut 960, et 960 sur une source de 1280), quand T ≤ 128, la passe horizontale prend des blocs choisis à la construction.
        Chaque bloc a p pixels source et q pixels de travail fixés à la compilation, avec les poids de chaque pixel de travail sur tout le bloc, en tableaux fixes (zéro hors de son aire).
        Le total tient sur 32 bits et s'arrondit par `((total + T/2) × (⌊2²³ / T⌋ + 1)) >> 23`, sans dépasser 32 bits ; un test exhaustif le vérifie contre `(total + T/2) / T` pour chaque T de 1 à 128 et chaque total de 0 à 255 × T.
        LLVM vectorise alors la boucle des blocs dans le binaire livré, ce que la forme générale ne permettait pas (poids lus par indice, inverse sur 128 bits) : en `x86-64-v2`, 4 blocs 3:2 par tour, lus au pas de 3 par `pinsrd` ; en aarch64, lus au pas de 3 par `ld3.4s`, puis `mla.4s` et `ushr.4s #23`.
        Sans LTO, la crate seule reste scalaire en aarch64 : la vectorisation ne se juge que sur le binaire.
        Les autres rapports gardent la forme générale.
        Sur un fil, en `release` : 0,63 ms par image à 960 hors gris contre 1,23 sur lumiel, 0,44 ms gris compris contre 1,18 sur le M1 Max ([09](09-profilage.md)).
      - Taille inchangée (1:1, #159) : quand la source a déjà la largeur et la hauteur de travail, la réduction n'est que le gris, sans aire, sans arrondi et sans copie du gris de la source.
        Sur un fil, en `release` : 0,41 ms par image à 1440 contre 1,84.
        Sur `092-1440` : 158,5 s de CPU contre 166,1 (−4,6 %, trois paires A/B en alternance avec `main`), 613 Mo de mémoire résidente contre 628.
      - Essayés et écartés par #159, mesurés à 960 contre 1,17 ms par image sur le M1 Max : les champs de l'arrondi pris dans les fermetures (1,18 ms, LLVM le faisait déjà), des poids contigus avec totaux et inverse sur 32 bits (1,39 ms), l'inverse sur 32 bits seul (1,21 ms).
    - `blur` (#150) : noyau gaussien de σ = `target_sigma / échelle` pixels de travail, de portée ⌈4σ⌉ (celle d'OpenCV pour tous les σ du banc).
      Les poids `exp(−d²/2σ²)` (le poids central posé à 1, sans formule) se partagent 4 096 parts par les plus forts restes, par paires symétriques : la somme est exacte, le noyau symétrique, et aucun poids n'est négatif.
      σ = 0 donne l'identité, `[4096]`, et σ = 1 le noyau `[1, 18, 221, 991, 1634, …]`.
      Dès un σ de travail d'environ 11,6, le centre, qui ne reçoit que l'unité impaire, peut tomber d'une part sous son voisin ; cela n'arrive jamais dans le banc (σ de travail de 2 au plus).
      Un côté de travail plus étroit que le noyau (2 × portée + 1) est refusé (`TooSmall`) avant toute allocation.
    - `prepare` : la réduction, puis le flou (horizontal à longueur de noyau fixée à la compilation de 1 à 17, une boucle générique au-delà ; bords en miroir sans répéter le pixel du bord), avec un seul arrondi final en 1/64 de niveau (`u16`, 16 320 au plus ; passe verticale en `u32`, où 255 × 4 096² tient puisque le noyau somme exactement 4 096).
      Le niveau d'une image est la somme de ses unités hors du masque : la somme de chaque ligne, moins celle de ses colonnes masquées.
      - Noyau plié (#159) : le noyau étant symétrique, la passe verticale et la passe horizontale à longueur fixée additionnent les deux pixels à même distance du centre avant de multiplier par leur poids commun.
        Chaque somme partielle reste sous le total, donc au plus 255 × 4 096², et rien ne déborde, ce que vérifie une image blanche de σ = 0 à 2,75 en profil `ci`.
        Flou seul, sur un fil, en `release` : 0,59 ms par image contre 0,77 à 960, 1,76 contre 2,50 à 1440 ; sur la 092, 109,5 s de CPU contre 111,1 (−1,4 %, trois paires A/B).
    - `network` : la médiane par le réseau de Batcher coupé à n entrées et élagué aux comparaisons qui atteignent le milieu, sur des blocs de 64 pixels.
      Il est testé en 0-1 exhaustif de 1 à 20 (64 motifs par appel) et par un proptest contre un tri de 1 à 128.
      Si n est pair, la médiane est la demi-somme des deux du milieu.
      Il y a un réseau par compte, de 1 au maximum (#151), avec les paires en `Box<[_]>` : 2,3 Ko à 11 échantillons, 177 Ko à 61, 8,7 Mo à 301, 566 Mo à 1 801 (60 s au pas de 1), soit 23 à 30 % de la fenêtre de 1 801 images.
      - Onze échantillons (#213), la fenêtre par défaut : une table de 28 comparateurs (`MEDIAN_OF_ELEVEN`) au lieu des 32 de l'élagage.
        On l'a trouvée hors du dépôt en retirant un à un des comparateurs du réseau élagué tant que la médiane restait exacte sur les 2 048 entrées 0/1.
        Une construction aléatoire suivie du même élagage n'est pas descendue sous 28, sans preuve que ce soit le minimum.
        Le test 0-1 exhaustif la couvre comme les autres comptes.
        Le réseau prend 12,5 % de cycles en moins, et la détection de la 092 2,3 % de temps en moins sur lumiel ; garder les 11 lignes dans les registres était plus lent ([09](09-profilage.md)).
    - `threshold` : résidu `f32` = image − fond − (niveau − niveau moyen), le gain se calculant en entiers jusqu'à une seule division.
      Les candidats sont au-dessus du plancher (12 niveaux).
      Avec au moins 7 échantillons et k > 0, le seuil se relève à max(12, 8 × 1,4826 × MAD des écarts corrigés du gain), pour les seuls candidats ; avec k = 0, il vaudrait le plancher et n'est pas calculé.
      Un candidat sous le masque est écarté avant le seuil relevé.
      - Blocs masqués (#159) : un bloc de 64 pixels tout entier sous le masque est sauté avant la copie des échantillons et le réseau.
        Sur un fil, en `release`, à 960 avec la bande `osd` : 2,19 ms par image contre 2,36 ; sur l'extrait aux réglages `osd`, 14,66 s de CPU contre 14,94 (−1,9 %, trois paires A/B).
      - Tampons : les lignes du réseau sont sur la pile jusqu'à 64 échantillons (`STACKED_SAMPLES`, 8 Ko), sur le tas de chaque fil au-delà ; écarts et gains vivent dans des tampons de chaque fil, dimensionnés une fois.
        Les mêmes lignes sur le tas coûtaient 7 % de temps CPU sur la 092 (117 s contre 110 s, mesuré en alternance avec `main`).
        Par ablation, ni le test du masque, ni les écarts, ni la copie des deux valeurs du milieu, ni l'inlining du tri n'en sont la cause, et le mécanisme n'est pas isolé.
        Les deux chemins sont testés (16, 64, 65 et 80 échantillons).
    - `mask` (#151) : les zones OSD en pixels de travail, une liste d'intervalles de colonnes fusionnés par ligne (vide sans zone).
      La marge est la portée du noyau, ⌈4σ⌉ pixels de travail (⌈3σ⌉ dans la v0.1.1), si bien que les unités préparées sous la bande ne dépendent pas de l'écran, au bit près.
      Si aucun pixel ne reste, la détection refuse `WholeFrame` (`detection::Error::Mask`).
      Le niveau moyen exclut le masque : une bande qui scintille ne change pas le gain.
      Sur le banc, la marge de 4σ ne perd aucune détection (`osd` et `width_1440_osd`).
    - `closing` : fermeture par le disque de rayon `round(merge_radius / échelle)`, moitié vers le haut (#150).
      La v0.1.1 arrondit au pair, d'où un rayon d'écart quand ce quotient vaut x,5, par exemple 3,75 à l'échelle 1,5 ; aux défauts, le rayon est 4, de demi-largeurs `[0, 3, 3, 4, 4, 4, 3, 3, 0]`.
      La demi-largeur de chaque ligne vaut `round(√(r² − dy²))`, comme l'ellipse d'OpenCV, et `sqrt` est exact par IEEE 754.
      Au rayon 0, la fermeture rend les pixels allumés.
      Le disque ne garde que les lignes et les demi-largeurs qui peuvent tomber dans l'image, ce qui borne sa taille sans rien changer.
      La fermeture est creuse : seuls les pixels touchés par la dilatation sont marqués, puis érodés, et le dehors de l'image compte comme allumé.
      Un proptest la compare à une fermeture dense.
    - `components` : composantes 8-connexes par union-find, dans l'ordre de balayage de leur premier pixel (haut, puis gauche).
      La v0.1.1 suit la numérotation d'OpenCV, par blocs de 2×2 pixels : une tache qui commence une ligne plus bas peut sortir avant (dans sa référence, 12 280 des 29 650 images à plusieurs détections ont au moins une inversion d'une ligne).
      Le suivi en dépend en cas d'égalité, et le Rust ne l'imite pas (règle 5).
      La boîte et le barycentre sont ceux de la composante fermée ; la surface vaut pixels allumés × échelle², l'amplitude le plus grand |résidu| en niveaux de gris, et le filtre garde `min_area` ≤ surface ≤ `max_area`.
      `components` rend des `Measures`, sans image : le détecteur les range dans `FrameDetections` par `push`, dans l'ordre des images, et le test des fils et des lots (`detector`, 1, 3 ou 4 fils, lots de 1 à 64) garde cet ordre.
      `threshold` range les pixels au-dessus des trois quarts du seuil dans une seconde liste (`Scratch::weak`, #21) ; `closing` et `components` prennent les deux listes sans les fusionner, puisque la fermeture ne dépend pas de l'ordre, et une composante est forte si elle a un pixel de la première (`Strength`, dans `types`).
    - `bracketing` (#21) : après `Detector::finish`, garde une détection faible encadrée par deux fortes (règle et mesures dans [10](10-logique-de-detection.md#5-ter-garder-les-taches-faibles-qui-comblent-un-vol)).
      Il est dans `detect_injected`, donc vaut pour la commande, `compare` et le banc.
    - `window` : les échantillons se prennent une image sur 3 depuis le début de la fenêtre, coupée aux bords de la vidéo.
    - `detector` : lots de 32 images (copie du plan Y).
      Chaque lot est préparé, puis les cibles prêtes (demi-fenêtre suivante lue, ou fin) sont détectées, en groupes contigus zippés avec une réserve de tampons (un par fil `rayon`), et rangées dans l'ordre.
      La lecture anticipée passe à 36 images pour la détection, alors que `read` en garde 4.
    - `background` : le fond de l'image résumé (#104).
      Les 25 images aux rangs `(2i + 1) × N / 50` (N annoncé ; toutes si N < 25) sont gardées en YUV entier pendant la lecture.
      Le rang est exact, sans le positionnement imprécis d'OpenCV, et un rang perdu en fin de vidéo donne moins d'images, comme dans la v0.1.1.
      - À la fin de la lecture, les images qui tombent dans une plage abîmée sont écartées, sauf s'il n'en reste aucune (décision de l'utilisateur, 12 octobre 2026).
        Leurs pixels masqués par FFmpeg dépendent du nombre de fils (règle 16), et elles sont la majorité sur 121 à 124 : 15 à 21 rangs sur 25 sur les plages de la v0.1.1, et en Rust 16 écartées sur la 122 (9 gardées), 21 sur la 124 (4 gardées).
      - La médiane se prend plan par plan, échantillon par échantillon, par sélection (`select_nth_unstable`) sur un tampon réutilisé.
        En nombre pair, c'est la demi-somme des deux valeurs du milieu, arrondie au plus proche (règle 5).
        Elle prend 0,29 s sur la 092 (profil `ci`).
      - Les 25 images gardées sont libérées à la fin de la lecture.
    - Différences avec la v0.1.1, hors gris :
      - le Rust passe en gris puis réduit, quand la v0.1.1 réduit l'image en couleur puis passe en gris ;
      - le noyau est quantifié sur 4 096, quand la v0.1.1 floute en `float32` avec le noyau exact ;
      - médiane et MAD portent sur des unités de 1/64 de niveau ;
      - résidu et écarts sont en `f32` (`float64` en v0.1.1), et le niveau moyen est exact ;
      - l'ordre des taches diffère (voir `components`).
    - Le test de saut de luminosité à seuil fixe de la v0.1.1 n'a pas d'équivalent aux réglages par défaut : un saut de 30 niveaux allume l'image au bord du saut, ce que la stabilité retire (tests de `compare`).
    - Règle 16 : même sortie au bit près avec 1, 3 ou 4 fils et des lots de 1 à 64 (test rapide).
      Sur le canari de l'extrait (`slow`), 1 fil de décodage et 1 fil `rayon` donnent, contre l'automatique, les mêmes plages et les mêmes détections hors plages.
      Sur l'extrait, la 092 et les 26 vidéos, le parallèle redonne le séquentiel au bit près hors plages abîmées ; dans les plages, les pixels dissimulés varient d'une lecture à l'autre.
    - Une boucle de comparaisons écrite élément par élément se vectorisait mal une fois inlinée en `release` (détection 4 fois plus lente qu'en `ci`) ; écrite sur des tableaux entiers, elle se vectorise.
    - Mémoire (`tests/slow_memory_flash.rs`, `dhat`, 4 fils, 64 images) : un éclair plein cadre dans chaque groupe de cibles, au premier lot, coûte 67,1 Mo de plus qu'une vidéo calme, soit 24,27 octets par pixel de travail et par fil.
      Le compte : les pixels allumés (8 octets), touchés et fermés (4 octets chacun) d'un fil montent jusqu'à 2²⁰ éléments, 24,3 octets par pixel, gardés jusqu'à la fin (environ 170 Mo à 10 fils).
      Le test exige plus de 16 et au plus 32 octets ; les allocations de FFmpeg, les mêmes dans les deux vidéos, ne sont pas vues.
      `WORK_PIXELS` vient de `WorkScale` et de la configuration.
      - `tests/slow_memory_flash_wide.rs`, à 1440 pixels de travail (1 440 × 1 080) : 134,2 Mo de plus, 21,58 octets par pixel et par fil.
      - `tests/slow_memory_flash_long.rs`, 2 s au pas de 1, soit 61 échantillons : 67,1 Mo de plus, 24,27 octets par pixel et par fil.
        Les éclairs y tombent dans le vidage de l'image 64 (images 3, 11, 19 et 27, cibles 2 à 33 en 4 groupes de 8), où chacun des 4 fils a une cible éclairée à cette demi-fenêtre de 30.
        Le pic de la vidéo calme, 319 Mo, tombe dans ce même vidage et porte la fenêtre (2 octets par pixel et par image gardée).
      - Le pic au calme tombe pendant la lecture, avec les 37 tampons de la lecture anticipée.
        Quand le fil de lecture libérait ses tampons à sa fin de fichier, le pic au calme du cas long dépendait de la vitesse du décodage (244,8 Mo ou 272 à 275 Mo), et le surcoût mesuré pouvait tomber sous la borne basse (#321).
        Mesuré sur lumiel depuis : 24,27, 21,58 et 24,27 octets par pixel et par fil, à 2 Ko près sur 50 passes (cas long 20 à 12 cœurs et 10 à 3 cœurs, 10 de chacun des deux autres), et le même pic au calme du cas long, à 8 Ko près sur 6 profils, avec une lecture rapide ou ralentie.
      - Avec les pixels faibles (#21), dans leur propre liste et sans recopie des forts : 24,32 octets par pixel et par fil pour le flash et pour le cas long, 21,58 pour le cas large (lumiel, après #321 et #24).
        Non testé : un éclair fort puis un éclair entre 0,75 et 1 du seuil sur une autre image, où chaque liste garde sa capacité, environ 8 octets de plus par pixel et par fil.
  - **Gris de la détection** (#118, exception à la règle 5) : chaque Y passe par une table de 256 gris mesurée sur OpenCV (`GRAY` dans `gray`, privée, lue par `gray::gray(y)`, `const fn` publique).
    La table vient d'une rampe Y = 0 à 255, U = V = 128, encodée sans perte en plage `tv`, lue par `cv2.VideoCapture` puis `COLOR_BGR2GRAY`.
    Le calcul passe par sa forme close (`gray::computed`, #214), `clamp((Y × 4769 − 81 520) >> 12, 0, 255)`, égale à la table sur les 256 valeurs (test exhaustif) : la lecture de la table restait scalaire en `x86-64-v2`, alors que la forme close se vectorise dans le binaire livré (`pmaddwd` en `x86-64-v2` ; `ushll`, `mla.4s` et `sshr.4s #12` en aarch64, 16 octets par tour).
    La réduction pondère ensuite ces gris et arrondit à 8 bits avant la gaussienne.
    La table suit `(Y − 16) × 255 / 219` de 1,2 à 2,3 niveaux en dessous, en dents de scie, à 0 jusqu'à Y = 17 et à 255 dès Y = 237.
    Ablation (M1 Max, un choix à la fois, taux d'appariement des détections référence / Rust contre la v0.1.1, hors plages abîmées et instables de `video.json`) :

    | Variante | Extrait | 092 | 089 | Pire des 26 | Sous 98 % | Points de piste perdus |
    |---|---|---|---|---|---|---|
    | (a) table et arrondi à 8 bits | 100 / 99,75 | 100 / 99,87 | 98,19 / 98,46 | 98,19 | 0 | 19 |
    | (b) formule, écrêtée, arrondie | 98,51 / 98,75 | 96,15 / 95,30 | 70,36 / 72,92 | 70,36 | 25 | 415 |
    | (c) formule écrêtée, exacte | 97,51 / 98,25 | 95,38 / 96,00 | 75,58 / 76,76 | 75,58 | 22 | 340 |
    | (d) formule exacte, sans écrêtage | 97,76 / 98,74 | 95,13 / 95,13 | 72,12 / 70,61 | 70,61 | 23 | 358 |

    - Les écarts de (d) sont surtout du bruit au seuil : sur la 089, 489 des 508 détections perdues sont des taches de 4,5 px² sous 20 niveaux, et autant sont ajoutées.
      Un décalage de quelques dixièmes de niveau suffit à les faire basculer.
    - Au niveau des pistes, sur les 966 de la référence : (a) en apparie 963 (958 concordantes), en perd 3 et en ajoute 9 ; (d) en apparie 936 (912), en perd 30 et en ajoute 35.
      Sans jumelle à 1 s, (d) perd vraiment 17 pistes, dont la 097 0:34 (102 points) et la 100 2:00 (24 points), et en ajoute vraiment 22.
    - La table s'écarte aussi du `BGR2GRAY` d'OpenCV quand la chrominance n'est pas neutre : dans la bande d'affichage des jumelles (les 7 % du haut de l'image, colorés, V jusqu'à 255), l'écart va jusqu'à 56 niveaux sur l'image 40 de l'extrait (972 pixels à 3 niveaux ou plus), et il est nul ailleurs (U = V = 128).
      C'est pourquoi `sigma_0` et `window_2s` s'écartent de la v0.1.1 dans cette bande (#150, #151) ; l'utilisateur a gardé la table.
    - La formule sera retentée après la 0.2.0 (issue en Backlog).
- **`cli`** → `engine`, `tracking`, `files`, `summary`, `render`, `draw`, `media`, `types` : paquet `batdetect`, binaire `batdetect` (#123).
  Tout texte vu par l'utilisateur vit dans les deux catalogues Fluent `locales/{fr,en}.ftl`, lus par `texts.rs` (#114) ; `messages.rs` fait les messages à partir des vidéos et des erreurs, `arguments.rs` la ligne de commande.
  La bibliothèque (`failure`, `jobs`, `language`, `options`, `process`, `progress`, `settings`, `tally`, `writable`) sert aux tests.
  - `batdetect VIDÉO|DOSSIER… [-o DOSSIER]`.
    Sans `-o`/`--out-dir`, les résultats de chaque vidéo vont dans `<dossier de la vidéo>/batdetect/` (#86), ce que dit l'aide (« [par défaut : batdetect/ à côté de chaque vidéo] »).
  - **Autres commandes** (#58, #59) : `commands::other_command` (binaire) lit le premier argument avant `arguments::parse`.
    `doctor` et `completion` partent vers leur module, et tout le reste vers le parseur, sans changer `batdetect <vidéos>`.
    Une vidéo ou un dossier nommé ainsi se donne en `./doctor` ou `./completion`, ce que dit l'aide (`other-commands`, sous les options de `-h` et de `--help`).
    - `batdetect completion zsh` écrit sur la sortie standard le script `_batdetect` que produit `clap_complete` depuis le parseur complet (`arguments::full_command`, `Level::Full`), dans la langue du système.
      La ligne du positionnel est réécrite (`completion::inputs_spec`) : répétée (`*:`), elle appelle `_batdetect_inputs`, ajoutée au script (`completion::inputs_function`).
      Cette fonction propose `doctor` et `completion` avec leur description au premier mot seulement (`CURRENT == 2`), puis les vidéos par `_files -g "[^.]*.(#i)(mp4|mov|avi|mkv|m4v)"`, aux extensions de `jobs::video_extensions()`, et les dossiers, sans fichiers cachés.
      `-o` complète des dossiers (`ValueHint::DirPath`, `_files -/`), `--zoom` ses trois valeurs.
      Quand la sortie standard est un terminal, la sortie d'erreur donne les lignes de `~/.zshrc`, avec et sans Homebrew ; la commande n'écrit jamais dans le profil.
      Un autre shell est refusé (code 2).
    - `batdetect doctor`, sans argument, vérifie le dossier courant (`checkup::checkup`, bibliothèque) et écrit sur la sortie standard une ligne par point (`doctor::report_of`) :
      - version de batdetect et de FFmpeg ;
      - essai réel de l'encodeur, le même que le traitement (`Fallback::for_this_platform().trial`, 1440 × 1080 à 30 i/s) ;
      - place libre du volume (`disk::volume`, la même lecture que le contrôle de place au lancement : `statfs` sous macOS, `statvfs` sous Linux, depuis le plus proche dossier existant ; non mesurée sous Windows) ;
      - droit d'écrire (`writable::writing_refused`, la sonde du contrôle d'écriture).

      ✗ et un « à faire » marquent un encodeur absent (`NoEncoder` : réinstaller), un essai impossible faute de dossier temporaire (`Scratch`, `Destination` : place sur le disque du système, pas une réinstallation), moins de 20 Go libres (`checkup::ENOUGH_FREE_BYTES`, affichés arrondis par défaut au dixième) ou l'écriture refusée, et donnent le code 1.
      ⚠ marque openh264 quand VideoToolbox ne répond pas, ou une place non mesurée : rien d'obligatoire, code 0.
  - **Aide** : usages courants en tête (#56), au format de `tldr` (« - description : », puis la commande en retrait).
    Six exemples : un dossier de soirée, plusieurs soirées, une vidéo, `-o`, `--zoom all`, `--annotated`.
    Le nom de la commande est `env!("CARGO_BIN_NAME")` (`arguments::PROGRAM`), écrit à un seul endroit du code livré.
    Les textes sont dans `texts.rs` (`common_uses`, `all_options`, `files_produced`) et les `.ftl`.
    - Sans aucun argument : la phrase d'en-tête, les usages et « Toutes les options et les fichiers produits : batdetect --help », sur la sortie standard, code 0.
      `-o` sans vidéo reste une erreur d'usage (code 2).
    - `-h` : la phrase, les usages (`before_help`), l'essentiel, et le même pied (`after_help`).
    - `--help` : la phrase, les usages, tout, puis « Fichiers produits » et « Mode d'emploi complet : https://github.com/zaratan/chiro-dvr#readme ».
      Les noms des fichiers sont ceux de `files::names`, dans la langue de l'aide : `Names::numbered_clip` donne un nom d'extrait sans `Track`, `period_suffix` et `Names::part_suffix` les suffixes de l'image résumé, et `RESULTS_FOLDER` (`batdetect`) le dossier de résultats.
      Aucun nom n'est écrit en dur dans les textes de l'aide.
    - `style` (binaire) porte `STYLES` et `HELP`, que `arguments`, `errors` et `texts` importent, pour qu'il n'y ait pas de cycle entre ces modules.
      Chaque nom est sur sa ligne, en jaune, avec son rôle en retrait dessous : clap replie `before_help` et `after_help` à la largeur du terminal, et deux colonnes s'y casseraient.
      Les commandes des exemples sont en cyan gras, les titres en bleu gras, comme le reste de l'aide.
  - **Options** (décidées le 10 octobre 2026, #113), en builder `clap`, sur deux niveaux d'aide :
    - `-h` : l'essentiel, `VIDÉO…`, `-o/--out-dir`, `--zoom`, `-h`, `--version` ;
    - `--help` : tout, avec l'utile (`--annotated`, `--max-tracks`, `--clip-margin`, `--osd-region` répétable) et, sous « Settings that change the detection » (« Réglages qui changent la détection »), `--threshold`, `--min-hits`, `--max-blobs` (0 coupe le filtre), `--work-width`.
    - L'apparence est celle de l'aide d'argparse 3.14 (demande de l'utilisateur) : une ligne par option, le texte en colonne, coupé à la largeur du terminal (feature `wrap_help`, qui ajoute `terminal_size`, accepté par cargo deny).
      clap passe toujours le texte à la ligne suivante en aide longue : `-h` et `--help` sont donc tous deux l'aide courte de clap (`ArgAction::HelpShort`), et le niveau se choisit avant la lecture (`--help` présent avant `--` : tout est visible ; sinon l'utile et l'avancé sont cachés).
    - Couleurs (`Styles`) : titres et « Usage » en bleu gras, options en cyan gras, noms de valeurs en jaune, valeurs des défauts et valeurs possibles en vert, `error:` en rouge gras.
      clap les coupe hors terminal et avec `NO_COLOR`, et les force avec `CLICOLOR_FORCE` ; aide et erreurs passent par `clap::Error::print`.
      Limites de clap acceptées : `-h` et `--help` de la même couleur, nom du programme en cyan.
    - Les défauts ne sont jamais coupés et vont sur leur propre ligne (choix de l'utilisateur).
      clap coupe ses lignes sur l'espace ASCII et à chaque changement de couleur, même sans espace (`StyledStr::wrap` passe chaque segment de couleur à `LineWrapper::wrap`, qui peut couper avant son premier mot) : son « [default: 20] », en trois segments, laissait parfois « ] » seul sur la ligne suivante.
      La commande écrit donc elle-même le défaut (`hide_default_value`), après un retour à la ligne : « [default: » et « ] » en texte normal, espace insécable, la valeur en vert.
      clap ne coupe jamais le premier mot d'une ligne, et la ligne est courte.
      Les valeurs possibles de `--zoom` suivent sur leur ligne, sous la même forme, et `--osd-region` a aussi son défaut sur sa ligne (« [default: none] »).
    - La colonne du texte se cale par titre : clap calcule sa largeur pour chaque section séparément, sans réglage pour la partager entre « Options » et les réglages avancés.
    - Sous environ 95 colonnes, clap repasse le texte de toutes les options d'une section à la ligne suivante dès qu'une seule ne tient plus (règle des 40 % de `help_template`).
      Les tests rendent l'aide à 100 colonnes fixes.
    - `-V/--version` est une option de la commande, au ton des autres (« print the version »), comme `-h/--help`.
      Les deux écrivent les mêmes deux lignes, version puis FFmpeg et sa licence (#196) : la commande n'a qu'une `version`, la longue.
    - Le gabarit de la commande (`help_template`, #114) reprend celui de clap avec notre titre d'usage, et garde `{before-help}` et `{after-help}`, où se logent les exemples.
    - Les défauts viennent des configurations, et l'aide les affiche ; les autres réglages restent au banc.
    - Une valeur refusée (type, borne d'un constructeur, zone mal formée) est une erreur d'usage, code 2, qui nomme l'option et sa plage, sans ligne d'usage (`--threshold must be a finite number > 0, not -1`).
      Une valeur illisible (`--max-tracks many`) dit la même phrase avec la valeur telle qu'écrite (#55) : `parsing::Reader` lit chaque option et porte l'option, la plage et la valeur dans `parsing::Unreadable`, que `errors.rs` traduit.
      Les entiers ne disent que leur borne basse (`un entier ≥ 0`), pas le maximum du type.
      Une virgule décimale, sur `--threshold` et `--clip-margin` seulement, reçoit le conseil « écrire 11.5, avec un point », et une valeur vide dit « une valeur est attendue ».
      `--osd-region` accepte une valeur qui commence par un tiret (`allow_hyphen_values`), refusée ensuite par la phrase de la zone.
    - Les refus que la ligne de commande ne peut pas atteindre (`TrackConfig`, `StabilityConfig`, une option sans valeur par défaut) nomment l'option et demandent de le signaler ; ceux d'un réglage interne (`min_area` contre `max_area`, `trail_s`) disent « les réglages par défaut sont refusés ».
      `render::config::Error::Seconds` porte le réglage refusé (`Seconds::ClipMargin`), et non son nom en texte.
    - `--osd-region` se lit par `FromStr for engine::settings::Region` (`x0,y0,x1,y1`), la même lecture que le banc, et `Display` l'écrit de la même façon.
      Une zone dont les bornes ne vérifient pas `0 ≤ x0 < x1 ≤ 1` et `0 ≤ y0 < y1 ≤ 1` est refusée par son numéro parmi les zones données : « la zone n° 2 de --osd-region ne vérifie pas … : -0.1,0,1,1 » (« region no. 2 of --osd-region … » en anglais, #221).
  - **Entrées.** Un dossier donne ses vidéos (`mp4`, `mov`, `avi`, `mkv`, `m4v`), par ordre de nom, au premier niveau seulement : un `batdetect/` voisin n'est pas repris.
    - Un nom qui commence par « . » est écarté (#204), comme la page écarte les dossiers cachés : ce sont surtout les `._x.mp4` (AppleDouble) que macOS dépose à côté de chaque fichier sur un volume exFAT ou FAT.
      Un dossier qui n'en contient que devient un dossier sans vidéo.
    - Une vidéo nommée, cachée ou non, est jugée sur sa seule extension.
    - Une vidéo donnée deux fois, sous n'importe quelle écriture (son dossier et son nom, `./`, un lien), n'est traitée qu'une fois, sous sa première écriture (#194).
      La clé est son chemin réel (`fs::canonicalize`), ou, pour un fichier absent, le dossier réel suivi du nom, si bien qu'un chemin introuvable donné deux fois n'est refusé qu'une fois.
  - **Refus en bloc avant tout calcul** (`jobs::plan`, #55).
    Toutes les entrées sont relevées avant la première détection, et une seule fautive arrête tout le lot (code 2, aucun dossier créé).
    Chaque refus est une ligne « nom : raison », sous « aucune vidéo n'est traitée : », sans ligne d'usage ni renvoi à l'aide.
    Sous chaque refus vient, en retrait, sa ligne « à faire » (#239), comme sous un échec du bilan.
    Ses textes sont des clés du même message `failure-todo`, propres aux entrées : `input-read-refused` (lecture refusée), `input-missing` (introuvable, ou un élément du chemin qui n'est pas un dossier : « le dossier ou la vidéo a peut-être été déplacé ou renommé », quand l'échec `missing` ne parle que de la vidéo), `unreachable` (inaccessible), `not-a-video-name`, `no-video`, `shared-folder`.
    Les raisons, typées par `jobs::Reason` :
    - chemin absent ;
    - fichier nommé dont l'extension n'est pas une des cinq ;
    - dossier sans vidéo, parmi d'autres entrées ;
    - chemin ou dossier illisible, la raison étant prise dans la table fermée `access::Access` (refusé, pas un dossier, introuvable, lecture seule, disque plein, sinon inaccessible) ;
    - plusieurs vidéos vers le même dossier de résultats : une ligne par dossier, qui nomme toutes ses vidéos, après les refus d'entrées puisque ce sont des sorties ;
    - vidéo qui ne s'ouvre pas (`Reason::Unopened`, #62), vue par l'inventaire, après les précédentes.
      Sa ligne est la phrase de l'échec équivalent, et son « à faire » celui de cet échec (`failure::reading_cause`, la classification d'une lecture ratée), suivi du détail anglais quand ce « à faire » y renvoie (`unsupported`).
      Une vidéo introuvable ou dont la lecture est refusée prend les clés d'entrée, `input-missing` et `input-read-refused`, qui portent son chemin.
      Le bilan reprend les mêmes lignes, détail compris, puisque ce « à faire » y renvoie ; un échec à son tour, lui, y perd son détail.

    La ligne refusée nomme l'entrée par le dernier nom de son chemin (`messages::last_name`, #247), comme on la voit dans le Finder, parce qu'un chemin glissé dans le Terminal est absolu et noyait la raison.
    Le chemin entier reste dans le « à faire » quand il sert à agir : `input-missing` (« vérifier le chemin {chemin} »), `input-read-refused` (« vérifier les droits de {chemin} ») et `no-video` (« vérifier que {chemin} est le bon dossier »).
    Le disque trop petit (#61) garde le chemin entier du dossier de résultats, qui nomme le disque, et les dossiers où l'on ne peut pas écrire gardent le leur.
    La page (`sheet.rs`) raccourcit la ligne de la même façon, et le test qui compare la page au terminal le tient.

    Si toutes les entrées sont des dossiers sans vidéo, le refus devient « aucune vidéo (mp4, mov, avi, mkv ou m4v) trouvée dans ce dossier : », suivi des dossiers, chacun avec sa ligne « à faire ».
  - **Question des refus** (`consent::Asking`, décision de l'utilisateur du 16 octobre 2026) : quand des vidéos valides restent à côté des refus, la commande peut les traiter.
    - Si l'entrée standard est un terminal, la liste des refus passe sous « 2 entrées sont refusées : », puis la sortie d'erreur pose la question « Traiter quand même les 3 vidéos valides ? (O/n) » (« (Y/n) » en anglais).
      Entrée, o, O, y ou Y valent oui, dans les deux langues ; n ou N valent non ; toute autre réponse fait reposer la question ; la fin de l'entrée ou une erreur de lecture vaut non.
      Sur un oui, les vidéos valides sont traitées et le code de sortie vaut 1 ; sur un non, « aucune vidéo n'est traitée », code 2.
    - `--skip-refused` vaut un oui sans question, avec ou sans terminal.
    - Sans terminal et sans l'option (script, redirection), le refus en bloc reste, comme quand aucune entrée n'est valide, avec ou sans l'option.
    - Le bilan, toujours sur la sortie standard, compte « 2 entrées refusées » dans son titre et les liste sous « Entrées refusées, non traitées : », chacune avec sa ligne « à faire ».
    - Le chemin avec question se teste dans `consent`, sur une entrée fabriquée, parce qu'un test de commande ne peut pas fabriquer de terminal sans pseudo-terminal.
  - **Inventaire au départ** (`inventory::take_stock`, #62), après `jobs::plan` et avant la question des refus.
    - La reprise se décide là, une fois pour toutes (`reuse::earlier`, sauf `--force`) : une vidéo déjà faite n'est pas ouverte, et sa durée est celle de son `params.json`.
    - Chaque autre vidéo est ouverte par `engine::input::Input::open`, en un seul fil, puis refermée aussitôt ; sa durée est le nombre d'images annoncé divisé par la cadence.
    - Une vidéo qui ne s'ouvre pas rejoint les refus d'entrées, avec la même question, le même `--skip-refused` et le même code 2.
    - Coût, mesuré le 16 octobre 2026 sur lumiel (binaire release, fichiers en cache, `-o` vers un dossier en lecture seule pour s'arrêter juste après l'inventaire) : 0,39 à 0,44 s pour les 26 vidéos de `BATDETECT_VIDEOS_DIR`, dont 0,39 s de CPU, pic de mémoire 15,6 Mo.
    - Une vidéo fausse, tronquée ou au flux effacé échoue à l'ouverture (essai sur la vidéo fabriquée) : un échec à son tour vient surtout de l'écriture.
      Les tests de commande le provoquent par un fichier ordinaire posé à la place de `<racine>/<vidéo>/`, qui fait échouer la vidéo sur tous les systèmes, Windows et root compris (« erreur inattendue »).
    - Les causes d'une lecture ratée servent encore pour un fichier qui disparaît ou change entre le départ et son tour.
  - **Dossiers de sortie** (`jobs::plan_jobs`, #86, #91).
    La racine d'une vidéo est `-o`, ou à défaut `<dossier de la vidéo>/batdetect/`, et chaque vidéo a son dossier, à son nom, dans sa racine.
    Les racines et les homonymes se comparent sur le dossier réel (#194) ; une racine écrite de deux façons reprend sa première écriture, et le bilan n'a qu'un titre par dossier.
    - Les homonymes se comptent par racine : sans `-o`, `nuit1/v.mp4` et `nuit2/v.mp4` donnent `nuit1/batdetect/v` et `nuit2/batdetect/v`.
    - Deux homonymes dans une même racine prennent `<dossier parent>_<nom>_<extension>` avec `-o`, et `<nom>_<extension>` sans `-o`, puisque le dossier est déjà celui de la vidéo.
    - Les homonymes se comparent à la casse près (#91), sur tous les systèmes.
      La clé est le nom en NFC (`files::names::composed`, celle du ménage), puis `to_lowercase`.
      Un nom non UTF-8 garde ses octets pour la clé, comme dans le ménage : par `to_string_lossy`, `\xff` et `\xfe` auraient la même clé et seraient refusés comme un seul dossier.
      - Avec `-o`, `a/Vol.mp4` et `b/vol.mp4` donnent `a_Vol_mp4` et `b_vol_mp4` ; sans cette clé, ils partageraient un dossier sur APFS ou NTFS, et le ménage de la seconde effacerait les extraits de la première.
      - Sous Linux, la règle va plus loin que le système de fichiers : deux homonymes à la casse près reçoivent deux noms qui n'étaient pas nécessaires, et deux dossiers `Nuit/x.mp4` et `nuit/x.mp4` sont refusés comme un seul (code 2), avec ou sans `-o`.
    - Deux vidéos qui écriraient au même endroit, à la même clé près, sont refusées avant tout calcul (code 2), avec les autres entrées fautives (#55).
  - **Contrôle d'écriture** (`writable::unwritable_folders`, #86), avant tout calcul, pour chaque racine, celle de `-o` comprise.
    Il passe dans `admission`, après l'inventaire et avant le contrôle de place, si bien qu'un dossier de résultats où rien ne s'écrit n'est refusé qu'une fois, pour l'écriture.
    `create_dir_all` ne suffit pas, puisqu'il réussit sur un dossier existant en lecture seule.
    - Une sonde est créée de façon exclusive (`.batdetect-probe-…`, par `tempfile`), puis retirée, dans le plus proche dossier existant de la racine (`.` pour une racine relative qui n'existe pas encore).
      Rien n'est créé avant la détection : une vidéo au chemin mal tapé ne laisse pas de `batdetect/` vide.
    - Un `batdetect/<vidéo>/` existant mais en lecture seule passe la sonde de sa racine, et sa vidéo échoue seule, après la détection.
    - Si une racine refuse, aucune vidéo n'est traitée (code 2) : « aucune vidéo n'est traitée, les résultats ne peuvent pas s'écrire dans ce dossier : », puis une ligne « dossier : raison » par dossier, sans ligne d'usage ni renvoi à l'aide (#55).
      La raison vient de la table fermée `access::Access` : écriture refusée, pas un dossier, introuvable, disque en lecture seule, disque plein, sinon inaccessible.
    - Chaque dossier refusé a sa ligne « à faire » (#82), comme une entrée refusée.
      Écriture refusée et disque plein reprennent les clés de l'échec (`forbidden`, `disk-full`).
      Disque en lecture seule : « copier les vidéos sur le Mac, ou choisir un autre dossier avec -o » (`read-only`, décision du 9 octobre 2026).
      Pas un dossier ou introuvable : « choisir un autre dossier avec -o » (`other-folder`).
      Inaccessible : `unreachable`, comme pour une entrée.
    - Un `-o` qui est un fichier est refusé de même, avant la détection.
  - **Contrôle de place au lancement** (`space`, `disk`, `admission`, #61), après l'inventaire, sauf `--ignore-disk`.
    - Chaque vidéo que la reprise ne saute pas compte sa taille multipliée par le coefficient des sorties, et par celui de la vidéo annotée en plus sous `--annotated` (`space::launch_estimate` ; coefficients et mesure dans [02](02-methode.md), « Place prise »).
    - Les besoins s'additionnent par volume du dossier de résultats (`-o`, ou le `batdetect/` voisin), pas par volume des vidéos : un disque externe pour les vidéos et le Mac pour les résultats se comptent à part.
      Le volume se reconnaît à `MetadataExt::dev()` du plus proche dossier existant.
      Sous btrfs, chaque sous-volume a son propre `dev()` : deux dossiers d'un même système de fichiers s'y comptent chacun contre toute la place libre, ce qui sous-estime le besoin (cas rare, sans effet sur le Mac).
    - La place libre est `f_bavail × f_bsize` de `rustix::fs::statfs` sous macOS (compteurs de blocs sur 64 bits, quand ceux de `statvfs` n'en ont que 32), et `f_bavail × f_frsize` de `rustix::fs::statvfs` sous Linux, exFAT monté sous `/Volumes/…` compris.
      Sous APFS, elle ne compte pas l'espace purgeable, que le Finder ajoute : le contrôle est donc plus prudent que le Finder.
      Sous Windows, la place n'est pas lue et rien n'est refusé.
    - Un volume est refusé si l'estimation plus la marge (10 % de l'estimation, au moins 1 Go, décision du 9 octobre 2026) dépasse sa place libre.
      La ligne de refus nomme le premier dossier de résultats de ce volume, un chemin que la naturaliste reconnaît : « le disque qui contient … : 12,3 Go à prévoir, 4,1 Go libres », en Go décimaux, à prévoir arrondi au-dessus et libre au-dessous.
      Son « à faire » cite `--annotated` seulement quand il est donné (`disk-short`, `disk-short-annotated`).
    - Le refus suit le circuit des refus d'entrées : liste, question dans un terminal, `--skip-refused` vaut un oui, code 2 sinon (décision de l'utilisateur du 16 octobre 2026).
      Avant la question, il a son bloc à lui, « le disque des résultats semble trop petit : », après celui des entrées refusées ; hors terminal, il reste sous « aucune vidéo n'est traitée : » avec elles.
      Un oui lance tout le lot (`Stock::go_ahead_despite_the_disks`).
      Le refus de place ne compte alors pas parmi les entrées refusées du bilan, ni dans `.batdetect/refus.json` et sur la page (#267) : seul un lot lancé écrit ce fichier, et un lot malgré un disque trop petit ne l'est qu'après ce oui, qui retire le refus.
  - **Contrôle de place par vidéo** (`process::room_for_clips`, #61), après le suivi et l'image résumé, juste avant les extraits, même sous `--ignore-disk`.
    - L'estimation est presque exacte : secondes des fenêtres d'extrait (piste et marges, `render::windows::clip_windows`) et de zoom (pistes que `ZoomConfig::wants` retient), multipliées par le débit de la vidéo d'entrée et par leur coefficient, plus la vidéo annotée (`space::render_estimate`).
      Au-delà de `--max-tracks`, rien n'est compté, puisque rien n'est rendu.
    - Si elle dépasse la place libre du volume du dossier de la vidéo, sans marge, la vidéo échoue avec `Error::DiskSpace`, classée « disque plein ».
      Son « à faire » dit de libérer de la place « sur le disque qui contient » le dossier, la même tournure que le contrôle au lancement (#221).
      `params.json` n'est pas écrit, la vidéo sera refaite à la relance, et la suivante est tentée.
    - Les tests des deux contrôles prennent une vraie vidéo allongée par `set_len` à 10 To, en fichier épars : FFmpeg la lit comme l'originale, et son estimation dépasse tout disque, sans couture dans le code.
      Ils ne tournent que sous Unix.
  - **Bilan de fin de soirée** (#65), écrit une fois après `run_all`, sur la sortie standard.
    - Le modèle, sans texte, est dans la bibliothèque (`tally::Tally`), la mise en forme dans le binaire (`board::write`).
      `Tally::roots()` ne rend que les dossiers de résultats qui existent (#196) : une racine qui n'a que des vidéos en échec n'a de titre que si son dossier est sur le disque, et ses vidéos restent sous « vidéos en échec ».
    - Titre : « Bilan : 12 vidéos, 10 réussies, 1 avec trop de pistes, 1 en échec » (les états absents ne sont pas nommés).
      Puis un tableau par dossier de résultats (#86), les vidéos dans l'ordre du lot : vidéo, durée, pistes, non analysé, calcul.
      - La vidéo est nommée par son dossier de résultats, qui porte déjà la levée d'ambiguïté des homonymes.
      - Le nom est à gauche, les nombres et les durées à droite.
        Les largeurs se comptent en `char`, après composition NFC, sans crate, et une cellule inconnue vaut « — ».
      - Pistes : le nombre par le formateur de #114, « échec », ou « 412, trop ».
      - Durées en « 1 h 12 min 3 s » : la durée et le calcul sont tronqués à la seconde, le non-analysé arrondi au-dessus.
        `summary::header::duration`, celui de l'image résumé et de la page, écrit les heures de la même façon au-delà d'une heure (#338).
    - La durée va de la première image à la fin de la dernière, sans le premier `pts`, et le non-analysé est l'union des plages abîmées et instables (`process::extent`).
    - Sous le tableau viennent des sections, omises quand elles sont vides, dont :
      - « Trop de pistes (tableau écrit, ni extraits ni image résumé) », avec le « à faire » de #64 : « ouvrir le tableau des pistes et regarder la vidéo à quelques-uns de ses temps ; si ce sont de vrais passages, relancer avec un --max-tracks plus grand » (#221) ;
      - « 1 vidéo sur 12 en échec », avec la raison et le « à faire » de #64, sans le détail anglais, qui reste sur la sortie d'erreur.
        Le « à faire » qui renvoie au détail devient, dans le bilan, « avec le détail écrit pendant le traitement » (`tally-failure-todo`) ;
      - « Tableau laissé tel quel » (#115), pour une entrée au nom du tableau qui n'est pas un fichier, ou pour un fichier qui occupe encore le nom (autre casse) ;
      - « Réglages changés : --zoom all, --max-tracks 0, --threshold 11,5 ».
        Ce sont toutes les options qui font recalculer une vidéo à la relance (#221, #247).
        `Options::changed` compare les réglages à ceux par défaut avec ce que compare la reprise (`settings::compared` : `params.json` sans les fils ni les encodeurs, partagé avec `reuse`), champ par champ (`settings::differing`), et nomme chaque champ par son option.
        Elles viennent dans l'ordre de l'aide : `--zoom`, `--annotated`, `--max-tracks`, `--clip-margin`, `--osd-region` avec son nombre de zones, `--threshold`, `--min-hits`, `--max-blobs`, `--work-width`.
        La valeur s'écrit comme elle se tape, avec la virgule décimale en français, comme le reste du bilan.
        Un champ sans option, qui ne peut pas changer depuis la commande, se nommerait `section.champ` (`Change::Unnamed`).
        Deux tests du binaire tiennent la liste : toute option de la commande est rangée parmi celles qui font recalculer ou celles qui laissent les résultats tels quels, et chacune des premières, donnée seule, est nommée.
    - Codes de sortie : 1 si une vidéo a échoué ou a trop de pistes, et aussi quand le bilan ne peut pas s'écrire.
      Si une écriture au fil de l'eau échoue, `run_all` s'arrête et il n'y a pas de bilan.
  - **Page de résultats** (#71), à la racine de chaque dossier de résultats.
    - `written::write_page` relit la racine (`files::results`, et le fichier de verdicts par `files::verdicts`, passé tel quel à la page, #228), y ajoute les vidéos du lot pas encore traitées (« en attente »), ouvre celles du lot, et écrit `resultats.html` ou `results.html` par `.partial` puis renommage, en retirant la page de l'autre langue.
      Il n'écrit ni ne retire une page qui n'a pas la marque de `page`, et ne crée jamais la racine.
    - `main` l'appelle avant le lot, puis dans le rappel de fin de chaque vidéo.
      Une erreur va dans `Tally::unwritten`, dite à la fin du bilan (pour une page étrangère du même nom, ce qu'il faut faire ; sinon le chemin et le détail), et rend le code 1 sans arrêter le lot.
    - Au Ctrl-C, ou si la boucle du lot s'arrête sur une erreur, chaque page est réécrite avant la sortie avec `Run::Stopped`, à la position de la vidéo en cours (`Status::position`).
      Ce que le disque garde l'emporte ; sinon la vidéo en cours est « inachevée », et les suivantes du lot « pas encore traitée » (#247).
      Avant les pages, `written::create_reached_root` crée la racine de la vidéo en cours, et d'aucune autre, pour qu'un Ctrl-C pendant la première vidéo laisse sa page.
      Puis le `Ledger`, partagé avec le gestionnaire du Ctrl-C, écrit les échecs qui attendaient cette racine, et les refus s'y écrivent.
    - Les clés des vidéos sont les noms de dossier composés en NFC : un dossier créé en NFD et la même vidéo tapée en NFC font une seule entrée.
      Un nom de dossier non UTF-8 garde ses octets pour la clé, comme dans le ménage et dans `jobs` (#227) : `vol_\xe9` et `vol_\xe8`, tous deux affichés `vol_�`, font deux entrées.
    - `ledger::Ledger::note`, dans le même rappel et avant la page : une vidéo en échec écrit `.batdetect/echecs/<dossier>.json` à la racine (clé stable de `Failure` : `Cause::key`, ou `internal` pour une panique ; place, détail, date UTC), et une réussite ou « trop de pistes » l'efface.
      Un échec dont la racine n'existe pas encore attend qu'une autre vidéo du lot la crée ; sinon rien n'est créé pour lui.
    - Le détail technique n'est montré que pour les causes dont le conseil y renvoie (`failure::REFERS_TO_DETAIL`).
    - `refusals::Refusals::note` (#267), avant les pages et dans le même rappel : les entrées refusées au départ (`jobs::Refused`, de `plan` puis de `inventory`) s'écrivent dans `.batdetect/refus.json` de chaque racine du lancement, dès qu'elle existe, une fois par racine, et un lancement sans refus efface ce fichier.
      Chaque entrée garde son chemin, la clé de sa raison (`Reason::key`), celle de son conseil (`Reason::todo`), la cause d'une vidéo qui ne s'ouvre pas et son détail quand le conseil y renvoie (`failure::REFERS_TO_DETAIL`), et les vidéos d'un dossier de résultats commun.
      Une interruption (Ctrl-C) ou un arrêt de la boucle les écrit aussi avant les pages.
      Ces deux clés sont la seule table entre raison et texte, partagée par le terminal (`messages`) et la page (`sheet`).
      Le texte de la raison est encore produit deux fois, par `messages::input_refused` pour le terminal et `sheet::refused_input` pour la page : un test de `sheet` vérifie, pour chaque raison, chaque accès et chaque cause d'ouverture, dans les deux langues, que la page dit les mêmes lignes que le terminal, détail compris.
      La page les montre dans une carte avant les vidéos, liée depuis le total.
    - `notify` (#69) ouvre, après le bilan, la page de chaque racine dans la langue du lancement (`written::page_to_open`), et rien pour une racine sans page à nous.
  - **Vidéo et étape en cours** (#63), sur la sortie d'erreur ; la sortie standard ne porte que le bilan de fin de soirée.
    - `process` annonce quatre étapes (`progress::Progress`) : la détection (lecture comprise), le suivi (ménage, CSV, essai des encodeurs et `params.json` compris), l'image résumé et les extraits (zooms et vidéo annotée compris).
      Il n'y a pas d'avancement image par image : l'heure de fin se compte d'une vidéo à l'autre.
    - Une écriture ratée de l'étape fait échouer la vidéo avant sa détection (`Error::Progress`).
      Le rapport de cette vidéo échoue ensuite sur la même sortie d'erreur, et le lot s'arrête là, en code 1.
    - Avant le premier rang, une ligne annonce le lot sur la sortie d'erreur (#62) : « 12 vidéos, 1 h 2 min 5 s de vidéo, dont 3 déjà faites », la fin seulement s'il y en a.
      La durée s'écrit comme la colonne « durée » du bilan (`clock`), vidéos déjà faites comprises.
    - Le binaire (`status.rs`) écrit « [3/12] video_092 — détection », puis « — suivi », « — image résumé », « — extraits ».
      La vidéo est nommée sans extension, sauf quand une autre vidéo du lancement a le même nom sans extension : les deux gardent alors la leur, « video_092.mov » et « video_092.mp4 » (#247, `messages::homonyms`).
      Quand l'extension ne suffit pas (même nom de fichier dans deux dossiers, comme `batdetect nuit-12/ nuit-13/`, la numérotation des jumelles repartant à zéro), la ligne nomme aussi le dossier parent : « nuit-12/video_001.mp4 » et « nuit-13/video_001.mp4 ».
      Le rang compte dans le lot.
    - La ligne d'une vidéo finie reste : « [3/12] video_092 — 14 pistes », « — trop de pistes » (le mot du bilan, #196) ou « — échec ».
    - Heure de fin estimée de la soirée (#68) : « [3/12] video_094 — détection · fin vers 22:41 » sur un terminal, à chaque étape ; hors terminal, à la fin de la ligne d'une vidéo finie, jamais sur une étape.
      Rien tant qu'aucune vidéo n'a été calculée, ni quand il ne reste rien à calculer (dernière ligne d'une vidéo finie hors terminal).
      - `eta::Evening`, pur : les secondes passées sur les vidéos calculées depuis le début du lot, divisées par leurs images, s'appliquent aux images restantes.
        Les images sont celles qu'annonce l'inventaire (`Listed::frames_to_compute`), et une vidéo que la reprise saute en a 0.
        Pendant une vidéo, le temps qu'elle a déjà pris se retire de sa part, sans descendre sous 0.
      - Seule une vidéo calculée jusqu'au bout donne du temps (`Status::computed`, après l'écriture de la page), parce qu'une vidéo en échec, souvent arrêtée tôt, tirerait l'estimation vers le bas.
        Le temps se mesure de l'entrée dans la vidéo (`Status::enter`) à ce moment, par `Instant`.
      - L'heure est locale (`jiff::Zoned::now`), arrondie à la minute la plus proche, sans jour : au-delà de 24 h de calcul restant, elle ne dit pas le jour.
      - La coupe à la largeur du terminal prend la fin de la ligne : sur une fenêtre trop étroite pour la ligne entière, l'heure de fin disparaît la première.
    - Ctrl-C (#196) : un gestionnaire `ctrlc`, posé par `main` une fois le lot admis, efface la ligne d'étape sur un terminal, écrit « lot interrompu à la vidéo 3/12 ; relancer la même commande reprend là où le lot s'est arrêté » (« refait le lot » sous `--force`, #70) sur la sortie d'erreur et sort en 130.
      Il sort même si la ligne ne s'écrit pas, et garde la sortie d'erreur jusqu'à la sortie. `Status` retient la vidéo en cours.
      Rien d'autre ne se fait : un `.partial` laissé par l'interruption dans le dossier d'une vidéo est retiré par le ménage de la relance (#115), et les autres restent (`names::partial_file`).
    - Les lignes d'événement (extrait refait, cause d'un échec) s'écrivent au-dessus de cette dernière ligne.
    - Le mode se choisit par `std::io::IsTerminal` sur la sortie d'erreur. Redirigée, elle reçoit une ligne par étape, et rien n'est effacé.
      Sur un terminal, la ligne d'étape est redessinée sur place, par `\r`, des espaces jusqu'à la longueur précédente, puis `\r`, sans séquence d'échappement (console Windows ancienne) ; la longueur compte des caractères.
      Sous Unix, la ligne est coupée à la largeur du terminal moins une colonne, lue à chaque étape par `tcgetwinsize` (`terminal.rs`), parce qu'une ligne qui passerait à la rangée suivante ne s'effacerait qu'à partir de sa dernière rangée.
      Sous Windows, la largeur n'est pas lue, et une ligne plus large que le terminal garde ce défaut.
  - **Chaîne d'une vidéo** : `engine::detection::detect` (fils automatiques), puis la `Timeline` des `pts` lus, puis `exclude`.
    Les plages abîmées viennent du moteur, et les images signalées sont l'union des images en erreur et après un saut.
    Puis viennent `track_detections`, le ménage, le CSV et `params.json` de `files`.
  - **Ménage** (#115) : `files::previous::remove_previous_outputs`, après la détection et le suivi, avant le CSV, si bien qu'une détection ratée ne détruit rien.
    - Il retire les sorties des deux langues et des anciens noms : sur l'extrait, une passe en français, puis en anglais, puis en français dans le même `-o`, ne laisse que les fichiers de la dernière, sous les noms de la v0.1.1.
    - Le tableau des pistes appartient à l'outil (#228) : un fichier à son nom est retiré, quoi qu'il contienne.
      Les tableaux mis de côté sous un nom daté par une version antérieure (#83) gardent un nom que le ménage ne reconnaît pas, et restent.
    - Une entrée au nom du tableau qui n'est pas un fichier est laissée, et le bilan le dit sous « Tableau laissé tel quel » : « {vidéo} : {csv} n'a pas la forme d'un tableau des pistes ».
      Si elle porte le nom de la langue courante, ou si un fichier occupe encore ce nom après le ménage (`Error::Taken`, autre casse), le nouveau tableau n'est pas écrit par-dessus : `Processed::csv` vaut `None`, et la ligne finit par « ; le nouveau tableau n'est pas écrit ».
      La vidéo reste réussie.
  - **Image résumé** (#104) : `overview`, appelée après `params.json`, si bien qu'un échec du fond laisse le CSV et les réglages.
    Le fond passe par `Converter::new(…, SourceRange::FullBlackAt16)`, puis `summary::pages`, et chaque page prend son nom par `Names::summary_png` ; sans fond, l'erreur est `NoBackground`.
    - Le fond de `pages` s'écrit avant les pages, sous `Names::background_png` (#259), un par vidéo quel que soit le nombre de tranches et de parties (0,87 Mo pour la 092).
      `sheets` recopie la mise en page de chaque page dans `params.json` (`summaries`, secondes au centième, fichier relatif au dossier de la vidéo), et `listing` y met la trace de chaque piste dessinée (`trace`, pixels entiers de la vidéo, points consécutifs égaux fusionnés).
      Rien n'est recalculé après `summary` : `results` et la page relisent ces valeurs. Sur la 092, `params.json` passe à 13,5 Ko.
  - **Extraits** (#112) : `render_clips`, appelée après l'image résumé.
    - Le dossier des extraits (`extraits` ou `clips`, selon la langue), retiré par le ménage, n'est recréé qu'avec au moins un extrait à écrire, après l'essai d'encodeur réussi (#195) : 0 piste, trop de pistes ou aucun encodeur ne laissent aucun dossier.
    - Garde `max_tracks` (300) : au-delà, aucun extrait n'est écrit, et la vidéo échoue avec sa ligne (« 301 pistes, au-delà de 300 : extraits non écrits ») après le CSV, `params.json` et l'image résumé.
    - Encodeurs : un essai d'encodage (`Fallback::trial`) à la taille et à la cadence de la vidéo, avant `params.json`, donne la liste employée (VideoToolbox puis openh264 sur Mac, openh264 ailleurs).
      Il est sauté au-delà de la garde.
      S'il échoue, `params.json` porte `encoders: []`, l'image résumé est écrite, et la vidéo échoue à l'étape des extraits, après le vidage du dossier.
    - `Source` reçoit les images-clés de la lecture du moteur (`Reading::key_frames`) et ses images en erreur ; chaque extrait prend son nom par `Names::clip`, à partir de `files::time::frame_seconds` de sa première image.
    - Un extrait refait en openh264 après un échec de VideoToolbox écrit une ligne sur la sortie d'erreur, avant la ligne de réussite : « {vidéo} : {extrait} réécrit avec openh264 après une erreur de VideoToolbox : {causes} ».
    - La première fois que VideoToolbox ne répond pas en 10 s (#313), une seule ligne le dit pour tout le lancement, avant la ligne de fin de la vidéo où c'est arrivé (réussie ou en échec) : « VideoToolbox n'a pas répondu en 10 s : les extraits s'écrivent avec openh264 jusqu'à la fin du lancement » (`Status::hearing`, `hardware-silent`).
  - **Langue du système** : la première variable non vide parmi `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` et `LANG`.
    Dans une liste (`LANGUAGE=de:fr`), c'est la première langue qui a des messages qui compte : `fr*` pour le français, `en*` pour l'anglais.
    Si aucune variable n'est posée ou si toutes sont vides, `sys-locale` lit la langue de l'interface (macOS, Windows), et à défaut c'est l'anglais.
    `LANG=C` reste en anglais, puisque la variable est posée. Les tests qui lancent le binaire fixent tous `LANG`.
  - **Langues** (#114) : toute la commande parle la langue choisie, la ligne de commande comme les lignes de console et le bandeau de l'image résumé ; les noms de fichiers restent dans `files::names`.
    - Deux catalogues Fluent plats, `crates/cli/locales/fr.ftl` et `en.ftl`, inclus dans le binaire (`include_str!`), sans termes, ni attributs, ni `NUMBER()`.
      Le français reprend mot pour mot les textes de la v0.1.1 quand ils existent.
    - `texts::Message` énumère les messages avec leurs arguments typés, `texts::Fixed` ceux qui n'en ont pas.
      Le test des clés exige que chaque message se forme sans erreur dans les deux langues, que chaque argument serve dans au moins une langue (l'anglais n'accorde ni « réécrit » ni « illisible »), qu'aucune clé ne soit orpheline, que les deux catalogues se lisent sans erreur, et qu'aucune marque d'isolement U+2068/U+2069 n'apparaisse.
      Ces marques sont coupées par `set_use_isolating(false)` : Roboto n'a pas leurs glyphes, et elles entreraient dans les chemins copiés depuis la console.
    - Les deux bundles vivent dans deux `static OnceLock`, construits au premier usage sans `Result`, et `texts::Texts` ne porte que la langue : un bundle n'est pas `RefUnwindSafe`, et le travail de `run_all` ne capture ainsi que la langue.
    - La bibliothèque reste sans texte : `process(job, &dyn Wording, …)` reçoit le trait `Wording` (les libellés du bandeau et la langue des noms), et ses tests passent des libellés de débogage en anglais.
    - Nombres : un formateur par langue (`numbers.rs`), branché sur le bundle.
      En français, virgule décimale et milliers séparés par U+202F dans la console, par U+00A0 dans le bandeau (Roboto n'a pas U+202F) ; en anglais, point, sans séparateur.
      `-0` s'écrit `0`. Le CSV, `params.json` et les noms de fichiers n'y passent pas.
      L'aide affiche donc « [par défaut : 1,5] », alors que l'option s'écrit `--clip-margin 1.5`.
    - Aide : la commande se construit avec les textes de la langue (aides, noms de valeurs, titres par `help_heading`, « Utilisation : » par le gabarit).
      Les titres français finissent par une espace insécable, puisque clap colle « : ». L'apparence est la même dans les deux langues.
      L'usage reste celui que clap génère avec nos noms de valeurs : « [OPTIONS] » s'y lit aussi en français, et `override_usage` n'est pas pris.
      Chaque texte d'aide d'une option ou d'un argument commence par une majuscule, dans les deux langues (« Vidéos, ou dossiers… », « Print the version »), mais pas les noms de valeurs, les titres ni « [par défaut : …] » ; un test le garde (#115).
    - Erreurs de clap : un `ErrorFormatter` par langue (`errors.rs`), branché par `clap::Error::apply`, les rend depuis `ErrorKind` et `ContextKind` ; impression, couleurs et codes de sortie restent ceux de clap.
      Nos refus portent leur phrase dans `ContextKind::Custom`, et non par `Command::error`, dont le message tout fait ne passe pas par le formateur.
      En anglais, les erreurs de syntaxe essayées par le test (option inconnue, avec option ou valeur proche et conseil `--`, valeur hors choix ou vide, valeur donnée à un interrupteur, option répétée, argument manquant) sont rendues comme clap les écrit, couleurs comprises, au pied près.
      Un genre non prévu donne « les arguments sont refusés », sans le texte anglais du genre (#55).
    - Ligne d'usage et pied (#55) : la ligne d'usage ne suit que les erreurs de syntaxe de clap.
      Les refus de valeurs ne l'ont pas et gardent le pied, « Pour en savoir plus, essayez 'batdetect --help'. », dont le nom vient de `CARGO_BIN_NAME`.
      Les refus d'entrées et de dossiers n'ont ni l'une ni l'autre (`errors::rejection`, marqués par `ErrorKind::Io`).
    - Causes : les `Display` des erreurs des bibliothèques restent en anglais dans le seul détail d'un échec (`failure-detail`, #64) et dans `clips-redone`.
      Aucun refus avant calcul n'interpole de texte anglais (#55).
    - Noms d'options : `engine::settings::Setting::flag()` (`threshold`, `work-width`, …) nomme l'option dans la commande comme au banc, pour `Arg::long` comme pour les refus.
      La phrase du refus reste en deux copies : anglaise au banc, Fluent dans la commande.
  - **Raisons en clair** (#64) : une vidéo en échec écrit trois lignes sur la sortie d'erreur, au-dessus de « [k/n] vidéo — échec ».
    Les autres vidéos continuent, et le code de sortie vaut 1 si une vidéo a échoué.
    - Les trois lignes : « {vidéo} : {phrase} », « à faire : {conseil} », « détail : {chaîne anglaise des causes} ».
      « Trop de pistes » a sa phrase et un « à faire », mais pas de détail.
      Cette phrase finit par « ni extraits ni image résumé » (#221) : au-delà de `--max-tracks`, `process` n'écrit pas non plus l'image résumé ni son fond (#259), et n'annonce ni l'étape « image résumé » ni celle des extraits ; la vidéo passe de « suivi » à « trop de pistes ».
      `params.json` a alors `background` à `null`, `summaries` vide et des pistes sans couleur ni `trace` (`Drawn::default`), et la page ne dessine rien pour elle.
    - `failure::cause(&process::Error, &Options) -> Cause` classe sans texte.
      Le `match` est exhaustif, sans `_`, sur `process::Error`, `detection::Error`, `input::Error` et `decode::Error`.
      `detection::Error::Input` est transparent : `input::Error` n'apparaît jamais dans `source()`, et seul le `match` y mène.
      Côté écriture (`Folder`, `Previous`, `Files`, `Overview`, `Encoders` sauf `Scratch`, `Clips`), c'est la chaîne `source()` qui classe, sans `match` sur les énumérations de `files`, `render` ou `fallback`.
    - Causes :
      - à l'ouverture, par `io_kind()` : introuvable, lecture refusée, sinon « pas une vidéo lisible, ou copie incomplète », parce qu'une copie tronquée depuis les jumelles perd le `moov`, écrit en fin de fichier, et que FFmpeg ne la distingue pas d'un fichier qui n'est pas une vidéo ;
      - nom illisible par FFmpeg ;
      - vidéo abîmée (nombre d'images, lecture, décodage) ;
      - format non géré (décodeur absent, format de pixel…) ;
      - réglages, seulement si `--work-width` ou `--osd-region` a été donné, sinon format non géré ;
      - disque plein, écriture refusée et, sous Windows, fichier ouvert ailleurs (erreurs 32 et 33 au ménage, #115).
        Ces trois-là ne se cherchent que du côté écriture, sur toute la chaîne `source()`, par `io::Error` ou `LibraryError::io_kind`.
        Le fichier ouvert se reconnaît sur `Previous`, aux variantes `Remove` et `Read`, les seules qui portent le chemin ; pour un dossier d'extraits, c'est le dossier qui est nommé, pas le fichier ouvert dedans ;
      - extraits non écrits ;
      - trop de pistes ;
      - erreur inattendue ;
      - erreur interne (panique).
  - **Tenue aux pannes** (#113) : `outcome::run_all(vidéos, travail, compte rendu)` passe chaque vidéo dans `catch_unwind`, le seul du code livré (règle 10).
    Le travail est `Fn + RefUnwindSafe`, si bien que le compilateur vérifie qu'il ne modifie rien de ce qu'il capture.
    `run_all` appelle le compte rendu au fil de l'eau (le binaire n'y fait qu'écrire) et rend un `Row` par vidéo (#65) : le `Job`, le temps de calcul (`Instant` autour de `catch_unwind`, détection comprise) et un `State`, `Done`, `TooManyTracks` ou `Failed(Failure)` (`Failure::Error`, ou `Failure::Panic` avec son message et son lieu).
    Une panique donne « erreur interne », avec le message et le lieu en détail, compte comme un échec (code 1), et la vidéo suivante est traitée.
    - Crochet (#64) : `outcome::quiet_panics()`, appelé une fois par `main`, garde le crochet par défaut.
      Pendant une vidéo (un `AtomicBool` mis par `run_all` autour de `catch_unwind`), il n'écrit rien et garde la première panique dans un `static Mutex`, repris même empoisonné.
      Il est global et non par fil, parce que `resume_unwind` n'appelle pas le crochet ; hors d'une vidéo, il passe la main au crochet par défaut.
      Le lieu est celui que rustc connaît : `crates/…` sous macOS et Linux, `crates\…` sous Windows, un chemin absolu pour une dépendance.
    - La vidéo suivante peut repartir parce que le travail ne capture que des références vers des données qu'il ne modifie pas (options, langue, versions), et que les fils d'une vidéo sont rejoints ou arrêtés avant la relance (`read_ahead`, `serve_ahead`, `schedule`, rayon), ce que deux tests `should_panic` vérifient pour une panique du consommateur.
    - Ce qu'une panique peut laisser : les extraits passent par un temporaire effacé au déroulement, mais pas le CSV ni `params.json`, si bien qu'un CSV peut rester tronqué, comme un échec laisse ses fichiers déjà écrits.
      Une panique dans l'initialisation de FFmpeg (un `Once`) l'empoisonne, et les vidéos suivantes paniquent aussi.
      Le manque de mémoire, une panique pendant une panique et une panique dans un `rayon::spawn` ne sont pas rattrapés ; la borne de la fenêtre du fond (`engine`) écarte la première cause connue.
  - **`params.json`** (contenu dans [02](02-methode.md#sorties)) : réglages rangés par configuration, avec les clés de la v0.1.1 (`DetectConfig`, `TrackConfig`, `StabilityConfig`, par `tracking::settings`), un filtre coupé s'écrivant 0.
    - `threads` donne les fils réellement employés : `decoding` et `decoding_kind` (`single`, `frame`, `slice`) pour le décodeur, `detection` pour `rayon`.
    - `version` est celle de `--version`, gravée par `crates/cli/build/engraved.rs` (#113) : la variable `BATDETECT_VERSION` si elle est posée, sinon `git describe --tags --match 'v[0-9]*' --dirty --always` sans le `v` (par exemple `0.1.1-81-g40591c2-dirty` ; le hachage court seul dans un clone sans tags), sinon `CARGO_PKG_VERSION`.
      Le script ne se relance que si `HEAD`, la branche qu'il nomme, `refs/tags`, `packed-refs` (chemins absolus, worktree compris, seulement ceux qui existent), `crates/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/`, `packaging/` ou `tests/` changent ; ce sont les chemins de la provenance de `record`, qui partage ce script (#295).
      Un `git fetch` ou un commit d'un autre worktree ne le relancent donc pas, « -dirty » ne voit pas une modification de `docs/` ou de `mise.toml`, et toute modification de source recompile `cli`, la crate feuille.
    - `ffmpeg` est la version de FFmpeg liée.
    - `video` est écrit par `to_string_lossy` (#227) : un nom non UTF-8 y garde ses caractères lisibles, et ses octets illisibles deviennent `�`, au lieu de faire échouer l'écriture.
      C'est un texte à lire, pas un chemin à rouvrir : la relance compare la taille, la version, la langue et les réglages, jamais ce nom, et la page retrouve le tableau par son nom lossy (`results`).
      Des octets échappés dans la chaîne auraient changé la forme du champ pour tous les noms et demandé un décodage à chaque lecteur.
    - `RenderConfig` (#112, #103) : `box_pad`, `trail_s`, `clip_margin_s`, `max_tracks`, `zoom` (`auto`, `all` ou `none`), `zoom_below_area` (`100.0`), `zoom_below_amplitude` (`40.0`), `annotated`, et `encoders`, la liste des encodeurs retenue par l'essai (vide au-delà de la garde).
    - Écarts à la v0.1.1 : `mode` et `ffprobe` retirés ; dans `RenderConfig`, `crf`, `encoder` et `vt_quality` ne sont pas écrits, et `encoders` les remplace pour l'encodage ; `version`, `ffmpeg` et `threads` ajoutés ; les nombres réels s'écrivent `12.0` et non `12`.
      `DetectConfig` s'écrit par `engine::settings::DetectSettings`, `osd_regions` compris, en objets `{x0, y0, x1, y1}`.
  - **Tests** : `tests/command.rs` lance le binaire sur une vidéo fabriquée (openh264, 1440×1080), qu'il passe en français, en anglais, puis en français dans le même `-o` (#115) ; `slow_command.rs` passe l'extrait (4 pistes appariées, chauve-souris de 3:58) ; `reference_092.rs` passe la 092 ([Gardes de détection](#gardes-de-détection)).
    - Sorties de l'extrait (`tests/local_outputs.rs`, `#[ignore = "local"]`, `BATDETECT_REFERENCE_DIR`, #115) : le binaire passe l'extrait en français, en anglais, puis en français, dans le même `-o`.
      Entre la première et la deuxième passe, le test ajoute des anciens noms et des parties en trop.
      Après chaque passe, la liste des fichiers égale, au nom près, celle de la v0.1.1 (`outputs/{fr,en}/` de la référence, hors dépôt) : CSV, image résumé, `params.json` et les 4 extraits.
      Comme la v0.1.1, le Rust ne fait aucun zoom sur l'extrait (`--zoom auto`).
      Le test prend environ 20 s (18,5 et 23,7 s le 14 octobre 2026 ; il ne mesure pas de temps).
    - `tests/local_command.rs` (`#[ignore = "local"]`, `BATDETECT_REFERENCE_DIR`, `BATDETECT_VIDEOS_DIR` et `BATDETECT_REFERENCE_V020_DIR`) passe `process` (la fonction qui écrit le CSV) sur les 26 vidéos et apparie ses pistes à celles de la v0.2.0 par `match_tracks` ([Gardes de détection](#gardes-de-détection)).
      Les plages abîmées sont celles du moteur.
- **`tracking`** → `types` : exclusion des images abîmées et instables (#120), puis suivi (#121).
  `compare` et `cli` enchaînent `exclude` puis `track_detections`.
  - `exclusion` est le seul point d'entrée.
    `exclude` reçoit les détections, les plages abîmées toutes faites, le nombre d'images signalées et la `FrameRate`, et ne peut pas échouer.
    Les images abîmées sont retirées du calcul de stabilité, puis les détections des plages abîmées et instables sont vidées, images gardées.
    `Exclusion` rend aussi le nombre d'images ignorées et celui des images abîmées, chacune comptée une fois.
  - `settings` (#108) : `TrackSettings` et `StabilitySettings` écrivent et lisent les réglages de suivi et de stabilité aux clés de la v0.1.1, pour que `params.json`, `bench.json` et `run.json` partagent une seule forme.
    Un filtre coupé s'écrit 0 (`twin_distance`, `max_blobs`).
  - `stability` : `StabilityConfig::new(max_blobs, pad_s)`, par défaut 20 et 1,0 s.
    `max_blobs` est un `Option<NonZeroU32>`, et `None` coupe le filtre (la commande traduit `--max-blobs 0`) ; `pad_s` est fini, positif ou nul, sinon `Error::Setting` avec `Setting::PadS`.
    - La limite de foule vaut `max_blobs + 6 × médiane` du nombre de détections des images lisibles, vides comprises, calculée en entiers.
    - Une image qui l'atteint donne une plage de `pad_s × cadence` images de chaque côté, arrondi au plus proche, moitié vers le haut (la v0.1.1 arrondit au pair : une image d'écart quand ce produit vaut x,5, jamais avec les réglages par défaut).
      Le rembourrage est borné à la longueur de la vidéo.
    - Seuls les bords sont bornés aux premières et dernières images lisibles : au milieu, une plage instable peut recouvrir une plage abîmée (093, 097).
    - Deux plages séparées d'au plus le rembourrage se fondent.
    - La médiane porte sur toute la vidéo, si bien que la stabilité dépend de sa longueur : la concaténation 089-091 ignore 27,9 s, contre 28,9 s en morceaux (mesuré par #94 ; la médiane comme cause est une hypothèse, non vérifiée).
  - `spans` (interne) : les images couvertes par des plages, bornées à la vidéo, pour les comptes d'`Exclusion` ; les détections se vident par `FrameDetections::without`.
  - Les 3 tests de saut de luminosité de la v0.1.1 sont dans `compare`, parce qu'ils enchaînent détection et stabilité.
  - `committed_exclusion` (19 cas du dépôt) et `local_exclusion` (26 vidéos) comparent à la v0.2.0, avec une ligne figée par cas (`Kept`) : détections gardées en moins et en plus (écart net des totaux), et plages instables changées, toutes à 0 (#277).
  - `track` : `track_detections(&Exclusion, SourceSize, TrackConfig)` rend des `Track` numérotées à partir de 1, triées par (image de début, `x` du premier point).
    Elle prend l'`Exclusion` entière (détections et plages ignorées) et la taille de l'image, dont le recollage a besoin (#24).
    - `TrackConfig::new(max_jump, max_gap, min_hits, min_travel, twin_distance, max_median_turn, rejoin)`, par défaut 120, 6, 6, 45, `Some(36)`, 0,8 et `Some(RejoinConfig::default())` (19 images, 60 px, morceaux d'au moins 2 détections, allures 0,25 et 0,45).
      `without_rejoin()` rend le suivi sans recollage, celui des références v0.1.1 et v0.2.0.
      Les accesseurs portent les noms des clés de `params.json`.
      `max_gap` et `min_hits` sont des `NonZeroUsize`, `twin_distance` un `Option<f64>` (`None` coupe la fusion ; la commande traduit `0`).
      Un réglage hors bornes rend `Error::Setting` avec son `Setting`.
      `Error::Track` ne vient d'aucune entrée : c'est un garde-fou, qui signale un bogue de l'association ou de la fusion des jumelles (deux points sur une même image).
      La renumérotation finale passe par `Track::with_id`.
    - Écarts voulus à la v0.1.1 : NaN refusé pour `max_jump`, `min_travel` et `twin_distance` ; infini refusé pour les quatre réglages `f64` ; `Some(0)` refusé pour `twin_distance`, le `0` de la v0.1.1 devenant `None`.
    - Image par image, les pistes restées plus de `max_gap` images sans détection finissent.
      Chaque piste prédit sa position par sa vitesse sur au moins 3 images (l'avant-dernier point à défaut).
      L'appariement est glouton sur (pistes qui ont une vitesse d'abord, distance ≤ `max_jump`, ordre de création, ordre des détections), et une détection restée seule ouvre une piste.
    - Détections faibles (#21) : l'association attribue d'abord les détections fortes, puis les faibles aux pistes restées sans détection.
      Une faible libre n'ouvre pas de piste, et un point réuni par les jumelles est fort si l'un des deux l'est.
      Sur des détections toutes fortes, comme celles des références, rien ne change ([10](10-logique-de-detection.md#7-relier-les-taches-dans-le-temps--le-suivi)).
    - Jumelles : pistes qui coexistent, partagent au moins une image et restent à ≤ `twin_distance` sur chaque image commune.
      Les groupes se font par union-find sur les pistes triées par image de début, et sortent dans l'ordre de leur premier membre.
      Les points d'une même image se combinent : boîte englobante, surfaces additionnées, barycentre pondéré, amplitude maximale.
    - Filtres après fusion : `min_hits`, corde ≥ `min_travel`, virage médian ≤ `max_median_turn` (mesures de `types::track::Track`).
    - Recollage après les filtres (`track/rejoin.rs`, #24) : règle et mesures dans [10 § 9 bis](10-logique-de-detection.md#9-bis-recoller-un-vol-caché-derrière-la-végétation), valeurs dans [02 § 7 bis](02-methode.md#7-bis-recollage).
      `RejoinConfig::new(max_hidden, max_distance)` refuse une distance hors bornes (`Setting::RejoinDistance`).
      `TrackConfig::new` refuse un `max_hidden` qui ne dépasse pas `max_gap` (`Setting::RejoinHidden`), puisque le recollage n'aurait plus rien à faire.
      Les liens se prennent du plus petit écart au plus grand, une fois par fin et par début ; les chaînes donnent une seule piste, et aucun point n'est ajouté dans le trou.
      `TrackSettings` a `rejoin_hidden` et `rejoin_distance` (0 = coupé), lus à 0 dans un fichier qui ne les a pas.
      `--replay` et `--targets` jugent avec les réglages de la commande, donc avec recollage par défaut ; seul `--against` exige les réglages enregistrés, d'où `--rejoin-hidden 0` contre la v0.1.1 (`local_guard` juge avec `without_rejoin()`).
      Une piste recollée garde les bouts dont elle est faite (`Track::rejoined`, `Track::parts`), et le zoom automatique la prend si l'un d'eux y avait droit seul : sans cela, 10 pistes des 095, 096, 097, 098 et 100 perdaient leur zoom.
    - Morceaux et allure (#311, `track/rejoin/pace.rs`) : `track_detections` met de côté les pistes écartées par les filtres qui ont de `piece_points` à `min_hits − 1` points (`RejoinConfig::takes_as_piece`) et les passe à `rejoin(kept, pieces, …)`.
      Un lien entre deux morceaux n'existe pas.
      Les liens entre pistes gardées sont pris avant ceux des morceaux, qui ne peuvent donc pas défaire une soudure, et une chaîne sans piste gardée est jetée.
      `Track::rejoined(id, Vec<Stretch>)` reçoit la chaîne en `Stretch::Part` (piste gardée) et `Stretch::Piece` (morceau) : les points des morceaux sont dans la piste, mais `parts()` ne rend que les pistes gardées, seules jugées par le zoom ; une piste de morceaux seuls est refusée (`Error::PiecesOnly`).
      Un `piece_points` au moins égal à `min_hits` laisse la pile vide, sans erreur, si bien que `--min-hits 2` reste permis.
      `RejoinPace::keeps` juge chaque lien, entre pistes gardées comme avec un morceau : le saut doit valoir au moins `junction` × la vitesse de chaque piste gardée du lien (5 points du côté du trou), et un morceau d'au moins 2 points doit avoir une vitesse propre d'au moins `piece` × celle de la piste.
      `RejoinConfig::with_pieces(piece_points, pace)` ; `RejoinPace::new` refuse une allure négative ou non finie (`Setting::RejoinJunctionPace`, `RejoinPiecePace`).
      `TrackSettings` a `rejoin_piece_points`, `rejoin_junction_pace` et `rejoin_piece_pace`, lus à 0 dans un fichier plus ancien ; le banc, `--rejoin-piece-points`, `--rejoin-junction-pace` et `--rejoin-piece-pace`.
    - Portée (#326, `track/reach.rs`, `track/edge.rs`) : `Gate { max_jump, reach, size }` décide dans `Association::pair` si une piste peut prendre une tache, dans les deux tours, fort et faible.
      Une piste qui a une vitesse ne la prend qu'à `miss ≤ Reach::farthest(way)`, soit `margin + travel × way`, où `way` est `Growing::predicted_travel`, calculé une fois par piste et par image (`Gate::reach_of`) ; une piste d'un point n'a que `max_jump`.
      La portée ne joue pas si le dernier point, la tache ou `Growing::speed_measured_from` est au bord (`edge::at_edge`, 30 px, partagé avec le recollage).
      `predicted_travel` prend la vitesse sur les 5 derniers points (`Growing::tail_speed`, `TAIL_POINTS`) quand celle de la prédiction tombe sous 0,4 fois elle (`SLUMP_SHARE`) ; la prédiction reste la même au bit près.
      `TrackConfig::with_reach(Option<Reach>)`, par défaut `Some(Reach::default())` (50 px, 1) ; `None` coupe la portée, et `Reach::new` refuse une marge non positive ou non finie et un `travel` négatif ou non fini (`Setting::ReachMargin`, `ReachTravel`).
      `TrackSettings` a `reach_margin` et `reach_travel`, à 0 quand la portée est coupée et dans un fichier plus ancien ; le banc, `--reach-margin` (0 coupe) et `--reach-travel`.
    - Limite : plusieurs seuils tombent sur une distance exacte, et `f64::hypot` peut différer d'un ulp de celui du Python.
      Le critère des tests est donc l'appariement avec tolérances ; l'identité est mesurée en plus.
    - Gardes (#277) : le suivi courant tourne sur les détections, les plages abîmées et le nombre d'images signalées de la [référence v0.2.0](#référence-v020), sans décodage, donc sans dépendre du poste ni du décodeur.
      Ses pistes se comparent à celles de la v0.2.0 par `track_matching` (début ±5 images, points ±3).
      Chaque cas a sa ligne figée (`FROZEN`) : pistes de la référence, suivies, appariées, concordantes, perdues et ajoutées, `Tracks::same(n)` tant qu'il n'y a pas d'écart.
      Les 19 cas du dépôt n'ont aucune plage abîmée, et seul `sigma_0` a des plages instables : en CI, le suivi sur des plages abîmées n'est gardé que par les tests unitaires de `tracking`, et sur des données réelles par `local_tracking` et `local_exclusion` seulement.
      Une PR de suivi met à jour ces lignes, avec la cause dans la même série de changements, et les colonnes de pistes des [gardes de détection](#gardes-de-détection), qui disent l'effet.
      - `committed_tracking` : les 19 cas du dépôt (extrait, 092, 1440, 16 réglages), en CI sur les trois systèmes, avec en plus le nombre de pistes identiques au bit près et les numéros 1..n.
        Le recollage (#24) n'y change rien. La portée (#326) change trois cas, qui gardent 92 pistes identiques au lieu de 125 :
        - `sigma_0` (l'extrait sans seuil par pixel) passe de 30 à 17 pistes : 13 appariées et concordantes, 17 perdues, 4 ajoutées, une seule identique.
          Les 13 pistes de moins sont du bruit, et les 4 chauves-souris de l'extrait restent, qui commencent aux images 144, 707, 781 et 972 au lieu de 144, 705, 783 et 972 ;
        - `width_720` garde ses 5 pistes appariées, dont 2 ne concordent plus : les deux pistes jumelles entre lesquelles la chauve-souris des images 705-722 était déjà coupée se la partagent autrement ;
        - la 092 à 1440 garde ses 14 pistes appariées et concordantes, dont 2 ont d'autres points.
          La piste de 2:02.85 ne commence plus par deux points sur une tache immobile (images 3688-3689, à 77 px du premier point de la chauve-souris).
          À 1:49.75, un morceau de deux points, que la portée arrête là, reste à 27 et 34 px de la piste 6 et s'y fond comme une jumelle, ce qui déplace ses points des images 3294 et 3296 de 24 et 2 px.
      - `local_tracking` : les 26 vidéos (`BATDETECT_REFERENCE_V020_DIR`), 972 pistes dans la référence, 902 suivies avec la portée.
        Ses lignes sont `changed(référence, suivies, appariées, concordantes, perdues, ajoutées)`, puisque la portée ne fait pas que souder : 849 appariées, 774 concordantes, 123 perdues et 53 ajoutées.
        Le nombre de pistes identiques (même numéro, mêmes points au bit près) est figé vidéo par vidéo à côté des écarts (`Identical(n)`, #340) : 171 en tout sur lumiel le 18 octobre 2026, après #21, #311 et #326.
        Une piste recollée reste appariée à sa première partie mais n'est plus concordante, et sa seconde partie compte comme perdue.
      - `committed_metrics` déclare les réglages gagnés depuis la v0.2.0 (`judged_now` : recollage, `reach_margin` et `reach_travel`) et fige leur effet sur les deux bancs.
        Au transit : 163 pistes injectées, 4 cibles changées par rapport à la v0.2.0 (la 79 commence 23 images plus tôt, la 151 trois plus tard, la 213 finit 3 images plus tôt, la 167 tient en un fragment au lieu de 2), et 3 classes changées (passage d'amplitude −15 à sigma 5 : délai médian de début de 4,5 images au lieu de 6,5).
        À la chasse : 498 pistes injectées au lieu de 514, 12 cibles moins fragmentées, et une cible perdue, dont la piste a été soudée à celle d'une autre cible qui la précédait au même endroit (454 trouvées au lieu de 455).
- **`files`** → `types` : les fichiers écrits par la commande (#122), dans la langue de la commande (`types::Language`).
  - `names` : `Names::new(langue)` et ses méthodes.
    Français : `extraits`, `piste`, `_pistes`, `_resume`, `_annotee`, `_fond`, sans accent ; anglais : `clips`, `track`, `_tracks`, `_summary`, `_annotated`, `_background`.
    `params.json` n'est pas traduit, et `_zoom` est commun.
    - Le fond de la page (`_fond`, `_background`, #259) est reconnu par le ménage comme une image.
    - Le nom d'extrait se construit depuis la `Track`, avec le temps de début passé par l'appelant en secondes.
      Il s'écrit en minutes et secondes entières, tronquées comme dans la v0.1.1 : à 59,996 s, le nom dit `0m59s` et le CSV `1:00.00`.
    - Le numéro de piste prend la largeur du nombre de pistes, 2 chiffres au moins.
    - La période de l'image résumé est donnée en minutes (`_000m-010m`).
    - Une image coupée en parties porte sa partie, numérotée à la largeur du total : `_3_sur_8`, `_03_sur_12`, `_3_of_8` en anglais, après la période s'il y en a une (`_000m-010m_2_sur_4`).
    - Les noms de vidéo sont des `OsStr`, gardés tels quels.
    - Une troisième table, celle des anciens noms (`split`, `.tracks.csv`, `.tracks.png`, `_boxes.mp4`, ceux de la v0.1.0), ne sert qu'à les reconnaître (#115).
  - `previous` (#115) : `remove_previous_outputs`, le ménage des sorties d'une passe précédente dans le dossier d'une vidéo, dans les trois tables.
    Comme la v0.1.1, il compare en NFC le nom de chaque entrée à celui de la vidéo.
    - Il retire les dossiers d'extraits, les CSV, les vidéos annotées, les images résumé avec leur période (`_000m-010m`, 3 chiffres au moins) et leur partie (`_3_sur_8`, `_3_of_8`), et les partiels des vidéos annotées, des images, des CSV et de `params.json` que laisse un arrêt brutal, sous les deux formes de `names::partial_file`.
      `params.json` est réécrit, pas retiré, et le reste du dossier n'est pas touché.
    - Une entrée à ce nom qui n'est pas un fichier ordinaire (dossier, lien) est laissée en place et rendue (`Kept::Left`, avec `ours` vrai si elle porte le nom de la langue courante) ; le nouveau tableau n'est alors pas écrit.
    - Un lien qui porte le nom d'un dossier d'extraits est retiré, sa cible non (sous Windows, un lien de dossier par `remove_dir`).
  - `names::partial_file` crée le partiel de tout fichier écrit : un fichier vide `.<nom>.<6 lettres ou chiffres>.partial` à côté du fichier final, au nom tiré au hasard et créé de façon exclusive (`tempfile::Builder::make_in` et `File::create_new`, #245).
    Deux écrivains du même fichier, comme le fil du Ctrl-C qui réécrit la page pendant que le fil principal l'écrit, ou deux `batdetect -o` sur le même dossier, ont donc chacun le leur.
    Le dernier renommage gagne et chaque fichier final est entier, sauf pour le tableau des pistes, où le premier gagne (voir `table::write_tracks_csv`).
    Un chemin sans nom de fichier est refusé (`InvalidInput`).
    `table`, `params`, `failures`, `summary::image_file` et la page de `cli` l'emploient ; `media::path` a la même forme, liée par un test de `cli`.
    Le ménage reconnaît cette forme, quel que soit le mot de 6 lettres ou chiffres (`.v_pistes.csv.backup.partial` compris), et l'ancienne, `.<nom>.partial`, laissée par la 0.2.0.
    `tempfile_in` n'est pas employé, parce qu'il crée le fichier en `0600`, que le fichier final garderait.
    Un partiel laissé par un arrêt brutal à la racine (page) ou dans `.batdetect/echecs/` n'est pas écrasé au passage suivant : il reste, caché.
  - `table::write_tracks_csv` écrit dans son partiel, puis le renomme sans jamais écraser un fichier présent (`exclusive::rename`, sinon `Error::Taken`).
    - Le renommage exclusif (`TempPath::persist_noclobber`) passe en premier.
      Sur toute autre erreur que « nom déjà pris », dont le refus de macOS sur exFAT (#236), le nom final est réservé (`exclusive::reserve`), puis le partiel renommé par-dessus.
      Si la réservation échoue, c'est son erreur qui remonte ; si le renommage échoue, la réservation vide est retirée, et si ce retrait échoue, c'est son erreur qui remonte.
    - Un arrêt brutal pendant l'écriture ne laisse qu'un partiel, que le ménage retire, et jamais un CSV tronqué qui passerait pour étranger.
    - La réservation ne sert qu'en repli : un arrêt brutal entre elle et le renommage laisserait un CSV vide, que le ménage retirerait au calcul suivant.
    - `tests/slow_exfat.rs`, sous macOS seulement, monte une image exFAT par `hdiutil` sans `sudo`.
      `persist_noclobber` y est refusé ; le tableau s'y écrit, refuse d'écraser un tableau annoté, est retiré par le ménage et se réécrit.
    - macOS 27 écrit sur cette image un `._x` (AppleDouble) à côté de chaque fichier : la liste des fichiers restants des tests les écarte, mais garde les partiels cachés (#335, `files/tests/listing.rs`).
      Le ménage n'y touche pas, et macOS retire le `._x` avec son fichier.
    - Sur un disque insensible à la casse (APFS, NTFS), un `vol_pistes.csv` annoté et la vidéo `VOL.mp4` ont le même chemin.
      Le ménage, qui compare les octets, ne reconnaît pas ce fichier, et le renommage refuse de l'écraser.
  - `time` : `clock` écrit `h:mm:ss`, tronqué à la seconde comme le nom de l'extrait.
    C'est aussi le seul format du début d'une piste sur la page (liste, survol de la frise, tête du lecteur, libellés), écrit par le binaire dans la cellule `.clock` de la liste et relu par le script (#304).
    Les temps d'axe (minutes de la frise et intervalle au-dessus de l'image) restent en `m:ss` (`summary::header::clock`), comme sur l'image résumé.
    - Le temps d'une image vient de son `pts` (`Timeline::seconds_by_pts`).
      Une image sans `pts` reprend celui de la dernière image qui en a un, plus un pas de cadence par image ; la cadence seule ne sert qu'en l'absence de tout `pts`.
    - La fin d'une plage est le début de l'image suivante.
    - Le temps n'est pas rapporté au premier `pts`, puisque les 26 vidéos de la référence commencent à `pts` 0 ; un `pts` négatif reste à décider avec #116.
  - `table` : CSV des pistes pour un tableur (#84), 12 colonnes dans l'ordre de `table::COLUMNS` ; colonnes et forme par langue dans [02](02-methode.md#sorties).
    Les en-têtes sont des textes Fluent, donnés par le binaire à travers le trait `table::Headings`, et `files` ne les connaît pas.
    Les verdicts n'y sont pas : ils vivent dans `verdicts.csv` (#228).
    - `dialect` porte la forme d'une langue (BOM, séparateur, virgule ou point décimal), CRLF dans les deux.
    - Une case qui contient le séparateur ou un guillemet est mise entre guillemets ; les cases des nombres et des temps n'en ont jamais.
    - Les nombres sont arrondis, puis écrits sous leur forme la plus courte, avec `.0` pour un entier (`12.0`, `12,0`) ; `-0.0` devient `0.0`.
    - La surface max est entière, arrondie à mi-chemin vers le haut.
    - Durée = fin − début, au moins une image.
  - `params` : `Params<S>` écrit `video` (un chemin), `fps`, les réglages de l'appelant aplatis, `ignored_s` et `damaged_s` (au centième), `damaged_frames`, `version` (fixée à la compilation par l'appelant) et `ffmpeg` (version liée).
    - `files` n'impose pas les clés des réglages : garder celles de la v0.1.1 (une clé par configuration, au nom de sa classe, dont les seuils de zoom dans `RenderConfig`) revient à l'appelant, et `cli` porte le test des clés et celui des seuils de zoom (`cli/src/settings/tests.rs`).
    - Une clé `video` ou `fps` venue des réglages serait écrite en double.
  - CSV écrit depuis les pistes de la référence v0.1.1, comparé à son `tracks.csv` (`files/tests/committed_table.rs`, mesuré le 11 octobre 2026, relu par valeurs le 16 octobre 2026) :
    - sur l'extrait, les valeurs sont les mêmes, début et fin tronqués à la seconde ;
    - sur la 092 (14 lignes), numéro, points, corde, chemin, amplitude, images comblées et durée sont égaux ;
    - les temps sont plus tôt d'un ou deux centièmes : débuts décalés sur 11 pistes sur 14 (de 33,94 → 33,93 s à 237,18 → 237,16 s), fins sur 12, et, tronquée à la seconde, seule la fin de la piste 6 change (1:50.00 → 1:49.99, soit `0:01:49`).
      La v0.1.1 divisait le numéro d'image par la cadence moyenne (30,027 i/s), alors que les `pts` avancent de 333/10 000 s par image (30,03 i/s) ;
    - `speed_px_s` est plus haute de 0,1 à 0,2 px/s sur 10 pistes, jamais plus basse ;
    - `max_area_px` change sur les pistes 1, 5, 6 et 10 : 274 → 275, 400 → 401, 490 → 491 et 274 → 275.
      Les maxima tombent sur un demi (sur la 092, les surfaces sont des multiples de 2,25) : le Rust arrondit à mi-chemin vers le haut, la v0.1.1 au pair, et l'écart est gardé (règle 5).
  - Lecture d'un dossier de résultats pour la page (#71) :
    - `results::read_results` lit chaque sous-dossier non caché, dans l'ordre des noms.
      Avec un `params.json` lu (`Written`, avec `Deserialize` sur `Finish` et `TrackEntry`), la vidéo est terminée ; avec un `params.json` à nous sans `state`, c'est une ancienne version.
      Sans `params.json`, avec une sortie nommée d'après la vidéo (`names_the_video`, le nom du dossier pris pour celui de la vidéo ; un dossier `clips/` seul ne compte pas), elle est inachevée.
      Ni l'un ni l'autre : le dossier est ignoré, pour ne jamais lister un dossier de l'utilisateur.
      Les échecs de `.batdetect/echecs/` s'y ajoutent, avec ou sans dossier.
    - Le fond (`background` de `params.json`) est donné avec sa taille, lue dans l'en-tête PNG ; absent du disque ou du fichier, il manque, et `summaries` est vide (`#[serde(default)]`).
    - Les images résumé sont celles que nomment les pistes de `params.json`, dans leur ordre, ou l'image sans suffixe s'il n'y a pas de piste ; leur taille vient de l'en-tête PNG (`picture`).
    - Le tableau des pistes se cherche sous le nom tiré de `video`.
      S'il est absent et que ce nom contient `�`, il se cherche sous le fichier du dossier dont le nom lossy est le même (`on_disk`, #227 ; à égalité, le premier que liste le système).
      Une vidéo au nom non UTF-8 retrouve ainsi son tableau, mais pas ses images résumé ni sa vidéo annotée, cherchées sous le nom lossy.
    - `verdicts::read_verdicts_file` (#228) : le plus récent des fichiers `verdicts*.csv` de la racine (le navigateur numérote les doubles), son nom, sa date et le nombre de candidats.
      Il ne lit que les octets, sans décodage ni lecture du CSV : c'est le JS de la page qui les lit, et batdetect n'écrit jamais ce fichier.
    - `failures` : un fichier par vidéo en échec dans `.batdetect/echecs/`, écrit par `.partial` et renommage ; un fichier illisible ne cache que lui-même.
    - `refusals` (#267) : `.batdetect/refus.json`, les entrées refusées au dernier lancement, écrit par `.partial` et renommage ; illisible ou vide, il ne donne rien.
    - `names::results_page` : `resultats.html` / `results.html`.
- **`render`** → `media`, `draw`, `types` (#110, #112) : les extraits, du choix des images à l'écriture.
  `clips::render(source, Colored, Outputs { clips, zooms }, RenderConfig, encodeurs)` en est le point d'entrée ; `cli` l'appelle et choisit les zooms par `ZoomConfig::wants` (#103).
  - `colored` (#287) : `Colored::new(pistes, couleurs)` réunit les pistes et leurs couleurs, construit une fois par `process`.
    Une tranche de couleurs vide (boîte rouge et trace jaune pour toutes) ou d'autant de couleurs que de pistes est acceptée ; toute autre longueur est refusée par `Mismatch`, au lieu de retomber en silence sur la boîte rouge.
  - `drawn`, derrière la feature `testing` (tests seulement, #198) : `drawn(source, Colored, Outputs, RenderConfig)` passe par le même rendu, sur un fil, mais chaque encodeur est remplacé par un enregistreur qui garde l'empreinte de chaque image poussée, fichier par fichier.
    Les tests figés de `cli` s'en servent.
  - `source` : `Source::new(chemin, Timeline, images-clés, images en erreur)`, puis `serve(plage, visite)`, qui rend chaque image de la plage une fois, dans l'ordre, avec son `FrameIndex`.
    Les images-clés et les images en erreur viennent du moteur (`FrameInfo::is_key`, `Damage::error_frames`) : `render` ne dépend pas d'`engine`.
  - Les images se choisissent par `pts` : chaque `pts` rendu par le décodeur est cherché dans la `Timeline` (recherche inverse, interne à `render`).
    Une `Timeline` dont les `pts` connus ne croissent pas strictement est refusée, comme une plage qui sort de la vidéo ou dont une image n'a pas de `pts`, avant tout décodage.
  - Contrôle de la suite servie : un `pts` inconnu de la `Timeline`, une image qui revient après une plus récente, une image de la plage que le décodeur saute ou qui manque en fin de fichier sont des erreurs typées.
    Une erreur de la visite arrête la lecture et revient telle quelle, comme pour `engine::input`.
  - Image de reprise : la dernière image-clé à ou avant le début de la plage qui n'est pas elle-même en erreur.
    La dissimulation d'une image-clé en erreur puise dans les images d'avant, et seule une image-clé saine remet le décodeur à zéro.
    Il n'y a pas de marge réglée : la marge de 30 du prototype supposait des GOP de 15 (ici de 8 à 16).
  - Fils de décodage : un seul (`Threads::Exactly(1)`) quand la plage lue, depuis l'image de reprise, contient une image en erreur, car les pixels dissimulés ne se reproduisent qu'à un fil (voir « Fils et pixels dissimulés » dans `engine`).
    Sinon, ils sont automatiques, car une vidéo saine donne les mêmes pixels quel que soit le nombre de fils (`slow_source`, `local_clips`).
    L'appelant sert une plage par groupe de fenêtres chevauchantes, pour ne décoder qu'une fois une image partagée ; le parallélisme vient des groupes.
    Il borne aussi les fenêtres à la dernière image, puisque `serve` refuse une plage qui sort de la vidéo (la v0.1.1 ne les bornait pas).
  - Placement raté : si le placement échoue, ou si la première image rendue vient après l'image de reprise (conteneur sans index), la plage est relue depuis le début.
    `serve` rend d'où il a lu : `Origin::Key(image)`, ou `Origin::Start` avec sa raison (`Restart::NoCleanKey`, `Unplaced` avec l'erreur de FFmpeg, `Overshot` avec l'image de reprise et la première image rendue).
    - Le dépassement est testé : une image-clé forgée avant la première image du mp4 est ramenée par FFmpeg à l'image 0 (`mov.c`), qui vient après elle.
    - Aucun mp4 ne fait échouer le placement : une cible avant le début est ramenée à la première image.
      `Unplaced` et l'erreur `OutOfOrder` (une image rendue après une plus récente, impossible tant que le décodeur rend dans l'ordre des `pts` et que la `Timeline` croît strictement) restent des gardes non couvertes.
  - Hypothèse : une image-clé saine remet le décodeur à zéro.
    C'est vrai d'une IDR, mais FFmpeg marque aussi image-clé une image à point de reprise (SEI `recovery_frame_cnt` nul), qui n'est pas forcément indépendante.
    Seuls l'extrait et les 103, 123 et 124 le confirment.
    Le placement passe par le flux vidéo par défaut de FFmpeg, le décodeur par `best(Video)` : ils coïncident tant que le fichier n'a qu'un flux vidéo.
  - `Source` ne garde de la `Timeline` que le nombre d'images et la table `pts` ↔ image : sans durée de vie, `Send` et `Sync`.
    Un test sert trois plages depuis trois fils à la fois.
  - Mesures (M1 Max, `--profile ci`) : chaque plage est comparée, empreinte par image, à une lecture séquentielle à un fil.
    - Test rapide : une vidéo openh264 1440×1080 de 48 images, une image-clé toutes les 8, dont la sixième part sur douze de `mdat` est écrasée, donne 40 images, 8 perdues d'un coup, et l'image-clé 16 en erreur.
      Les 40 plages d'au plus 4 images et la plage entière sont égales ; celles qui commencent de 16 à 23 reprennent à l'image-clé 8.
      Reprendre à l'image-clé 16 en erreur fait échouer la plage 16–19 (essai fait une fois).
    - `slow_source` : 12 plages de 1 à 380 images tirées dans l'extrait (SplitMix64, graine fixe), égales, en 24 s.
    - `local_source` : la fenêtre 6580–6684 de la 103, seul extrait des 26 vidéos qui traverse une perte d'images, reprend à l'image-clé 6573 et rend ses 105 images égales (63 s).
      Les 8 fenêtres d'extraits de la 123 et les 3 de la 124 qui touchent une plage abîmée (pistes de la référence, marge `round(1,5 × cadence)` = 45, bornée à la vidéo) sont égales (140 s pour les deux).
      Aucune de ces 12 fenêtres ne saute une image-clé en erreur : seule la vidéo fabriquée exerce cette règle.
  - `config` : `RenderConfig` (`box_pad` 12 px, `trail_s` 1 s, `clip_margin_s` 1,5 s, `max_tracks` 300, 0 pour sans limite), valeurs de la v0.1.1, validées à la construction (ni `NaN`, ni infini, ni négatif).
    Les secondes deviennent des images par `round(secondes × cadence)` (`f64::round`).
    `ZoomConfig` (#103) : `Zoom::Auto`, `All` ou `None`, et les seuils `max_area < 100` px² ou `peak_amplitude < 40`, comparés strictement sur les mesures de chacun des bouts de la piste (`Track::parts`, un seul pour une piste non recollée) ; seuils finis et au-dessus de 0.
  - `windows` : la fenêtre d'une piste va de sa première image moins la marge à sa dernière plus la marge, bornée à la vidéo.
    Les fenêtres qui se chevauchent ou se touchent forment un groupe, décodé une fois sur son enveloppe et encodé une fois.
  - `overlay` et `picture` : sur chaque image, toutes les pistes présentes sont dessinées, une fois, et l'image dessinée est partagée par les extraits du groupe.
    Boîte de 2 px sur un point détecté, de 1 px sur un point interpolé, `#N` au-dessus, trace sur `trail_s`.
    Chaque piste a son encre (`ink`, #13) : la couleur que `cli` lui passe, celle de l'image résumé, pour la boîte, le numéro et la trace, avec un liseré `INK` de 1 px de part et d'autre de la boîte et autour du numéro ; sans couleur, c'est `BOX` et `TRAIL` sans liseré.
    Les couleurs et les tailles viennent de `draw`, et les constantes propres aux extraits restent dans `overlay`.
    `Picture::draw` prend tout ce qui implémente `Marks` (ses zones, son dessin) : l'`Overlay` des extraits et la trace du zoom.
    Les zones viennent de `zones` (paires, arrondies à 16, bornées, fusionnées), les boîtes bornées et les points interpolés de `positions`.
  - **Boîte bornée** (`overlay`, #112) : elle ne sert qu'à l'affichage ; la détection, le suivi et le CSV ne changent pas.
    Chaque côté de la boîte dessinée est borné à 4 fois la médiane du même côté sur les points détectés de la piste (médiane de deux valeurs centrales : leur moyenne).
    Une boîte bornée est recentrée sur le barycentre de la détection, puis élargie de `box_pad` comme les autres, et les points interpolés viennent des boîtes déjà bornées.
    - Le cas qui l'a motivée : la 092, piste 13, image 6945 (3:51).
      La tache, étalée, a une boîte de 339 × 235,5 px pour 353 px de surface, quand la piste a pour médiane 31,5 × 23,25 : on dessinait 363 × 259 px autour de rien, comme la v0.1.1.
      Bornée, la boîte fait 126 × 93 px avant `box_pad`.
    - Mesure sur les 966 pistes de la référence des 26 vidéos (33 865 points, `rapports/mesure-112/boxes.py`, hors dépôt) :

      | Facteur | Boîtes bornées | Pistes | Réduites de plus de 12 px |
      | --- | --- | --- | --- |
      | 3 | 129 (0,38 %) | 84 | 11 |
      | 4 (retenu) | 33 (0,10 %) | 27 | 7 |
      | 5 | 15 (0,04 %) | 12 | 6 |

      Avec 4, les 7 boîtes réduites de plus d'un `box_pad` sont : 092 piste 13 image 6945, 094 piste 7 image 4444, 096 piste 79 images 4817 à 4819, 096 piste 113 image 6538 et 100 piste 34 image 2896.
      Les 26 autres boîtes bornées perdent au plus 10,5 px (médianes de leurs pistes : de 3 à 15 px).
  - **Zoom** (`zoom`, #103) : la vue lisible d'une petite cible, `<extrait>_zoom.mp4` ; ce qu'il montre est décrit dans [02](02-methode.md), « Extrait zoomé et ralenti ».
    - Fenêtre : celle de l'extrait, mêmes images ; cadence déclarée divisée par 4 (`FrameRate::slowed`, rationnel exact : 22527500/3000927 sur la 092, quand la v0.1.1 écrivait 908336/121001).
    - Cadre fixe pour la piste, par la formule de la v0.1.1 appliquée aux **boîtes bornées** : leur enveloppe plus 40 px, en unités du PGCD de la largeur et de la hauteur (4×3 en 1440×1080), au moins le quart de l'image (×4 au plus), centré puis borné à l'image.
      Placement par `f64::round` (la v0.1.1 arrondissait au pair : un pixel d'écart sur une égalité exacte).
    - Taille de sortie : celle de la vidéo.
      Une sortie en 720×540 pour les cadres qui y tiennent (86 % des zooms), essayée le 13 octobre 2026, faisait la 099 en 36,3 s au lieu de 41,8 s, et des zooms de 260 Mio au lieu de 413.
      L'utilisateur ne l'a pas retenue : la pleine taille montre mieux les objets diffus.
    - Agrandissement au plus proche, en YUV : pixel de sortie `x` ← `gauche + x × largeur du cadre / largeur de sortie` (entiers), chroma de sortie `cx` ← chroma de la source de la luma `2·cx`.
      Les lignes répétées se recopient.
    - Trace seule, 1 px, de la couleur de la piste (#13) : centres des points détectés sur `trail_s` et position courante (boîte bornée, interpolée entre deux points), coupée dans la boîte courante élargie de 16 px (Liang-Barsky), placée à `(x − gauche + 0,5) × échelle`.
      Ni boîte ni `#N`. Affichage et conversion par zones comme les extraits.
    - Encodé en **openh264**, règle fixe ([Zooms en openh264](#zooms-en-openh264)). Un échec d'openh264 sur un zoom fait échouer la vidéo.
  - **Conversion par zones** (#112, décision de l'utilisateur) : l'image entière passe par les tables d'affichage de `Converter`, et seules les zones autour des dessins passent en RGBA, reçoivent le dessin et reviennent en YUV.
    La zone d'une marque couvre sa boîte, son `#N` (deux hauteurs de chiffre au-dessus de la ligne de base) et sa trace, plus 4 px.
    Elle est calée sur des coordonnées paires (4:2:0), sa taille est arrondie au multiple de 16 (pour le cache de contextes) et bornée à l'image, et les zones qui se touchent sont fusionnées.
    Un test vérifie, image par image, que tout pixel dessiné tombe dans une zone.
    Sur la 099, les tuiles de 16 px touchées par un dessin font 0,56 % de l'image en moyenne, 3,8 % au plus.
    `ACCURATE_RND` et `BITEXACT` sont gardés ; ils coûtent environ 2,1 ms par image entière, presque tout dans le sens YUV → RGBA.
  - **Vidéo annotée** (`--annotated`, #103) : `<nom>_annotee.mp4` (`_annotated`), toutes les images qui ont un `pts`, avec le dessin des extraits, aux `pts` de la source (en phase avec le CSV après une perte d'images).
    Elle se lit par tronçons (`Source::pieces`) : une image en erreur isole le tronçon qui va de l'image-clé d'avant à l'image-clé d'après, lu sur un fil, et le reste se lit sur tous les fils ; une image sans `pts` coupe un tronçon et n'est pas écrite.
    Un seul encodeur sert toute la vidéo, VideoToolbox d'abord, avec une image-clé toutes les `round(cadence)` images pour les deux encodeurs.
    Elle ne partage rien avec les extraits.
    Relevé (13 octobre 2026, un passage) : +19,7 s sur la 092 (18,4 → 38,1 s), +23,0 s sur la 103 (37,7 → 60,7 s).
    Quand VideoToolbox casse, elle est refaite entière en openh264, sur un fil ; tout en openh264, la 092 avec `--annotated` prend 133 s.
    Le ménage de `cli` (#115) efface la vidéo annotée des deux langues et son `.partial`, même sans `--annotated`, avant la garde `max_tracks` et l'essai d'encodeur ; au-delà de la garde, le message dit « clips and annotated video skipped » quand elle était demandée.
  - Ordonnancement (`clips`, `work`, `schedule`, `group`, `ahead`) :
    - les travaux sont la vidéo annotée, les groupes d'extraits et les zooms, chacun relu sur sa plage ; la vidéo annotée part la première (la plus longue), puis les autres par image de début, le groupe avant le zoom qui commence avec lui ;
    - les travaux partent dans l'ordre, sur la moitié des cœurs (`available_parallelism / 2`), et chaque fil garde son `Picture`, car `Converter` n'est pas `Send` ;
    - dans un groupe, un fil décode en avance (file de 4 images, cases recyclées), et l'image dessinée est poussée une fois dans l'encodeur du groupe ;
    - un groupe réserve sa session VideoToolbox (une seule) avant de partir et la garde jusqu'au bout ; l'encodeur de chaque fenêtre est celui de son groupe, le même d'un passage à l'autre ;
    - une erreur ou une panique arrête les autres groupes à l'image suivante, et les groupes qui attendent ne partent pas ; l'erreur rendue est celle du premier groupe, dans l'ordre, parmi ceux qui ont échoué avant d'être arrêtés ;
    - les extraits déjà finis restent dans le dossier quand la vidéo échoue ; la ligne d'erreur nomme la vidéo, et le lancement suivant vide le dossier.
  - Encodeurs (`encoders`) : VideoToolbox pour chaque groupe tant qu'il n'est pas écarté, sinon openh264, qui pose une image-clé par seconde environ (`with_key_interval(round(cadence))`).
    Un fichier s'écrit dans son partiel au nom unique, créé par `File::create_new` (droits habituels de l'utilisateur, umask compris), puis renommé ; il n'en reste aucun après une erreur.
  - Reprise : quand VideoToolbox casse en cours d'écriture (hors erreur de destination), le groupe entier est noté, VideoToolbox est écarté pour la suite de la vidéo, et le groupe est refait en openh264 une fois les groupes finis.
    `Rendered.redone` nomme ses extraits avec la cause, et un échec d'openh264 fait échouer la vidéo.
    Une erreur de destination (chemin refusé, temporaire non créé, fichier non ouvert ou non renommé) arrête la vidéo sans reprise ; l'erreur nomme l'extrait qu'elle concerne.
  - Tests des extraits :
    - une fausse ouverture (`testing::Fake`, un vrai `Fanout` en openh264) compte les encodeurs et les sessions, et casse à la n-ième image ou au renommage : elle exerce la reprise en openh264, l'absence de reprise après une erreur de destination et l'arrêt sur un échec d'openh264, sans `.partial` ;
    - un fil et quatre donnent les mêmes fichiers à l'octet en openh264, zooms compris ;
    - boîte qui suit la cible après un trou (`cli`, `process/tests.rs`) : vol fabriqué de 80 images, une image-clé toutes les 8, un dixième du `mdat` écrasé (8 images perdues, deux pistes de part et d'autre).
      Sur chaque point détecté, les quatre bords rouges lus dans l'image tombent à 2 px au plus de la boîte attendue, et le carré clair est dans la boîte de détection ; un dessin décalé d'une image fait échouer le test (essai fait une fois) ;
    - `slow_clips` (l'extrait, pistes de la référence, extraits de 380, 108, 115 et 117 images) : chaque image servie est celle de la lecture séquentielle, et chaque image écrite est comparée, dans l'ordre, à l'image source servie par `pts` et passée par la même conversion.
      Seule la vidéo annotée garde les `pts` de la source, comparés à ceux de la source ; les extraits et les zooms comptent leurs images à la cadence de l'encodeur (le quart pour les zooms), depuis 0.
      Le SSIM se prend sur Y par blocs 8×8 disjoints, hors blocs qui touchent un dessin (boîte, `#N` et trace, avec une marge).
      Mesuré sur le Mac : VideoToolbox 0,9659 à 0,9667, openh264 0,9671 à 0,9677, pour des seuils de 0,963 et 0,964 (le Y mesuré par le filtre `ssim` de FFmpeg contre l'original valait 0,9688 et 0,9698).
      Les couleurs relevées sur la boîte et la trace s'écrivent dans la sortie du test, sans seuil.
      Une seule lecture séquentielle de l'extrait sert les trois tests ; les `pts` et le nombre d'images de la vidéo annotée se relèvent pendant sa comparaison, sans seconde lecture, et chaque bloc se compare dans un tableau fixe de 64 paires, sans allocation.
      Le calcul de la similarité a son test rapide, `tests/similarity.rs`.
      Le binaire entier prend 76 s et 59 s sur le M1 Max, sous le verrou (217 s et 212 s de CPU), et la vidéo annotée seule 58 s ;
    - `local_clips` (`cli`, du moteur à `Source`, fils automatiques) : sur la 103, la 123 et la 124, les images-clés et les images en erreur viennent de la lecture du moteur, et chaque groupe qui touche une plage abîmée (1, 6 et 3 groupes) est servi avec les pixels d'une lecture séquentielle à un fil, en 300 s.
      Aucun n'est relu depuis le début (`Origin::Start` : 0, épinglé).
      Les fenêtres des 26 vidéos se comparent à celles des pistes de la v0.2.0 (`BATDETECT_REFERENCE_V020_DIR`, [gardes de détection](#gardes-de-détection)), après une détection de chacune.
- **`compare`** → `engine`, `media`, `tracking`, `reference`, `types` (#119) : outil de développement, binaire `compare`, non livré.
  `media` n'y sert qu'à choisir les fils du décodeur (`Threads::Automatic`) et, dans `slow_same_as`, à couper l'extrait (`payload::wipe`, #176).
  - `compare extract|092|092-1440|settings|local [--reference v0.2.0|v0.1.1] [--frames DOSSIER] [--same-as DOSSIER]` lance `engine::detection::detect` sur l'extrait, la 092, la 092 à 1440 pixels de travail (nommée par son dossier pour ne pas écraser la 092), les 16 cas de réglages ([Réglages](#réglages)) ou les 26 vidéos (`BATDETECT_VIDEOS_DIR`, 6 vidéos à la fois, résultats dans l'ordre des dossiers).
    Il compare le résultat à une référence : la [v0.2.0](#référence-v020) par défaut (`BATDETECT_REFERENCE_V020_DIR` pour `local`), ou la v0.1.1 (`--reference v0.1.1`, `BATDETECT_REFERENCE_DIR`).
    Les réglages de détection sont ceux de l'enregistrement : `config.DetectConfig` du `video.json`, lu par `VideoSummary::detect_config` en `DetectSettings`, puis `DetectConfig::try_from`.
    Pistes et stabilité restent aux défauts, comme à l'enregistrement.
    On le lance depuis la racine du dépôt, sous le verrou : `verrou lourd cargo run --profile ci -p compare --bin compare -- local`.
  - La règle est la même pour les deux références (#275) :
    - **Entrées.**
      Le nombre d'images, les `pts` et les images-clés de la lecture courante doivent être ceux de `frames.jsonl` de la référence, et, contre la v0.2.0, les pertes aux bords celles de `decoder.losses` (la v0.1.1 ne les a pas).
      Sinon la vidéo échoue (`Error::Inputs`, première image et champs qui diffèrent), et rien d'autre ne se compare.
      Les erreurs de décodage ne sont pas exigées, puisque le décodeur de lumiel en voit 14 de plus que celui du Mac (voir `engine`, « Dégâts contre la référence ») ; elles sont comptées de chaque côté, avec les images où elles diffèrent (`inputs::DecodeErrors`).
    - **Plages abîmées.**
      Les images en erreur, les sauts et les pertes aux bords des deux côtés (`decoder` de la v0.2.0, `ffprobe` de la v0.1.1, `Detected::damage` de la lecture courante) sont réunis, puis tracés en plages par la règle courante (`Damage::reported`, marge = demi-fenêtre de la référence ; `damaged::joined`).
      Une plage que la règle change (#140) se voit donc dans les pistes.
    - **Détections**, hors de la réunion des plages abîmées de la référence et des plages retracées, fondue avant d'être comptée (`span::merged`), parce que dans les plages de la référence ses détections viennent de pixels dissimulés.
      D'abord l'égalité exacte, image par image : nombre d'images qui diffèrent, dix au plus listées.
      Puis l'appariement à moins de 3 px (`reference::detection_matching`) : paires, perdues, ajoutées, taux dans les deux sens.
      Les plages instables ne sont pas écartées : elles se déduisent des détections, qu'on compare brutes.
    - **Pistes** : `exclude` avec les plages retracées et le nombre d'images signalées de la réunion, puis `track_detections` aux réglages par défaut, puis `match_tracks` (début à ±5 images, points à ±3).
    - Il n'y a pas de seuil : un écart se lit dans le rapport, et les gardes figent les leurs par vidéo (`Comparison::gaps` pour la détection, #276 ; #277 pour le suivi).
  - Sortie : par vidéo, les entrées, les détections et les pistes ; puis les écarts de détections groupés par vidéo et par temps (30 images entre deux écarts au plus), les écarts de pistes, le temps de détection de chaque vidéo, et enfin `differing from the reference: N` avec les vidéos qui ont un écart.
    Avec `local`, les temps sont pris pendant que 5 autres vidéos tournent : seul `compare 092` chronomètre la détection seule.
    Une vidéo en échec (vidéo absente, demi-fenêtre autre que celle de la référence, entrées différentes…) est écrite sur la sortie d'erreur avec ses causes, sans effacer le rapport des autres.
    Le code de sortie vaut 1 si une vidéo échoue, entrées comprises, si une vidéo diffère du passage de `--same-as`, ou si aucune vidéo n'est comparée (`Outcome::holds`) ; jamais pour un écart à la référence, sans quoi `compare` resterait rouge dès la première PR de détection.
    Avec `settings`, `--frames` écrit dans un dossier par cas (`DOSSIER/<cas>/frames.jsonl`), les cas partageant leur vidéo.
  - `--frames` écrit `DOSSIER/<vidéo>/frames.jsonl` au format de la référence (`frames::frames_of`, puis `reference::frames::Frames::write`) : `pts`, image-clé et erreur de décodage vus par le décodeur de `media` (par ffprobe dans la référence v0.1.1), détections brutes avant exclusion.
    Sur l'extrait, `pts`, images-clés et erreurs sont égaux à ceux de la référence sur les 1 230 images.
  - `--same-as DOSSIER` (#176) dit si ce passage donne les mêmes détections qu'un passage précédent écrit par `--frames DOSSIER`, hors des plages abîmées.
    Les pixels que FFmpeg dissimule dans ces plages varient d'une lecture à l'autre : deux passages de `main` y différaient sur 17 vidéos et 2 351 images (#159), si bien que `cmp -r` entre deux dossiers `--frames` ne prouve rien.
    - Avant la détection, `DOSSIER/<vidéo>/frames.jsonl` est relu ; absent ou illisible, la vidéo échoue sans être détectée.
    - Après la détection, les entrées sont d'abord comparées (nombre d'images, puis `pts`, image-clé et erreur de décodage de chaque image) ; si elles diffèrent, les détections ne sont pas comparées.
    - Sinon, les détections le sont image par image, à l'égalité exacte, hors des plages abîmées que le moteur vient de trouver (`Detected::damage`, marge comprise).
      La comparaison se fait en direct plutôt qu'entre deux dossiers parce que `frames.jsonl` ne garde pas les pertes avant la première image et après la dernière, qui ouvrent aussi une plage.
      Les plages sont donc celles du seul nouveau passage : des pertes au bord vues par l'ancien seul donnent « differs », du côté prudent ; vues par le nouveau seul, elles écartent des images que l'ancien ne jugeait pas abîmées.
      Les plages instables ne sont pas écartées : elles se déduisent des détections, que l'on compare brutes.
    - Sortie : une section à la fin du rapport, `same as DOSSIER, outside the damaged spans`, puis une ligne par vidéo, `identical`, `differs` (images hors plage, dix au plus puis `… and N more`) ou `inputs differ` (première image et champs qui diffèrent, ou nombres d'images de chaque côté).
      Chaque ligne donne ensuite le nombre de plages, leurs images et les images qui diffèrent dedans ; enfin vient le bilan, `N videos: identical …, differing …, inputs differ …`.
    - Le test lent `slow_same_as` coupe un vingtième de l'extrait par `media::payload::wipe`, puis le détecte à un fil et en fils automatiques : le résultat est identique hors de sa plage.
      Sur lumiel, la plage couvre les images 601 à 644, avec 27 images différentes dedans, un nombre que le test ne demande pas parce qu'il varie d'une lecture et d'une machine à l'autre ; une détection déplacée hors plage est listée avec sa vidéo et son image.
      Le test exige que les fils automatiques décodent par images (`ActiveThreads::Frame`), sans quoi ses deux passages liraient la vidéo de la même façon.
      `slow_extract` relance l'extrait avec `--same-as` sur le dossier qu'il vient d'écrire, par le chemin de la commande.
  - `record extract|092|092-1440|settings|local --out DOSSIER [--same-as DOSSIER]` (#250), second binaire de la crate, enregistre la [référence v0.2.0](#référence-v020).
    Pour chaque cas, il relit le `video.json` de la v0.1.1, détecte avec ses réglages en fils automatiques et vérifie que le flux lu par `media` est celui de la v0.1.1 (taille, cadence, base de temps).
    Il suit ensuite comme `cli::process::tracked` : `exclude` avec les plages abîmées du décodeur courant, puis `track_detections`, aux réglages par défaut.
    Il écrit enfin un enregistrement au format 2 dans `DOSSIER`, avec la même arborescence que la v0.1.1 (`<cas>/`, `settings/<cas>/`, `data/<vidéo>/`, `Case::folder`).
    - `video.json` (`compare::decoded`) : `video` et `stream` recopiés de la v0.1.1 (`VideoSummary::decoded`) ; `decoder` donne les images en erreur, les sauts et les pertes de `Damage` et de `Reading::losses`, ainsi que les fils actifs.
      Les plages abîmées sont exactes, marge comprise ; `config` contient les réglages avec lesquels le moteur a tourné, `versions` le binaire, le commit, rustc, FFmpeg et le système.
      Pas de `tracks.csv`.
    - Provenance : le script de construction de `cli` (`crates/cli/build/engraved.rs`), que `compare` reprend par la clé `build` de son `Cargo.toml` (#295), grave `git describe --dirty`, le commit, `rustc --version` et ce qu'il tire de `git status` sur les chemins surveillés : `clean`, `changed`, ou rien si `git status` n'a pas pu tourner (#275).
      Un fichier non suivi sous `crates/`, `.cargo/`, `packaging/` ou `tests/` rend aussi le binaire `-dirty`, parce que `git describe` ne les voit pas, alors que Cargo prend tout seul un `build.rs` non suivi et que `.cargo/config.toml` choisit le FFmpeg lié.
      `record` refuse de tourner si le binaire est `-dirty`, si `git status` n'a pas pu tourner, s'il a été construit hors de git, sans rustc ou après aucun tag de version (`provenance::Build::engraved`).
      La référence prend le nom de ce tag (`v0.2.0`), préversion comprise : `v0.3.0-rc1-4-g28d5526` donne `v0.3.0-rc1`, pas `v0.3.0` (#275).
      L'enregistrement se fait donc depuis un clone propre du commit, et toujours par `cargo run` : les fixtures de la v0.1.1 sont lues à l'exécution, et seul `cargo run` reconstruit, donc marque `-dirty`, un binaire dont une fixture a changé.
      Le script est seul dans son dossier `build/` parce que `cargo shear` prend le dossier du script pour la racine des sources de `compare`, et il n'est pas partagé par `#[path]`, parce que `cargo fmt` suivrait le `mod tests` du module (#295).
    - Le flux est vérifié à l'ouverture de la vidéo, avant toute détection, puis la demi-fenêtre du fond après la détection (`Error::HalfWindow`, comme `compare`).
      Avant toute détection aussi, `--out` est refusé s'il est dans `tests/fixtures/`, dans le dossier de `BATDETECT_REFERENCE_DIR` ou de `BATDETECT_REFERENCE_V020_DIR`, dans le dossier de `--same-as`, ou s'il contient l'un d'eux : l'arborescence étant la même, `record` écraserait la référence (`recorder::refuse_overlap`).
      Le refus ne crée pas `--out` : le chemin se résout depuis son plus proche parent existant, `..` compris (#275), et les dossiers ne se créent qu'à l'écriture de chaque cas.
      La v0.2.0 s'enregistre donc ailleurs, puis se copie.
    - `--same-as DOSSIER` compare les détections à celles d'un passage `compare --frames DOSSIER` (`DOSSIER/<vidéo>/frames.jsonl`, à plat), comme `compare --same-as`, et non à un dossier écrit par `record`.
      Un écart, une vidéo en échec ou aucun cas enregistré donnent le code 1 (`recorder::holds`).
    - Le suivi est celui de `cli::process::tracked` aux réglages par défaut : un test de `cli` exige les mêmes exclusions et les mêmes pistes sur `spot.mp4`.
      Le test lent `slow_record` enregistre l'extrait et exige au bit près les détections de `tests/fixtures/rust/`.
  - **Identité entre systèmes.**
    Une seule opération de la chaîne de détection (`engine`, et `WorkScale` dans `types`) dépend de la libm : `exp`, qui calcule le noyau du flou une fois par vidéo.
    - Pour chaque σ de travail du banc (0,5, 0,75, 1, 1,5 et 2 ; 0 n'appelle pas `exp`) et pour 3, un test vérifie que chaque part du noyau est à plus de 10⁻⁶ d'un entier et que la répartition des restes ne bascule pas pour 10⁻⁶ : un ulp d'écart ne change aucun poids.
    - Hors de ces σ, rien n'est garanti.
      Sur 99 901 σ de 0,01 à 10, aucun poids ne change quand chaque `exp` varie au hasard de −1 à +2 ulp (8 tirages par σ, relecture de #150) ; ce n'est pas une preuve.
    - Le reste de la chaîne n'emploie que les quatre opérations, `abs`, `midpoint`, `min`, `max`, `round`, les conversions et le tri par `total_cmp`, toutes exactes ou fixées bit à bit par IEEE 754.
      La seule exception, `max` entre +0 et −0, n'arrive pas : on compare toujours au plancher du seuil ou à un pic qui le dépasse.
    - `area_to_source` multiplie par `échelle × échelle`, et non par `powi(2)`, dont Rust ne fixe pas la précision ; Rust ne contracte jamais a × b + c en FMA.
    - Le décodage H.264 est exact au bit près par la norme, sauf dans les images dissimulées, et l'extrait n'en a aucune.
    - Hors de la chaîne, `hypot` (distance de l'appariement, association du suivi) et `atan2` (virage médian) viennent de la libm.
      Un écart d'un ulp ne ferait basculer qu'une paire à 3 px tout juste, ou un choix entre deux distances égales, et il ne touche pas `frames.jsonl`.
  - **Sauts de luminosité** (`tests/slow_brightness_jump.rs`, les 3 tests de stabilité de la v0.1.1) :
    - une vidéo de 90 images 1440×1080 à 30 i/s (gris 200, bruit σ 3, saut à l'image 45), encodée par l'openh264 de `media`, passe par `detect`, `exclude` puis `track_detections`.
      Le test passe par une vidéo plutôt que par une entrée directe du détecteur, pour n'ouvrir aucune API nouvelle dans `engine` ;
    - il est au niveau `slow` : 3,3 s en profil `ci`, mais 368,8 s sous l'instrumentation de la couverture, qui ralentit la détection ;
    - un saut de 20, 30 ou 40 niveaux ne donne aucune piste.
      Un saut de 30 donne une plage instable, [9, 80], autour du saut, et le test vérifie qu'elle laisse des images de chaque côté ;
    - un saut de 20 n'allume aucun pixel, alors que la v0.1.1, à l'échelle 3 et sans filtre, y voyait quelques taches ;
    - la bande où un saut donne des taches sans plage instable n'a ici qu'un niveau de large, 28 (22 détections) ; 29 en donne 1 617, avec la plage [9, 80].
      Le test du saut modéré garde donc 20, et vérifie que rien n'est allumé ni ignoré ;
    - le saut de 30 n'est qu'à un niveau du bord : si openh264 n'encode pas la même chose sur un autre système, ce test le montrera, et l'écart remonte alors à l'utilisateur.
- **`bench`** → `engine`, `tracking`, `reference`, `media`, `types` (#107 à #109) : le banc de mesure, bibliothèque et binaire `batdetect-bench`, sans dépendance de plus au workspace.
  Sa commande et sa méthode sont dans [08](08-banc-de-mesure.md).
  - `random` : SplitMix64 et xoshiro256\*\*, écrits dans le dépôt et épinglés sur les sorties publiées par leurs auteurs.
    Chaque cible a sa graine, tirée de `--seed` dans l'ordre des numéros, comme `SeedSequence.spawn` de numpy : le tirage d'une cible ne dépend pas du nombre d'essais des précédentes.
    `uniform(a, b)` rend une valeur de `[a, b]` (l'arrondi peut rendre `b`), et `below(n)` tire par rejet, sans biais.
  - `trajectory` (saccades, Bézier quadratique, transit), `hunting` (virages, lignes droites, entrée par un bord, chasse et cercle) et `sampling` (tirage loin des pistes de référence et des cibles déjà placées, 500 essais) portent le calcul de la v0.1.1, pas l'ordre de ses tirages (règle 5), avec les mêmes constantes.
  - Le tirage passe par `exp`, `atan2`, `powf`, `hypot`, `sin` et `cos` de la bibliothèque mathématique du système : un ulp d'écart peut changer un arrondi ou un rejet, puis tout ce qui suit.
    Une graine n'est donc reproductible que sur un même système ; aucun tirage n'est épinglé en CI, et les tests épinglent des propriétés.
  - `injection` : sur le plan Y, avant la détection, par `detect_injected`. La fenêtre `[⌊x⌋ − ⌈3σ⌉, ⌊x⌋ + ⌈3σ⌉]` est bornée à l'image.
    Pour chaque pixel, le gris visé est `gris(Y) + ⌊a·exp(−d²/2σ²)⌋` borné à [0, 255], puis Y devient le Y dont le gris est le plus proche (`nearest_luma`, table calculée à la compilation).
    La table `GRAY` saute un gris sur six environ, et un gris sauté se rend alternativement un niveau plus sombre et un niveau plus clair (décision de #107).
    Mesure de `local_injection` sur les 986 patchs enregistrés (471 046 pixels, fonds Y de 80 à 215) : partie entière de la gaussienne identique à la v0.1.1 sur 100 % des pixels, changement de gris à un niveau près sur 100 %, biais moyen de +0,0010 niveau (−0,1558 en allant toujours vers le Y le plus sombre, +0,1558 vers le plus clair).
    Le contraste effectif est le changement de gris réalisé, un entier (la v0.1.1 enregistrait le changement non arrondi).
  - Appariement et métriques (`matching`, `metrics`, `stats`) sont ceux de la v0.1.1, décrits dans [08](08-banc-de-mesure.md), calculés sans arrondi (règle 5).
    La visibilité écarte en plus une cible dont le centre tombe dans une zone OSD (`Region::contains`, bornes comprises), et les images des plages ignorées du passage injecté ; à égalité, une piste va à la première cible rencontrée.
  - Un passage écrit `run.json`, au format propre au Rust (vidéo, taille, cadence exacte, plages, images signalées, demi-fenêtre, réglages de détection, provenance des cibles), à côté de `reference.jsonl.gz`, `injected.jsonl.gz` et `bats.jsonl.gz`, aux formats de la v0.1.1.
  - `report` : `bench.json` aux clés de la v0.1.1, plus `targets` (tirées, graine ; ou rejouées, chemin) et `seconds` (détections, évaluation, total).
    `sampling` vaut `null` pour des cibles rejouées, et `code` est le `git describe --always --dirty` du dossier courant, ou `unknown`.
  - `guard`, `against` : la [garde](#garde) contre un banc de la v0.1.1.
  - Commande : `batdetect-bench VIDÉO [-o DOSSIER]` avec les options de la v0.1.1 (`--per-class`, `--amplitudes`, `--sigmas`, `--motions`, `--seed`, `--match-radius`, les dix de détection, `--max-blobs`, `--unstable-pad`, les six de suivi), sans `--mode` ni `--workers`, plus `--targets FICHIER`, `--replay DOSSIER` et `--against DOSSIER`.
    La sortie va dans `<-o>/<nom de la vidéo>/` (défaut `out/bench`).
    - `--targets` rejoue positions et amplitudes ; l'évaluation prend le contraste mesuré par le Rust.
      Il est refusé avec une option de tirage ou `--against`.
    - `--replay` réévalue un dossier écrit par le banc, sans détection, avec d'autres réglages de suivi, de stabilité ou d'appariement ; la détection et les cibles viennent de `run.json`, et leurs options sont refusées.
      Il écrit `bench.json` directement dans `-o`.
      Avec la vidéo telle qu'elle a été tapée, `run.json` garde le chemin relatif au dossier de lancement : relu d'ailleurs, `video` ne situe plus le fichier.
      La demi-fenêtre y est gardée pour information, puisque les plages abîmées sont déjà élargies.
    - Une sortie qui écraserait l'entrée (`--replay` dans le dossier rejoué, `--targets` dont le fichier est dans le dossier de sortie) est refusée avant toute détection.
    - `--against` écrit dans `<-o>/<vidéo>_<dossier enregistré>/`, pour que transit et chasse ne s'écrasent pas.
    - Les nombres négatifs sont acceptés derrière une option ; une vidéo placée après `--amplitudes` serait lue comme une amplitude (comme avec argparse), et se met donc avant.
      Les erreurs de réglage nomment l'option (`--bg-window`, pas `bg_window_s`).
    - Code 0, 1 en cas d'échec ou de garde en échec, 2 pour une erreur d'usage.
  - Tests : une vidéo de 96 × 72 et 60 images fabriquée par openh264 (fond bruité), avec une cible lente rejouée (−60, σ 3), trouvée ; les cibles tirées n'y seraient presque jamais trouvées, et le tirage se teste par ses propriétés.
    `tests/command.rs` lance le binaire (cibles rejouées, tirées, rejeu, garde contre son propre passage).
    La vidéo et la cible lente viennent de `bench::noisy`, derrière la feature `testing` du banc (#154) : `bench` dépend de lui-même en dev-dependency avec cette feature, pour que `tests/command.rs` les voie aussi.
    `local_injection` et `local_guard` lisent `BATDETECT_REFERENCE_DIR/bench/{transit,hunt}`.
    `committed_metrics` (#277) relit chaque passage v0.2.0 du dépôt par `folder::read`, le réévalue par `evaluate(&run, Judging::default())`, comme `--replay`, et exige le `bench.json` enregistré : entiers égaux, flottants à 1e−9 près (`matching.rs` passe par `hypot`), durées exclues.
    Les réglages track, stability et match de `bench.json` doivent donc être ceux de `Judging::default()`, si bien qu'un changement des défauts ne déplace pas la base sans le dire.
    Ce qui vient de l'évaluation se fige par banc dans `FROZEN` (`Metrics` : non visibles, fausses pistes, pistes des deux passages, cibles évaluées et trouvées, cibles et classes changées par rapport au `bench.json`, ces deux dernières à 0 à l'enregistrement), et les lignes changées s'affichent ; l'effet actuel est décrit à `tracking`.
    Une PR de suivi met à jour `FROZEN`, jamais la référence.

## Réglages

Le banc d'essai du moteur hors des réglages par défaut compte 16 cas, chacun une référence à d'autres réglages de détection, que `compare settings` relit (#149).
Tout réglage valide se compare.

### Vidéo 720×540

`tests/fixtures/video_092_original_3m24-4m05_720x540.mp4` (44 336 736 octets, sous LFS) est l'extrait réduit de moitié, fait une fois le 12 octobre 2026 par le ffmpeg 9.0.2 de Homebrew (libx264), sous le verrou :

```bash
ffmpeg -i tests/fixtures/video_092_original_3m24-4m05.mp4 -map 0:v:0 -an -sn -dn -map_metadata -1 \
  -vf scale=720:540:flags=area -c:v libx264 -preset slow -crf 16 -g 15 -keyint_min 15 \
  -sc_threshold 0 -bf 0 -threads 1 -pix_fmt yuv420p -color_range tv -colorspace bt709 \
  -color_primaries bt709 -color_trc bt709 -fps_mode passthrough -video_track_timescale 10000 \
  tests/fixtures/video_092_original_3m24-4m05_720x540.mp4
```

- Mêmes 1 230 images, même cadence (10000/333), même base de temps (1/10 000), mêmes images-clés (toutes les 15 images), sans image B, plage `tv` et BT.709 comme l'extrait.
- Elle ne se refait pas : un autre encodeur, ou le même sur un autre système, ne donnerait pas les mêmes images, et la référence ne vaudrait plus.
  Les tâches CI `rust-slow` et `rust-settings` la tirent avec l'extrait ; `rust-windows-slow` ne tire que l'extrait.
- CRF 16 garde le bruit du capteur : 44 Mo, contre 60 Mo pour l'extrait en 1440×1080.

### Les 16 cas

Les références v0.1.1 sont dans `tests/fixtures/reference/settings/<cas>/` (1,6 Mo en tout : `frames.jsonl.gz`, `tracks.jsonl`, `tracks.csv`, `video.json`).
Elles ont été enregistrées le 12 octobre 2026 par `tools/reference_v011.py data <vidéo> <dossier> --gzip --detect '<JSON>'` (script au tag `python-final`), au code du tag `v0.1.1`, un cas par appel sous le verrou, avec les mêmes versions que la référence de #94.
Les références v0.2.0 des mêmes cas sont dans `tests/fixtures/reference-v0.2.0/` ([Référence v0.2.0](#référence-v020)).
Tous les cas portent sur l'extrait, sauf le dernier.

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

- `max_area` vaut 400 et non 500 : la plus grande tache de l'extrait fait 488,25 px², et la v0.1.1 rendait à 500 les détections des défauts octet pour octet (décision de l'utilisateur).
- `sigma_0` est le seul cas où la v0.1.1 trouve des plages instables (2) : sans flou, le bruit du capteur passe le seuil par endroits.
- `crates/compare/tests/settings_agreement.rs` exige les 16 cas comparés sans échec, avec leurs écarts figés à la référence v0.2.0 ([Gardes de détection](#gardes-de-détection)).
  `crates/reference/tests/committed_settings.rs` relit les 16 cas en test rapide.

### La 092 à 1440 pixels de travail

La référence `tests/fixtures/reference/video_092_original_width_1440/` (`--detect '{"work_width": 1440}'`) se compare par `compare 092-1440` et par le niveau `reference`, dans `cli/tests/reference_092.rs`.
Dans `rust:reference`, les deux passages de la 092 tournent en même temps dans le même binaire, et leurs temps de détection s'y mélangent.

## Banc de mesure

La commande et la méthode du banc sont dans [08](08-banc-de-mesure.md), et le détail de la crate dans `bench` ([Crates](#crates)).
`bench.json` note la version du code (`code`, `git describe --always --dirty` du dossier courant).
Cette section décrit la garde contre les bancs de la v0.1.1.

### Garde

`batdetect-bench VIDÉO --against DOSSIER`, et le test `local_guard`, qui appelle la même fonction `bench::against::against`, rejouent un banc enregistré par la v0.1.1 :

- `DOSSIER` contient `bench.json` et `bats.jsonl.gz` (`$BATDETECT_REFERENCE_DIR/bench/` `transit` ou `hunt`).
  Un banc fait à d'autres réglages de détection, de suivi, de stabilité ou d'appariement est refusé, le réglage nommé.
- Un banc d'une autre vidéo est refusé (#198) : le nom de fichier de `video` dans `bench.json`, sans son dossier, doit être celui de `VIDÉO`, vérifié avant toute détection, et le nombre d'images d'`injected.jsonl.gz` doit être celui que décode le Rust, vérifié après les deux détections.
  On lit `injected.jsonl.gz` plutôt que `bench-run.json`, parce que seul le premier existe aussi dans un banc écrit par le Rust.
- Les cibles sont les mêmes des deux côtés : positions et amplitudes de `bats.jsonl.gz`, placées par le numéro d'image du décodeur de `media` (identique à celui d'OpenCV sur la 092 ; une vidéo où ils diffèrent décalerait les cibles).
  Le Rust injecte et détecte.
- Le passage Rust est évalué deux fois. `bench.json` et le tableau prennent le contraste mesuré par le Rust, comme tout passage.
  La garde prend le contraste enregistré par la v0.1.1, qui décide donc de la visibilité des deux côtés ; seules les plages ignorées propres à chaque passage injecté peuvent encore la changer.
- Par classe, sur les cibles visibles des deux côtés : leur nombre, les trouvées de chaque côté, la borne basse de Wilson de la v0.1.1 recalculée sur ces cibles (elle diffère du `found_ci95` de son `bench.json` dès qu'une cible n'est visible que d'un côté), les cibles perdues et gagnées (numéro et image de début).
  À part : les cibles visibles d'un seul côté, celles dont la visibilité change avec le contraste mesuré par le Rust, les fausses pistes et les pistes de référence de chaque côté (14 et 14 sur la 092 : un nombre différent dirait une autre vidéo).
- **Échec automatique** (code 1) seulement si la proportion trouvée par le Rust tombe sous cette borne basse ; sinon la liste est jugée par l'utilisateur.
  Une classe dont toutes les cibles quittent le dénominateur ne peut pas échouer : elle n'apparaît que dans la liste des cibles visibles d'un seul côté.
- Sortie : `<-o>/<vidéo>_<dossier>/`, avec le passage, `bench.json` et `guard.txt`, le même texte que la sortie standard.
- `--replay` d'un passage tiré avec un `--match-radius` plus grand que celui du tirage rompt l'écart de 3 rayons entre cibles sur lequel repose la règle d'égalité de `matching`, et rien ne le refuse.

Le 15 octobre 2026 (f364698, M1 Max, `release`, la 092 du dépôt), la garde passait sur les deux bancs.
Au transit, 253 cibles visibles des deux côtés, 148 trouvées de chaque côté, aucune perdue ni gagnée.
À la chasse, 584 cibles, 456 trouvées par la v0.1.1 et 455 par le Rust : perdues la 19 (image 3 221) et la 543 (image 6 935), gagnée la 555 (image 7 948), et visibilité changée par le contraste du Rust pour les 84, 120, 341, 602 et 665.
De chaque côté, 14 pistes de référence et aucune fausse piste. L'évaluation seule prend moins de 10 ms : un `--replay` est immédiat.

## Référence v0.1.1

La référence de la v0.1.1, enregistrée par le Python, reste lue par des tests : son format, son emplacement et ses versions sont dans le README de `tests/fixtures/reference/` (#94).
Elle ne se réenregistre que depuis le tag `python-final` (`tools/reference_v011.py`, qui exige `src/` identique à `v0.1.1`).
La crate `reference` la lit, et le test local `crates/reference/tests/local_reference.rs` relit les 26 vidéos (966 pistes) et les apparie chacune à elle-même.
Les 16 cas de réglages sont dans `tests/fixtures/reference/settings/` ([Réglages](#réglages)), relus par `crates/reference/tests/committed_settings.rs`.

Attendus de l'image résumé : `tools/summary_v011.py` (au tag `python-final`) relit `tracks.jsonl` et `video.json` d'un enregistrement, sans relancer la détection, avec la même garde que `tools/reference_v011.py`.
Tranches, couleurs et horloges viennent des fonctions de la v0.1.1 (`split_by_period`, `assign_colors`, `format_clock`, `format_duration`) ; l'en-tête est recomposé en lignes structurées selon `header_lines`, sans les textes.
Ceux de l'extrait et de la 092 sont dans `crates/summary/tests/fixtures/`, ceux des 26 vidéos dans `summary/` du dossier de référence local, à côté de `data/`.

Le binaire de la v0.2.0 retrouvait 963 des 966 pistes de la v0.1.1 sur les 26 vidéos (958 concordantes), en perdait 3 et en ajoutait 9 (`compare local`, mesuré le 12 octobre 2026 puis le 15 octobre sur f364698 ; 089 à 98,19 % côté référence et 98,46 % côté Rust, aucune vidéo sous 98 %).
Ces 17 écarts tombaient sur 9 endroits :

- 094 1:42 : perdue, revient 26 images plus tard ;
- 097 0:20 : perdue, revient 7 images plus tard ;
- 097 2:42 : 24 points au lieu de 86, et une piste ajoutée à 2:43 ;
- 098 0:05 : ajoutée ;
- 098 0:41 : perdue ;
- 099 de 0:14 à 0:19 : trois écarts de points et une piste ajoutée ;
- 099 0:27 : ajoutée ;
- 100 1:36 : deux pistes ajoutées ;
- 126 1:31 : 27 points au lieu de 32, et une piste ajoutée à 1:32.

## Référence v0.2.0

Ce que la v0.2.0 en Rust détecte et suit, enregistré par `record` (#250) le 17 octobre 2026 sur lumiel, depuis un clone propre du commit 1e4cf6b.
Format, emplacements, provenance, preuve et recette sont dans le README de `tests/fixtures/reference-v0.2.0/`.
Dans le dépôt : l'extrait, la 092, la 1440, les 16 réglages et les deux bancs de la 092 (5,5 Mo).
Hors dépôt : les 26 vidéos (`BATDETECT_REFERENCE_V020_DIR`, 31 Mo, préfixe `reference-v0.2.0/` du seau, `rust:data`).

- Preuve par le binaire de `v0.2.0`, construit à part :
  - les 19 cas du dépôt ont les mêmes `frames.jsonl` à l'octet ;
  - les 26 vidéos ont les mêmes entrées et les mêmes détections hors plages abîmées (`--same-as`) ;
  - les pistes de l'extrait et de la 092 sont celles du CSV de `batdetect` de `v0.2.0`, par valeurs ;
  - les deux bancs donnent les mêmes fichiers, `bench.json` par valeurs hors `code` et durées.
- Elle ne se reproduit que hors des plages abîmées (#176), et `SHA256SUMS` change à chaque enregistrement des 26 vidéos.
- Les gardes de `compare` (#275), de la détection (#276) et du suivi (#277 : `committed_tracking`, `committed_exclusion`, `local_tracking`, `local_exclusion`, `committed_metrics`) portent sur elle.

### Gardes de détection

Chaque garde compare à la v0.2.0 par la règle de `compare` et fige ses écarts **par vidéo et par cas** (`Comparison::gaps`, #276) : images qui diffèrent, détections perdues et ajoutées, puis pistes appariées, concordantes, perdues et ajoutées.
Les détections faibles encadrées ([10](10-logique-de-detection.md#5-ter-garder-les-taches-faibles-qui-comblent-un-vol), #21) ne font qu'ajouter des détections, et aucune n'est perdue : `Gaps::added_only(images, ajoutées, [appariées, concordantes, perdues, ajoutées])`, mesuré sur lumiel le 17 octobre 2026.
Avant #21, ils valaient tous 0 sur lumiel (`Gaps::agreeing(n)`).
Une PR qui change la détection les met à jour avec leur cause ([05](05-outillage.md)).
Les gardes `local_*` lisent la v0.2.0 des 26 vidéos par `BATDETECT_REFERENCE_V020_DIR`, les vidéos par `BATDETECT_VIDEOS_DIR`, et `local_command` lit les zooms et la 103 annotée de la v0.1.1 par `BATDETECT_REFERENCE_DIR`.

- `compare/tests/slow_extract` : l'extrait, 6 détections faibles ajoutées et ses 4 pistes, plus le bit près de `tests/fixtures/rust/` et `--same-as`.
- `compare/tests/settings_agreement` : les 16 cas.
  Les détections faibles en ajoutent 755 en tout, dont 664 sans filtre (`sigma_0`) ; ailleurs, elles ajoutent au plus 2 pistes (`width_720`), une perdue et une ajoutée à `sigma_3`, une ajoutée à `max_area_400`, et `threshold_25` ne change pas.
  La portée (#326) change deux cas : `sigma_0` a 15 pistes appariées, 11 concordantes, 15 perdues et 9 ajoutées (les 4 chauves-souris de l'extrait restent, le bruit perd des pistes, comme dans `committed_tracking`), et `width_720` n'a plus que 2 pistes concordantes sur 5, les deux jumelles de la chauve-souris des images 705-722 se la partageant autrement.
  L'écart de `sigma_0` et `window_2s` à la v0.1.1 dans la bande d'affichage s'explique par la table `GRAY` (voir `engine`, « Gris de la détection »).
- `compare/tests/local_agreement` : 26 lignes figées, une par vidéo, 972 pistes.
  Les détections faibles en ajoutent 236 622 (155 968 sur la 103), et aucune n'est perdue.
  Pistes : 842 appariées, 694 concordantes, 130 perdues et 79 ajoutées, effet de #21, du recollage (#24) et de la portée (#326) ensemble, qui se remesure sur l'arbre qui les a tous et ne s'additionne pas.
  Une piste de la v0.2.0 réunie à une autre par le recollage compte pour perdue, puisqu'une piste ne s'apparie qu'à une seule.
  Le recollage fait 81 réunions sur les 26 vidéos : 67 sur les seules détections de la v0.2.0, 13 de plus qui réunissent des bouts que les détections faibles changent (cause non classée une à une), et une de plus avec la portée.
  La portée coupe des pistes qui sautaient vers une autre tache, dont les bouts comptent comme perdus ou ajoutés.
  Les endroits changés ont été jugés à l'œil ([10](10-logique-de-detection.md#7-relier-les-taches-dans-le-temps--le-suivi)), et la 092 ne bouge pas.
- `cli/tests/reference_092` :
  - la 092 : 14 pistes appariées et concordantes, 20 détections faibles ajoutées sur 19 images ;
  - la 092 à 1440 pixels : 9 détections faibles ajoutées, 14 pistes appariées et concordantes.
    Un second morceau du passage de 1:48.93, à 27-36 px de lui, atteindrait 6 points grâce à la détection faible de 1:49.83 (image 3298) et ferait une piste de plus ; avec la portée, cette détection est trop loin pour lui, et il s'arrête à 2 points pour se fondre dans la piste 6 comme une jumelle ;
  - la commande sur la 092 : 14 pistes appariées et concordantes, sans écart, ses 15 extraits figés et son zoom ;
  - les faits de la naturaliste, seuls affirmés sans référence : les 13 chauves-souris confirmées le 9 octobre ([03](03-resultats-092.md)) ont chacune leur piste, qui commence à ±5 images de leur début (temps aux `pts` de la lecture du calcul).
    À côté d'elles, le test tolère au plus une piste à chacun de ces deux endroits : l'objet lent de 3:29, qui n'en est pas une, et la cible connue de 3:42.10, objet réel manqué aujourd'hui par le réglage par défaut et pas encore vu par la naturaliste ([03](03-resultats-092.md#cibles-connues-pour-les-issues-suivantes)).
    Toute autre piste nouvelle fait échouer le test : c'est une question pour elle.
    Le nombre de pistes n'est affirmé que par les écarts figés, parce qu'un recollage peut passer de 14 à 13 pistes sans perdre un passage.
- `cli/tests/slow_command` : les 4 pistes de l'extrait, sans écart ; la colonne des détections du CSV est celle de `tests/fixtures/rust/…/tracks.csv`, en-tête exclu.
- `cli/tests/local_command` : les pistes de la commande sur les 26 vidéos, une ligne figée par vidéo (`[référence, appariées, concordantes, perdues, ajoutées]`, les mêmes que `local_agreement`), où une piste recollée est comparée à tous ses bouts.
  Le test compare le zoom de chaque piste à celui de toutes les pistes de la v0.2.0 qu'elle contient (premier point compris dans les siens), et exige que chaque piste de la v0.2.0 soit contenue dans une piste et une seule.
  Deux listes figées tiennent les exceptions.
  - `UNHELD`, les pistes de la v0.2.0 dont aucune piste de la commande ne tient le premier point, en a 11.
    Quatre étaient déjà perdues avec les seules détections faibles, sans recollage : 090 #14 et #17, 096 #44, 097 #4.
    Sept viennent de la portée, à des endroits jugés à l'œil ([10](10-logique-de-detection.md#7-relier-les-taches-dans-le-temps--le-suivi)) :
    - 090 #5 (2:09.80, trois pistes devenues deux) et #7 (2:12.33, la piste qui perd deux points, seul de ces endroits jugé pire) ;
    - 094 #7 (2:28.11, trop courte une fois le saut refusé) ;
    - 099 #60 (1:32.09, la piste perd 4 points) et #90 (3:22.07, trois pistes devenues deux) ;
    - 100 #36 (1:36.56, trois pistes devenues deux) et #43 (1:55.58, des points changent de tache).

    La 098 #61 est tenue : le morceau de #311 qui prolonge la #59 contient son premier point, et le suivi rejoué sans portée la tient aussi.
  - `ZOOM_SWITCHES`, les pistes dont le zoom automatique diffère de celui des pistes de la v0.2.0 dont elles tiennent le premier point, a 31 lignes : 30 zooms gagnés, dont 25 par des pistes qui ne tiennent aucun début de la v0.2.0, et un perdu, la 098 #114.
    La piste qui tient le début de la 098 #114 suit la seconde chauve-souris jusqu'à l'image 7713, où sa tache atteint 164 px² et une intensité de 65, au-dessus des seuils du zoom (100 et 40) ; la première chauve-souris, coupée de la seconde, gagne son zoom (#112, 76 px² au plus, et #113).
    Avant la portée, cinq zooms gagnés venaient de pistes qui tiennent un début de la v0.2.0 :
    - la 094 #2 : une petite tache repasse, une image plus tard et à moins de 8 px, là où était la grande, et leurs deux pistes échangent leur seconde moitié après deux détections faibles ;
    - la 097 #85 : la tache unique de deux chauves-souris qui se croisent (2:41.72-2:41.76) va à l'autre piste, mieux prédite grâce à une détection faible ;
    - la 099 #90 : le début de 7 points qui reste après la réunion juste de #88 et #90 ;
    - deux pistes recollées qui prennent le zoom d'un bout ajouté par les détections faibles, la 096 #89 de la v0.2.0, réunie au bout de 6 points des images 5523-5530, et la 098 #126, réunie à celui des images 8452-8459.
  - Les zooms enregistrés par la v0.1.1, décidés sur ses propres pistes, se vérifient à part (`the_zooms_recorded_with_the_v011_are_those_its_own_tracks_ask_for`, sans décodage).
- `cli/tests/local_clips` : les fenêtres des extraits des 26 vidéos, toutes exigées, comparées à celles des pistes de la v0.2.0, vidéo par vidéo.
  17 vidéos ont des fenêtres qui changent avec leurs pistes : 180 fenêtres propres à la commande et 234 propres à la v0.2.0, avec le nombre de fenêtres de chaque côté et de celles propres à chacun figé par vidéo.
- `process` et `tracked` n'écartent que les plages du décodeur courant : `reference_092`, `slow_command`, `local_command` et `local_clips` dépendent donc du poste sur les 8 vidéos où lumiel voit 14 images en erreur de plus que le Mac.
  Leurs lignes sont figées sur lumiel, et restent à vérifier sur le Mac. `compare` réunit les deux lectures (#275) et n'en dépend pas.
- Restent sur la v0.1.1 :
  - les entrées, avec ffprobe pour témoin : `slow_extract` (entrées de l'extrait), `reference_092` (9 011 images de la 092), `local_command` (`pts` de la 103 annotée) et, dans `engine`, `slow_input`, `local_input` et `local_damage`, dont la règle de lumiel reste celle de #250 ;
  - l'aval nourri des pistes de la v0.1.1 (`summary`, `files`, `render`) ;
  - les sorties (`local_outputs`), le banc (`local_guard`) et le format v1.
