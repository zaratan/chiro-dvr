# Le Rust : construction, charte et tests

Ce document s'adresse à qui modifie le code Rust de `batdetect` ([qui tient quoi](README.md#qui-tient-quoi)).
Il couvre la chaîne de construction, les licences, la charte, les tests, l'encodage, puis le détail de chaque crate.
La carte courte des crates est dans [CLAUDE.md](../CLAUDE.md).
La logique de la détection est dans [10](10-logique-de-detection.md) et la valeur de chaque réglage dans [02](02-methode.md).

Le Rust doit détecter au moins aussi bien que la v0.1.1 sans la reproduire bit à bit.
La v0.1.1 est la dernière version en Python (tag `python-final`).
Les détections et les pistes se comparent par appariement avec des tolérances. Chaque écart se juge à l'œil.
Les comparaisons à la v0.1.1 qui restent dans ce document expliquent une règle ou un test encore en vigueur.
Le reste de l'histoire de la réécriture est dans git et le [CHANGELOG](../CHANGELOG.md).

## Construction

```bash
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:ffmpeg   # construit FFmpeg une fois par poste
mise run rust:static      # fmt, clippy, règles de la charte, cargo deny, cargo shear
mise run rust:coverage    # tests rapides sous cargo-llvm-cov, couverture ≥ 95 %
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:slow       # l'extrait LFS
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:settings   # les 16 cas de réglages du banc d'essai
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:reference  # la 092 entière
```

`verrou lourd` est le verrou de mesure des agents de ce poste (`MESURES.md`, hors dépôt). Ailleurs ces commandes se lancent sans lui.

- **Toolchain** : `rust-toolchain.toml` fixe Rust 1.99.0 en édition 2024 (clippy, rustfmt, llvm-tools).
  C'est la seconde source de versions du dépôt à côté de `mise.toml`. Une montée de version se fait dans un changement à part.
- **FFmpeg 7.1.1 exactement et openh264 2.6.0.** `packaging/ffmpeg-versions` donne les versions, les adresses et les sha256.
  La signature de l'archive FFmpeg a été vérifiée avec la clé publiée sur ffmpeg.org. La somme d'openh264 est celle de la formule Homebrew.
  nasm 2.16.03 n'est construit que sur x86_64 pour l'assembleur. Il n'entre pas dans le binaire.
  La 7.1.1 est la version de la roue OpenCV de la v0.1.1.
  Avec elle le prototype lisait les mêmes images qu'OpenCV : 1 230 sur 1 230 sur l'extrait.
  FFmpeg 9.0.2 s'en écartait jusqu'à 3 niveaux (rapport d'exploration, branche `exploration/rust`).
  La comparaison à la référence v0.1.1 part donc d'une même lecture.
  L'épinglage rend la lecture des drapeaux d'erreur (`decode_error_flags`) reproductible d'un poste à l'autre.
- **Ligne `configure`** : en lecture elle active le démultiplexeur mov, les parsers et décodeurs h264 et hevc, le protocole file et swscale.
  En écriture elle active l'encodeur libopenh264 et le muxeur mp4.
  Sur macOS elle active aussi VideoToolbox (décodage et encodeur `h264_videotoolbox`).
  `--disable-autodetect` écarte les bibliothèques du système.
  `--enable-pic` est explicite (#220) : sans lui `configure` n'active le PIC que si le compilateur en produit par défaut.
  Le clang de macOS et le gcc d'Ubuntu le font (PIE par défaut). Le gcc de Rocky 10 ne le fait pas.
  Or Rust lie en PIE sous Linux et l'édition de liens échoue sur des `.a` non PIC.
- **Script** : `packaging/ffmpeg-unix.sh` est en bash portable (bash 3.2 de macOS compris).
  - Il construit dans un dossier temporaire du cache puis le déplace par `mv`.
    Si une construction concurrente a déjà posé la cible, `mv` refuse et la cible existante est gardée.
  - **Cache partagé entre worktrees** : `${XDG_CACHE_HOME:-~/.cache}/batdetect/ffmpeg/<empreinte>-<os>-<arch>/`.
    L'empreinte est un sha256 du script et des seules clés `FFMPEG_`, `OPENH264_` et `NASM_` de `ffmpeg-versions` : le script Windows n'y touche pas.
  - Chaque worktree a un lien `.ffmpeg` vers sa cible. Ce lien est ignoré par git.
    Les `.pc` sont réécrits en `${pcfiledir}/../..` : le cache marche à travers le lien à n'importe quel chemin.
  - `--fingerprint` imprime l'empreinte (clé du cache CI).
    `--link` (`mise run rust:link`) pose le lien et échoue si le cache manque : `mise run check` ne construit jamais FFmpeg.
- **Liaison** :
  - `.cargo/config.toml` force `PKG_CONFIG_LIBDIR` vers `.ffmpeg/lib/pkgconfig` et vide `PKG_CONFIG_PATH`, `CPATH` et `C_INCLUDE_PATH`.
    Ainsi le FFmpeg 9 de Homebrew, ses `.pc` et ses en-têtes ne peuvent pas être pris. `cargo` seul suffit sans variable d'environnement.
  - Nos `.pc` mettent toutes les dépendances dans `Libs:` et le dossier ne contient que des `.a` : la liaison est statique.
  - On n'utilise pas `FFMPEG_DIR` : sa route dans ffmpeg-sys-next 9.0.1 oublie openh264 et la bibliothèque C++.
    On n'utilise pas non plus la feature `static` de ffmpeg-next.
    Elle ajoute 16 frameworks macOS figés et inutiles, dont QTKit, AppKit, OpenGL et OpenCL.
    QTKit est absent en arm64 et l'éditeur de liens le signale.
  - Ce que le binaire lie en dynamique (`otool -L`, 16 octobre 2026) :
    - sur macOS, libc++, libSystem, libiconv, libobjc et six frameworks (Foundation, CoreFoundation, VideoToolbox, CoreMedia, CoreVideo, CoreServices).
      La bibliothèque standard de Rust tire libiconv. libobjc et Foundation viennent de ctrlc par objc2.
    - sous Linux, libstdc++, libgcc_s, libc et libm.

    `packaging/check.sh` contrôle cette liste sur le binaire livré ([Empaquetage](#empaquetage)).
- **Version liée vérifiable** : FFmpeg est construit avec `--extra-version=<empreinte>`.
  `batdetect --version` affiche donc `FFmpeg 7.1.1-<empreinte>`. Un test compare cette empreinte à la cible de `.ffmpeg`.
- **Reconstruction** : cargo ne suit pas les `.a`.
  - La rlib de ffmpeg-sys-next n'en contient aucune copie : elle pèse 5 516 320 octets alors que `libavcodec.a` pèse à elle seule 13 Mo.
    La mesure date du 11 octobre 2026 sous macOS. Sur ces octets, 5 515 160 sont des métadonnées : les liaisons de bindgen.
    S'y ajoute un objet de 336 octets sans symbole défini.
  - Le build script ne déclare que des `rustc-link-lib=avcodec` sans `static=`, avec la recherche dans `.ffmpeg/lib`.
    Les `.a` sont donc lus à l'édition de liens de chaque binaire. Ni ce script ni l'édition de liens ne se refont quand les `.a` changent.
    Un FFmpeg changé laisserait des binaires liés à l'ancien et des liaisons tirées des anciens en-têtes.
  - Le script lance donc `cargo clean -p ffmpeg-sys-next` dans chaque profil (`target/llvm-cov-target` compris) quand `.ffmpeg` change de cible.
    Le nettoyage passe avant le changement du lien : s'il échoue, le passage suivant le refait.
  - Après une montée de version de FFmpeg, on fait un `cargo clean` complet car les liaisons de bindgen changent.
  - Un arrêt brutal (SIGKILL, délai de la CI dépassé) peut laisser un dossier `.build.*` dans le cache : il se supprime à la main.
- **Cible processeur** :
  - macOS arm64 garde le défaut (`apple-m1`) pour le Rust.
  - Linux x86_64 compile le Rust en `x86-64-v2` (`.cargo/config.toml`).
    Sans cela le min/max u16 du réseau de médiane ne serait pas vectorisé, car il demande SSE4.1.
  - FFmpeg choisit ses routines en assembleur à l'exécution.
    Son code C et celui d'openh264 sont en revanche compilés pour une cible épinglée dans `ffmpeg-unix.sh`, donc dans l'empreinte (#310).
    Cette cible est `-march=x86-64-v2` sous Linux et `-mcpu=apple-m1` sous macOS.
    FFmpeg la reçoit par `--extra-cflags` et openh264 par `CFLAGS_OPT="-O3 …"`.
    Le Makefile d'openh264 met `CFLAGS` dans ses compilations C, C++ et assembleur.
    Sans cible épinglée, ils suivent celle du compilateur du poste : `x86-64-v3` pour le gcc de Rocky 10 (`gcc -Q --help=target`).
    Le FFmpeg de lumiel s'arrêtait ainsi sur un `vmovsd` sous Intel SDE en Nehalem (#251).
    Il avait pourtant la même empreinte que celui de la CI.
  - Windows n'épingle rien : MSVC compile en SSE2 par défaut sur x64 quel que soit le poste.
  - Ne jamais poser `RUSTFLAGS` : il remplacerait ce réglage.
- **macOS 14 au minimum** : `MACOSX_DEPLOYMENT_TARGET=14.0` vaut pour openh264, FFmpeg et rustc.
  On a mesuré `minos` 14.0 sur les 411 objets et sur le binaire.
- **Profils** : `release` a `lto = "fat"`, `codegen-units = 1` et `panic = "unwind"`.
  `ci` hérite de `release` sans LTO et en 16 unités. Il ajoute `overflow-checks` et `debug-assertions`.
  Les tests tournent en `ci` et les temps se mesurent en `release`.
- **Construction de FFmpeg et d'openh264** (10 octobre 2026) : elle prend 42 à 50 s et un cache de 26 Mo sur le Mac M1 Max.
  Elle prend 224 s et 43 Mo sous Linux amd64 émulé (OrbStack).

### Empaquetage

```bash
mise run package          # binaire release, puis dist/batdetect-<cible>.tar.gz (zip sous Windows)
mise run package:check    # contrôles du livrable sur cette archive
```

- **Archives** (#223) : `batdetect-darwin-arm64.tar.gz`, `batdetect-linux-x86_64.tar.gz` et `batdetect-windows-x86_64.zip`.
  - Elles contiennent à la racine le binaire, `LICENSE`, `NOTICE` et `licenses/`.
  - Le nom n'a pas de numéro de version parce que la formule Homebrew en dépend.
  - `packaging/package.sh` lit la cible dans la ligne `host:` de `rustc -vV` et non dans `uname`. Sous MSYS2, `uname` vaut `UCRT64_NT-…`.
    Les scripts sont en bash puisque `mise` ne tourne pas sous Windows.
  - Sans `BATDETECT_VERSION`, il prend la version de `[workspace.package]` dans `Cargo.toml`.
    C'est celle que `--version` imprime : `mise run package:check` contrôle ainsi en local la même version qu'en CI.
- **Contrôles** : `packaging/check.sh ARCHIVE [VERSION]` joue tous les contrôles puis échoue à la fin si un seul a échoué.

  | Contrôle | Ce qu'il mesure |
  |---|---|
  | C1 | l'archive contient le binaire, `LICENSE`, `NOTICE` et `licenses/`, rien d'autre |
  | C2 | aucun lien dynamique hors d'une liste mesurée par système : `otool -L`, `readelf -d` (lignes NEEDED), `dumpbin -dependents` |
  | C3 | `minos` ≤ 14.0 sous macOS ; sous Linux, `GLIBC_` ≤ 2.35, `GLIBCXX_` ≤ 3.4.30, `CXXABI_` ≤ 1.3.13 et `GCC_` ≤ 12.0.0, la bibliothèque de l'Ubuntu 22.04 ; sous Windows, versions du système et du sous-système de l'en-tête PE ≤ 10.0 (`dumpbin -headers`), celle de Windows 10, que Windows 11 annonce aussi ; C3-cpu, sous Linux et Windows : l'extrait tourne sous Intel SDE en mode Nehalem (`-nhm`, qui couvre x86-64-v2) et donne le CSV de C6 |
  | C4 | `--version` donne `batdetect <VERSION>`, puis le FFmpeg épinglé, avec l'empreinte du script de construction et la LGPL |
  | C5 | le bloc `Rust crates:` de `NOTICE` égale `cargo tree -p batdetect -e normal,no-proc-macro` sur les trois cibles de `packaging/about.toml` ([Licences](#licences)) |
  | C6 | le CSV de l'extrait, en anglais, est identique à l'octet à `tests/fixtures/rust/video_092_original_3m24-4m05/tracks.csv` |
- **Preuves négatives** : chaque contrôle échoue seul sur une archive altérée.
  - Le 16 octobre 2026 sur le Mac, un fichier en trop et `NOTICE` retiré font échouer C1.
    Le binaire remplacé par `/usr/bin/curl` fait échouer C2, C3, C4 et C6.
    Une construction sans `BATDETECT_VERSION` fait échouer C4. Un octet changé dans le CSV fait échouer C6.
  - Un sous-système 11.0 posé par `editbin` sur une copie de l'archive Windows donne `FAIL C3-floor: subsystem 11.00, os 6.00, above 10.00` (CI de la PR #261).
    C4 et C6 échouent ensuite, car le chargeur refuse le binaire (`Exec format error`).
    L'archive intacte donne `ok C3-floor (subsystem 6.00, os 6.00)`.
  - C3-cpu, le 17 octobre 2026 : un binaire construit en `target-cpu=x86-64-v3` donne `FAIL C3-cpu: batdetect failed on the extract: TID 0 SDE-ERROR: Executed instruction not valid for specified chip (NEHALEM): … vzeroupper`.
    On l'a vu sur lumiel dans un conteneur `ubuntu:26.04` dont la glibc accepte Nehalem.
    La CI l'a ensuite montré sur Ubuntu et Windows (PR #309). L'archive Linux de `main` donne `ok C3-cpu` dans `ubuntu:22.04`.
    `--version` passe sur ce binaire car le code fautif ne tourne qu'au traitement. C'est pourquoi le contrôle traite l'extrait.
- **Intel SDE** (C3-cpu, #251) : `packaging/sde.sh` télécharge le kit de `packaging/sde-versions` (10.13.1) par son URL directe et vérifie le sha256 épinglé.
  Il l'extrait dans `${XDG_CACHE_HOME:-~/.cache}/batdetect/sde/` et imprime le chemin de `sde64` ou de `sde.exe`.
  SDE présente le CPUID de Nehalem : FFmpeg et openh264 choisissent alors les routines de ce processeur.
  SDE arrête le programme sur toute instruction que Nehalem n'a pas.
  Il n'émule que ce qui manque à l'hôte : l'extrait prend 19 s sur lumiel et donne les pistes de C6 à l'octet.
  - Licence : Intel Simplified Software License (février 2020, `Licenses/LICENSE.pdf` du kit).
    Elle permet l'usage et la redistribution sans modification et n'a pas de clause sur la CI.
    L'utilisateur l'a acceptée le 17 octobre 2026 : la page de téléchargement d'Intel demande un clic.
  - Sous Ubuntu, Pin demande `kernel.yama.ptrace_scope=0` (`README.txt` du kit) : une étape de `package.yml` le pose par `sudo sysctl` avant `check.sh`.
  - Le kit (36 Mo) n'a pas de cache CI : `sde.sh` fait confiance à un kit déjà extrait sans recalculer son sha256.
    Un cache restauré contournerait l'épinglage.
    Une release dépend donc du miroir d'Intel. S'il manque, elle échoue en clair (`could not fetch Intel SDE`).
  - SDE ne tourne pas sous macOS arm64.
    Sous Rocky 10, la glibc exige x86-64-v3 et SDE n'y tourne pas utilement : `sde64 -nhm -- /bin/true` y donne « CPU does not support x86-64-v3 ».
    La preuve se fait donc dans les tâches `package / build` d'Ubuntu et de Windows.
    Sur un tel hôte, C3-cpu échoue sur une sonde qui le dit (`sde64 -nhm -- /bin/true`) plutôt que sur l'extrait.
    `mise run package:check` y passe au rouge.
- **Mesure** (16 octobre 2026, Mac, `release`) : l'archive macOS pèse 3 511 951 octets pour un binaire de 8 130 992 octets.
  Le CSV des profils `ci` et `release` est identique à l'octet.

### Windows

La cible `x86_64-pc-windows-msvc` n'est construite que par la CI (tâches `rust-windows` et `rust-windows-slow`) car `mise` n'y tourne pas.
Le binaire Windows se construit et s'empaquette dans `package.yml` ([Empaquetage](#empaquetage)).
La construction sur un poste Windows est l'objet de #226.

- **Script** : `packaging/ffmpeg-windows.sh` est en bash et tourne sous MSYS2 avec MSVC dans le `PATH`.
  - Il lit le même `ffmpeg-versions`. Deux clés `WINDOWS_NASM_*` s'y ajoutent pour le binaire officiel de nasm 2.16.03.
    Ce préfixe reste hors du filtre de l'empreinte Unix : celle-ci ne change donc pas.
  - Son empreinte hache le script et les clés `FFMPEG_`, `OPENH264_` et `WINDOWS_`.
  - `pinned`, `sha256`, `fetch`, `relocate` et `clean_bindings` sont recopiées du script Unix.
    Un fichier commun changerait `ffmpeg-unix.sh`, donc son empreinte et son cache.
    Deux sont adaptées : `fetch` est coupée en `download` et `fetch`. `relocate` traite le chemin sous ses deux formes, POSIX et mixte.
    Un correctif du script Unix se reporte donc à la main.
  - Sa seule option est `--fingerprint`.
- **Même bibliothèque C partout** : openh264 se compile en `/MT` (`make OS=msvc`, avec son `CFLAGS_OPT` redonné sans `-Zi -FS`).
  FFmpeg se compile aussi en `/MT` (`--toolchain=msvc --extra-cflags=-MT`).
  Le Rust prend `+crt-static` (`.cargo/config.toml`, avec `target-cpu=x86-64-v2`).
  Le binaire ne demande donc ni `vcruntime140.dll` ni l'UCRT.
- **Liaison** : le script corrige deux pièges après `make install`.
  - FFmpeg nomme ses archives `libavcodec.a` même sous MSVC.
    Or rustc passe `avcodec.lib` à `link.exe` : le script renomme `libX.a` en `X.lib`.
  - Le `configure` de FFmpeg écrit les bibliothèques système des `.pc` à la manière de MSVC (`bcrypt.lib`, `-libpath:`).
    pkg-config-rs ignore en silence un `bcrypt.lib` nu et lit `-libpath:` comme `-l` suivi de `ibpath:`.
    Le script les réécrit en `-lbcrypt` et `-L`.

  La liaison reste statique sans rien corriger. Sans la feature `static`, ffmpeg-sys-next appelle pkg-config-rs en `statik(false)`.
  Celui-ci demande ces bibliothèques sans type, donc en dylib. Sous MSVC, `link.exe` prend aussi bien une archive.
- **Autres pièges** :
  - Sous MSVC, ffmpeg-sys-next essaie vcpkg avant pkg-config.
    Comme `msvc-dev-cmd` (vcvars) définit `VCPKG_ROOT`, la tâche pose `VCPKGRS_DISABLE=1`.
  - Le `link.exe` de coreutils (MSYS2) est renommé avant toute construction. Le script refuse un `link` qui n'est pas celui de Microsoft.
  - MSYS2 réécrit en chemin tout argument qui commence par `/`.
    Les options de MSVC s'écrivent donc `-x`. `cmd /c` passe par `MSYS2_ARG_CONV_EXCL='*'`.
  - Le cache se donne en chemin mixte (`D:/a/...`). Le script le normalise par `cygpath`.
  - Pour bindgen, `LIBCLANG_PATH` pointe vers le LLVM de l'image.
  - openh264 se compile sans `-Zi -FS`, donc sans informations de débogage.
    Avec `-Zi`, chaque édition de liens affichait 65 avertissements LNK4099 : un par objet dont le `.pdb` n'est pas installé.
    Ces avertissements n'avaient pas d'effet sur le binaire mais rustc les remontait.
- **Lien `.ffmpeg`** : c'est une jonction NTFS (`mklink /J`). Elle ne demande pas de droits.
  `fs::read_link` la lit : le test d'empreinte de la version liée vaut aussi sous Windows.
- **Gardes du script** :
  - `config.h` porte LGPL 2.1+ et `HAVE_W32THREADS 1`.
    Le multifil n'est gardé qu'à la compilation : aucun test ne vérifie que le décodage tourne sur plusieurs fils.
  - pkg-config ne rend ni antislash ni `.lib` nu ni `-libpath:` quand on l'interroge sans `--static` comme le fait cargo.
  - Aucune archive `.a` ne reste.
  - Chaque `.lib` demande `LIBCMT` et jamais `MSVCRT` (`dumpbin -directives`).
- **Fins de ligne** : l'image Windows extrait le texte en CRLF (`core.autocrlf`).
  `.gitattributes` force `eol=lf` pour les scripts `.sh`, `ffmpeg-versions`, `sde-versions`, `tests/fixtures/reference/**` et pour `LICENSE`, `packaging/NOTICE` et `packaging/licenses/**`.
  Les textes du zip sont ainsi ceux des archives macOS et Linux.
- **Mesures** (11 octobre 2026, runner `windows-latest`, image `windows-2025-vs2026` 20260925.250.1, MSVC 14.51) : FFmpeg et openh264 se construisent en 3 min 15 s.
  À chaud, la tâche prend 1 min 53 s (caches LFS, `target/` et FFmpeg restaurés).
  Le binaire dépend de `kernel32`, `ntdll`, `bcrypt` et `api-ms-win-core-synch-l1-2-0`.

## Licences

Le livrable est sous **MIT** pour le code de batdetect (`LICENSE`) et sous **LGPL 2.1 ou ultérieure** pour FFmpeg (décision du 16 octobre 2026, #224).
Il porte aussi openh264 en BSD-2 et Roboto en OFL-1.1.
La bibliothèque standard de Rust et 83 crates tierces y sont aussi, chacune sous sa licence.

`packaging/NOTICE` décrit le binaire composant par composant. Les textes de licence sont dans `packaging/licenses/`.
`package.sh` copie l'un et l'autre dans chaque archive ([Empaquetage](#empaquetage)).
Chaque texte vient de la source épinglée du composant :

| Fichier | Source |
|---|---|
| `FFmpeg-LGPL-2.1.txt` | `COPYING.LGPLv2.1` de `ffmpeg-7.1.1.tar.xz` |
| `openh264-BSD-2-Clause.txt` | `LICENSE` de l'archive openh264 2.6.0 |
| `Roboto-OFL-1.1.txt` | copie de `crates/draw/fonts/OFL.txt` |
| `Rust-MIT.txt`, `Rust-Apache-2.0.txt` | `LICENSE-MIT` et `LICENSE-APACHE` de `rust-lang/rust` au tag 1.99.0 |
| `crates.txt` | généré par `mise run notice` |

- **FFmpeg 7.1.1 en LGPL 2.1 ou ultérieure.** Il est construit sans `--enable-gpl` ni `--enable-nonfree`.
  `libavcodec` annonce « LGPL version 2.1 or later » : `--version` l'affiche et `check.sh` le contrôle (C4).
  Lié en statique, il oblige à fournir de quoi relier.
  Chaque release joint `ffmpeg-7.1.1.tar.xz` (#223) et les sources de batdetect du tag sont publiques.
  `NOTICE` dit comment relier : changer `FFMPEG_URL` et `FFMPEG_SHA256` dans `ffmpeg-versions`, lancer le script de construction puis `cargo build`.
- **openh264 2.6.0 en BSD-2.** Il est construit depuis ses sources, jointes à la release.
  Il n'a donc **pas** la licence de brevets H.264 que Cisco n'accorde qu'à ses propres binaires.
  Les brevets ont été acceptés le 16 octobre 2026.
  `NOTICE` dit que les codecs H.264 et HEVC peuvent être couverts par des brevets et qu'aucune licence de brevet n'est fournie avec ce binaire.
- **Roboto en SIL Open Font License 1.1** (#101) : `draw` embarque la police variable `crates/draw/fonts/Roboto.ttf` (`wdth` et `wght`).
  Sa licence est à côté (`crates/draw/fonts/OFL.txt`) et recopiée dans `packaging/licenses/`.
  Elle vient du fichier `ofl/roboto/Roboto[wdth,wght].ttf` du dépôt `google/fonts` au commit `6183fc0` du 27 janvier 2026 (sha256 `d7598e12…872a134`).
  Elle n'a pas de nom de police réservé.
- **Bibliothèque standard de Rust** : MIT OU Apache-2.0.
  Son avis complet couvre aussi ses dépendances : c'est `share/doc/rust/COPYRIGHT-library.html` de la toolchain (1,5 Mo de HTML).
  Ce fichier n'est pas joint à l'archive.
- **Crates livrées** : c'est l'union de `cargo tree -p batdetect -e normal,no-proc-macro --target <triple>` sur les trois triples de `packaging/about.toml`.
  Les crates du workspace en sont exclues.
  Mesurée le 16 octobre 2026, elle compte 83 crates : 78 sous macOS, 74 sous Linux et 75 sous Windows.
  - **`mise run notice`** croise cette liste avec le JSON de `cargo-about` (épinglé dans `mise.toml`).
    Il réécrit le bloc `Rust crates:` de `NOTICE` avec une ligne `- nom version, licence` par crate.
    Il réécrit aussi `licenses/crates.txt` : 41 textes distincts, chacun suivi des crates qui l'emploient.
    `cargo-about` ne sait pas écarter les proc-macros (13 crates de plus avec leurs dépendances) : c'est la liste de `cargo tree` qui fait foi.
  - **Avis de copyright** : chaque texte garde celui que la crate publie. Plusieurs textes n'en ont aucun, comme leur amont.
    C'est le cas du MIT de fdeflate, du WTFPL de ffmpeg-next et d'un texte MIT commun à 17 crates (adler2 à zmij, dont rustc-hash et tinyvec).
    - Cinq crates ont un texte que `cargo-about` ne trouve pas seul. `about.toml` le lui donne par `clarify`.
      Pour type-map, windows-sys et windows-link, le texte est dans la crate.
      Pour fluent-langneg et intl_pluralrules, il est dans le dépôt amont au commit publié.
    - objc2, objc2-encode, block2 et dispatch2 ne publient aucun texte de licence. Ils ont seulement un `LICENSE.md` qui renvoie à MIT.
      Leur ligne de copyright devient « No copyright notice is published with these crates », suivie des auteurs de leur `Cargo.toml`.
    - La tâche échoue sur tout autre modèle de texte (`<year>`, `<copyright holders>`).
  - **Contrôle C5** de `check.sh` : le bloc de `NOTICE` de l'archive égale la liste de `cargo tree` sans crate absente ni en trop.
    Une dépendance changée sans `mise run notice` fait donc échouer `package.yml`.
    `cargo tree` lit les manifestes des trois cibles et la machine ne les a pas toujours téléchargés : C5 a besoin du réseau.
    Preuve négative (16 octobre 2026) : une crate retirée du `NOTICE` puis une crate en trop font échouer C5 seul.
  - **Licence retenue** : dans une expression « OU », l'ordre de `accepted` (`about.toml`) choisit. MIT l'emporte pour 74 crates.
    Les autres suivent Unicode-3.0 pour `tinystr`, `yoke` et `zerofrom`, BSD-3-Clause pour `tiny-skia` et `tiny-skia-path`, BSD-2-Clause pour `arrayref`, WTFPL pour `ffmpeg-next` et `ffmpeg-sys-next`.
    Pour `self_cell`, l'expression `Apache-2.0 OR GPL-2.0-only` donne Apache-2.0.
    Aucune crate livrée n'a de fichier NOTICE à reprendre (Apache-2.0 §4d).
- **Licences admises** : `deny.toml` n'admet que MIT, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, WTFPL, Zlib et Apache-2.0 (avec ou sans LLVM-exception).
  Chacune sert au moins une crate de `Cargo.lock` (`cargo deny list`, 19 octobre 2026).
  Zlib sert par exemple `bytemuck`, `miniz_oxide` et `tinyvec`.
  D'autres licences n'apparaissent que comme alternatives dans une expression « OU ».
  `adler2` a 0BSD à côté de MIT, `memchr` Unlicense et `r-efi` LGPL-2.1-or-later.
  Pour `self_cell`, GPL-2.0-only n'est qu'à côté d'Apache-2.0. `r-efi` ne vise qu'UEFI et n'est livré sur aucune des trois cibles.
- **`jiff` et ses crates `jiff-*`** (#66, #83 : heure locale des noms datés et de `params.json`) sont sous Unlicense OU MIT.
  `about.toml` retient MIT. Sous Windows `jiff-tzdb` embarque la base des fuseaux horaires de l'IANA. Cette base est du domaine public.

## Lints

La [charte](#charte) est résumée dans [CLAUDE.md](../CLAUDE.md).
Les lints se règlent dans `[workspace.lints]` du `Cargo.toml` racine et jamais par attribut.

- **Activés** :
  - `clippy::pedantic`.
  - Des restrictions : `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr`, `exit`, `allow_attributes`, `let_underscore_must_use`, `unused_result_ok`, `map_err_ignore`, `iter_over_hash_type`, `mod_module_files`, `string_slice`, `clone_on_ref_ptr`.
  - Pour `unsafe` : `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block`, `unnecessary_safety_comment`.
  - Côté rustc : `unsafe_code = "forbid"`, `unreachable_pub`, `missing_debug_implementations`, `trivial_numeric_casts`, `rust_2018_idioms`, `non_ascii_idents`.
- **Sept lints de `pedantic` coupés** dans la table seulement :
  - `cast_possible_truncation`, `cast_possible_wrap`, `cast_precision_loss` et `cast_sign_loss` tombaient 30, 55, 48 et 31 fois sur les noyaux du prototype.
    Ces cas étaient des calculs de pixels où la règle 15 suffit.
  - `missing_errors_doc` et `missing_panics_doc` sont coupés parce que la règle du zéro commentaire ne demande pas de documentation.
  - `must_use_candidate` signalerait presque chaque accesseur.
- **Refusés** : `indexing_slicing`, `arithmetic_side_effects`, `as_conversions` et `missing_docs`, avec 272, 223, 184 et 62 occurrences sur le prototype.
  `nursery` et `cargo` sont refusés aussi.
- **`media` est la seule crate avec `unsafe`.** Elle porte sa propre table : une copie exacte de celle du workspace sans `unsafe_code`.
  Cargo refuse `workspace = true` avec une surcharge locale. Un contrôle compare les deux tables.
- **Règles vérifiées par script** (`mise run rust:rules`, dans `rust:static`) :
  - Aucun `#[allow]`, `#[expect]` ou `cfg_attr(…allow…)`.
  - Chaque crate porte `[lints] workspace = true` sauf `media`. Sa table est égale à celle du workspace sans `unsafe_code`.
  - Aucun `ffmpeg_next` hors de `media`.
  - Les dépendances des crates ne viennent que du workspace.
  - Au plus un appel à `catch_unwind` hors des tests, et seulement sous `crates/cli/src/` (#113).
    Les tests sont `*/tests.rs`, `*/testing.rs` et `*/tests/*`. Le compte exclut les lignes `use` et prend toutes les formes d'appel.
    Pas de `panic = "abort"` dans les profils.
  - Au plus 2 % de lignes de commentaire dans `crates/` en comptant les commentaires de fin de ligne et `/* */`.
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
   **Exception** (décision de l'utilisateur à #118) : le gris de la détection reprend la table mesurée sur OpenCV et l'arrondi à 8 bits après la réduction.
   Les seuils de la v0.1.1 sont en effet réglés sur ce gris (mesures dans `engine`, « Gris de la détection »).

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
    Tout texte vu par l'utilisateur vit dans un seul module de la commande.

Types et calcul :

13. L'index d'image et le `pts` ont chacun leur type (`FrameIndex`, `Pts`).
    L'index suit l'ordre d'affichage, celui où le décodeur rend les images. Pas de sentinelle : `Option<Pts>`.
14. Tout est en pixels de la vidéo d'origine hors de la détection. Seul `WorkScale` passe des pixels de travail aux pixels d'origine.
15. Conversions : `From` quand elle existe.
    Un `as` qui perd de l'information suit un `round` ou un `clamp` visible. Ou bien il vit dans une fonction de conversion nommée.
16. Même entrée, même sortie quel que soit le nombre de fils.
    Aucune itération sur `HashMap` ou `HashSet` n'atteint une sortie. Aucune somme flottante n'est parallèle.
    La règle vaut pour ce qui sort du moteur, plages abîmées exclues.
    Les pixels que FFmpeg dissimule sur une image abîmée dépendent du nombre de fils de décodage (mesure dans `engine`).
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

- Chaque dépendance se déclare une fois dans `[workspace.dependencies]` puis `x.workspace = true`. `--locked` est partout.
- `rayon` porte le parallélisme de la détection dans `engine`.
  `dhat` est une dépendance de développement d'`engine` qui mesure le tas des tests de mémoire.
- `serde` sert `types` et `reference`.
  `serde_json` a la feature `float_roundtrip` pour qu'un flottant de la référence se relise au bit près.
  `flate2` (moteur `miniz_oxide`, 100 % Rust) lit et écrit les `.gz` de la référence. `zlib-rs` figure dans `Cargo.lock` sans être compilé.
- **Pas d'`ab_glyph`** : il tire `ttf-parser`, que RUSTSEC-2026-0192 déclare non maintenu sans mise à jour sûre.
  L'utilisateur refuse d'ignorer l'avis.
- **Polices** (#101, 11 octobre 2026) :
  - `skrifa` 0.48.0 (fontations de Google, MIT OR Apache-2.0) est suggéré par l'avis.
    Il fournit la table des caractères, les métriques, les contours et les axes variables.
  - Le tracé passe par `tiny-skia` 0.12.0 (BSD-3-Clause) sans sa feature `png`.
  - `unicode-normalization` 0.1.25 (MIT OR Apache-2.0) donne la forme NFC.
  - `read-fonts` tire `yoke`, `zerofrom` et `synstructure`. Elles sont sous Unicode-3.0, une licence déjà admise.

  On écarte `fontdue` et `rusttype`, qui tirent `ttf-parser`.
  On écarte aussi `swash` (un second rastériseur à côté de tiny-skia) et `icu_normalizer` (une quinzaine de crates pour une seule normalisation).
- **PNG de l'image résumé** (#104) : `png` 0.18.1 (MIT OR Apache-2.0) sert en direct dans `summary`, en RGB 8 bits.
  Il tire `fdeflate` 0.3.7 (même licence). `draw` reste sans format de fichier. La feature `png` de tiny-skia tirerait les mêmes crates.
  `png` dépend de `miniz_oxide` 0.8 quand `flate2` prend la 0.9. Ce doublon existait déjà par `dhat`.
  Il entre avec `png` dans le binaire livré (avertissement de `cargo deny`).
- `clap` sert en builder, avec les features `string` et `wrap_help`.
  La commande se construit à l'exécution avec les textes de la langue (#114). `derive` ne le fait pas.
- **Langues** (#114) : `fluent-bundle` 0.16.0, `unic-langid` 0.9.6 et `sys-locale` 0.3.2 (MIT OR Apache-2.0).
  Ils ajoutent 13 crates à `Cargo.lock` (`fluent-syntax`, `fluent-langneg`, `intl-memoizer`, `intl_pluralrules`, `self_cell`, `type-map`, `tinystr`, `zerovec`, `displaydoc`, `unic-langid-impl`).
  Elles sont sous MIT, Apache-2.0 ou Unicode-3.0, des licences déjà admises.
  Écartés : `i18n-embed-fl` (macro et chargement de fichiers dont la commande n'a pas besoin) et ICU4X (des dizaines de crates pour formater deux sortes de nombres).
  La macro `langid!` n'est pas prise : la langue se lit par `"fr".parse()`. Le test des pluriels prouve qu'elle est reconnue.
- `cargo deny check` échoue sur un avis de sécurité, une licence hors liste, une source inconnue, un paquet retiré (`yanked`) ou une version sans borne (`*`).
  Il ne fait qu'avertir sur une version en double.

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
- `mise run check` joue `rust:static` et `rust:test` avant de rendre.
  `mise run check:ci` joue `rust:coverage`, `rust:slow` et `rust:settings`. La CI les répartit sur ses tâches.
  Durées du 16 octobre 2026 (bb3a668, M1 Max au calme, `verrou chrono`) : `rust:static` prend 3,1 s.
  `rust:test` prend 52,1 s, dont 22 s de compilation des 12 crates du workspace.
  `rust:coverage` prend 519 s, dont 28 s de compilation instrumentée et 253 s pour les tests unitaires de `batdetect` sous instrumentation.
  Sans recompilation, `check` prend 19,5 s réels et 57 s de CPU. Le hook pre-push de `lefthook.yml` le lance et prend 19,8 s.
- Un test rapide peut relancer son propre binaire sur un test `#[ignore = "run by …"]` pour lire ce qu'il écrit sur stderr (par exemple aucun message du décodeur pendant la lecture).
- La couverture se mesure sur les tests rapides en profil `ci`, avec un seuil de 95 %.
  Les lignes qui restent hors couverture sont des erreurs de FFmpeg qu'aucune vidéo de test ne provoque.
- Les tests de mémoire sont des tests `slow` hors de la couverture.
  Sous l'instrumentation, le premier prenait 83 s sous Linux et 55 s sous macOS (PR #145) contre 1 s en profil `ci` sur le M1 Max.
  Il ne couvrait pas une ligne de plus. Ils tournent sur les trois systèmes par `rust:slow`.
- Le test du détecteur à 61 et 81 échantillons de fond tourne sur une image de 144×24 (24 blocs de 64 pixels).
  Il garde les mêmes demi-fenêtres, fils et lots qu'à 360×180. Il prend 7 s sous la couverture sur le M1 Max, contre 146 s.
  Sous l'instrumentation, chaque voie du tri de `Network::order` incrémente un compteur atomique partagé par tous les fils (mesuré sur le Mac par `sample` et désassemblage).
  Le tri n'est plus vectorisé et les fils se disputent le compteur.
- Le niveau `reference` passe la 092 deux fois en parallèle : aux défauts et à 1440 pixels de travail (`compare`).
  Aux défauts, la détection passe par `engine::detection::detect`, puis `batdetect::process::process_detected` fait le suivi, les extraits et le zoom.
  - L'initialisation du binaire `cli/tests/reference_092.rs` lance les deux passages sur deux fils et range leurs résultats dans un `OnceLock`.
    Les cinq tests les relisent : la commande, la concordance aux défauts, la lecture, le décodage et la concordance à 1440.
    La lecture vérifie d'après ffprobe les horodatages, les images-clés et l'absence de plage abîmée.
    Le décodage vérifie la cadence, la taille, la plage et l'absence de paquet illisible.
  - Le parallèle se fait dans l'initialisation et non par libtest.
    Ce dernier lance les tests dans l'ordre de leur nom, 4 à la fois sur le runner. Celui de la 1440 viendrait en dernier.
  - Une erreur est rangée avec ses causes et une panique avec son message.
    Un `OnceLock` dont l'initialisation panique la relancerait pour chaque test en attente.
    La détection aux défauts, la commande qui la suit et la 1440 ont chacune leur résultat.
    Un rendu qui échoue ne fait donc tomber que le test de la commande.
  - Un test lancé seul par filtre fait quand même les deux passages : 66 s sur le M1 Max.
    Le binaire entier prend 66,0 s et 429 s de CPU, pour un pic de 1,65 Go. Avec 4 fils de test, il prend 70,6 s.
  - La concordance aux défauts vérifie d'abord que les réglages de l'enregistrement sont ceux de `DetectConfig::default()` : c'est la même détection que celle de la commande.
  - La 1440 ne touche pas `cli`. Elle est dans ce binaire pour partager le processus. De là vient la dépendance de test `cli` → `compare`.
  - Le décodage par le `Decoder` seul de `media` et la lecture du moteur ne se recoupent que sur l'extrait (`slow_decode`, `slow_input`) et pas sur la 092.
- Les fixtures se trouvent par `reference::fixture::fixture`. Cette fonction refuse un pointeur LFS non tiré avec le conseil `git lfs pull`.
- Les tests `local_` de `bench` lisent les bancs de la v0.1.1 (`BATDETECT_REFERENCE_DIR/bench/`).
  `local_injection` tourne sans vidéo en moins d'une seconde.
  `local_guard` rejoue les deux bancs sur la 092 du dépôt (LFS) sans recollage ni portée ([Garde](#garde)).
  Il prend 1 min 54 en profil `ci`. `committed_metrics` (#277) lit les deux bancs de la v0.2.0 qui sont dans le dépôt.
  Il tourne en CI sur les trois systèmes (0,07 s sur lumiel).
- `tests/fixtures/rust/video_092_original_3m24-4m05/frames.jsonl` : les détections de l'extrait par le Rust.
  Elles sont écrites sur le Mac depuis la racine du dépôt par `verrou lourd cargo run --profile ci -p compare --bin compare -- extract --frames tests/fixtures/rust`.
  C'est un fichier git ordinaire, avec une fin de ligne `\n` forcée par `.gitattributes`.
  Le test `slow_extract` de `compare` les redemande au bit près sur macOS, Linux et Windows.
  Un écart remonte à l'utilisateur (voir `compare`).
- `tests/fixtures/rust/video_092_original_3m24-4m05/tracks.csv` : le tableau des pistes de l'extrait.
  `check.sh` (C6) le contrôle à l'octet sur les trois systèmes.
  - Il se refait en anglais depuis la racine du dépôt par le binaire `release` : `verrou lourd env -u LANGUAGE -u LC_ALL -u LC_MESSAGES LANG=C target/release/batdetect tests/fixtures/video_092_original_3m24-4m05.mp4 -o out/csv`.
    On copie ensuite `out/csv/video_092_original_3m24-4m05/video_092_original_3m24-4m05_tracks.csv` à sa place.
  - Ses lignes finissent en CRLF comme tout tableau produit. `.gitattributes` le marque `-text` pour que git garde ces octets tels quels.
  - Ses valeurs diffèrent de la référence v0.1.1 sur trois cellules : le chemin de la piste 1 (912.3 au lieu de 912.7) et l'amplitude des pistes 1 et 3 (110.2 au lieu de 110.3, 99.0 au lieu de 99.1).
- Cinq petites fixtures hors LFS sont faites une fois par le ffmpeg 9.0.2 de Homebrew (libx264) et commitées :
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
    `tests/full_range.rs` vérifie que chaque image de l'un est lue en `Limited` ou `Unspecified` et chaque image de l'autre en `Full`.
    Il vérifie ensuite que leurs détections ne sont pas vides et qu'elles sont identiques au bit près.
- **CI** : sur macOS et Linux, deux tâches tournent en parallèle (#179). `rust` fait le statique et les tests rapides sous couverture.
  `rust-slow` fait les tests `slow`, les contrôles du binaire et l'essai matériel sous macOS.
  Elles ne partagent aucune compilation : la couverture compile dans `target/llvm-cov-target`, les tests lents et le binaire dans `target/ci`.
  - `rust` ne tire aucune vidéo LFS. Aucun test rapide n'en lit.
  - Deux autres tâches tournent sous Linux en parallèle : `rust-reference` (#146) et `rust-settings` (#167).
    `rust-settings` joue les 16 cas du banc d'essai.
    Les réglages ne tournent ni sous macOS ni sous Windows. `slow_extract` y garde l'identité au bit près sur les trois systèmes.
  - Quand l'empreinte de FFmpeg change, `rust-slow`, `rust-reference` et `rust-settings` le construisent aussi sans le sauver.
    Seule `rust` sauve le cache de FFmpeg. Ce cache est restauré, puis sauvé dès la construction même si la suite échoue.
  - Le binaire contrôlé se construit par `--workspace --bin batdetect`.
    Avec `-p batdetect`, les features de `syn` changent (`draw` en ajoute par `skrifa`).
    `bindgen`, `ffmpeg-sys-next` et `media` se recompilaient alors (11 s sous Linux, 22 s sous macOS, 60 s sous Windows à la PR #145).
  - L'action `setup` installe la toolchain, les outils cargo et le Python de `rust:rules`.
  - Deux actions composites servent à chaque tâche (#181).
    `.github/actions/ffmpeg-key` pose `XDG_CACHE_HOME` et calcule les clés des caches de FFmpeg et de `target/` par un seul script : `key.sh`.
    Il tourne sous `bash` ou sous MSYS2 sur Windows.
    `.github/actions/windows` met en place MSYS2, MSVC et rustup. Elle renomme le `link.exe` de coreutils.
- **Tâches `rust-windows` et `rust-windows-slow`** : elles sont indépendantes des autres et parallèles (#179).
  Quand l'empreinte de FFmpeg change, chacune le construit. Seule `rust-windows` le sauve.
  `rust-windows` lance les tests rapides, contrôle le binaire, puis `cargo clippy --profile ci -- -D warnings`.
  `rust-windows-slow` lance les tests `slow` (sans les réglages).
  - En profil `ci`, clippy reprend les scripts de construction et les macros des tests.
    En `dev`, il recompilait 46 unités, dont les liaisons de FFmpeg (82 s à la PR #145).
  - `dumpbin -dependents` n'admet que des DLL de Windows : `kernel32`, `ntdll`, `bcrypt`, `bcryptprimitives` et `api-ms-win-core-*`.
    C'est une liste blanche. `kernel32` doit y être pour que le contrôle ne puisse pas passer à vide.
  - Ni couverture ni `cargo deny` ni `mise` : ils tournent déjà sous Unix.
  - clippy vient en dernier pour qu'un échec propre à Windows laisse lisibles les résultats des tests.
  - Seule `rust-windows-slow` tire l'extrait. Son cache LFS se reprend depuis Linux (`enableCrossOsArchive`).
  - En cas d'échec, `config.log` de FFmpeg est téléversé en artefact (`ffmpeg-windows-logs` ou `ffmpeg-windows-slow-logs`).
- **Cache de `target/`** (`actions/cache`, une entrée par tâche) :
  - Il contient tout `target/` de sa tâche.
    Sous Unix, `rust` garde `debug` (clippy) et `target/llvm-cov-target` (couverture). `rust-slow` garde `ci` (tests lents, binaire).
    Sous Windows, les deux tâches gardent `ci`. Chacune garde la rlib de ffmpeg-sys-next avec ses liaisons.
  - La clé commence par le nom de la tâche.
    `rust` et `rust-slow` ont donc des entrées distinctes, comme `rust-windows` et `rust-windows-slow`.
    Le repli de l'une ne prend jamais l'entrée de l'autre.
    L'entrée de `rust` sur `main` garde `target/ci` jusqu'au prochain `Cargo.lock` parce qu'une clé trouvée telle quelle ne se resauve pas.
  - Il se sauve en fin de tâche réussie.
  - Sa clé porte dans cet ordre : la tâche, l'OS, l'architecture, l'image du runner `ImageOS`, **l'empreinte de FFmpeg**, `rust-toolchain.toml`, puis `Cargo.lock`.
    La clé se calcule dans l'action `ffmpeg-key`. La tâche y est celle qui appelle, sauf `rust-settings` qui passe `job: rust-reference`.
    Cargo ne suit pas les `.a`.
    Un `target/` construit avec un autre FFmpeg garderait donc des binaires déjà liés à l'ancien et les liaisons de bindgen tirées de ses en-têtes (voir « Reconstruction »).
  - En PR seulement, une clé de repli (`restore-keys`, #146) prend tout sauf `Cargo.lock`.
    Une PR qui change une dépendance reprend ainsi le `target/` de `main` pour la même image, le même FFmpeg et la même toolchain.
    Elle ne recompile que ce qui a changé. La clé de repli ne mélange jamais deux empreintes de FFmpeg.
  - Sur `main`, pas de repli : un nouveau `Cargo.lock` repart d'un `target/` vide.
    Un `target/` repris par le repli garde les artefacts des versions remplacées et grossirait sans cette remise à zéro à chaque changement de dépendance.
    Tailles à la PR #145 : 393 Mo pour `rust` sous Linux, 366 Mo sous macOS, 470 Mo sous Windows et 101 Mo pour `rust-reference`.
  - `rust-settings` ne sauve rien.
    Elle restaure l'entrée de `rust-reference`, qui a la même clé et a déjà compilé les dépendances en profil `ci`.
    Le quota du cache (10 Go) ne gagne aucune entrée.

### Tests figés

Six cas d'extraits et de zooms ont été jugés à l'aveugle par l'utilisateur le 15 octobre 2026 (sur f364698, en `release`, `LANG=fr_FR.UTF-8`) contre les sorties de la v0.1.1, puis figés.
Chaque cas avait deux fichiers, A et B tirés au sort. Ils étaient ouverts l'un après l'autre en plein écran dans QuickTime.
Une seule question était posée : « la cible se voit-elle aussi bien ? ».
Le texte dessiné (`#N`, Roboto contre la police d'OpenCV) trahit la provenance et n'était pas à juger.
Les six verdicts sont « aussi bien ». L'utilisateur voit peu ou pas de différence entre A et B.

| Cas | Vidéo | Piste (début v0.1.1) | Fichier |
|---|---|---|---|
| bordure avec zoom | 125 | 0:28.41 | zoom |
| passage rapide (1 926 px/s) | 092 | 3:48.29 | extrait |
| piste courte (6 images) | 097 | 1:54.63 | extrait |
| piste lente | 092 | 3:29.61 | extrait |
| boîte interpolée (30 images) | 097 | 1:48.54 | extrait |
| zoom ×4 | 125 | 0:47.12 | zoom |

Deux images résumé ont été jugées à découvert et côte à côte, sans rien à redire : la 097 (plages abîmées et instables) et la 089-091 (plages instables dans la 089 et la 091).

VideoToolbox n'est pas reproductible d'une machine à l'autre. Les cas validés deviennent donc des empreintes, jamais celle d'un mp4 :

- **Extraits et zooms** : les images dessinées que `render` pousserait à l'encodeur (`YuvFrame`, boîtes, traces et numéros compris).
  `render::drawn::drawn` (feature `testing`) les récupère sans encodeur.
  Chaque fichier a une empreinte `of_sequence` avec son nombre d'images.
  Les pistes sont celles du Rust et non celles de la référence. Ce sont elles que l'utilisateur a jugées.
  Elles viennent de `batdetect::process::tracked` (timeline, exclusion, suivi).
  `process_detected` l'appelle aussi : un changement de la commande entre la détection et le rendu se voit dans ces tests.
  - `cli/tests/slow_frozen.rs` tourne sur l'extrait LFS dans la CI des trois systèmes.
    Il couvre la piste lente (380 images) et le passage rapide (108 images).
    Sur l'extrait, elles portent les numéros `#1` et `#2` au lieu de `#11` et `#12`. Seuls les numéros diffèrent des images jugées.
    La CI de la PR #208 a montré les mêmes empreintes sous macOS, Linux et Windows : tiny-skia (avec SIMD) et swscale (`BITEXACT`) dessinent au pixel près d'un système à l'autre.
  - `cli/tests/local_frozen.rs` couvre les six cas sur leurs vidéos entières. Il ne tourne jamais en CI (vidéos hors dépôt).
    Trois ont été refigés après une planche avant/après jugée par l'utilisateur le 17 octobre 2026.
    Le zoom de la 125 à 0:47.12 et les deux extraits de la 097 (1:48.52 et 1:54.62) gardent leurs pistes et leurs débuts avec les détections faibles (#21).
    Ils gagnent des détections faibles à la place de positions estimées.
    Avec le recollage, 9 pistes de moins commencent avant ces deux extraits de la 097 (#24).
    Leurs numéros passent de #62 à #53 et de #67 à #58. Deux voisines changent de couleur.
    Les points, les boîtes et les traces ne changent pas.
- **Images résumé** : les pixels de chaque PNG écrit par `batdetect` en français (`of_pixels`) et le fond de chaque vidéo (`_fond.png`).
  Le test porte sur la 097 et la 089-091 et vit dans `cli/tests/local_frozen.rs`.
  Ce test ne peut pas vivre dans `summary` parce que le fond est la médiane calculée par la détection et que `summary` n'a ni FFmpeg ni moteur.
  Les images résumé ont été refigées le 18 octobre 2026 après #21, #311 et #326.
  La 097 tient en sept images, la tranche 0-10 min de la 089-091 en une.
  Les images de la 097 ont une empreinte par système. Vers 182 s, Linux voit une image abîmée de plus que macOS.
  La plage non analysée finit à 182,15 s au lieu de 181,68 s. Ses pistes et son fond sont identiques.
  La 089-091 est dans le seau (README de la référence).
- Un changement voulu du dessin, de la détection ou du suivi casse ces empreintes.
  On les refige en le disant, après un nouveau jugement à l'œil si le dessin a changé.

## Encodage

Sur le Mac, les extraits et la vidéo annotée passent par VideoToolbox matériel en q65. Le repli est openh264 en QP 23.
Ailleurs, ils passent par openh264 seul.
Les zooms passent toujours par openh264. Aucun de ces réglages n'est une option (décision de l'utilisateur, 11 octobre 2026).

### Choix des encodeurs

Mesure du 11 octobre 2026 sur le M1 Max, sous le verrou de mesure (#111). Elle passe par un programme Rust jetable lié au même FFmpeg.
Le programme est hors dépôt dans `~/Projects/dvr-wt/rapports/mesure-111/`, avec les scripts `one.sh` et `xpc.sh`, les journaux et `RESULTATS.md`.
Le SSIM vient du filtre `ssim` du ffmpeg 9.0.2 de Homebrew. Chaque sortie est comparée à l'extrait d'origine (1 230 images).
Les temps sont la médiane de 3 passages dont on retranche le décodage seul (6,98 s).

| Encodeur | Réglage | SSIM All (Y) | Taille | Encodage | CPU | Mémoire par encodeur |
|---|---|---|---|---|---|---|
| VideoToolbox matériel | q65, `allow_sw=0` | 0,9792 (0,9688) | 105,4 Mo | +0,4 s | ≈ 0 | ≈ 180 Mo |
| Logiciel d'Apple | `require_sw=1`, 20 Mb/s | 0,9809 (0,9714) | 105,5 Mo | 6,5 s | 54,8 s | ≈ 495 Mo |
| openh264 | profil high, QP 23 | 0,9799 (0,9698) | 136,9 Mo | 16,8 s | 16,3 s | ≈ 16 Mo |

- La cible d'openh264 est le SSIM de VideoToolbox q65 remesuré dans le moteur Rust (0,9792). Le plancher est 0,977.
- QP 24 donne 0,9761. Pour égaler QP 23 en débit, il faut 28 Mb/s. Le fichier pèse alors 143 Mo.
- L'utilisateur a validé QP 23 à l'œil le 11 octobre 2026 (« Franchement c'est pareil ») sur la piste 0:23.48 de l'extrait.
  C'est la moins contrastée (17 points, amplitude 68,7, images 660 à 767).
  Elle pèse 9,3 Mo en VideoToolbox q65 et 12,1 Mo en openh264 QP 23.
- openh264 sert de repli plutôt que le logiciel d'Apple : il prend 30 fois moins de mémoire et 3 fois moins de CPU.
  C'est aussi le même encodeur que sous Linux, donc testé partout. Le prix : des fichiers 30 % plus gros.
- Le logiciel d'Apple refuse q65 : l'ouverture échoue avec `-12900` (propriété `Quality` non gérée). Il ne marche donc qu'en débit fixé.
- **Couleur** : plage limitée et BT.709 pour la matrice, le transfert et les primaires (décision du 11 octobre 2026).
  Les deux encodeurs les écrivent dans le flux H.264. On suppose toutes les sources en BT.709. C'est vérifié sur les 27 vidéos connues.

### Pièges de FFmpeg 7.1.1

- **`allow_sw` ne choisit pas le logiciel d'Apple** : `allow_sw=1` prend le matériel s'il répond et `allow_sw=0` exige le matériel.
  Seul `require_sw=1` force le logiciel.
- **q65 dans le processus** : `global_quality = 65 × FF_QP2LAMBDA` plus le drapeau `QSCALE`. Ce réglage n'existe que sur arm64.
- **Profil d'openh264** : sans réglage, il encode en Constrained Baseline (CAVLC).
  Le profil high (CABAC) donne le même SSIM et un fichier 8 % plus petit. Il encode en revanche 38 % plus lentement (16,8 s contre 12,2 s).
- **QP d'openh264** : `rc_mode=off` laisse le QP à 26 quel que soit `qmin`/`qmax`.
  `rc_mode=quality` avec `qmin = qmax` applique au contraire le QP aux images I et P (vérifié par `trace_headers`).
- **Fils d'openh264** : ils ne changent rien avec une tranche par image (même fichier à l'octet, même temps).
  Le parallélisme vient de plusieurs extraits encodés à la fois.
- **Options inconnues** : `open_with` les ignore sans erreur. Les tests relisent donc le profil et le PSNR.

### Essai VideoToolbox

`Codec::VideoToolbox` est le matériel seul car le logiciel d'Apple refuse q65 (#111, décision de l'utilisateur du 11 octobre 2026).
Les tests rapides de VideoToolbox sur macOS passent donc aussi par le matériel.

- L'étape matérielle de la CI macOS encode 120 images de l'extrait (images 660 à 779) au réglage de production : q65 sans repli logiciel.
  Elle les relit ensuite.
  Elle vérifie le nombre d'images, la taille, la cadence 10000/333 exacte, la plage limitée et un PSNR de luminance supérieur à 37,9 dB.
  Sur le M1 Max, on mesure 38,9 dB en q65 et 34,9 dB sans q65.
- Cette étape bloque la CI (décision de l'utilisateur, 11 octobre 2026).
  Si elle casse à cause d'une machine de GitHub plutôt que du code, la question se rouvre avec l'utilisateur.
- Sur le runner `macos-latest`, le journal affiche « IOServiceMatchingfailed for: AppleM2ScalerParavirtDriver » avant le test.
  Le message n'a pas d'effet sur le résultat.
- Le test lent d'openh264 relit le même passage avec un seuil de 39,1 dB. Sur le M1 Max, on mesure 39,53 dB à QP 23 et 38,69 dB à QP 24.
  Placé à mi-chemin, le seuil rejette un QP qui ne serait pas appliqué.
  Il passe aussi sur l'openh264 x86 dans `rust-slow` sous Linux et dans `rust-windows-slow` (CI de `main` du 18 octobre 2026, ede141a).

### Plusieurs encodeurs à la fois

Mesuré le 11 octobre 2026 avec K encodeurs de 300 images chacun.

- **Mémoire** : le pic du processus plus l'empreinte du service `VTEncoderXPCService` où tourne VideoToolbox.
  openh264 prend environ 16 Mo par encodeur. Le matériel prend environ 130 Mo dans le service et 48 Mo dans le processus.
  Le logiciel d'Apple prend environ 480 Mo dans le service et 15 Mo dans le processus.
  `Codec::footprint` vaut 17 Mo pour openh264 (16,3 Mo mesurés, arrondis au-dessus) et 180 Mo pour VideoToolbox.
  Le matériel fait aussi monter la mémoire câblée du système de +0,7 à +3,4 Go selon K.
  La mesure est trop bruitée pour l'attribuer. Elle n'entre pas dans `Codec::footprint`.
- **Sessions matérielles** : 17 tiennent sur le M1 Max.
  À partir de 18, toutes s'ouvrent puis l'envoi d'images échoue (`-17691`). Le repli à l'ouverture ne couvre pas ce cas.
  Les extraits ouvrent donc au plus 8 sessions. Le seuil de la machine cible n'est pas mesuré.
- **Charge réelle** : la 125 (47 pistes, 39 zooms) n'a jamais plus de 8 fenêtres ouvertes à la fois.
- **Budget** : repère de 4 Gio pour l'ensemble des encodeurs ouverts.

### Dans les extraits

- **Un encodeur par groupe** (`media::encode::fanout::Fanout`, nuit du 12 au 13 octobre 2026).
  Chaque image dessinée d'un groupe est encodée une seule fois, sans images B.
  Ses paquets sont écrits dans chaque extrait ouvert, avec des `pts` rebasés à 0.
  Une image-clé est forcée sur la première image de chaque extrait (`AVFrame.pict_type = I`, que VideoToolbox et openh264 honorent).
  Un extrait se ferme (en-queue, renommage) dès que le paquet de sa dernière image arrive.
  - Gardes : le premier paquet de chaque extrait doit porter le drapeau clé (`NoKeyFrame` sinon).
    Un extrait doit avoir reçu toutes ses images (`ShortClip`). Un paquet sans `pts` est une erreur.
  - Vérifié par `ffprobe` sur les 117 extraits de la 099 : tous commencent par une image-clé (`key_frame=1`) et n'ont pas d'image B.
    Les tests vérifient que chaque extrait se décode seul.
  - Sans lui, une image commune à plusieurs extraits d'un groupe était encodée une fois par extrait : 16 016 envois pour 6 616 images distinctes sur la 099.
    Avec lui et le décodage à plusieurs fils, la commande passe de 58,68 s à 26,94 s sur la 099 (M1 Max, trois passages).
- **La limite est le débit du codeur matériel.**
  Pendant les extraits, VideoToolbox encode de 322 à 378 images 1440×1080 par seconde, toutes sessions confondues.
  Le temps des extraits suit « images envoyées à VideoToolbox / environ 350 ».
  En encodeur seul à 1440×1080, il fait 224 images par seconde avec une session et 366 à 392 avec 2, 4 ou 8.
  Le M1 Max a deux moteurs d'encodage saturés dès deux sessions. Les options `prio_speed` et `realtime` n'aident pas.
- **Sessions** : une session VideoToolbox par groupe, au plus 8 à la fois.
  Avec un groupe par fil sur la moitié des cœurs, ce plafond n'est atteint qu'au-delà de 16 cœurs.
- **Mémoire** : un encodeur par groupe en cours (un par fil) plus les images en vol et le cache de zones de chaque fil (borné à 4 millions de pixels, LRU).
  Mesurée sur la 099 : 1,04 Gio pour le processus avec le décodage à plusieurs fils, 0,81 Gio sans.
  Processus et service VideoToolbox restaient ensemble sous 2,2 Go sur la commande complète du 12 octobre 2026.
  Le service se mesure par la somme des `phys_footprint` des `VTEncoderXPCService` relevée chaque seconde, moins les 57 Mo des services ouverts au repos.
  Le chiffre est la somme des deux maxima, qui ne tombent pas forcément au même instant.
- **Image-clé d'openh264** : une par seconde environ (`round(cadence)` images). Un lecteur se place ainsi vite dans l'extrait.
  VideoToolbox garde son réglage.
- **Extrait refait** : quand VideoToolbox casse en cours d'écriture, le groupe est repris en openh264 (voir `render`).
  Aucun échec réel n'a été vu sur les vidéos mesurées. La reprise est exercée par une fausse ouverture dans les tests.
- **Droits** : chaque extrait est créé par `File::create` sous un nom temporaire, puis renommé.
  Comme le CSV et le PNG, il a donc les droits habituels de l'utilisateur (umask).

### Zooms en openh264

Les zooms ne prennent aucune session VideoToolbox et occupent les cœurs libres.
Mesure au calme (13 octobre 2026, 3 passages en alternance, `release`, zoom `auto`) contre une variante temporaire tout en VideoToolbox, zooms compris :

| Vidéo | Zooms openh264 | Tout VideoToolbox | CPU (openh264 / VT) | Cœurs occupés pendant le rendu |
| --- | --- | --- | --- | --- |
| 092 (1 zoom) | 18,38 s | 19,09 s | 130 s / 129 s | 3,5 / 3,7 |
| 099 (46 zooms) | 39,79 s | 43,81 s | 304 s / 231 s | 7,7 / 4,0 |

Les cœurs occupés sont le temps CPU du processus relevé chaque seconde après la détection.
La détection prend les 14 premières secondes, avec environ 8 cœurs dans les deux cas.
Sur la 099, openh264 prend 73 s de CPU de plus sur des cœurs qui restaient libres.
Il rend la commande 9 % plus courte : VideoToolbox plafonnait, partagé entre extraits et zooms.

## Plage réelle des sources

Mesure du 11 octobre 2026 (#101) sur 20 images par vidéo.
Elles sont réparties hors des plages abîmées de la référence avec une marge de 15 images (programme et table brute : `~/Projects/dvr-wt/rapports/mesure-101-plage/`, `plage.tsv`).
Les 26 vidéos annoncent une plage limitée (`tv`) sur leur conteneur comme sur chaque image.

| Vidéos | Y hors 16..235 | Pire image | U/V hors 16..240 | Extrêmes |
|---|---|---|---|---|
| 089 à 103 | 0,92 à 1,96 % | 7,81 % (094) | 0,020 à 0,035 % | Y 0-255, U 68-146, V 98-255 |
| 120 à 130 | 0,41 à 0,62 % | 0,75 % | 0,031 à 0,033 % | Y 0-255 (0-252 pour la 120), U 75-147, V 113-255 |

- Sur l'image 716 de l'extrait, les pixels hors plage sont le contenu thermique lui-même.
  Y < 16 est surtout en haut de l'image (arbres et ciel froids) et Y > 235 surtout en bas (sol chaud).
  V à 255 n'occupe qu'un petit carré. C'est vraisemblablement une icône rouge de l'affichage des jumelles.
- **Décision de l'utilisateur** : pour l'affichage, les sources se lisent en plage pleine quelle que soit leur étiquette.
  Les fichiers produits restent en plage limitée BT.709 (#111). Ils sont resserrés sans écrêtage.
- **Noir calé sur 16** (décision de l'utilisateur du 12 octobre 2026, à l'œil sur les fonds de la 092, de la 122 et de la 124, #104).
  - Lu en plage pleine, le ciel froid autour de Y = 16 sort gris et non noir, comme un voile.
    Le fond de l'image résumé de la 092 a un plancher à 10 (1er centile, après l'assombrissement à 0,6).
    Il n'a aucun pixel à 2 ou moins, contre 26,6 % en v0.1.1.
  - Trois variantes ont été comparées : plage pleine, plage limitée (comme la v0.1.1, blancs écrêtés au-dessus de Y = 235) et noir calé sur 16 sans écrêter les blancs.
  - Retenue : `Y' = max(0, Y − 16) × 255 / 239`, arrondi au plus proche.
    Elle s'applique avant la conversion en plage pleine et ne touche pas la chrominance.
    Y = 16 et en dessous deviennent noirs et Y = 255 reste blanc. Les zones chaudes gardent leurs nuances.
  - Fond de la 092 : plancher 0 et médiane 75 (80 en plage pleine, 79 en v0.1.1). Il a 24,8 % de pixels à 2 ou moins (26,6 % en v0.1.1).
  - Ce réglage ne touche que l'affichage : la détection lit Y brut.
  - Dans les extraits, l'image passe par les tables d'affichage (`Converter::display`) hors des zones dessinées.
    Dans les zones, elle passe par `FullBlackAt16`, le RGBA et le retour en plage limitée BT.709 (voir `render`).
    Un rendu en plage pleine ferait tomber le SSIM des pixels hors dessins à 0,917 environ au lieu de 0,966 (`slow_clips`, essai fait une fois avant les zones).

## Crates

La carte courte des crates et de leurs dépendances est dans [CLAUDE.md](../CLAUDE.md).
Pour chaque crate, cette section ne garde que ce que le code ne dit pas : contrats, pièges, raisons d'un choix et mesures qui le justifient.
Le workspace compte treize crates (`members = ["crates/*"]`). Le mot `unsafe` n'est permis que dans `media`.

### `types`

La crate tient le vocabulaire commun. Elle n'a ni entrée-sortie ni algorithme.
Un type n'y entre que s'il est produit par une crate et lu par au moins deux autres.

- `FrameIndex` compte les images à partir de 0 dans l'ordre où le décodeur les rend. C'est l'ordre d'affichage.
  `Pts` compte des ticks de la base de temps. `FrameRate` a son propre type pour ne pas s'échanger avec la base de temps.
- `Timeline` donne deux temps. Le temps par les `pts` est celui de la commande.
  Le temps par la cadence (numéro ÷ `avg_frame_rate`) est celui du CSV de la v0.1.1.
- `FrameDetections::push` est la seule façon de remplir la liste, tests compris.
  Chaque détection prend le rang de l'image qu'elle remplit : ni le détecteur ni la lecture de `reference` ne peuvent la ranger sous une autre image.
- Les constructeurs dérivés de `Span` (`single`, `hull`, `widened`, `intersection`) ne peuvent pas échouer.
  Les plages instables et abîmées se bâtissent ainsi sans variante d'erreur.
- `span::merged` et `merged_by` (#302) sont la seule fusion de plages : deux plages qui se recouvrent ou se touchent (l'une commence l'image après la fin de l'autre) n'en font qu'une.
  Les plages abîmées d'`engine`, les groupes d'extraits de `render` et la réunion des plages de `compare` passent par elles.
- Une piste recollée garde ses bouts (#311). `Stretch::Part` marque une piste gardée et `Stretch::Piece` un morceau qui relie ou prolonge.
  `parts()` ne rend que les pistes gardées : ce sont celles que juge le zoom.
- `WorkScale` fait seul le passage des pixels de travail aux pixels d'origine (règle 14).
  Une coordonnée devient `(c + 0,5) × échelle − 0,5`, une longueur × échelle et une surface × échelle².

### `media`

La crate est la frontière avec FFmpeg : aucun type de `ffmpeg_next` n'en sort. Chaque module a son erreur.

**Erreurs.**

- `LibraryError::io_kind()` (#64) lit le genre d'erreur système derrière une erreur d'FFmpeg.
  Sa table (ENOENT, EACCES, EPERM, ENOSPC, EROFS) est unique et sans `cfg`.
  FFmpeg porte aussi sous Windows l'errno du CRT et non un code Win32. Cet errno a les mêmes valeurs.
- `decode::Error::NoDecoder` est séparé de `Open` pour qu'un `.mov` en ProRes ne passe pas pour un fichier illisible.

**Décodage** (`decode`).

- `ffmpeg_next::format::input` ferait `to_str().unwrap()` sur le chemin (#227).
  Le chemin passe donc en octets sous Unix puis par `avformat_open_input`. Une vidéo au nom non UTF-8 s'ouvre ainsi sous Linux.
  `UnusablePath` reste pour un NUL. Sous Windows, il sert aussi pour un nom qui n'est pas de l'Unicode valide.
- `decode_errors` lit les `decode_error_flags` tels quels.
  Sans son cfg `ffmpeg_5_1`, le `DecodeError` de ffmpeg-next perd `CONCEALMENT_ACTIVE` et `DECODE_SLICES`.
  Une image est en erreur dès qu'un bit est posé, bit inconnu compris. Une tranche coupée donne 12 (dissimulation et tranches manquantes).
- `active_threads` dit ce que FFmpeg fait vraiment après l'ouverture : `Single`, `Frame(n)` ou `Slice(n)`.
  Un test exige plusieurs fils d'image en automatique sur une machine à plusieurs cœurs : un FFmpeg construit sans fils ferait échouer la CI.
- La couture des tests (`with_seam`) voit chaque paquet avant `send_packet`.
  Un paquet raccourci sans réécrire la longueur de son NAL est refusé en entier par le décodeur et compté illisible.
  Un paquet raccourci à 0 octet fait échouer `send_packet` (`EINVAL`).
- Un paquet illisible est compté, pas fatal. Une erreur rend le décodeur inutilisable.
- Fin de fichier : FFmpeg 7.1.1 rend l'erreur d'entrée-sortie du contexte (`pb->error` hors `EAGAIN`) à la place d'un `EOF` du démultiplexeur (`libavformat/demux.c:1530`).
  Elle sort en erreur `Read`. Une lecture tronquée sans erreur d'entrée-sortie donne moins d'images. La garde d'`engine` la refuse.
- Un `linesize` négatif rendrait faux le `data()` de ffmpeg-next (longueur `linesize as usize × hauteur`).
  Il est refusé (`FlippedFrame`) avant tout appel à `data()`. Le décodeur h264 n'en produit pas.
- Seuls `yuv420p` et `yuvj420p` sont acceptés. Le second se lit en plage pleine.
  La cadence est `avg_frame_rate`. Si elle est inconnue, c'est `r_frame_rate`.
- Placement (`decode::placed`, #110) : on se place sur la dernière image-clé à ou avant un `pts` que connaît l'index du conteneur.
  Cela vaut même après la fin du fichier.
  Après un placement, le compte des images rendues ne les numérote plus : les images placées n'ont qu'un `pts`.
  La cible passe aux microsecondes de FFmpeg en arrondissant vers le bas pour ne jamais dépasser l'image voulue.
  FFmpeg la ramène à la base de temps du flux à l'arrondi le plus proche. Elle ne dépasse donc pas l'image voulue.

**Encodage** (`encode`) avec les réglages et les mesures d'[Encodage](#encodage).

- Le fichier s'écrit sous un nom temporaire que `finish` renomme.
  Le ménage de `files` doit reconnaître ce temporaire. `media` ne peut pas l'appeler : c'est un test de `cli` qui l'exige.
- Une image en plage pleine est refusée : l'encodeur écrit une plage limitée BT.709.
- Sans réglage, openh264 n'écrit qu'une image-clé : la première.
- `open_stamped` (#103) garde les `pts` de la source au lieu de compter les images à la cadence de l'encodeur.
  Le flux prend la base de temps de la source (1/10000 sur les jumelles). Sur l'extrait, la durée égale celle de la source (40,959 s).
- Une écriture refusée (`Writing`) ne concerne pas la destination pour que la bascule vers openh264 joue.
  Un chemin refusé, un temporaire non créé ou un fichier non ouvert la concernent.
- `Fallback` essaie VideoToolbox matériel puis openh264 sur macOS. Ailleurs, il n'essaie qu'openh264.
  Au démarrage, `trial` retire en tête de liste les encodeurs qui échouent. À chaque extrait, `create` ouvre le premier qui répond.
  Une destination inutilisable rend tout de suite `Destination` sans essayer les encodeurs suivants.
  L'échec en cours d'écriture n'est pas couvert ici : `render` le reprend.
- **Ouverture bornée** (`bounded`, #313).
  Sur la CI macOS, une ouverture de session VideoToolbox a attendu sans fin la réponse du service (`xpc_connection_send_message_with_reply_sync`, PR #309).
  Le test matériel entier y prend pourtant de 5 à 7 s. L'ouverture (`avcodec_open2`) se fait donc sur un fil à part.
  On l'attend au plus 10 s (`VIDEOTOOLBOX_START_LIMIT`, décision de l'utilisateur).
  - Au-delà, elle rend `Unanswered`. L'essai comme l'extrait passent à openh264 comme pour un encodeur qui refuse.
  - Le silence se retient pour tout le processus : toute ouverture VideoToolbox suivante rend `Unanswered` aussitôt.
    L'essai des vidéos suivantes ne garde qu'openh264. Un service muet coûte ainsi au plus 10 s pour la soirée.
    Si plusieurs groupes ouvraient une session au même moment, il coûterait moins de 20 s.
    Un service lent qui répond avant 10 s n'est jamais déclaré muet : il coûte son attente à chaque ouverture. Ce coût n'a pas de plafond.
  - Le fil bloqué ne se tue pas. Il est détaché et reste dans son appel jusqu'à la fin du programme.
    Il coûte une pile réservée (2 Mio de mémoire virtuelle) et ce que VideoToolbox avait déjà alloué. Une réponse tardive est jetée.
  - Une panique pendant l'ouverture repasse au fil appelant (`resume_unwind`) pour ne pas passer pour un silence.
  - La borne ne couvre que l'ouverture.
    Deux autres appels attendent le service sans borne : la recréation de la session sur `kVTInvalidSessionErr` pendant `push` et la fermeture (`vtenc_close`).
    La fermeture attend les images en cours.
  - Les tests prouvent l'attente bornée sur un appel qui ne revient jamais, la mémoire du silence, la réponse tardive jetée et la panique repassée.
    `slow_videotoolbox_silent` rend muet le vrai `VIDEOTOOLBOX` dans son propre processus.
    Aucun test ne reproduit le blocage du service lui-même.
    La fin du programme avec un fil encore bloqué ne se jugera que sur macOS au prochain blocage.
    Un `atexit` de VideoToolbox qui attendrait la même connexion bloquerait après le bilan, Ctrl-C compris.

**Conversion** (`convert`, #101).

- BT.709 dans les deux sens (#111). La plage lue est un paramètre (`SourceRange`) et non l'étiquette de l'image.
  Les jumelles se lisent en plage pleine avec le noir calé sur 16 ([Plage réelle des sources](#plage-réelle-des-sources)).
  Le retour au YUV se fait toujours en plage limitée. La plage est resserrée sans écrêter : Y 0 devient 16 et Y 255 devient 235.
- Tables d'affichage (`display`, #112) : une table par plan évite le passage par le RGBA.
  Celle de Y est tirée de swscale sur une rampe de gris, noir calé sur 16 compris. Elle est donc égale à l'aller-retour sur tout pixel gris.
  Les tables se comparent à l'aller-retour par RGBA sur les 1 230 images de l'extrait (`slow_convert`).
  Les valeurs diffèrent pour 0,11 % en Y, 0,15 % en U et 0,11 % en V.
  Les écarts de plus de 2 niveaux touchent 0,02 %, 0,06 % et 0,04 % des valeurs. L'écart va au plus à 34 en Y, 18 en U et 51 en V.
  Le pire est un pixel de V à 255 : c'est l'icône rouge des jumelles. L'aller-retour l'écrête en RGB et la table le garde.
- Zones (#112) : la même conversion se fait sur un rectangle aux coordonnées paires par des contextes swscale à sa taille.
  Ils vivent dans un cache LRU de 32 tailles et 4 millions de pixels au plus.
  `render` arrondit la taille des zones au multiple de 16 pour que le cache serve.
- Filtres mesurés : « point » à la montée et « area » à la descente. La chrominance horizontale est pleine.
  Les aplats de la palette sont alignés sur des blocs 2×2. Ils font l'aller-retour à 2 niveaux au plus, contre 34 à 143 avec le bilinéaire.
- Image 716 de l'extrait sans dessin : son étiquette dit plage limitée. Lue ainsi, elle a 345 704 pixels en noir pur et 9 719 en blanc pur.
  Lue en plage pleine, elle en a 38 et 15 : ce sont exactement les pixels à Y 0 et Y 255 de la source.
  Réécrite en plage limitée puis relue, elle s'écarte en moyenne de 0,086. Les valeurs au-delà de 2 niveaux font 0,016 %.

**Pour les tests** (feature `testing`).

- `payload::wipe` écrase de `0xff` des parts égales de la boîte `mdat` d'un mp4.
  Les tests de `media`, d'`engine` et de `render` s'en servent pour fabriquer leurs vidéos abîmées.
- `fingerprint` (#154) est la seule empreinte d'image des tests.
  C'est un FNV-1a sur des mots de 64 bits écrit à la main. Chaque pas replie les bits hauts sur les bas.
  Sans ce repli, deux mots changés dans leur seul octet haut s'annulaient une fois sur 256.
  À la différence de `DefaultHasher`, il est le même sur chaque système et chaque version de Rust.
  Une empreinte peut donc s'enregistrer dans un test figé.

### `draw`

La crate fait du dessin pur sur une toile RGBA, sans FFmpeg ni entrée-sortie (#101).
Police et primitives ont été choisies sur une planche qui mettait l'image 716 de l'extrait à côté du rendu de la v0.1.1.
Roboto y était comparée à Noto Sans et à IBM Plex Sans.

- Le texte est en Roboto de graisse 400 et les numéros en 600 (choix de l'utilisateur, 11 octobre 2026).
  Une taille est la hauteur des chiffres en pixels. Les chiffres ont tous la même largeur.
- Le texte est normalisé en NFC. Un caractère absent devient U+FFFD. Il n'y a ni crénage ni placement d'accent par la table GPOS.
  Cela suffit pour des numéros et des légendes en caractères précomposés.
- Un texte se prépare une fois (`shape`) puis se redessine sans allocation. C'est le cas d'un `#N` à chaque image.
- La toile est toujours opaque. Ses octets sont donc du RGBA simple pour `media::convert`.
- Les primitives sont lissées sauf les rectangles (même planche).
  Un bord de 1 px couvre les pixels du bord de la boîte. Chaque pixel de plus l'élargit vers l'extérieur.
  La boîte de 2 px d'une image détectée et celle de 1 px d'une image interpolée gardent ainsi le même bord intérieur.
- L'assombrissement à 0,6 arrondit au plus proche alors que la v0.1.1 tronque. L'écart est d'un niveau au plus (règle 5).
- Un test épingle chacune des 12 couleurs de la palette au BGR inversé de `output/style.py` de la v0.1.1.
  Sur l'image résumé comme dans la page, le numéro des pastilles s'écrit en noir (`INK`) sur chaque couleur.
  Le contraste va de 6,94:1 (rouge 255 92 92) à 17,62:1 (gris clair).
  C'est au-dessus des 4,5:1 du texte en WCAG AA. Le blanc ne les atteint sur aucune (3,03:1 au plus).
  Le gris d'une piste « non » dans la page donne 6,49:1. Un test garde ce seuil si la palette change (#299).
- Les tailles du résumé suivent la largeur de l'image dans les proportions de la v0.1.1.
  Le rayon des pastilles est mesuré sur la nouvelle police.
  L'arrondi va à l'entier le plus loin de zéro (règle 5) : 1440 / 64 tombe pile sur 22,5, d'où un rayon 23 et des chiffres de 21 px en 1440.
  La v0.1.1 arrondissait au pair et donnait 22 et 20.

### `summary`

La crate dessine l'image résumé à raison d'une page par tranche de 10 minutes (#104).
Elle n'utilise pas FFmpeg : `cli` lui donne le fond déjà converti.
Les règles des couleurs (indice le moins employé autour) et des flèches (25° et 3 % de la longueur) sont celles de la v0.1.1.
Celles de la légende et des tranches aussi.
Les tris deviennent des clés entières. Les points ne sont pas arrondis avant le dessin (règle 5).

- `pages` rend aussi le fond (#259).
  Pour chaque piste dessinée, il rend son chemin, sa flèche, le centre de sa pastille et le départ de son trait de rappel.
  La page les redessine sans rien recalculer.
  Une pastille a un trait de rappel quand elle est à plus d'un rayon de la trajectoire (`Mark::leader`).
  C'est le seul critère, pour l'image comme pour la page.
- Les tests figés de `cli` prennent la couleur de chaque piste par `assigned::track_colors` (feature `testing`).
  C'est la même mise en page sans le dessin.
- Temps par les `pts` (#35) : la tranche et le temps de légende d'une piste viennent de sa première image.
  Les plages viennent de `spans_in_seconds`, comme `params.json`.
- Le bandeau est un modèle sans texte (`HeaderLine`). Le trait `Labels` le met en mots avec les libellés de la v0.1.1.
  - Les plages illisibles sont datées comme les plages hors analyse (#127).
    On écrit trois lignes, puis une dernière avec le nombre des autres et la durée illisible totale. Cette durée est arrondie au-dessus.
  - Les plages qui se touchent ou se recouvrent une fois arrondies à la seconde ne font qu'une ligne.
    Sur la 121, « 0:04 - 0:06 », « 0:05 - 0:08 » et « 0:07 - 0:09 » deviennent « 0:04 - 0:09 ».
    La frise et `params.json` gardent le détail exact.
  - Seule la ligne du nom de la vidéo reste plafonnée à 3 colonnes de légende. Les autres lignes sont bornées et élargissent le bandeau.
- **Frise du temps** (#127) : elle se place sous l'image et sur sa largeur.
  - La frise a deux bandes : « hors analyse » et « illisible ». Une bande vide est un cadre fin pour ne pas se lire comme une bande pleine.
  - Chaque plage fait au moins 2 px : les 87 plages de la 121 restent toutes visibles.
  - Les graduations vont au pas de 1, 2, 5 ou 10 minutes. Ces pas divisent la tranche : son début tombe toujours sur une graduation.
    Chaque piste a un repère à sa couleur au temps de sa première image (choix de l'utilisateur sur la planche).
  - Le bandeau reste à la hauteur de l'image. La page grandit de la hauteur de la frise (135 px en 1440).
  - Sur une image coupée, la frise couvre toute la tranche.
    Les départs des pistes de l'image sont en couleur et ceux des autres en gris (choix de l'utilisateur).
- **Images coupées au nombre de pistes** (#157) : une tranche de plus de 20 pistes (`MAX_TRACKS_PER_IMAGE`) donne `ceil(n / 20)` images.
  Chaque image prend un paquet égal de pistes consécutives par départ : 21 pistes donnent 11 + 10 et non 20 + 1.
  - L'utilisateur a choisi 20 sur une planche de la 097 et de la 099 à 15, 20 et 25.
    Sur la référence, 10 vidéos sur 26 ont une tranche de plus de 20 pistes.
  - Les images gardent les numéros de la vidéo entière, le même fond et la même mise en page.
    Couleurs et pastilles s'attribuent paquet par paquet pour que des voisines d'autres images ne retirent pas de couleurs à celle-ci (choix de l'utilisateur).
  - La plage de temps d'une partie va du premier départ à la dernière fin du paquet.
    Les plages hors analyse et illisibles sont celles de toute la tranche.
- Masques de l'affichage : les zones OSD sont peintes en gris de panneau avant l'assombrissement.
  `cli` les convertit sans marge sur la taille de la source, soit `[⌊x0·w⌋, ⌈x1·w⌉)` comme dans la v0.1.1.
  La liste reste vide tant que la commande n'a pas `--osd-region`.
- **Placement des pastilles** (choix de l'utilisateur, 12 octobre 2026) : aucune pastille n'est hors cadre et aucune n'en recouvre une autre.
  Chacune est au plus près de sa piste.
  - Les candidats se prennent tous les demi-rayons le long de la piste.
    Ils se classent par pastilles touchées, flèches touchées, croisements, puis écart au quart de la piste.
  - Une pastille qui touche une autre pastille s'écarte de sa piste par anneaux d'un demi-rayon (16 directions et 40 anneaux au plus).
    Un trait de rappel la relie alors au point le plus proche.
    Suivent jusqu'à 8 passes. À chacune, une pastille encore en collision est replacée si la nouvelle place touche moins que l'ancienne.
  - On compte les recouvrements (centres à moins de 2 rayons) sur les 966 pistes de la référence (26 vidéos).
    Il y en a 266 en v0.1.1, 349 pour un portage fidèle et 0 pour le placement retenu. Aucune pastille n'est hors cadre.
    121 pastilles s'écartent de leur piste de plus d'un rayon, presque toutes sur les vidéos denses. La plus éloignée est à 304 px (096).
- Les écarts voulus à la v0.1.1 sont le fond, la police et le rayon.
  Le fond est lu en plage pleine avec le noir calé sur 16 ([Plage réelle des sources](#plage-réelle-des-sources)).
  La police et le rayon sont ceux de `draw` (29 pour un numéro à trois chiffres contre 26).
  Avant le choix du noir calé sur 16, on a comparé le fond de la 092 à celui de la v0.1.1 sur la luminance hors traits (81 % des pixels).
  Le SSIM valait 0,702 tel quel en plage pleine. Il valait 0,984 une fois ramené en plage limitée (écart moyen 1,4 niveau).
- Le PNG s'écrit dans son partiel (`files::names::partial_file`, #245) puis il est renommé.
  Il garde les droits ordinaires d'un fichier créé : un temporaire de `tempfile` resterait en `0600`.
  `png` n'expose la cause d'une erreur d'écriture que par `cause()`.
  `Error::Write` garde donc l'`io::Error` pour qu'un disque plein ne se perde pas (#64).
- Tests : `tests/v011.rs` compare numéros, couleurs, tranches et en-tête aux attendus de la v0.1.1 sur l'extrait et la 092 ([Référence v0.1.1](#référence-v011)).
  `tests/local_v011.rs` fait de même sur les 26 vidéos avec 22 écarts listés.
  Par les `pts`, 13 temps de légende et une ligne « hors analyse » de la 096 diffèrent.
  La durée illisible totale de la 120 diffère aussi (1 min 14 s contre 1 min 13 s).
  Les autres écarts sont les plages hors analyse réunies à la seconde (089, 096, 098, 103).
  Les plages illisibles se comparent tranche par tranche sur leur nombre et leur durée totale.
  Une tranche coupée se compare pages réunies sur ses numéros et son en-tête.
  Ses couleurs sont attribuées par paquet et ne se comparent pas.

### `page`

La crate rend en HTML la page de résultats d'une soirée (#71). Elle n'a aucune dépendance : ni crate du workspace ni FFmpeg.
Elle ne lit pas le disque : `cli` lui donne un modèle et elle rend une chaîne.
Les verdicts se saisissent dans la page, qui les télécharge en `verdicts.csv` à côté d'elle (#228).
Ce fichier est lu en octets (`files::verdicts`) et batdetect ne l'écrit jamais.

- Les textes passent par le trait `Labels`. Le binaire le remplit avec les messages `page-*` des `.ftl`.
  Ceux dont le JS a besoin sont embarqués en JSON avec leurs trous écrits `{nom}`. Le JS remplit ces trous.
  Les en-têtes et le dialecte du fichier de verdicts (séparateur et virgule décimale) sont aussi des textes de la langue.
- Le texte est échappé (`& < > " '`).
  Les chemins sont encodés en pourcents segment par segment pour tout octet hors de `A-Z a-z 0-9 - . _ ~`.
  Un `#`, un `%` ou un `?` dans un nom reste ainsi un nom de fichier. Rien n'est normalisé : le lien suit l'octet du disque.
  Le JSON embarqué ne peut pas fermer le `<script>` qui le contient (`<`, `>` et `&` en `\u`).
- Limite (#227) : les noms arrivent au modèle en chaînes lossy.
  Pour une vidéo au nom non UTF-8 sous Linux, la page montre ses pistes mais ni l'image résumé ni la vidéo annotée.
  Les liens vers le tableau, les extraits et le dossier sont cassés.
- La marque `<meta name="generator" content="batdetect">` dit qu'une page est à nous. On la cherche dans le premier Kio.
  `cli` n'écrit ni ne retire une page sans elle.
  Les images ont `width` et `height` pour qu'un changement d'onglet ne fasse pas sauter la page.
- `assets/` contient `page.css` et onze scripts classiques.
  Les scripts sont inclus dans l'ordre dans un seul `<script>` parce qu'un `import` entre fichiers est bloqué en `file://` sous Chrome.
  La page tient en un fichier et se passe de réseau. Le lecteur de chaque vidéo a `preload="none"`.
  `csv.js` décode en UTF-8 strict, sinon en Windows-1252. `store.js` est le seul endroit où se construit le mot d'un verdict (#305).
- **Image redessinée** (#259) : c'est un `<svg>` en pixels de la vidéo.
  Le fond est en `<image>` et les calques suivent dans l'ordre de `summary`. La frise est en HTML.
  Sans fond ou sans mise en page, le panneau montre l'image complète.
  - Les traits gardent leur épaisseur à l'écran (`vector-effect: non-scaling-stroke`).
    Les pastilles et les numéros en police du système suivent l'échelle de la mise en page.
  - Les minutes de la frise sont dans un élément large d'une minute (`container-type: inline-size`).
    Des requêtes `@container` choisissent le pas selon la place et sans JS.
  - À 1180 px, la colonne verdict avait 82 px pour une puce de 78 px. Toutes les puces étaient coupées.
    La zone de jugement est donc un conteneur nommé.
    Sous 27rem, la colonne Zoom du tableau disparaît : la tête du jugement dit déjà si la piste a un zoom.
- **Traces cliquables** (#269) : un clic se ramène aux coordonnées de la vidéo (`getScreenCTM`) et retient les pistes à moins de 12 px d'écran.
  La plus proche passe d'abord.
  Si la piste jugée en fait partie, le clic prend la suivante. Un second clic au même endroit passe ainsi à l'autre trace sans état caché.
  Au clavier, les pastilles de la grande image sont des boutons à un seul arrêt de tabulation. Sans script, l'image reste une image.
- **Lecteur d'écran** (#338) : chaque temps lu a son double en mots.
  Un intervalle se dit « de 12 s à 31 s » parce que le tiret n'est souvent pas lu.
  Ces doubles (`.sr`) sont hors sélection (`user-select: none`) pour que copier une ligne ne colle que ce qui est affiché.
  Au-delà de 1024 px, la liste et les verdicts sans piste sont `position: relative` pour que les doubles défilent avec leur ligne.
  La frise reste `aria-hidden`.
- **Filtres et marques** (#268) : les boutons font 44 px et il y a un choix par vidéo. La piste jugée est toujours montrée.
  - Les signes d'un verdict sont dessinés en SVG parce que les glyphes ✓ ✕ manquent dans la police du système sous Linux.
  - Leurs couleurs sont fixes puisque la scène ne suit pas le thème.
    Elles sont choisies pour qu'au moins le signe ou son contour passe 3:1 sur la scène (ciel noir et roches).
  - Dans la frise, le départ d'une piste « non » pâlit au lieu de griser pour ne pas ressembler aux départs des autres parties.
    Deux départs proches se recouvrent : un clic mène à la piste la plus proche du pointeur dans 12 px.
- **Rattachement d'un verdict** (`store.js`) : les candidates sont les pistes de la même vidéo dont le début est à 0,2 s au plus.
  Les paires se classent par distance des points de départ quand les deux sont connus, sinon par écart de début.
  L'affectation est gloutonne : une piste pour un verdict.
  Un verdict sans piste libre est gardé et montré avec un bouton vers la piste la plus proche.
- Une vidéo sans extrait, sans zoom et sans image résumé n'a pas de zone de jugement (#247).
  Ses pistes ne sont pas jugeables et restent hors de « à voir ».
  Un verdict qu'un fichier leur rattache n'est pas montré. Il compte pourtant et se réécrit tel quel.
- **Brouillon** (`page.js`) : à chaque changement, les lignes du fichier à écrire vont dans `localStorage` sous `batdetect:verdicts:<chemin de la page>`.
  L'empreinte du fichier lu les accompagne (nom, date et taille).
  À la réouverture sur le même fichier, elles sont reprises. Sur un autre fichier, la page demande.
  `beforeunload` demande confirmation tant que des changements ne sont pas téléchargés.
  Un seul endroit écrit le texte et un seul l'envoie : c'est là que #252 branchera un enregistrement sans téléchargement.
- Tests du JS : `node --test` porte sur les fonctions pures (`csv.js`, `store.js`, `hits.js`, `filter.js`, `time.js`, le nom d'une pastille, #337).
  Il porte aussi sur les règles d'opacité de la vignette dans `page.css` (#336).
  Ils tournent par la tâche `page:js` de `mise` dans `check`. La tâche CI `page-js` les joue aussi, sous Linux seulement.
  Les vues se vérifient dans un navigateur.

### `reference`

La crate lit et écrit les références ([v0.1.1](#référence-v011) et [v0.2.0](#référence-v020)). Elle fait aussi l'appariement.

- `location` sépare les fichiers du dépôt (`tests/fixtures/reference/` et `tests/fixtures/reference-v0.2.0/`) des données hors dépôt (`BATDETECT_REFERENCE_DIR`, `BATDETECT_REFERENCE_V020_DIR` et `BATDETECT_VIDEOS_DIR`).
  Une variable absente ou vide est refusée. Les bancs de la v0.1.1 restent hors dépôt. Ceux de la v0.2.0 sont dans git (2,7 Mo).
- Un enregistrement a quatre fichiers (`video.json`, `frames.jsonl`, `tracks.jsonl` et `tracks.csv`).
  Un nom en `.gz` est compressé (gzip sans nom ni date). Les sections sans type de `video.json` sont gardées telles quelles.
  Relus puis réécrits, les fichiers redonnent les mêmes valeurs.
- `video.json` a deux versions (#274).
  - La version 1 est celle de la v0.1.1. Elle a un bloc `ffprobe` et un `tracks.csv`.
  - La version 2 est celle de la v0.2.0. Elle remplace `ffprobe` par `decoder`.
    Ce bloc dit ce que voit `media` : images en erreur, sauts, pertes aux bords et fils.
    Les fils y sont parce que leur nombre change ce que le décodeur dissimule dans les plages abîmées.
    Les pertes aux bords y sont parce qu'elles ouvrent une plage et que `frames.jsonl` ne les garde pas (#176).
  - La version 2 n'a pas de `tracks.csv`.
    C'est `compare` qui l'enregistre : il ne pourrait écrire le CSV du Rust (en-têtes Fluent de `cli`) sans faire de cycle avec `cli`.
  - Un résumé dont les blocs ne sont pas ceux de sa version est refusé, à la lecture comme à l'écriture.
- Les points d'une piste ne sont pas forcément des détections.
  Les points des pistes jumelles fusionnées sont combinés (17 sur 358 dans l'extrait et 63 sur 610 dans la 092).
- `detection_matching` apparie deux détections de la même image à une distance strictement inférieure à 3 px.
  Chaque détection est appariée au plus une fois, la plus proche d'abord.
  L'écart mesuré à la revue de #124 contre le plus proche voisin sans exclusivité du prototype reste sous 0,01 %.
  Il touche 8 détections sur 542 261 pour la 103, décalées d'1 px.
- `track_matching` apparie des pistes dont le début est à ±5 images.
  Il prend le début le plus proche d'abord, puis l'écart de points le plus faible. Une paire s'accorde si ses points diffèrent de 3 au plus.
  Il est glouton. Deux pistes à moins de 10 images l'une de l'autre peuvent donc laisser une paire possible de côté (un test le fige).
- `bench` (#107, #108) lit les fichiers du banc de la v0.1.1 et le Rust les réécrit au même format.
  Une ligne enregistrée se réécrit octet pour octet. Les entiers du Python se lisent en `f64`.

### `engine`

Le moteur : lecture, plages abîmées et détection. `input` est le seul point d'entrée de la lecture.
Les valeurs de `DetectConfig::default()` sont justifiées dans [02](02-methode.md).
La logique de la détection est dans [10](10-logique-de-detection.md).

**Lecture.**

- `Input::open` accepte toute taille et toute plage. Il refuse une vidéo qui annonce 0 image. La plage n'est pas exposée.
  La détection lit le plan Y tel quel, avec la même table quelle que soit l'étiquette (décision du 12 octobre 2026).
  L'affichage force la plage pleine (décision du 11 octobre). Une vraie source `pc` est donc détectée sur les mêmes Y qu'une source `tv`.
  Comme OpenCV la lit autrement, « au moins aussi bien que la v0.1.1 » ne s'y mesure pas. Aucune des 26 vidéos n'est concernée.
- Lecture anticipée (`read_ahead`) : un fil décode jusqu'à 4 images d'avance dans 5 tampons recyclés, 36 images pour la détection.
  Les tampons vivent jusqu'à ce que le consommateur ait fini, même quand le fil a déjà atteint la fin du fichier.
  Sinon le pic de mémoire dépendait de la vitesse du décodage (#321, voir la mémoire plus bas).
- Pertes (`Losses`) :
  - Aux sauts de `pts` : le pas est la moyenne des intervalles réguliers selon la médiane des écarts.
    Il est juste aussi pour un pas fractionnaire. Un écart de k pas arrondis perd k − 1 images.
  - Avant la première image : l'écart en pas arrondis entre son `pts` et le `pts` de départ du flux.
  - Après la dernière : les paquets illisibles qui la suivent, un par image.
    Ce compte est mesuré égal aux images perdues sur une fin abîmée avec 1, 3 fils ou en automatique.
- Garde du nombre d'images : décodées = annoncées − toutes les pertes.
  Une perte est inexpliquée quand elle ne laisse ni saut ni paquet illisible au bord.
  Seule une telle perte fait refuser la vidéo (`FrameCount`).
- Lecture des 26 vidéos (M1 Max, `--profile ci`) : l'extrait et la 092 donnent 1 230 et 9 011 images, avec les `pts` et les images-clés de ffprobe.
  Les 26 vidéos se lisent sans refus.

**Plages abîmées** (`damage`).

- Une plage part d'une image en erreur ou d'une image après un saut, comme `damaged_spans` de la v0.1.1.
  Elle va jusqu'à l'image qui précède l'image-clé suivante.
  Elle est élargie de la marge de chaque côté et bornée à la vidéo. Elle est ensuite fondue avec la précédente si elle la touche.
  La marge est la demi-fenêtre du fond.
- Des images perdues avant la première ou après la dernière ouvrent aussi une plage.
  Cette plage ne couvre que la marge : les images perdues n'entrent dans aucune fenêtre de fond.
  La v0.1.1 refusait ces vidéos. Le Rust les lit et exclut ces plages (décision de l'utilisateur à #116).
- Le saut se juge au pas des pertes : un écart de k images en vaut plus que k s'il dépasse k + ½ pas.
  La v0.1.1 jugeait à 1,5 × k × pas médian. Le résultat est le même quand tous les `pts` sont connus, comme sur les 26 vidéos.
  À travers un `pts` inconnu, le Rust compte l'image cachée et la v0.1.1 non.
- `Damage::reported` (#275) trace les plages à partir de ce qu'une lecture rapporte : `compare` s'en sert pour réunir ce que rapportent la référence et la lecture courante.
- **Canari** (`tests/slow_canary.rs`, sur chaque système de la CI) : la couture coupe le dernier tiers de la tranche de cinq images de l'extrait en réécrivant la longueur du NAL.
  La 248 est une P au milieu d'un GOP de 15 et la 555 une image-clé. La 860 et la 861 sont consécutives et la 1222 est dans le dernier GOP.
  Le Rust signale exactement {248, 555, 860, 861, 1222} avec 1 fil comme en automatique.
  ffprobe 9.0.2 lancé avec les arguments de la v0.1.1 (`-threads 1 -show_log 16`) en signalait autant sur une copie coupée de la même façon.
  Cette mesure n'a été faite qu'une fois. L'ensemble est écrit en constante.
- **Fils et pixels dissimulés** (règle 16) : une vidéo saine donne les mêmes pixels quel que soit le nombre de fils.
  Sur une tranche corrompue, les images en erreur, les sauts et les plages ne changent pas.
  Mais les pixels de l'image dissimulée et des suivantes jusqu'à l'image-clé peuvent changer.
  - Les 40 images qui diffèrent entre 1 fil et l'automatique sur le canari de l'extrait sont toutes dans les plages.
    Les 1 190 autres sont identiques au pixel près.
  - La 103, la 122 et la 124 (`tests/local_threads.rs`) ont 105, 1 189 et 1 472 images qui diffèrent.
    La mesure suivante en donne 105, 1 151 et 1 447. Toutes ces images sont dans les plages abîmées (marge 15).
    Les pixels dissimulés en fils d'image ne se reproduisent donc pas d'une lecture à l'autre.
- **Dégâts contre la référence v0.1.1** (marge 15, `tests/local_damage.rs`) : l'extrait et la 092 n'ont aucune image abîmée.
  Les images après un saut sont celles d'ffprobe sur les 26 vidéos.
  Pour 25 vidéos, le Rust trouve exactement ses images en erreur et ses plages.
  La 093 compte ainsi 11 images en erreur et 419 abîmées. La 122 en compte 195 et 6 043.
  - Sur le Mac, la 124 diffère : 310 images en erreur contre 309 et 7 458 abîmées contre 7 415 (+43).
    L'image en plus est la 2197. Elle porte le seul drapeau `CONCEALMENT_ACTIVE` avec 1 fil comme en automatique.
    ffprobe ne l'écrit qu'au niveau `info` (« concealing 392 DC, 392 AC, 392 MV errors in P frame »).
    La v0.1.1 ne lit pas ce niveau. C'est la seule image des 26 vidéos dans ce cas.
    Le Rust exclut donc environ 1,4 s de plus que la v0.1.1 sur la 124.
    Reste à juger si c'est souhaitable (image réellement dissimulée) ou trop prudent (passage possible perdu).
  - Sur lumiel, le même FFmpeg 7.1.1 signale 14 images de plus que ffprobe sur 8 vidéos (#250, 17 octobre 2026).
    La 2197 de la 124 n'y est pas comptée.
    Les images en plus sont 1307 et 7395 sur la 093, 5444 sur la 097, 4589 sur la 120, 8844 sur la 121, 2264 sur la 122, 3643 sur la 124 et 2830 sur la 128.
    La 123 en a six : 330, 1250, 2267, 3986, 4902 et 5417.
    On y compte 449 images abîmées sur la 093, 535 sur la 097, 2 215 sur la 120, 4 912 sur la 121, 6 073 sur la 122, 6 627 sur la 123, 7 458 sur la 124 et 1 006 sur la 128.
    Sur la 121, les plages sont inchangées. Sur la 124, le compte est le même que sur le Mac.
    Trois lectures de chaque vidéo donnent les mêmes images et les mêmes plages : deux en fils automatiques et une à un fil.
    La cause de l'écart entre les deux systèmes n'est pas isolée.
    `local_damage` accepte donc pour chaque vidéo soit aucune de ces images, soit exactement toutes avec leur nombre d'images abîmées (`FLAGGED_ONLY_BY_SOME_SYSTEMS`).

**Détection** (#118).

- `detect_injected` (#107) est la couture du banc : il appelle un crochet sur la copie du plan Y que garde le détecteur.
  Il l'appelle une fois par image dans l'ordre de lecture.
  Les images du fond de l'image résumé sont prises avant la copie : le crochet ne les touche donc pas.
  Le crochet est générique : la commande passe un crochet vide et ne paie donc ni appel indirect ni branche.
- Le pool `rayon` est celui de l'appelant (`ThreadPool::install`).
- `half_window` arrondit `fenêtre × cadence / 2` au plus proche, moitié vers le haut.
  La v0.1.1 arrondit au pair : il y a une image d'écart quand ce produit vaut x,5. Cela n'arrive jamais à 30 i/s.
  Moins de 3 images de fenêtre ou moins de 3 échantillons sont refusés.
  Sur la 092, on obtient 15 et 11 échantillons. On obtient 30 et 21 avec `window_2s`, et 61 avec 2 s au pas de 1.
- **Borne de la fenêtre** (#113) : la fenêtre garde toutes ses images quel que soit le pas, à 2 octets (`u16`) par pixel de travail.
  `half_window` refuse une fenêtre qui dépasse `BACKGROUND_BUDGET` = 4 Gio (`LongWindow`).
  Ce budget vaut 3 105 images à 960×720 et 1 379 à 1440×1080.
  Le refus se décide en `f64` avant tout calcul entier. Une fenêtre infinie est refusée aussi.
  Une fenêtre absurde (`--bg-window` du banc, `compare`) donne ainsi cette erreur au lieu d'un débordement ou d'un manque de mémoire.
  La commande n'expose pas la fenêtre. Le défaut en prend 31 images.
  - `dhat` a mesuré le pic du tas de la détection le 13 octobre 2026, sur une vidéo fabriquée de 3 200 images 1440×1080 au travail en 960×720.
    Il vaut 368 Mo pour une fenêtre de 31 images et 876 Mo pour 301 (884 Mo au pas de 1).
    Il atteint 1 857 Mo pour 1 003 images (1 992 Mo au pas de 1).
  - Entre 301 et 1 003 images, chaque image de fenêtre coûte 1,40 Mo sur environ 0,4 Go fixes.
    Cela fait 2,02 octets par pixel de travail : le modèle du budget tient à 1 % près.
    Le pas de 1 ajoute les réseaux de tri, soit 0,13 Go à 1 003 images.
  - Au plafond, la détection tient donc autour de 5 Go au pas de 3.
    Cela reste sous la mémoire de la machine de Manon (16 Go). Les points de 2 000 et 3 105 images n'ont pas été mesurés.
- `settings` (#149) : les dix réglages de la v0.1.1 gardent leurs noms Python.
  Ils sont lus dans `video.json` et écrits dans `params.json` dans l'ordre de la dataclasse de la v0.1.1.
  `Default` se déduit de `DetectConfig::default()` : une seule source des valeurs.
  `DetectConfig::try_from` refuse ce que la v0.1.1 refuse. Il refuse en plus NaN et infini.
  - Écart voulu (#151) : `merge_radius` est borné à 12 pixels d'origine. La v0.1.1 acceptait tout.
    Sur un fil en `release`, une image d'éclair plein cadre prend 0,34 s au rayon 6, 0,99 s au rayon 12 et 3,77 s au rayon 25 au travail en 1440.
    Au travail en 960, elle prend 0,81 s au rayon 25.
    La borne tient une seconde par image à toute largeur. Elle recolle encore les fragments écartés de 18 à 24 px de [02](02-methode.md).
- `geometry` : l'échelle des largeurs sert aux deux axes, comme dans la v0.1.1.
  Quand la hauteur de travail est arrondie, y s'écarte donc de moins de 0,5 pixel d'origine en bas de l'image.
- `reduction` (#150) : Y passe au gris, puis la réduction se fait par aire et axe par axe.
  Chaque pixel source pèse la longueur de son intersection avec le pixel de travail.
  Chaque axe a son propre rapport, comme `fx` et `fy` d'`INTER_AREA` (hauteur arrondie). Il n'y a qu'un arrondi à 8 bits.
  En 3:2, T vaut 9 et les poids sont 2, 1 puis 1, 2.
  Un proptest compare la réduction à la réduction 3:2 d'origine et à un calcul brut des aires.
  - **Blocs fixes** (#214) : la passe horizontale prend des blocs de taille et de poids fixés à la compilation pour les rapports 2:1, 3:1, 3:2 et 4:3 quand T ≤ 128.
    Ces rapports sont ceux du travail en 720 et 480, du défaut 960 et de 960 sur une source de 1280.
    Le total tient sur 32 bits et s'arrondit par `((total + T/2) × (⌊2²³ / T⌋ + 1)) >> 23` sans dépasser 32 bits.
    Un test exhaustif le vérifie contre `(total + T/2) / T` pour chaque T de 1 à 128 et chaque total de 0 à 255 × T.
    LLVM vectorise alors la boucle dans le binaire livré.
    La forme générale ne le permettait pas (poids lus par indice, inverse sur 128 bits).
    En `x86-64-v2`, la boucle traite 4 blocs 3:2 par tour lus par `pinsrd`. En aarch64, elle utilise `ld3.4s`, `mla.4s` et `ushr.4s #23`.
    Sans LTO, la crate seule reste scalaire en aarch64 : la vectorisation ne se juge que sur le binaire.
    Sur un fil en `release`, la réduction prend 0,63 ms par image à 960 hors gris contre 1,23 sur lumiel.
    Sur le M1 Max, elle prend 0,44 ms gris compris contre 1,18 ([09](09-profilage.md)).
  - **Taille inchangée** (1:1, #159) : quand la source a déjà la taille de travail, la réduction n'est que le gris.
    Elle se fait sans aire, sans arrondi et sans copie. Sur un fil en `release`, elle prend 0,41 ms par image à 1440 contre 1,84.
    Le passage sur `092-1440` prend 158,5 s de CPU contre 166,1 (−4,6 %, trois paires A/B en alternance avec `main`).
    Il occupe 613 Mo de mémoire résidente contre 628.
  - Trois variantes ont été essayées puis écartées par #159 à 960. La référence était de 1,17 ms par image sur le M1 Max.
    Prendre les champs de l'arrondi dans les fermetures donnait 1,18 ms : LLVM le faisait déjà.
    Des poids contigus avec totaux et inverse sur 32 bits donnaient 1,39 ms. L'inverse sur 32 bits seul donnait 1,21 ms.
- `blur` (#150) : noyau gaussien de σ = `target_sigma / échelle` pixels de travail et de portée ⌈4σ⌉.
  C'est la portée d'OpenCV pour tous les σ du banc. Les poids se partagent 4 096 parts par les plus forts restes, par paires symétriques.
  La somme est exacte, le noyau est symétrique et aucun poids n'est négatif.
  Le centre ne reçoit que l'unité impaire. Dès un σ de travail d'environ 11,6, il peut tomber d'une part sous son voisin.
  Cela n'arrive jamais dans le banc (σ de travail de 2 au plus).
- `prepare` fait un seul arrondi final en 1/64 de niveau (`u16`) et une passe verticale en `u32`.
  Dans cette passe, 255 × 4 096² tient sur un `u32` puisque le noyau somme exactement 4 096.
  - **Noyau plié** (#159) : comme le noyau est symétrique, les deux pixels à même distance du centre s'additionnent avant la multiplication par leur poids commun.
    Rien ne déborde : une image blanche de σ = 0 à 2,75 le vérifie en profil `ci`.
    Sur un fil en `release`, le flou seul prend 0,59 ms par image contre 0,77 à 960 et 1,76 contre 2,50 à 1440.
    La 092 prend 109,5 s de CPU contre 111,1 (−1,4 %, trois paires A/B).
- `network` prend la médiane par le réseau de Batcher sur des blocs de 64 pixels.
  Le réseau est coupé à n entrées et élagué aux comparaisons qui atteignent le milieu.
  Il est testé en 0-1 exhaustif de 1 à 20 et par un proptest contre un tri de 1 à 128. Il y a un réseau par compte, de 1 au maximum (#151).
  L'ensemble pèse 2,3 Ko à 11 échantillons, 177 Ko à 61, 8,7 Mo à 301 et 566 Mo à 1 801 (60 s au pas de 1).
  Cela fait 23 à 30 % de la fenêtre de 1 801 images.
  - **Onze échantillons** (#213) : c'est la fenêtre par défaut.
    Une table de 28 comparateurs (`MEDIAN_OF_ELEVEN`) y remplace les 32 de l'élagage.
    On l'a trouvée hors du dépôt en retirant un à un des comparateurs du réseau élagué tant que la médiane restait exacte sur les 2 048 entrées 0/1.
    Une construction aléatoire suivie du même élagage n'est pas descendue sous 28. Rien ne prouve que ce soit le minimum.
    Le réseau prend 12,5 % de cycles en moins. La détection de la 092 prend 2,3 % de temps en moins sur lumiel.
    Garder les 11 lignes dans les registres était plus lent ([09](09-profilage.md)).
- `threshold` : le résidu est `image − fond − (niveau − niveau moyen)` en `f32`. Le gain se calcule en entiers jusqu'à une seule division.
  Les candidats sont au-dessus du plancher (12 niveaux).
  Avec au moins 7 échantillons, le seuil se relève à max(12, 8 × 1,4826 × MAD des écarts corrigés du gain).
  - **Blocs masqués** (#159) : un bloc de 64 pixels tout entier sous le masque est sauté avant la copie des échantillons et le réseau.
    Une image à 960 avec la bande `osd` prend 2,19 ms contre 2,36 sur un fil en `release`.
    L'extrait aux réglages `osd` prend 14,66 s de CPU contre 14,94 (−1,9 %, trois paires A/B).
  - Les lignes du réseau sont sur la pile jusqu'à 64 échantillons (`STACKED_SAMPLES`, 8 Ko). Au-delà, elles sont sur le tas de chaque fil.
    Les mêmes lignes sur le tas coûtaient 7 % de temps CPU sur la 092 (117 s contre 110 s, en alternance avec `main`).
    L'ablation montre que ni le test du masque ni les écarts ni la copie des deux valeurs du milieu ni l'inlining du tri n'en sont la cause.
    Le mécanisme n'est pas isolé. Les deux chemins sont testés.
- `mask` (#151) : la marge autour des zones OSD est la portée du noyau, soit ⌈4σ⌉ pixels de travail (⌈3σ⌉ dans la v0.1.1).
  Les unités préparées sous la bande ne dépendent donc pas de l'écran, au bit près.
  Le niveau moyen exclut le masque : une bande qui scintille ne change pas le gain.
  Si aucun pixel ne reste, la détection refuse (`WholeFrame`).
  La marge de 4σ ne perd aucune détection sur le banc (`osd` et `width_1440_osd`).
- `closing` : fermeture par le disque de rayon `round(merge_radius / échelle)`, moitié vers le haut (#150).
  La v0.1.1 arrondit au pair : il y a un rayon d'écart quand ce quotient vaut x,5. C'est le cas de 3,75 à l'échelle 1,5.
  Aux défauts, le rayon est 4 et ses demi-largeurs sont `[0, 3, 3, 4, 4, 4, 3, 3, 0]`.
  La demi-largeur de chaque ligne vaut `round(√(r² − dy²))`, comme l'ellipse d'OpenCV. `sqrt` est exact par IEEE 754.
  La fermeture est creuse : seuls les pixels touchés par la dilatation sont marqués puis érodés.
  Le dehors de l'image compte comme allumé. Un proptest la compare à une fermeture dense.
- `components` : composantes 8-connexes, dans l'ordre de balayage de leur premier pixel.
  La v0.1.1 suit la numérotation d'OpenCV par blocs de 2×2 pixels : une tache qui commence une ligne plus bas peut sortir avant.
  Dans sa référence, 12 280 des 29 650 images à plusieurs détections ont au moins une inversion d'une ligne.
  Le suivi en dépend en cas d'égalité. Le Rust ne l'imite pas (règle 5).
  La surface vaut pixels allumés × échelle². L'amplitude est le plus grand |résidu|.
  Les pixels au-dessus des trois quarts du seuil vont dans une seconde liste (#21).
  Fermeture et composantes prennent les deux listes sans les fusionner. Une composante est forte si elle a un pixel de la première.
- `bracketing` (#21) garde une détection faible encadrée par deux fortes.
  La règle et les mesures sont dans [10](10-logique-de-detection.md#5-ter-garder-les-taches-faibles-qui-comblent-un-vol).
  Il est dans `detect_injected` et vaut donc pour la commande, `compare` et le banc.
- `detector` travaille par lots de 32 images.
  Chaque lot est préparé, puis les cibles prêtes sont détectées en groupes contigus avec un tampon par fil `rayon`.
  Elles sont ensuite rangées dans l'ordre.
  Une boucle de comparaisons écrite élément par élément se vectorisait mal une fois inlinée en `release` (détection 4 fois plus lente qu'en `ci`).
  Écrite sur des tableaux entiers, elle se vectorise.
- `background` (#104) : les 25 images aux rangs `(2i + 1) × N / 50` (N annoncé) sont gardées en YUV entier pendant la lecture.
  Si N < 25, toutes le sont. Le rang est exact, sans le positionnement imprécis d'OpenCV.
  Un rang perdu en fin de vidéo donne moins d'images, comme dans la v0.1.1.
  - Les images qui tombent dans une plage abîmée sont écartées à la fin de la lecture, sauf s'il n'en reste aucune (décision de l'utilisateur, 12 octobre 2026).
    Leurs pixels dépendent du nombre de fils (règle 16).
    Elles sont la majorité sur 121 à 124 : 15 à 21 rangs sur 25 sur les plages de la v0.1.1.
    En Rust, 16 sont écartées sur la 122 (9 gardées) et 21 sur la 124 (4 gardées).
  - La médiane se prend par sélection (`select_nth_unstable`) sur un tampon réutilisé.
    En nombre pair, c'est la demi-somme des deux du milieu arrondie au plus proche (règle 5). Elle prend 0,29 s sur la 092 (profil `ci`).
- Différences avec la v0.1.1, hors gris :
  - Le Rust passe en gris puis réduit. La v0.1.1 réduit l'image en couleur puis passe en gris.
  - Le noyau est quantifié sur 4 096. La v0.1.1 floute en `float32` avec le noyau exact.
  - Médiane et MAD portent sur des unités de 1/64 de niveau.
  - Résidu et écarts sont en `f32` (`float64` en v0.1.1). Le niveau moyen est exact.
  - L'ordre des taches diffère (voir `components`).
- Le test de saut de luminosité à seuil fixe de la v0.1.1 n'a pas d'équivalent aux réglages par défaut : un saut de 30 niveaux allume l'image au bord du saut.
  La stabilité retire cette image (tests de [`compare`](#compare)).
- Règle 16 : même sortie au bit près avec 1, 3 ou 4 fils et des lots de 1 à 64 (test rapide).
  Sur le canari de l'extrait, 1 fil de décodage et 1 fil `rayon` donnent les mêmes plages que l'automatique et les mêmes détections hors plages.
  Le parallèle redonne le séquentiel au bit près hors plages abîmées sur l'extrait, la 092 et les 26 vidéos.

**Mémoire** (`tests/slow_memory_flash.rs`, `dhat`, 4 fils, 64 images).

- Un éclair plein cadre dans chaque groupe de cibles du premier lot coûte 67,1 Mo de plus qu'une vidéo calme.
  Cela fait 24,27 octets par pixel de travail et par fil.
  Le compte porte sur les pixels allumés, touchés et fermés d'un fil. Un pixel allumé prend 8 octets et un pixel touché ou fermé 4.
  Leurs listes montent jusqu'à 2²⁰ éléments et sont gardées jusqu'à la fin (environ 170 Mo à 10 fils).
  Le test exige plus de 16 et au plus 32 octets. Les allocations de FFmpeg ne sont pas vues : ce sont les mêmes dans les deux vidéos.
- `slow_memory_flash_wide` coûte 134,2 Mo de plus à 1440 pixels de travail, soit 21,58 octets par pixel et par fil.
- `slow_memory_flash_long` travaille à 2 s au pas de 1, soit 61 échantillons.
  Il coûte 67,1 Mo de plus, soit 24,27 octets par pixel et par fil.
  Les éclairs y tombent dans le vidage de l'image 64, aux images 3, 11, 19 et 27. Chacun des 4 fils y a une cible éclairée.
  Le pic de la vidéo calme vaut 319 Mo. Il tombe dans ce même vidage et porte la fenêtre (2 octets par pixel et par image gardée).
- Quand le fil de lecture libérait ses tampons à sa fin de fichier, le pic au calme du cas long dépendait de la vitesse du décodage (244,8 Mo ou 272 à 275 Mo).
  Le surcoût mesuré pouvait alors tomber sous la borne basse (#321).
  Sur lumiel, les mesures faites depuis donnent 24,27, 21,58 et 24,27 octets par pixel et par fil à 2 Ko près sur 50 passes.
  Le pic au calme du cas long est le même à 8 Ko près sur 6 profils, avec une lecture rapide ou ralentie.
- Les pixels faibles (#21) ont leur propre liste.
  Sur lumiel après #321 et #24, le flash et le cas long coûtent alors 24,32 octets par pixel et par fil. Le cas large en coûte 21,58.
  Un cas n'est pas testé : un éclair fort puis un éclair entre 0,75 et 1 du seuil sur une autre image.
  Chaque liste y garde sa capacité, soit environ 8 octets de plus par pixel et par fil.

**Gris de la détection** (#118, exception à la règle 5).

Chaque Y passe par une table de 256 gris mesurée sur OpenCV.
La mesure part d'une rampe Y = 0 à 255 avec U = V = 128, encodée sans perte en plage `tv`.
Elle est lue par `cv2.VideoCapture` puis passée par `COLOR_BGR2GRAY`.
La table suit `(Y − 16) × 255 / 219` de 1,2 à 2,3 niveaux en dessous, en dents de scie. Elle vaut 0 jusqu'à Y = 17 et 255 dès Y = 237.
La réduction pondère ensuite ces gris et arrondit à 8 bits avant la gaussienne.
Le calcul passe par sa forme close `clamp((Y × 4769 − 81 520) >> 12, 0, 255)` (`gray::computed`, #214).
Elle est égale à la table sur les 256 valeurs (test exhaustif).
La lecture de la table restait scalaire en `x86-64-v2`. La forme close se vectorise dans le binaire livré avec `pmaddwd`.
En aarch64, elle traite 16 octets par tour avec `ushll`, `mla.4s` et `sshr.4s #12`. L'ablation change un choix à la fois (M1 Max).
Les taux sont ceux de l'appariement des détections référence / Rust contre la v0.1.1, hors plages abîmées et instables de `video.json` :

| Variante | Extrait | 092 | 089 | Pire des 26 | Sous 98 % | Points de piste perdus |
|---|---|---|---|---|---|---|
| (a) table et arrondi à 8 bits | 100 / 99,75 | 100 / 99,87 | 98,19 / 98,46 | 98,19 | 0 | 19 |
| (b) formule, écrêtée, arrondie | 98,51 / 98,75 | 96,15 / 95,30 | 70,36 / 72,92 | 70,36 | 25 | 415 |
| (c) formule écrêtée, exacte | 97,51 / 98,25 | 95,38 / 96,00 | 75,58 / 76,76 | 75,58 | 22 | 340 |
| (d) formule exacte, sans écrêtage | 97,76 / 98,74 | 95,13 / 95,13 | 72,12 / 70,61 | 70,61 | 23 | 358 |

- Les écarts de (d) sont surtout du bruit au seuil : sur la 089, 489 des 508 détections perdues sont des taches de 4,5 px² sous 20 niveaux.
  Autant sont ajoutées. Un décalage de quelques dixièmes de niveau suffit à les faire basculer.
- La référence compte 966 pistes. La variante (a) en apparie 963 dont 958 concordantes, en perd 3 et en ajoute 9.
  La variante (d) en apparie 936 dont 912 concordantes, en perd 30 et en ajoute 35.
  Sans jumelle à 1 s, (d) perd vraiment 17 pistes, dont la 097 0:34 (102 points) et la 100 2:00 (24 points). Elle en ajoute vraiment 22.
- La table s'écarte aussi du `BGR2GRAY` d'OpenCV quand la chrominance n'est pas neutre.
  La bande d'affichage des jumelles couvre les 7 % du haut de l'image, en couleurs (V jusqu'à 255).
  L'écart y va jusqu'à 56 niveaux sur l'image 40 de l'extrait (972 pixels à 3 niveaux ou plus). Il est nul ailleurs (U = V = 128).
  C'est pourquoi `sigma_0` et `window_2s` s'écartent de la v0.1.1 dans cette bande (#150, #151). L'utilisateur a gardé la table.
- Retenter la formule avec des seuils réajustés au banc est l'objet de #142, ouverte.

### `cli`

Le paquet et le binaire `batdetect` (#123).
Tout texte vu par l'utilisateur vit dans les deux catalogues Fluent `locales/{fr,en}.ftl` (#114). Seul le binaire les lit.
La bibliothèque n'a aucun texte : elle reçoit ses libellés par des traits (`Wording` et `Labels`).
Ses tests passent des libellés de débogage en anglais. Le binaire porte les textes, la ligne de commande, la mise en forme et le terminal.

#### Ligne de commande

- `doctor` et `completion` (#58, #59) s'aiguillent sur le premier argument avant le parseur. `batdetect <vidéos>` ne change pas.
  Une vidéo ou un dossier nommé ainsi se donne en `./doctor` ou `./completion`. L'aide le dit.
- `completion zsh` écrit le script de `clap_complete` dans la langue du système. Ce script est tiré du parseur complet.
  La ligne du positionnel est réécrite.
  Elle propose les deux commandes au premier mot seulement, puis les vidéos (aux extensions de `jobs`) et les dossiers.
  Elle ne propose pas les fichiers cachés. La commande n'écrit jamais dans le profil.
  Sur un terminal, la sortie d'erreur donne les lignes de `~/.zshrc` avec et sans Homebrew. Un autre shell est refusé (code 2).
- `doctor` vérifie le dossier courant par les mêmes fonctions que le traitement.
  Il fait un essai réel de l'encodeur, mesure la place libre (`disk`) et lance la sonde d'écriture (`writable`).
  Un encodeur absent, un essai impossible faute de dossier temporaire, moins de 20 Go libres (`ENOUGH_FREE_BYTES`) ou l'écriture refusée donnent le code 1.
  openh264 à la place de VideoToolbox ou une place non mesurée ne donnent qu'un avertissement et le code 0.
- **Aide** sur deux niveaux (décidée le 10 octobre 2026, #113) : `-h` donne l'essentiel et `--help` tout.
  Dans `--help`, les réglages qui changent la détection ont leur propre titre.
  Les usages courants viennent en tête au format de `tldr` (#56). Sans aucun argument, la commande les écrit et sort en 0.
  `-o` sans vidéo reste en revanche une erreur d'usage.
  Les noms de fichiers de l'aide viennent de `files::names` dans la langue de l'aide. Ils ne sont jamais écrits en dur.
  Dans les deux langues, l'apparence est celle de l'aide d'argparse 3.14 (demande de l'utilisateur).
- **Pièges de clap** contournés ou acceptés :
  - clap passe toujours le texte à la ligne suivante en aide longue.
    `-h` et `--help` sont donc tous deux l'aide courte de clap. Le niveau se choisit avant la lecture.
  - clap coupe ses lignes sur l'espace ASCII et à chaque changement de couleur, même sans espace.
    Son « [default: 20] » est fait de trois segments et laissait parfois « ] » seul sur la ligne suivante.
    La commande écrit donc elle-même le défaut sur sa propre ligne avec une espace insécable.
    C'est le choix de l'utilisateur : un défaut n'est jamais coupé.
  - La colonne du texte se cale par titre. Aucun réglage ne la partage entre deux sections.
    Sous environ 95 colonnes, toute une section passe à la ligne suivante dès qu'une seule option ne tient plus (règle des 40 % de `help_template`).
    Les tests rendent donc l'aide à 100 colonnes fixes.
  - `before_help` et `after_help` sont repliés à la largeur du terminal.
    Chaque nom des exemples est sur sa ligne et son rôle en retrait dessous, parce que deux colonnes s'y casseraient.
  - Les titres français finissent par une espace insécable parce que clap colle « : ».
    « [OPTIONS] » reste dans l'usage français parce que `override_usage` n'est pas pris.
  - Limites acceptées : `-h` et `--help` ont la même couleur et le nom du programme est en cyan.
  - `-V` et `--version` écrivent les mêmes deux lignes : la version, puis FFmpeg et sa licence (#196).
    La commande n'a que la version longue.
- **Valeurs refusées** : une erreur d'usage en code 2 nomme l'option et sa plage (`--threshold must be a finite number > 0, not -1`).
  Elle n'a pas de ligne d'usage. Une valeur illisible dit la même phrase avec la valeur telle qu'écrite (#55).
  Les entiers ne disent que leur borne basse et pas le maximum du type.
  Sur `--threshold` et `--clip-margin`, une virgule décimale reçoit le conseil d'écrire un point.
  Les refus que la ligne de commande ne peut pas atteindre nomment l'option et demandent de le signaler.
- `--osd-region` se lit par `FromStr for engine::settings::Region`. Le banc le lit de la même façon.
  Il accepte une valeur qui commence par un tiret (`allow_hyphen_values`).
  La phrase de la zone la refuse ensuite et la numérote parmi les zones données (#221).
- **Erreurs de clap** : un formateur par langue est branché par `clap::Error::apply`.
  L'impression, les couleurs et les codes de sortie restent ceux de clap.
  Nos refus portent leur phrase dans `ContextKind::Custom` parce que le message de `Command::error` ne passe pas par le formateur.
  Les erreurs de syntaxe essayées par le test sont rendues en anglais comme clap les écrit, au pied près.
  Un genre non prévu ne recopie pas le texte anglais de clap (#55). Seules les erreurs de syntaxe ont la ligne d'usage.
  Les refus de valeurs gardent le pied qui renvoie à `--help`. Les refus d'entrées et de dossiers n'ont ni l'un ni l'autre.

#### Admission, avant tout calcul

Le binaire admet le lot dans cet ordre (`admission`). Il relève d'abord les entrées par `jobs::plan`.
Viennent ensuite l'inventaire, le contrôle d'écriture et le contrôle de place, puis la question des refus.
Rien n'est créé avant la détection : une vidéo au chemin mal tapé ne laisse pas de `batdetect/` vide.

- **Entrées** : un dossier donne ses vidéos (`mp4`, `mov`, `avi`, `mkv` et `m4v`) par ordre de nom.
  Il ne les donne qu'au premier niveau. Un `batdetect/` voisin n'est donc pas repris.
  - Un nom qui commence par « . » est écarté (#204) : ce sont surtout les `._x.mp4` (AppleDouble) que macOS dépose à côté de chaque fichier sur un volume exFAT ou FAT.
    Une vidéo nommée est jugée sur sa seule extension, qu'elle soit cachée ou non.
  - Une vidéo donnée deux fois sous n'importe quelle écriture n'est traitée qu'une fois (#194).
    La clé est son chemin réel. Pour un fichier absent, c'est le dossier réel suivi du nom.
- **Refus en bloc** (#55) : toutes les entrées sont relevées avant la première détection.
  Une seule fautive arrête le lot (code 2, aucun dossier créé).
  - Les raisons sont des tables fermées et sans texte (`jobs::Reason` et `access::Access`).
    Comme un échec du bilan, chaque refus a sa ligne « à faire » (#239).
  - La ligne nomme l'entrée par le dernier nom de son chemin (#247). Un chemin glissé dans le Terminal est absolu et noyait la raison.
    Le chemin entier reste dans le « à faire » quand il sert à agir. Il reste aussi dans le refus de place, où il nomme le disque.
  - Une vidéo que l'inventaire ne peut pas ouvrir rejoint ces refus avec la raison de l'échec équivalent.
- **Inventaire** (#62) : la reprise se décide là une fois pour toutes (`reuse`, sauf `--force`). Une vidéo déjà faite n'est pas ouverte.
  Chaque autre vidéo est ouverte en un seul fil puis refermée. Cela donne sa durée et refuse tôt celle qui ne s'ouvre pas.
  - Coût mesuré le 16 octobre 2026 sur lumiel (binaire release, fichiers en cache) : 0,39 à 0,44 s pour les 26 vidéos, dont 0,39 s de CPU.
    Le pic de mémoire est de 15,6 Mo.
  - Une vidéo fausse, tronquée ou au flux effacé échoue donc à l'ouverture. Un échec à son tour vient surtout de l'écriture.
    Les tests de commande le provoquent par un fichier ordinaire posé à la place du dossier de la vidéo.
    Ce fichier la fait échouer sur tous les systèmes, Windows et root compris.
- **Dossiers de sortie** (#86, #91) : la racine est `-o` ou à défaut `<dossier de la vidéo>/batdetect/`.
  Chaque vidéo a son dossier à son nom dans sa racine.
  - Deux homonymes dans une même racine prennent `<dossier parent>_<nom>_<extension>` avec `-o`.
    Sans `-o`, le dossier est déjà celui de la vidéo et ils prennent `<nom>_<extension>`.
  - Sur tous les systèmes, les homonymes se comparent à la casse près. La clé est le nom en NFC (celle du ménage) passé par `to_lowercase`.
    Sans elle, `a/Vol.mp4` et `b/vol.mp4` partageraient un dossier sur APFS ou NTFS.
    Le ménage de la seconde effacerait alors les extraits de la première.
    Sous Linux, la règle va donc plus loin que le système de fichiers : deux dossiers `Nuit/x.mp4` et `nuit/x.mp4` sont refusés comme un seul.
  - Un nom non UTF-8 garde ses octets pour la clé. `\xff` et `\xfe` auraient la même clé par `to_string_lossy`.
- **Contrôle d'écriture** pour chaque racine (#86) : une sonde est créée de façon exclusive dans le plus proche dossier existant de la racine, puis retirée.
  `create_dir_all` ne suffit pas : il réussit sur un dossier existant en lecture seule.
  Un `batdetect/<vidéo>/` existant mais en lecture seule passe la sonde de sa racine. Sa vidéo échoue seule après la détection.
- **Contrôle de place au lancement** (#61) : sauf sous `--ignore-disk`, chaque vidéo que la reprise ne saute pas compte sa taille multipliée par le coefficient des sorties.
  Sous `--annotated`, le coefficient de la vidéo annotée s'y ajoute.
  Les coefficients et leur mesure sont dans la section « Place prise » de [02](02-methode.md).
  - Les besoins s'additionnent par volume du dossier de résultats et non des vidéos.
    Un disque externe pour les vidéos et le Mac pour les résultats se comptent donc à part.
    Le volume se reconnaît à `MetadataExt::dev()` du plus proche dossier existant.
    Sous btrfs, chaque sous-volume a son propre `dev()`. Le besoin est alors sous-estimé (cas rare, sans effet sur le Mac).
  - Sous macOS, la place libre est `f_bavail × f_bsize` de `statfs` : les compteurs de blocs de `statvfs` n'y ont que 32 bits.
    Sous Linux, c'est `f_bavail × f_frsize` de `statvfs`.
    Sous APFS, elle ne compte pas l'espace purgeable que le Finder ajoute. Le contrôle est donc plus prudent que le Finder.
    Sous Windows, la place n'est pas lue et rien n'est refusé.
  - Un volume est refusé si l'estimation plus la marge dépasse sa place libre.
    La marge vaut 10 % de l'estimation et au moins 1 Go (décision du 9 octobre 2026).
    La ligne nomme le premier dossier de résultats de ce volume, un chemin que la naturaliste reconnaît. Elle compte en Go décimaux.
  - Le refus suit le circuit des refus d'entrées : question dans un terminal, `--skip-refused`, code 2 sinon (décision de l'utilisateur du 16 octobre 2026).
    Un oui lance tout le lot.
    Ce refus ne compte alors ni parmi les entrées refusées du bilan, ni dans `.batdetect/refus.json` et sur la page.
- **Question des refus** (`consent`, décision de l'utilisateur du 16 octobre 2026) : des vidéos valides peuvent rester à côté des refus.
  Si l'entrée standard est un terminal, la commande demande alors s'il faut les traiter.
  Toute autre réponse que oui ou non fait reposer la question. La fin de l'entrée ou une erreur de lecture vaut non.
  Un oui traite les vidéos valides et le code de sortie vaut alors 1. `--skip-refused` vaut un oui sans question.
  Sans terminal et sans l'option, le refus en bloc reste. Ce chemin se teste dans `consent` sur une entrée fabriquée.
  Un test de commande ne peut pas fabriquer de terminal sans pseudo-terminal.

#### Une vidéo

- **Ordre** (`process`) : détection en fils automatiques, suivi, ménage et tableau des pistes.
  Viennent ensuite l'essai des encodeurs, l'image résumé, le contrôle de place, les extraits, puis `params.json`.
  `params.json` s'écrit en dernier et seulement pour une vidéo réussie ou qui a trop de pistes.
  C'est la marque d'un dossier terminé, que relit la reprise (#66).
  Une détection ratée ne détruit donc rien. Une vidéo en échec sera refaite à la relance.
- **Ménage** après la détection et le suivi (#115, `files::previous`) :
  - Le tableau des pistes appartient à l'outil (#228) : un fichier à son nom est retiré quoi qu'il contienne.
    Les tableaux mis de côté sous un nom daté par une version antérieure (#83) gardent un nom que le ménage ne reconnaît pas et restent.
  - Une entrée au nom du tableau qui n'est pas un fichier est laissée et le bilan le dit.
    Le nouveau tableau n'est pas écrit par-dessus si elle porte le nom de la langue courante ou si un fichier occupe encore ce nom (autre casse).
    La vidéo reste réussie.
- **Encodeurs** : un essai d'encodage à la taille et à la cadence de la vidéo donne la liste employée (VideoToolbox puis openh264 sur Mac, openh264 ailleurs).
  S'il échoue, l'image résumé est écrite et la vidéo échoue à l'étape des extraits.
  Le dossier des extraits n'est recréé qu'après un essai réussi et avec au moins un extrait à écrire (#195).
  0 piste, trop de pistes ou aucun encodeur ne laissent donc aucun dossier.
- **Trop de pistes** (au-delà de `--max-tracks`) : le tableau est écrit, mais ni l'image résumé, ni son fond, ni les extraits (#259, #221).
  `--max-tracks` vaut 300 par défaut et 0 pour sans limite. La vidéo passe de l'étape « suivi » à « trop de pistes ».
  `params.json` a alors `background` à `null`, `summaries` vide et des pistes sans couleur ni `trace`. La page ne dessine rien pour elle.
- **Image résumé** (#104) : le fond s'écrit une fois par vidéo, quel que soit le nombre de tranches et de parties (0,87 Mo pour la 092).
  La mise en page de chaque page (`sheets`) et la trace de chaque piste dessinée (`listing`) vont dans `params.json` (#259).
  La page relit ces valeurs sans rien recalculer. Sur la 092, `params.json` passe à 13,5 Ko.
- **Contrôle de place par vidéo** (#61) : il a lieu juste avant les extraits, même sous `--ignore-disk`. L'estimation est presque exacte.
  Elle multiplie les secondes des fenêtres d'extrait et de zoom par le débit de la vidéo d'entrée et par leur coefficient, puis ajoute la vidéo annotée.
  Elle se compare à la place libre sans marge.
  Si elle la dépasse, la vidéo échoue (« disque plein ») et `params.json` n'est pas écrit. La suivante est tentée.
  Les tests des deux contrôles prennent une vraie vidéo allongée à 10 To par `set_len` en fichier épars.
  FFmpeg la lit comme l'originale et son estimation dépasse tout disque. Il n'y a aucune couture dans le code.
  Ils ne tournent que sous Unix.
- Un extrait refait en openh264 après un échec de VideoToolbox écrit une ligne sur la sortie d'erreur.
  Une seule ligne dit pour tout le lancement la première fois que VideoToolbox ne répond pas en 10 s (#313).

#### Pendant et après la soirée

- **Vidéo et étape en cours** sur la sortie d'erreur (#63). La sortie standard ne porte que le bilan.
  - Quatre étapes sans avancement image par image : l'heure de fin se compte d'une vidéo à l'autre.
    Si l'étape ne s'écrit pas, la vidéo échoue et le lot s'arrête en code 1.
  - Une ligne annonce d'abord le lot (#62) : nombre de vidéos, durée totale et vidéos déjà faites.
  - La vidéo est nommée sans extension. L'extension reste quand une autre vidéo du lancement a le même nom sans extension (#247).
    Quand l'extension ne suffit pas, la ligne nomme aussi le dossier parent.
    C'est le cas de `batdetect nuit-12/ nuit-13/` quand la numérotation des jumelles repart à zéro.
  - **Heure de fin** (#68, `eta`, pur) : les secondes passées sur les vidéos calculées sont divisées par leurs images.
    Ce temps par image s'applique aux images restantes que l'inventaire a annoncées. Seule une vidéo calculée jusqu'au bout donne du temps.
    Une vidéo en échec est souvent arrêtée tôt et tirerait l'estimation vers le bas. L'heure est locale et arrondie à la minute.
    Elle ne dit pas le jour, même au-delà de 24 h de calcul restant.
  - Le mode se choisit par `IsTerminal` sur la sortie d'erreur. Redirigée, elle reçoit une ligne par étape et rien n'est effacé.
    Sur un terminal, la ligne est redessinée par `\r` et des espaces sans séquence d'échappement (console Windows ancienne).
    Sous Unix, la largeur du terminal est lue à chaque étape.
    La ligne est coupée à cette largeur moins une colonne parce qu'une ligne passée à la rangée suivante ne s'effacerait qu'à partir de sa dernière rangée.
    La coupe prend la fin de la ligne : sur une fenêtre étroite, l'heure de fin disparaît la première.
    Sous Windows, la largeur n'est pas lue.
  - **Ctrl-C** (#196) : le gestionnaire est posé une fois le lot admis.
    Il efface la ligne d'étape, dit comment reprendre et sort en 130 même si la ligne ne s'écrit pas.
    Un `.partial` laissé dans le dossier d'une vidéo est retiré par le ménage de la relance (#115).
- **Bilan** écrit une fois sur la sortie standard (#65).
  Son modèle sans texte est dans la bibliothèque (`tally`) et sa mise en forme dans le binaire (`board`).
  - Il y a un tableau par dossier de résultats (#86).
    La vidéo y est nommée par son dossier, qui porte déjà la levée d'ambiguïté des homonymes.
    Seules les racines qui existent sur le disque ont un titre (#196).
  - Les largeurs se comptent en `char` après composition NFC et sans crate.
    Durée et calcul sont tronqués à la seconde et le non-analysé est arrondi au-dessus.
    La durée va de la première image à la fin de la dernière, sans le premier `pts`.
  - Dans le bilan, le « à faire » d'un échec qui renvoie au détail anglais dit que ce détail a été écrit pendant le traitement.
    Ce détail reste sur la sortie d'erreur.
  - « Réglages changés » nomme toutes les options qui font recalculer une vidéo à la relance (#221, #247).
    Ce sont les champs que compare la reprise (`settings::compared`). Chacun est nommé par son option dans l'ordre de l'aide.
    Deux tests du binaire tiennent la liste : toute option est rangée parmi celles qui font recalculer ou non.
    Chacune des premières est nommée quand elle est donnée seule.
  - Le code de sortie vaut 1 si une vidéo a échoué ou a trop de pistes, ou si le bilan ne s'écrit pas.
    Si une écriture au fil de l'eau échoue, le lot s'arrête et il n'y a pas de bilan.
- **Page de résultats** à la racine de chaque dossier de résultats (#71).
  - `written` l'écrit avant le lot puis à la fin de chaque vidéo, par `.partial` puis renommage. Il retire alors la page de l'autre langue.
    Il n'écrit ni ne retire une page qui n'a pas la marque de `page`. Il ne crée jamais la racine.
    Une erreur est dite à la fin du bilan. Elle rend le code 1 sans arrêter le lot.
  - Chaque page est réécrite avant la sortie au Ctrl-C ou si la boucle du lot s'arrête sur une erreur.
    Seule la racine de la vidéo en cours est créée, pour qu'un Ctrl-C pendant la première vidéo laisse sa page.
  - Les clés des vidéos sont les noms de dossier composés en NFC : un dossier créé en NFD et la même vidéo tapée en NFC font une seule entrée.
    Un nom de dossier non UTF-8 garde ses octets pour la clé, comme dans le ménage et dans `jobs` (#227).
  - Les échecs (`ledger`, `.batdetect/echecs/`) et les refus du lancement (`refusals`, `.batdetect/refus.json`, #267) s'écrivent à la racine pour la page sous des clés stables.
    Une réussite efface son échec. Un échec dont la racine n'existe pas encore attend qu'une autre vidéo du lot la crée.
  - Les clés de raison et de conseil d'un refus sont la seule table entre raison et texte.
    Le texte est pourtant encore produit deux fois : pour le terminal (`messages`) et pour la page (`sheet`).
    Un test de `sheet` vérifie que la page dit les mêmes lignes que le terminal dans les deux langues.
    Il le fait pour chaque raison, chaque accès et chaque cause d'ouverture.
  - Sous macOS et dans un terminal, `notify` (#69) ouvre après le bilan la page de chaque racine.
    Il n'ouvre rien pour une racine sans page à nous.

#### Langues

- La langue du système est la première variable non vide parmi `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` et `LANG`.
  Dans une liste comme `LANGUAGE=de:fr`, c'est la première langue qui a des messages.
  Sans variable, `sys-locale` lit la langue de l'interface (macOS et Windows). Sinon c'est l'anglais.
  `LANG=C` reste en anglais puisque la variable est posée. Les tests qui lancent le binaire fixent tous `LANG`.
- Les catalogues Fluent sont inclus dans le binaire. Ils sont plats, sans termes ni attributs ni `NUMBER()`.
  Le français reprend mot pour mot les textes de la v0.1.1 quand ils existent.
  Le test des clés exige que chaque message se forme dans les deux langues.
  Chaque argument doit servir dans au moins une (l'anglais n'accorde pas les participes). Aucune clé ne doit être orpheline.
- Les marques d'isolement U+2068/U+2069 sont coupées (`set_use_isolating(false)`) : Roboto n'a pas leurs glyphes.
  Elles entreraient aussi dans les chemins copiés depuis la console.
- Les deux bundles vivent dans deux `static OnceLock` et `Texts` ne porte que la langue.
  Un bundle n'est pas `RefUnwindSafe` : le travail passé à `run_all` ne capture ainsi que la langue.
- Nombres : le français a la virgule décimale et sépare les milliers par U+202F dans la console.
  Le bandeau prend U+00A0 parce que Roboto n'a pas U+202F. L'anglais a le point et aucun séparateur.
  Le CSV, `params.json` et les noms de fichiers n'y passent pas.
  L'aide affiche « [par défaut : 1,5] » alors que l'option s'écrit `--clip-margin 1.5`.
- Les causes techniques restent en anglais dans le seul détail d'un échec (#64) et la ligne d'un extrait refait.
  Aucun refus avant calcul n'interpole de texte anglais (#55).
- `Setting::flag()` nomme l'option pareil dans la commande et au banc.
  La phrase d'un refus de réglage reste en deux copies : anglaise au banc et Fluent dans la commande.

#### Échecs et pannes

- **Raisons en clair** (#64) : une vidéo en échec écrit sa raison, son « à faire » et le détail anglais de ses causes sur la sortie d'erreur.
  Les autres vidéos continuent. « Trop de pistes » a sa raison et son « à faire », sans détail.
- `failure::cause` classe sans texte.
  Côté lecture, le `match` est exhaustif et sans `_` sur les erreurs de `process`, de la détection, de l'entrée et du décodeur.
  `detection::Error::Input` est transparent : seul le `match` mène donc à `input::Error`.
  Côté écriture, c'est la chaîne `source()` qui classe sans `match` sur les énumérations de `files`, `render` ou `fallback`.
- Une copie tronquée depuis les jumelles perd le `moov`, écrit en fin de fichier.
  À l'ouverture, FFmpeg ne la distingue pas d'un fichier qui n'est pas une vidéo : les deux ont la même raison.
- Les réglages ne sont mis en cause que si `--work-width` ou `--osd-region` a été donné. Sinon c'est un format non géré.
- Disque plein, écriture refusée et fichier ouvert ailleurs sous Windows (erreurs 32 et 33 au ménage, #115) ne se cherchent que du côté écriture.
  Le fichier ouvert se reconnaît sur les seules erreurs du ménage qui portent le chemin.
  Pour un dossier d'extraits, c'est le dossier qui est nommé.
- **Pannes** (#113) : `outcome::run_all` passe chaque vidéo dans `catch_unwind`. C'est le seul du code livré (règle 10).
  - Le travail est `Fn + RefUnwindSafe`. Le compilateur vérifie ainsi qu'il ne modifie rien de ce qu'il capture.
    Une panique donne « erreur interne » avec le message et le lieu en détail. Elle compte comme un échec et la vidéo suivante est traitée.
  - Elle peut repartir parce que les fils d'une vidéo sont rejoints ou arrêtés avant la relance.
    Deux tests `should_panic` le vérifient pour une panique du consommateur.
  - Le crochet de panique est global et non par fil parce que `resume_unwind` n'appelle pas le crochet.
    Pendant une vidéo, il n'écrit rien et garde la première panique. Hors d'une vidéo, il passe la main au crochet par défaut.
    Le lieu est celui que rustc connaît : `crates/…` sous macOS et Linux et `crates\…` sous Windows.
  - Une panique laisse les fichiers déjà écrits, comme un échec.
    Une panique dans l'initialisation de FFmpeg (un `Once`) l'empoisonne. Les vidéos suivantes paniquent aussi.
    Le manque de mémoire, une panique pendant une panique et une panique dans un `rayon::spawn` ne sont pas rattrapés.
    La borne de la fenêtre du fond (`engine`) écarte la première cause connue.

#### `params.json`

Son contenu est décrit dans [02](02-methode.md#sorties).

- Les réglages sont rangés par configuration aux clés de la v0.1.1. Un filtre coupé s'écrit 0.
  `threads` donne les fils réellement employés pour le décodeur et pour `rayon`.
- `version` est celle de `--version`. Elle est gravée par `crates/cli/build/engraved.rs` (#113).
  C'est `BATDETECT_VERSION` si elle est posée, sinon `git describe --tags --match 'v[0-9]*' --dirty --always` sans le `v`, sinon `CARGO_PKG_VERSION`.
  - Le script ne se relance que si `HEAD`, la branche qu'il nomme, les tags, `crates/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/`, `packaging/` ou `tests/` changent.
    Ce sont les chemins de la provenance de `record`, qui partage ce script (#295).
  - Un commit d'un autre worktree ne le relance donc pas. « -dirty » ne voit pas une modification de `docs/` ou de `mise.toml`.
    Toute modification de source recompile `cli`, la crate feuille.
- `video` est écrit par `to_string_lossy` (#227) : un nom non UTF-8 y garde ses caractères lisibles et ses octets illisibles deviennent `�`.
  C'est un texte à lire et non un chemin à rouvrir.
  La relance compare la taille, la version, la langue et les réglages. Elle ne compare jamais ce nom.
  Des octets échappés dans la chaîne auraient changé la forme du champ pour tous les noms et demandé un décodage à chaque lecteur.
- Écarts à la v0.1.1 : `mode` et `ffprobe` sont retirés.
  `crf`, `encoder` et `vt_quality` sont remplacés par `encoders`, la liste retenue par l'essai (vide au-delà de `--max-tracks`).
  `version`, `ffmpeg` et `threads` sont ajoutés. Les nombres réels s'écrivent `12.0` et non `12`.

#### Tests de la commande

- `tests/command.rs` lance le binaire sur une vidéo fabriquée : en français, en anglais, puis en français dans le même `-o` (#115).
  `slow_command.rs` passe l'extrait. `reference_092.rs` passe la 092 ([Gardes de détection](#gardes-de-détection)).
- `tests/local_outputs.rs` passe l'extrait trois fois dans le même `-o`.
  Entre la première et la deuxième passe, il ajoute des anciens noms et des parties en trop.
  Après chaque passe, la liste des fichiers égale celle de la v0.1.1 au nom près. Comme elle, le Rust ne fait aucun zoom sur l'extrait.
- `tests/local_command.rs` passe `process` sur les 26 vidéos et apparie ses pistes à celles de la v0.2.0 ([Gardes de détection](#gardes-de-détection)).

### `tracking`

Cette crate exclut les images abîmées et instables (#120) puis fait le suivi (#121).
`compare` et `cli` enchaînent `exclude` puis `track_detections`.
La logique est dans [10](10-logique-de-detection.md). Les valeurs par défaut sont dans [02](02-methode.md).

- `exclude` ne peut pas échouer. Les images abîmées sont retirées du calcul de stabilité.
  Puis les détections des plages abîmées et instables sont vidées. Les images sont gardées.
- Les réglages s'écrivent et se lisent aux clés de la v0.1.1 (#108).
  Ainsi `params.json`, `bench.json` et `run.json` partagent une seule forme.
  Un filtre coupé s'écrit 0. Les clés venues depuis (recollage, morceaux, portée) se lisent à 0 dans un fichier plus ancien.
- **Stabilité** : la limite de foule vaut `max_blobs + 6 × médiane` du nombre de détections des images lisibles.
  Les images vides comptent dans cette médiane. Le calcul se fait en entiers.
  - Le rembourrage ajoute `pad_s × cadence` images de chaque côté et s'arrondit moitié vers le haut.
    La v0.1.1 arrondit au pair. L'écart est sans effet aux réglages par défaut.
  - Seuls les bords sont bornés aux premières et dernières images lisibles.
    Au milieu, une plage instable peut recouvrir une plage abîmée (093, 097).
  - La médiane porte sur toute la vidéo.
    La stabilité dépend donc de sa longueur : la concaténation 089-091 ignore 27,9 s contre 28,9 s en morceaux (mesuré par #94).
    La médiane comme cause est une hypothèse non vérifiée.
  - Les 3 tests de saut de luminosité de la v0.1.1 sont dans `compare` parce qu'ils enchaînent détection et stabilité.
- **Suivi** : les pistes sont numérotées à partir de 1. Elles sont triées par image de début puis par `x` du premier point.
  `Error::Track` ne vient d'aucune entrée : c'est un garde-fou contre un bogue de l'association ou de la fusion des jumelles (deux points sur une même image).
  Écarts voulus à la v0.1.1 : NaN et infini sont refusés pour les réglages réels et `twin_distance` à 0 devient `None`.
- **Détections faibles** (#21) : l'association attribue d'abord les détections fortes.
  Elle attribue ensuite les faibles aux pistes restées sans détection.
  Une faible libre n'ouvre pas de piste. Un point réuni par les jumelles est fort si l'un des deux l'est.
  Rien ne change sur des détections toutes fortes comme celles des références ([10](10-logique-de-detection.md#7-relier-les-taches-dans-le-temps--le-suivi)).
- **Recollage** (`track/rejoin.rs`, #24) : la règle et les mesures sont dans [10 § 9 bis](10-logique-de-detection.md#9-bis-recoller-un-vol-caché-derrière-la-végétation).
  Les valeurs sont dans [02 § 7 bis](02-methode.md#7-bis-recollage).
  - `max_hidden` doit dépasser `max_gap`. Sinon le recollage n'aurait rien à faire.
  - Les liens se prennent du plus petit écart au plus grand. Chaque fin et chaque début n'entre que dans un lien.
    Une chaîne de liens donne une seule piste. Aucun point n'est ajouté dans le trou.
  - Une piste recollée garde ses bouts. Le zoom automatique la prend si l'un d'eux y avait droit seul.
    Sans cela, 10 pistes des 095, 096, 097, 098 et 100 perdaient leur zoom.
- **Morceaux** (`track/rejoin/pace.rs`, #311) : les pistes écartées par les filtres qui ont de `piece_points` à `min_hits − 1` points peuvent relier ou prolonger une piste gardée.
  Elles ne relient ni ne prolongent jamais un autre morceau.
  - Les liens entre pistes gardées sont pris d'abord. Un morceau ne peut donc pas défaire une soudure.
    Une chaîne sans piste gardée est jetée.
  - Un `piece_points` au moins égal à `min_hits` laisse la pile vide sans erreur. `--min-hits 2` reste donc permis.
  - Chaque lien se juge à l'allure.
    Le saut doit valoir au moins `junction` fois la vitesse de chaque piste gardée du lien (5 points du côté du trou).
    Un morceau d'au moins 2 points doit aller au moins à `piece` fois la vitesse de la piste.
- **Portée** (`track/reach.rs`, `track/edge.rs`, #326) : une piste qui a une vitesse ne prend une tache qu'à `margin + travel × trajet prédit` de sa prédiction.
  Cela vaut dans les deux tours, fort et faible. Une piste d'un point n'a que `max_jump`.
  - La portée ne joue pas si le dernier point, la tache ou le point d'où se mesure la vitesse est au bord.
    Le bord fait 30 px : c'est la même marge que pour le recollage.
  - Le trajet prédit prend la vitesse des 5 derniers points quand celle de la prédiction tombe sous 0,4 fois elle.
    La prédiction elle-même reste la même au bit près.
- `--replay` et `--targets` du banc jugent aux réglages de la commande, donc avec recollage et portée.
  Seul `--against` exige les réglages enregistrés. Il faut donc `--rejoin-hidden 0 --reach-margin 0` contre un banc de la v0.1.1.
  `local_guard` coupe de même recollage et portée (`without_rejoin().with_reach(None)`).
- Limite : plusieurs seuils tombent sur une distance exacte. Or `f64::hypot` peut différer d'un ulp de celui du Python.
  Le critère des tests est donc l'appariement avec tolérances. L'identité est mesurée en plus.
- **Gardes** (#277) : le suivi courant tourne sur les détections, les plages abîmées et le nombre d'images signalées de la [référence v0.2.0](#référence-v020).
  Il ne décode rien et ne dépend donc ni du poste ni du décodeur. Ses pistes se comparent à celles de la v0.2.0 par `track_matching`.
  Chaque cas a sa ligne figée : pistes de la référence, suivies, appariées, concordantes, perdues et ajoutées.
  Une PR de suivi met à jour ces lignes et donne la cause dans la même série de changements.
  Elle met aussi à jour les colonnes de pistes des [gardes de détection](#gardes-de-détection). Ces colonnes disent l'effet.
  - Les 19 cas du dépôt n'ont aucune plage abîmée. Seul `sigma_0` a des plages instables.
    En CI, le suivi sur des plages abîmées n'est donc gardé que par les tests unitaires.
    Sur des données réelles, seuls `local_tracking` et `local_exclusion` le gardent.
  - `committed_exclusion` (19 cas du dépôt) et `local_exclusion` (26 vidéos) comparent l'exclusion à la v0.2.0.
    Les détections gardées en moins et en plus et les plages instables changées sont toutes à 0.
  - `committed_tracking` couvre les 19 cas du dépôt (extrait, 092, 1440, 16 réglages) en CI sur les trois systèmes.
    Sa ligne porte en plus le nombre de pistes identiques au bit près et les numéros 1..n.
    Le recollage (#24) n'y change rien. La portée (#326) change trois cas. Ces trois cas gardent 92 pistes identiques au lieu de 125 :
    - `sigma_0` (l'extrait sans seuil par pixel) passe de 30 à 17 pistes : 13 appariées et concordantes, 17 perdues, 4 ajoutées, une seule identique.
      Les 13 pistes de moins sont du bruit. Les 4 chauves-souris de l'extrait restent.
      Elles commencent aux images 144, 707, 781 et 972 au lieu de 144, 705, 783 et 972.
    - `width_720` garde ses 5 pistes appariées, mais 2 ne concordent plus.
      Les deux pistes jumelles entre lesquelles la chauve-souris des images 705-722 était déjà coupée se la partagent autrement.
    - La 092 à 1440 garde ses 14 pistes appariées et concordantes. 2 d'entre elles ont d'autres points.
      La piste de 2:02.85 ne commence plus par deux points sur une tache immobile (images 3688-3689).
      Cette tache est à 77 px du premier point de la chauve-souris.
      À 1:49.75, la portée arrête là un morceau de deux points. Il reste à 27 et 34 px de la piste 6 et s'y fond comme une jumelle.
      Cette fusion déplace les points de la piste 6 aux images 3294 et 3296 de 24 et 2 px.
  - `local_tracking` porte sur les 26 vidéos : 972 pistes dans la référence et 902 suivies.
    On y trouve 849 pistes appariées et 774 concordantes. 123 sont perdues et 53 ajoutées.
    Le nombre de pistes identiques (même numéro, mêmes points au bit près) est figé vidéo par vidéo (#340).
    Il valait 171 en tout sur lumiel le 18 octobre 2026, après #21, #311 et #326.
    Une piste recollée reste appariée à sa première partie mais n'est plus concordante. Sa seconde partie compte comme perdue.
  - `committed_metrics` déclare les réglages gagnés depuis la v0.2.0 (recollage, `reach_margin` et `reach_travel`) et fige leur effet sur les deux bancs.
    Au transit, 163 pistes sont injectées et 4 cibles changent par rapport à la v0.2.0.
    La 79 commence 23 images plus tôt et la 151 trois plus tard. La 213 finit 3 images plus tôt. La 167 tient en un fragment au lieu de 2.
    On compte aussi 3 classes changées (passage d'amplitude −15 à sigma 5 : délai médian de début de 4,5 images au lieu de 6,5).
    À la chasse, 498 pistes sont injectées au lieu de 514 et 12 cibles sont moins fragmentées.
    Une cible est perdue (454 trouvées au lieu de 455) : sa piste a été soudée à celle d'une autre cible qui la précédait au même endroit.

### `files`

Les fichiers écrits par la commande (#122) suivent la langue de la commande.

- **Noms** (`names`) : les noms français sont sans accent. Ce sont `extraits`, `piste`, `_pistes`, `_resume`, `_annotee` et `_fond`.
  En anglais, ce sont `clips`, `track`, `_tracks`, `_summary`, `_annotated` et `_background`.
  `params.json` n'est pas traduit et `_zoom` est commun.
  - Le nom d'un extrait dit son début en minutes et secondes entières.
    Elles sont tronquées comme dans la v0.1.1 : à 59,996 s, le nom dit `0m59s` et le CSV `1:00.00`.
  - Les noms de vidéo sont des `OsStr` gardés tels quels.
  - Une troisième table donne les anciens noms (ceux de la v0.1.0). Elle ne sert qu'à les reconnaître au ménage (#115).
- **Ménage** (`previous`, #115) : il retire d'abord `params.json` et ses partiels. Ainsi un dossier interrompu ne passe jamais pour terminé.
  Puis il retire les sorties d'une passe précédente dans les deux langues et sous les anciens noms. Le reste du dossier n'est pas touché.
  - Comme la v0.1.1, il compare en NFC le nom de chaque entrée à celui de la vidéo.
  - Une entrée au nom du tableau qui n'est pas un fichier ordinaire (dossier, lien) est laissée en place et rendue à l'appelant.
  - Un lien qui porte le nom d'un dossier d'extraits est retiré. Sa cible ne l'est pas.
- **Partiels** (#245) : tout fichier s'écrit d'abord dans un partiel `.<nom>.<6 lettres ou chiffres>.partial` à côté du fichier final, puis il est renommé.
  Le nom du partiel est tiré au hasard et le partiel est créé de façon exclusive.
  - Deux écrivains du même fichier ont donc chacun le leur.
    C'est le cas du fil du Ctrl-C qui réécrit la page pendant que le fil principal l'écrit, ou de deux `batdetect -o` sur le même dossier.
    Le dernier renommage gagne et chaque fichier final est entier. Pour le tableau des pistes, c'est le premier qui gagne.
  - `tempfile_in` n'est pas employé parce qu'il crée le fichier en `0600`. Le fichier final garderait ce mode.
  - Le ménage reconnaît cette forme quel que soit le mot de 6 lettres ou chiffres.
    Il reconnaît aussi l'ancienne forme `.<nom>.partial` laissée par la 0.2.0.
    Un partiel laissé par un arrêt brutal à la racine (page) ou dans `.batdetect/echecs/` n'est pas écrasé au passage suivant : il reste caché.
- **Tableau des pistes** : le renommage n'écrase jamais un fichier présent.
  - Le renommage exclusif passe en premier.
    Sur toute autre erreur que « nom déjà pris », le nom final est réservé puis le partiel est renommé par-dessus.
    Le refus de macOS sur exFAT (#236) est l'une de ces erreurs.
  - Un arrêt brutal pendant l'écriture ne laisse ainsi qu'un partiel, que le ménage retire.
    Il ne laisse jamais un CSV tronqué qui passerait pour étranger.
  - Un `vol_pistes.csv` annoté et la vidéo `VOL.mp4` ont le même chemin sur un disque insensible à la casse (APFS, NTFS).
    Le ménage compare les octets et ne reconnaît donc pas ce fichier. Le renommage refuse de l'écraser.
  - Sous macOS seulement, `tests/slow_exfat.rs` monte une image exFAT par `hdiutil` sans `sudo`.
    Le tableau s'y écrit et refuse d'écraser un tableau annoté. Puis il est retiré par le ménage et se réécrit.
    macOS 27 écrit sur cette image un `._x` (AppleDouble) à côté de chaque fichier.
    La liste des fichiers restants des tests les écarte mais garde les partiels cachés (#335).
- **Temps** (`time`) : le temps d'une image vient de son `pts`.
  - Une image sans `pts` reprend celui de la dernière image qui en a un, plus un pas de cadence par image.
    La cadence seule ne sert qu'en l'absence de tout `pts`.
  - La fin d'une plage est le début de l'image suivante.
  - Le temps n'est pas rapporté au premier `pts` puisque les 26 vidéos de la référence commencent à `pts` 0.
    Aucune vidéo qui commence ailleurs n'a été vue.
  - `clock` écrit `h:mm:ss` tronqué à la seconde, comme le nom de l'extrait.
    C'est aussi le seul format du début d'une piste sur la page. Le binaire l'écrit et le script le relit (#304).
    Les temps d'axe y restent en `m:ss` comme sur l'image résumé.
- **CSV** (`table`, #84) : le tableau a 12 colonnes.
  Le binaire donne les en-têtes Fluent à travers le trait `table::Headings`. `files` ne les connaît donc pas.
  Les colonnes et leur forme par langue sont dans [02](02-methode.md#sorties).
  Les verdicts n'y sont pas : ils vivent dans `verdicts.csv` (#228).
  - Les nombres sont arrondis puis écrits sous leur forme la plus courte, avec `.0` pour un entier. `-0.0` devient `0.0`.
  - La surface max est entière, arrondie à mi-chemin vers le haut.
- **`params`** : `files` n'impose pas les clés des réglages.
  Garder celles de la v0.1.1 revient à l'appelant. `cli` porte le test des clés et celui des seuils de zoom.
- **CSV contre la v0.1.1** : le CSV est écrit depuis les pistes de la référence (`files/tests/committed_table.rs`).
  La comparaison a été mesurée le 11 octobre 2026 et relue par valeurs le 16 octobre 2026.
  - Sur l'extrait, les valeurs sont les mêmes avec début et fin tronqués à la seconde.
  - Numéro, points, corde, chemin, amplitude, images comblées et durée sont égaux sur la 092 (14 lignes).
  - Les temps sont plus tôt d'un ou deux centièmes.
    Les débuts sont décalés sur 11 pistes sur 14 (de 33,94 → 33,93 s à 237,18 → 237,16 s). Les fins le sont sur 12.
    Tronquée à la seconde, seule la fin de la piste 6 change (1:50.00 → 1:49.99, soit `0:01:49`).
    La v0.1.1 divisait le numéro d'image par la cadence moyenne (30,027 i/s). Or les `pts` avancent de 333/10 000 s par image (30,03 i/s).
  - `speed_px_s` est plus haute de 0,1 à 0,2 px/s sur 10 pistes et jamais plus basse.
  - `max_area_px` change sur les pistes 1, 5, 6 et 10 : 274 → 275, 400 → 401, 490 → 491 et 274 → 275.
    Les maxima tombent sur un demi (les surfaces de la 092 sont des multiples de 2,25).
    Le Rust arrondit à mi-chemin vers le haut et la v0.1.1 au pair. L'écart est gardé (règle 5).
- **Lecture d'un dossier de résultats** pour la page (`results`, #71) :
  - Un sous-dossier non caché avec un `params.json` est une vidéo terminée.
    Sans `params.json` mais avec une sortie nommée d'après la vidéo, elle est inachevée. Un dossier `clips/` seul ne compte pas.
    Ni l'un ni l'autre : le dossier est ignoré pour ne jamais lister un dossier de l'utilisateur.
  - Les images résumé sont celles que nomment les pistes de `params.json`. Leur taille vient de l'en-tête PNG.
  - Un tableau absent sous un nom qui contient `�` se cherche sous le fichier du dossier dont le nom lossy est le même (#227).
    Une vidéo au nom non UTF-8 retrouve ainsi son tableau.
    Elle ne retrouve pas ses images résumé ni sa vidéo annotée, cherchées sous le nom lossy.
  - `verdicts` (#228) prend le plus récent des fichiers `verdicts*.csv` de la racine parce que le navigateur numérote les doubles.
    Il n'en lit que les octets : c'est le JS de la page qui les lit.
  - Un fichier d'échec ou de refus illisible ne cache que lui-même.

### `render`

La crate rend les extraits, les zooms et la vidéo annotée du choix des images jusqu'à l'écriture (#110, #112).
`cli` choisit les zooms par `ZoomConfig::wants` (#103). Les couleurs et les tailles viennent de `draw` et les encodeurs de `media`.

- Pistes et couleurs se réunissent une fois (`Colored`, #287).
  Une liste de couleurs vide (boîte rouge et trace jaune pour toutes) ou d'autant de couleurs que de pistes est acceptée.
  Toute autre longueur est refusée au lieu de retomber en silence sur la boîte rouge.
- `drawn` (feature `testing`, #198) passe par le même rendu sur un fil.
  Chaque encodeur y est remplacé par un enregistreur qui garde l'empreinte de chaque image poussée.
  C'est l'entrée des [tests figés](#tests-figés).
- **Lecture** (`source`) : les images-clés et les images en erreur viennent de la lecture du moteur. `render` ne dépend donc pas d'`engine`.
  - Les images se choisissent par `pts`, cherchés dans la `Timeline`.
    Une `Timeline` dont les `pts` connus ne croissent pas strictement est refusée avant tout décodage.
    Un `pts` inconnu, une image qui revient ou une image de la plage qui manque sont des erreurs typées.
  - Image de reprise : la dernière image-clé à ou avant le début de la plage qui n'est pas elle-même en erreur.
    La dissimulation d'une image-clé en erreur puise dans les images d'avant. Seule une image-clé saine remet le décodeur à zéro.
    Il n'y a pas de marge réglée : celle de 30 du prototype supposait des GOP de 15 (ici de 8 à 16).
  - Le décodage se fait sur un seul fil quand la plage lue contient une image en erreur.
    Les pixels dissimulés ne se reproduisent en effet qu'à un fil (voir « Fils et pixels dissimulés » dans [`engine`](#engine)).
    Sinon les fils sont automatiques, car une vidéo saine donne les mêmes pixels quel que soit leur nombre.
  - L'appelant sert une plage par groupe de fenêtres chevauchantes pour ne décoder qu'une fois une image partagée.
    Le parallélisme vient des groupes. Il borne aussi les fenêtres à la vidéo. La v0.1.1 ne les bornait pas.
  - La plage est relue depuis le début si le placement échoue ou si la première image rendue vient après l'image de reprise (conteneur sans index).
    Ce dépassement est testé par une image-clé forgée avant la première image du mp4. FFmpeg la ramène à l'image 0 (`mov.c`).
    Aucun mp4 ne fait échouer le placement. `Unplaced` et `OutOfOrder` restent des gardes non couvertes.
  - Hypothèse : une image-clé saine remet le décodeur à zéro.
    C'est vrai d'une IDR. Mais FFmpeg marque aussi image-clé une image à point de reprise (SEI `recovery_frame_cnt` nul).
    Une telle image n'est pas forcément indépendante. Seuls l'extrait et les 103, 123 et 124 confirment l'hypothèse.
    Le placement passe par le flux vidéo par défaut de FFmpeg et le décodeur par `best(Video)`.
    Les deux coïncident tant que le fichier n'a qu'un flux vidéo.
  - `Source` ne garde de la `Timeline` que le nombre d'images et la table `pts` ↔ image.
    Sans durée de vie, elle est `Send` et `Sync`. Un test sert trois plages depuis trois fils.
- **Mesures de la lecture** (M1 Max, `--profile ci`) : chaque plage est comparée à une lecture séquentielle à un fil, empreinte par image.
  - Test rapide : une vidéo openh264 1440×1080 de 48 images a une image-clé toutes les 8.
    La sixième part sur douze de son `mdat` est écrasée. Elle donne 40 images et l'image-clé 16 en erreur : 8 images sont perdues d'un coup.
    Les plages qui commencent de 16 à 23 reprennent à l'image-clé 8.
    Reprendre à l'image-clé 16 en erreur fait échouer la plage 16–19 (essai fait une fois).
  - `slow_source` tire dans l'extrait 12 plages de 1 à 380 images (SplitMix64, graine fixe). Elles sont égales et le test prend 24 s.
  - `local_source` lit la fenêtre 6580–6684 de la 103. C'est le seul extrait des 26 vidéos qui traverse une perte d'images.
    Elle reprend à l'image-clé 6573 et rend ses 105 images égales (63 s).
    Les 8 fenêtres d'extraits de la 123 et les 3 de la 124 qui touchent une plage abîmée sont égales (140 s pour les deux).
    Aucune de ces 12 fenêtres ne saute une image-clé en erreur : seule la vidéo fabriquée exerce cette règle.
- Réglages (`config`) : ce sont ceux de la v0.1.1, validés à la construction.
  Les secondes deviennent des images par `round(secondes × cadence)`.
  Les seuils du zoom se comparent strictement sur les mesures de chacun des bouts de la piste (un seul pour une piste non recollée).
- **Dessin** : sur chaque image, toutes les pistes présentes sont dessinées une fois.
  L'image dessinée est partagée par les extraits du groupe.
  La boîte fait 2 px sur un point détecté et 1 px sur un point interpolé. `#N` est au-dessus et la trace couvre `trail_s`.
  Chaque piste a la couleur que `cli` lui passe, celle de l'image résumé (#13).
  Un liseré noir (`INK`) de 1 px borde le trait de la boîte de part et d'autre et entoure le numéro.
  Sans couleur, c'est la boîte rouge et la trace jaune sans liseré.
- **Boîte bornée** (`overlay`, #112) : elle ne sert qu'à l'affichage. La détection, le suivi et le CSV ne changent pas.
  Chaque côté de la boîte dessinée est borné à 4 fois la médiane du même côté sur les points détectés de la piste.
  Une boîte bornée est recentrée sur le barycentre de la détection puis élargie de `box_pad` comme les autres.
  Les points interpolés viennent des boîtes déjà bornées.
  - Le cas qui l'a motivée : la 092, piste 13, image 6945 (3:51).
    La tache étalée a une boîte de 339 × 235,5 px pour 353 px de surface. La piste a pour médiane 31,5 × 23,25.
    On dessinait 363 × 259 px autour de rien, comme la v0.1.1. Bornée, la boîte fait 126 × 93 px avant `box_pad`.
  - Mesure sur les 966 pistes de la référence des 26 vidéos (33 865 points, `rapports/mesure-112/boxes.py`, hors dépôt) :

    | Facteur | Boîtes bornées | Pistes | Réduites de plus de 12 px |
    | --- | --- | --- | --- |
    | 3 | 129 (0,38 %) | 84 | 11 |
    | 4 (retenu) | 33 (0,10 %) | 27 | 7 |
    | 5 | 15 (0,04 %) | 12 | 6 |

    Avec 4, les 7 boîtes réduites de plus d'un `box_pad` sont : 092 piste 13 image 6945, 094 piste 7 image 4444, 096 piste 79 images 4817 à 4819, 096 piste 113 image 6538 et 100 piste 34 image 2896.
    Les 26 autres boîtes bornées perdent au plus 10,5 px (médianes de leurs pistes : de 3 à 15 px).
- **Zoom** (`zoom`, #103) : c'est la vue lisible d'une petite cible.
  Ce qu'il montre est décrit dans la partie « Extrait zoomé et ralenti » de [02](02-methode.md).
  - La cadence déclarée est divisée par 4 en rationnel exact. Elle vaut 22527500/3000927 sur la 092. La v0.1.1 écrivait 908336/121001.
  - Le cadre est fixe pour la piste. Il suit la formule de la v0.1.1 appliquée aux boîtes bornées.
    C'est leur enveloppe plus 40 px, en unités du PGCD de la largeur et de la hauteur (4×3 en 1440×1080).
    Il fait au moins le quart de l'image, puis il est centré et borné.
    Le placement passe par `f64::round` et la v0.1.1 arrondissait au pair. L'écart est d'un pixel sur une égalité exacte.
  - Taille de sortie : celle de la vidéo.
    Une sortie en 720×540 a été essayée le 13 octobre 2026 pour les cadres qui y tiennent (86 % des zooms).
    Elle faisait la 099 en 36,3 s au lieu de 41,8 s et des zooms de 260 Mio au lieu de 413.
    L'utilisateur ne l'a pas retenue : la pleine taille montre mieux les objets diffus.
  - L'agrandissement se fait au plus proche, en YUV.
    Seule la trace est dessinée, en 1 px et à la couleur de la piste. Elle est coupée dans la boîte courante élargie de 16 px.
    Il n'y a ni boîte ni `#N`.
  - Le zoom est encodé en openh264 ([Zooms en openh264](#zooms-en-openh264)). Un échec d'openh264 sur un zoom fait échouer la vidéo.
- **Conversion par zones** (#112, décision de l'utilisateur) : l'image entière passe par les tables d'affichage.
  Seules les zones autour des dessins passent en RGBA, reçoivent le dessin et reviennent en YUV.
  - La zone d'une marque couvre sa boîte, son `#N` et sa trace, plus 4 px.
    Elle est calée sur des coordonnées paires (4:2:0) et arrondie au multiple de 16 pour le cache de contextes.
    Elle est bornée à l'image. Les zones qui se touchent sont fusionnées.
  - Un test vérifie, image par image, que tout pixel dessiné tombe dans une zone.
  - Sur la 099, les tuiles de 16 px touchées par un dessin font 0,56 % de l'image en moyenne et 3,8 % au plus.
  - `ACCURATE_RND` et `BITEXACT` sont gardés. Ils coûtent environ 2,1 ms par image entière, presque tout dans le sens YUV → RGBA.
- **Vidéo annotée** (`--annotated`, #103) : elle montre toutes les images qui ont un `pts` avec le dessin des extraits.
  Elle les écrit aux `pts` de la source et reste donc en phase avec le CSV après une perte d'images.
  - Elle se lit par tronçons. Une image en erreur isole un tronçon, de l'image-clé d'avant à l'image-clé d'après.
    Ce tronçon se lit sur un fil et le reste sur tous les fils.
  - Un seul encodeur sert toute la vidéo, VideoToolbox d'abord. Il pose une image-clé toutes les `round(cadence)` images.
    La vidéo annotée ne partage rien avec les extraits.
  - Relevé du 13 octobre 2026 sur un passage : +19,7 s sur la 092 (18,4 → 38,1 s) et +23,0 s sur la 103 (37,7 → 60,7 s).
  - Elle est refaite entière en openh264 sur un fil quand VideoToolbox casse. La 092 avec `--annotated` prend 133 s tout en openh264.
  - Le ménage de `cli` l'efface dans les deux langues même sans `--annotated`.
- **Ordonnancement** :
  - La vidéo annotée part la première parce que c'est la plus longue.
    Puis partent les groupes et les zooms par image de début, le groupe avant le zoom qui commence avec lui.
  - Les travaux partent sur la moitié des cœurs (`available_parallelism / 2`).
    Chaque fil garde son `Picture` car `Converter` n'est pas `Send`.
  - Un groupe réserve sa session VideoToolbox avant de partir et la garde jusqu'au bout.
  - Une erreur ou une panique arrête les autres groupes à l'image suivante.
    L'erreur rendue est celle du premier groupe dans l'ordre parmi ceux qui ont échoué avant d'être arrêtés.
  - Les extraits déjà finis restent dans le dossier quand la vidéo échoue. Le lancement suivant vide le dossier.
- **Reprise** : quand VideoToolbox casse en cours d'écriture hors erreur de destination, il est écarté pour la suite de la vidéo.
  Le groupe est refait en openh264 une fois les groupes finis.
  Une erreur de destination arrête la vidéo sans reprise et nomme l'extrait qu'elle concerne.
- **Tests** :
  - Une fausse ouverture (`testing::Fake`, un vrai `Fanout` en openh264) compte les encodeurs et les sessions.
    Elle casse à la n-ième image ou au renommage.
    Elle exerce la reprise en openh264, l'absence de reprise après une erreur de destination et l'arrêt sur un échec d'openh264, sans `.partial`.
  - Un fil et quatre donnent les mêmes fichiers à l'octet en openh264, zooms compris.
  - Boîte qui suit la cible après un trou (`cli`, `process/tests.rs`) : un vol fabriqué de 80 images a une image-clé toutes les 8.
    Un dixième de son `mdat` est écrasé (8 images perdues).
    Sur chaque point détecté, les bords rouges lus dans l'image tombent à 2 px au plus de la boîte attendue.
    Un dessin décalé d'une image fait échouer le test (essai fait une fois).
  - `slow_clips` (l'extrait, pistes de la référence, extraits de 380, 108, 115 et 117 images) : chaque image écrite est comparée à l'image source servie par `pts` et passée par la même conversion.
    Le SSIM se prend sur Y par blocs 8×8 disjoints, hors blocs qui touchent un dessin.
    Sur le Mac, VideoToolbox mesure 0,9659 à 0,9667 et openh264 0,9671 à 0,9677. Les seuils sont de 0,963 et 0,964.
    Le Y mesuré par le filtre `ssim` de FFmpeg contre l'original valait 0,9688 et 0,9698.
    Une seule lecture séquentielle de l'extrait sert les trois tests.
    Le binaire prend 76 s et 59 s sur le M1 Max sous le verrou (217 s et 212 s de CPU). La vidéo annotée seule prend 58 s.
  - `local_clips` (dans `cli`) porte sur la 103, la 123 et la 124.
    Chaque groupe qui touche une plage abîmée (1, 6 et 3 groupes) est servi avec les pixels d'une lecture séquentielle à un fil.
    Le test prend 300 s. Aucun groupe n'est relu depuis le début (épinglé).
    Les fenêtres des 26 vidéos se comparent à celles des pistes de la v0.2.0 ([gardes de détection](#gardes-de-détection)).

### `compare`

`compare` est un outil de développement (#119). Ses binaires `compare` et `record` ne sont pas livrés.
`media` n'y sert qu'à choisir les fils du décodeur et à couper l'extrait dans `slow_same_as` (`payload::wipe`, #176).

- `compare extract|092|092-1440|settings|local` détecte puis compare à une référence.
  C'est la [v0.2.0](#référence-v020) par défaut ou la v0.1.1 par `--reference v0.1.1`.
  - Les réglages de détection sont ceux de l'enregistrement, lus dans son `video.json`.
    Pistes et stabilité restent aux défauts comme à l'enregistrement.
  - `local` détecte les 26 vidéos 6 à la fois : seul `compare 092` chronomètre la détection seule.
  - On le lance depuis la racine du dépôt, sous le verrou.
- **La règle** est la même pour les deux références (#275) :
  - **Entrées.** Le nombre d'images, les `pts` et les images-clés de la lecture courante doivent être ceux de la référence.
    Contre la v0.2.0, les pertes aux bords doivent l'être aussi. Sinon la vidéo échoue et rien d'autre ne se compare.
    Les erreurs de décodage ne sont pas exigées puisque le décodeur de lumiel en voit 14 de plus que celui du Mac (voir « Dégâts contre la référence » dans [`engine`](#engine)).
    Elles sont comptées de chaque côté.
  - **Plages abîmées.** Les images en erreur, les sauts et les pertes aux bords des deux côtés sont réunis.
    Ils sont tracés en plages par la règle courante (`Damage::reported`, marge = demi-fenêtre de la référence).
    Une plage que la règle change (#140) se voit donc dans les pistes.
  - **Détections** : elles se comparent hors de ces plages.
    Dans les plages de la référence, les détections de celle-ci viennent de pixels dissimulés.
    On cherche d'abord l'égalité exacte image par image, puis l'appariement à moins de 3 px.
    Les plages instables ne sont pas écartées : elles se déduisent des détections, qu'on compare brutes.
  - **Pistes** : on passe `exclude` avec les plages retracées puis `track_detections` aux réglages par défaut.
    Vient ensuite l'appariement des pistes.
  - Il n'y a pas de seuil : un écart se lit dans le rapport. Les gardes figent les leurs par vidéo.
    Le code de sortie vaut 1 si une vidéo échoue, si elle diffère du passage de `--same-as` ou si aucune vidéo n'est comparée.
    Il ne vaut jamais 1 pour un écart à la référence. Sinon `compare` resterait rouge dès la première PR de détection.
- `--frames` écrit les détections brutes de chaque vidéo avant exclusion, au format de la référence.
  Avec `settings`, il écrit un dossier par cas.
  `pts`, images-clés et erreurs de l'extrait sont égaux à ceux de la référence sur les 1 230 images.
- **`--same-as`** (#176) dit si ce passage donne les mêmes détections qu'un passage précédent écrit par `--frames`, hors des plages abîmées.
  - Les pixels que FFmpeg dissimule dans ces plages varient d'une lecture à l'autre.
    Deux passages de `main` y différaient sur 17 vidéos et 2 351 images (#159). `cmp -r` entre deux dossiers `--frames` ne prouve donc rien.
  - La comparaison se fait en direct plutôt qu'entre deux dossiers.
    En effet `frames.jsonl` ne garde pas les pertes aux bords, qui ouvrent aussi une plage.
    Les plages sont donc celles du seul nouveau passage : des pertes au bord vues par l'ancien seul donnent « differs ».
    C'est le côté prudent.
  - `slow_same_as` coupe un vingtième de l'extrait par `payload::wipe`. Il le détecte ensuite à un fil et en fils automatiques.
    Le résultat est identique hors de sa plage. Sur lumiel, la plage couvre les images 601 à 644 et contient 27 images différentes.
    Le test ne demande pas ce nombre parce qu'il varie d'une lecture et d'une machine à l'autre.
    Le test exige que les fils automatiques décodent par images. Sinon ses deux passages liraient la vidéo de la même façon.
  - `slow_extract` relance l'extrait avec `--same-as` sur le dossier qu'il vient d'écrire.
- **`record`** (#250) enregistre la [référence v0.2.0](#référence-v020).
  - Pour chaque cas, il détecte avec les réglages de la v0.1.1.
    Il vérifie que le flux lu par `media` est celui de la v0.1.1 (taille, cadence, base de temps).
    Il suit comme `cli::process::tracked` et écrit au format 2 avec la même arborescence que la v0.1.1.
    Un test de `cli` exige les mêmes exclusions et les mêmes pistes que `tracked` sur `spot.mp4`.
    `slow_record` exige au bit près les détections de `tests/fixtures/rust/`.
  - Provenance : le script de construction de `cli` (`crates/cli/build/engraved.rs`) grave `git describe --dirty`, le commit, `rustc --version` et l'état de `git status` sur les chemins surveillés.
    `compare` reprend ce script par la clé `build` de son `Cargo.toml` (#295).
    Un fichier non suivi sous `crates/`, `.cargo/`, `packaging/` ou `tests/` rend aussi le binaire `-dirty`.
    Il le faut parce que `git describe` ne voit pas un tel fichier.
    Or Cargo prend tout seul un `build.rs` non suivi et `.cargo/config.toml` choisit le FFmpeg lié.
  - `record` refuse de tourner si le binaire est `-dirty` ou si `git status` n'a pas pu tourner.
    Il refuse aussi un binaire construit hors de git, sans rustc ou après aucun tag de version.
    La référence prend le nom de ce tag, préversion comprise. `v0.3.0-rc1-4-g28d5526` donne ainsi `v0.3.0-rc1` et non `v0.3.0` (#275).
    L'enregistrement se fait donc depuis un clone propre du commit et toujours par `cargo run`.
    Les fixtures de la v0.1.1 sont lues à l'exécution.
    Seul `cargo run` reconstruit un binaire dont une fixture a changé, et donc le marque `-dirty`.
  - Le script est seul dans son dossier `build/` parce que `cargo shear` prend le dossier du script pour la racine des sources de `compare`.
    Il n'est pas partagé par `#[path]` parce que `cargo fmt` suivrait le `mod tests` du module.
  - `--out` est refusé avant toute détection s'il est dans `tests/fixtures/` ou dans le dossier d'une référence ou de `--same-as`.
    Il l'est aussi s'il contient l'un d'eux. Comme l'arborescence est la même, `record` écraserait sinon la référence.
    Le refus ne crée pas `--out`. La v0.2.0 s'enregistre donc ailleurs puis se copie.
  - `--same-as` compare les détections à celles d'un passage `compare --frames` et non à un dossier écrit par `record`.
- **Identité entre systèmes.**
  - Une seule opération de la chaîne de détection (`engine`, et `WorkScale` dans `types`) dépend de la libm.
    C'est `exp`, qui calcule le noyau du flou une fois par vidéo.
    Les σ de travail du banc sont 0,5, 0,75, 1, 1,5 et 2. Le σ 0 n'appelle pas `exp`.
    Pour chacun de ces σ et pour 3, un test vérifie que chaque part du noyau est à plus de 10⁻⁶ d'un entier.
    Il vérifie aussi que la répartition des restes ne bascule pas pour 10⁻⁶. Un ulp d'écart ne change donc aucun poids.
  - Rien n'est garanti hors de ces σ.
    Sur 99 901 σ de 0,01 à 10, aucun poids ne change quand chaque `exp` varie au hasard de −1 à +2 ulp (8 tirages par σ, relecture de #150).
    Ce n'est pas une preuve.
  - Le reste de la chaîne n'emploie que les quatre opérations, `abs`, `midpoint`, `min`, `max`, `round`, les conversions et le tri par `total_cmp`, toutes exactes ou fixées bit à bit par IEEE 754.
    La seule exception est `max` entre +0 et −0. Elle n'arrive pas : on compare toujours au plancher du seuil ou à un pic qui le dépasse.
  - `area_to_source` multiplie par `échelle × échelle` et non par `powi(2)`, dont Rust ne fixe pas la précision.
    Rust ne contracte jamais a × b + c en FMA.
  - Le décodage H.264 est exact au bit près par la norme, sauf dans les images dissimulées. L'extrait n'en a aucune.
  - Hors de la chaîne, `hypot` (appariement, association du suivi) et `atan2` (virage médian) viennent de la libm.
    Un écart d'un ulp ne ferait basculer qu'une paire à 3 px tout juste ou un choix entre deux distances égales.
    Il ne touche pas `frames.jsonl`.
- **Sauts de luminosité** (`tests/slow_brightness_jump.rs`, les 3 tests de stabilité de la v0.1.1) :
  - Une vidéo de 90 images 1440×1080 à 30 i/s (gris 200, bruit σ 3, saut à l'image 45) est encodée par l'openh264 de `media`.
    Elle passe par la détection, l'exclusion et le suivi.
    Le test passe par une vidéo plutôt que par une entrée directe du détecteur pour n'ouvrir aucune API dans `engine`.
  - Il est au niveau `slow` : il prend 3,3 s en profil `ci` mais 368,8 s sous l'instrumentation de la couverture.
  - Un saut de 20, 30 ou 40 niveaux ne donne aucune piste. Un saut de 30 donne une plage instable [9, 80].
    Le test vérifie qu'elle laisse des images de chaque côté.
  - Un saut de 20 n'allume aucun pixel. À l'échelle 3 et sans filtre, la v0.1.1 y voyait quelques taches.
  - La bande où un saut donne des taches sans plage instable n'a ici qu'un niveau de large : 28 (22 détections).
    Un saut de 29 en donne 1 617 avec la plage [9, 80]. Le test du saut modéré garde donc 20 et vérifie que rien n'est allumé ni ignoré.
  - Le saut de 30 n'est qu'à un niveau du bord.
    Si openh264 n'encode pas la même chose sur un autre système, ce test le montrera. L'écart remonte alors à l'utilisateur.

### `bench`

`bench` est le banc de mesure (#107 à #109). C'est une bibliothèque et le binaire `batdetect-bench`.
Sa commande et sa méthode sont dans [08](08-banc-de-mesure.md). Sa garde contre la v0.1.1 est dans [Garde](#garde).

- `random` porte SplitMix64 et xoshiro256\*\*. Ils sont écrits dans le dépôt et épinglés sur les sorties publiées par leurs auteurs.
  Chaque cible a sa graine, tirée de `--seed` dans l'ordre des numéros comme `SeedSequence.spawn` de numpy.
  Le tirage d'une cible ne dépend donc pas du nombre d'essais des précédentes.
  `uniform(a, b)` peut rendre `b` par l'arrondi. `below(n)` tire par rejet, sans biais.
- Trajectoires (saccades, Bézier quadratique, transit, chasse) et placement portent le calcul de la v0.1.1 avec les mêmes constantes.
  Ils ne portent pas l'ordre de ses tirages (règle 5).
- Le tirage passe par `exp`, `atan2`, `powf`, `hypot`, `sin` et `cos` de la bibliothèque mathématique du système.
  Un ulp d'écart peut changer un arrondi ou un rejet et ensuite tout ce qui suit.
  Une graine n'est donc reproductible que sur un même système. Aucun tirage n'est épinglé en CI. Les tests épinglent des propriétés.
- **Injection** : elle se fait sur le plan Y avant la détection, par `detect_injected`.
  Pour chaque pixel, le gris visé est `gris(Y) + ⌊a·exp(−d²/2σ²)⌋` borné à [0, 255]. Y devient ensuite le Y dont le gris est le plus proche.
  - La table `GRAY` saute un gris sur six environ.
    Un gris sauté se rend alternativement un niveau plus sombre et un niveau plus clair (décision de #107).
  - `local_injection` mesure les 986 patchs enregistrés (471 046 pixels, fonds Y de 80 à 215).
    La partie entière de la gaussienne est identique à la v0.1.1 sur 100 % des pixels.
    Le changement de gris est le même à un niveau près sur 100 %.
    Le biais moyen est de +0,0010 niveau, contre −0,1558 en allant toujours vers le Y le plus sombre et +0,1558 vers le plus clair.
  - Le contraste effectif est le changement de gris réalisé, un entier. La v0.1.1 enregistrait le changement non arrondi.
- Appariement et métriques sont ceux de la v0.1.1 ([08](08-banc-de-mesure.md)). Ils sont calculés sans arrondi (règle 5).
  La visibilité écarte en plus une cible dont le centre tombe dans une zone OSD, bornes comprises.
  Elle écarte aussi les images des plages ignorées du passage injecté. À égalité, une piste va à la première cible rencontrée.
- Un passage écrit `run.json` à côté des fichiers aux formats de la v0.1.1. Ce fichier a un format propre au Rust.
  `bench.json` garde les clés de la v0.1.1, plus `targets` et `seconds`. `code` est le `git describe --always --dirty` du dossier courant.
- Options : ce sont celles de la v0.1.1 sans `--mode` ni `--workers`, plus `--targets`, `--replay` et `--against`.
  - `--targets` rejoue positions et amplitudes. `--replay` réévalue sans détection un dossier écrit par le banc.
    Une sortie qui écraserait l'entrée est refusée avant toute détection.
  - `run.json` garde le chemin de la vidéo tel qu'il a été tapé : relu d'ailleurs, `video` ne situe plus le fichier.
  - Comme avec argparse, une vidéo placée après `--amplitudes` serait lue comme une amplitude. Elle se met donc avant.
  - Le code de sortie vaut 1 en cas d'échec ou de garde en échec et 2 pour une erreur d'usage. Sinon il vaut 0.
- Tests : une vidéo de 96 × 72 et 60 images au fond bruité est fabriquée par openh264.
  Une cible lente y est rejouée et trouvée. Les cibles tirées n'y seraient presque jamais trouvées. Le tirage se teste par ses propriétés.
  La vidéo et la cible viennent de `bench::noisy`, derrière la feature `testing` (#154).
  `bench` dépend de lui-même en dev-dependency avec cette feature pour que `tests/command.rs` les voie.
- `committed_metrics` (#277) relit chaque passage v0.2.0 du dépôt et le réévalue comme `--replay`.
  Il exige le `bench.json` enregistré, durées exclues.
  Les entiers doivent être égaux et les flottants à 1e−9 près (`matching.rs` passe par `hypot`).
  - Les réglages de `bench.json` doivent être ceux de `Judging::default()`.
    Ainsi un changement des défauts ne déplace pas la base sans le dire.
  - Ce qui vient de l'évaluation se fige par banc dans `FROZEN`. Les lignes changées s'affichent.
    L'effet actuel est décrit à [`tracking`](#tracking). Une PR de suivi met à jour `FROZEN`, jamais la référence.

## Réglages

Le banc d'essai du moteur hors des réglages par défaut compte 16 cas.
Chacun est une référence à d'autres réglages de détection et `compare settings` les relit (#149). Tout réglage valide se compare.

### Vidéo 720×540

`tests/fixtures/video_092_original_3m24-4m05_720x540.mp4` est l'extrait réduit de moitié (44 336 736 octets, sous LFS).
Cette vidéo a été faite une fois le 12 octobre 2026 sous le verrou, par le ffmpeg 9.0.2 de Homebrew (libx264) :

```bash
ffmpeg -i tests/fixtures/video_092_original_3m24-4m05.mp4 -map 0:v:0 -an -sn -dn -map_metadata -1 \
  -vf scale=720:540:flags=area -c:v libx264 -preset slow -crf 16 -g 15 -keyint_min 15 \
  -sc_threshold 0 -bf 0 -threads 1 -pix_fmt yuv420p -color_range tv -colorspace bt709 \
  -color_primaries bt709 -color_trc bt709 -fps_mode passthrough -video_track_timescale 10000 \
  tests/fixtures/video_092_original_3m24-4m05_720x540.mp4
```

- Elle garde les 1 230 images de l'extrait et sa cadence (10000/333).
  Sa base de temps reste 1/10 000 et ses images-clés tombent toutes les 15 images.
  Comme l'extrait, elle n'a pas d'image B et elle est en plage `tv` et BT.709.
- Elle ne se refait pas : un autre encodeur ne donnerait pas les mêmes images, ni le même encodeur sur un autre système.
  La référence ne vaudrait plus. Les tâches CI `rust-slow` et `rust-settings` la tirent avec l'extrait.
  `rust-windows-slow` ne tire que l'extrait.
- CRF 16 garde le bruit du capteur. La vidéo pèse 44 Mo contre 60 Mo pour l'extrait en 1440×1080.

### Les 16 cas

Les références v0.1.1 sont dans `tests/fixtures/reference/settings/<cas>/`.
Elles pèsent 1,6 Mo en tout : `frames.jsonl.gz`, `tracks.jsonl`, `tracks.csv`, `video.json`.
Elles ont été enregistrées le 12 octobre 2026 par `tools/reference_v011.py data <vidéo> <dossier> --gzip --detect '<JSON>'`, un cas par appel sous le verrou.
Le script est celui du tag `python-final` et le code celui du tag `v0.1.1`. Les versions sont les mêmes que pour la référence de #94.
Les références v0.2.0 des mêmes cas sont dans `tests/fixtures/reference-v0.2.0/` ([Référence v0.2.0](#référence-v020)).
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

- `max_area` vaut 400 et non 500 (décision de l'utilisateur). La plus grande tache de l'extrait fait 488,25 px².
  À 500, la v0.1.1 rendait les détections des défauts octet pour octet.
- `sigma_0` est le seul cas où la v0.1.1 trouve des plages instables (2). Sans flou, le bruit du capteur passe le seuil par endroits.
- `crates/compare/tests/settings_agreement.rs` exige les 16 cas comparés sans échec.
  Leurs écarts à la référence v0.2.0 y sont figés ([Gardes de détection](#gardes-de-détection)).
  `crates/reference/tests/committed_settings.rs` relit les 16 cas en test rapide.

### La 092 à 1440 pixels de travail

La référence `tests/fixtures/reference/video_092_original_width_1440/` (`--detect '{"work_width": 1440}'`) se compare par `compare 092-1440` et par le niveau `reference` dans `cli/tests/reference_092.rs`.
Dans `rust:reference`, les deux passages de la 092 tournent en même temps dans le même binaire. Leurs temps de détection s'y mélangent.

## Banc de mesure

La commande et la méthode du banc sont dans [08](08-banc-de-mesure.md). Le détail de la crate est dans `bench` ([Crates](#crates)).
`bench.json` note la version du code dans `code` : le `git describe --always --dirty` du dossier courant.
Cette section décrit la garde contre les bancs de la v0.1.1.

### Garde

Le test `local_guard` appelle la même fonction `bench::against::against` que `batdetect-bench VIDÉO --against DOSSIER`.
Tous deux rejouent un banc enregistré par la v0.1.1 :

- `DOSSIER` contient `bench.json` et `bats.jsonl.gz` (`$BATDETECT_REFERENCE_DIR/bench/` `transit` ou `hunt`).
  Un banc fait à d'autres réglages de détection, de suivi, de stabilité ou d'appariement est refusé. Le refus nomme le réglage.
  Le suivi doit donc tourner sans recollage ni portée comme à la v0.1.1.
  C'est `--rejoin-hidden 0 --reach-margin 0` sur la commande et `without_rejoin().with_reach(None)` dans `local_guard`.
- Un banc d'une autre vidéo est refusé (#198). Le nom de fichier de `video` dans `bench.json` doit être celui de `VIDÉO`, sans son dossier.
  Il est vérifié avant toute détection. Le nombre d'images d'`injected.jsonl.gz` doit être celui que décode le Rust.
  Il est vérifié après les deux détections.
  On lit `injected.jsonl.gz` plutôt que `bench-run.json` parce que seul le premier existe aussi dans un banc écrit par le Rust.
- Les cibles sont les mêmes des deux côtés : positions et amplitudes de `bats.jsonl.gz`.
  Elles sont placées par le numéro d'image du décodeur de `media`. Sur la 092, ce numéro est identique à celui d'OpenCV.
  Une vidéo où ils diffèrent décalerait les cibles. Le Rust injecte et détecte.
- Le passage Rust est évalué deux fois. `bench.json` et le tableau prennent le contraste mesuré par le Rust comme pour tout passage.
  La garde prend le contraste enregistré par la v0.1.1. Ce contraste décide donc de la visibilité des deux côtés.
  Seules les plages ignorées propres à chaque passage injecté peuvent encore la changer.
- Par classe et sur les cibles visibles des deux côtés, la garde donne leur nombre, les trouvées de chaque côté, la borne basse de Wilson de la v0.1.1 recalculée sur ces cibles, puis les cibles perdues et gagnées (numéro et image de début).
  Cette borne diffère du `found_ci95` de son `bench.json` dès qu'une cible n'est visible que d'un côté.
  La garde donne à part les cibles visibles d'un seul côté, celles dont la visibilité change avec le contraste mesuré par le Rust, les fausses pistes et les pistes de référence de chaque côté.
  Sur la 092, il y a 14 pistes de référence de chaque côté : un nombre différent dirait une autre vidéo.
- **Échec automatique** (code 1) seulement si la proportion trouvée par le Rust tombe sous cette borne basse.
  Sinon la liste est jugée par l'utilisateur.
  Une classe dont toutes les cibles quittent le dénominateur ne peut pas échouer : elle n'apparaît que dans la liste des cibles visibles d'un seul côté.
- La sortie va dans `<-o>/<vidéo>_<dossier>/` avec le passage, `bench.json` et `guard.txt`.
  `guard.txt` porte le même texte que la sortie standard.
- `--replay` d'un passage tiré avec un `--match-radius` plus grand que celui du tirage rompt l'écart de 3 rayons entre cibles sur lequel repose la règle d'égalité de `matching`.
  Rien ne le refuse.

La garde passait sur les deux bancs le 15 octobre 2026 (f364698, M1 Max, `release`, la 092 du dépôt).
Au transit, 253 cibles étaient visibles des deux côtés et 148 trouvées de chaque côté. Aucune n'était perdue ni gagnée.
À la chasse, la v0.1.1 trouvait 456 cibles sur 584 et le Rust 455.
Le Rust perdait la 19 (image 3 221) et la 543 (image 6 935) et gagnait la 555 (image 7 948).
Le contraste du Rust changeait la visibilité des 84, 120, 341, 602 et 665. Chaque côté avait 14 pistes de référence et aucune fausse piste.
L'évaluation seule prend moins de 10 ms : un `--replay` est immédiat.

## Référence v0.1.1

La référence de la v0.1.1 a été enregistrée par le Python et reste lue par des tests.
Son format, son emplacement et ses versions sont dans le README de `tests/fixtures/reference/` (#94).
Elle ne se réenregistre que depuis le tag `python-final`, par `tools/reference_v011.py` qui exige `src/` identique à `v0.1.1`.
La crate `reference` la lit.
Le test local `crates/reference/tests/local_reference.rs` relit les 26 vidéos (966 pistes) et apparie chacune à elle-même.
Les 16 cas de réglages sont dans `tests/fixtures/reference/settings/` et `crates/reference/tests/committed_settings.rs` les relit ([Réglages](#réglages)).

Les attendus de l'image résumé viennent de `tools/summary_v011.py` (au tag `python-final`).
Il relit `tracks.jsonl` et `video.json` d'un enregistrement sans relancer la détection. Il a la même garde que `tools/reference_v011.py`.
Tranches, couleurs et horloges viennent des fonctions de la v0.1.1 (`split_by_period`, `assign_colors`, `format_clock`, `format_duration`).
L'en-tête est recomposé en lignes structurées selon `header_lines`, sans les textes.
Les attendus de l'extrait et de la 092 sont dans `crates/summary/tests/fixtures/`.
Ceux des 26 vidéos sont dans `summary/` du dossier de référence local, à côté de `data/`.

Le binaire de la v0.2.0 retrouvait 963 des 966 pistes de la v0.1.1 sur les 26 vidéos, dont 958 concordantes.
Il en perdait 3 et en ajoutait 9. C'est la mesure de `compare local` du 12 octobre 2026, refaite le 15 octobre sur f364698.
La 089 était à 98,19 % côté référence et 98,46 % côté Rust. Aucune vidéo n'était sous 98 %. Ces 17 écarts tombaient sur 9 endroits :

- 094 1:42 : perdue puis revenue 26 images plus tard.
- 097 0:20 : perdue puis revenue 7 images plus tard.
- 097 2:42 : 24 points au lieu de 86 et une piste ajoutée à 2:43.
- 098 0:05 : ajoutée.
- 098 0:41 : perdue.
- 099 de 0:14 à 0:19 : trois écarts de points et une piste ajoutée.
- 099 0:27 : ajoutée.
- 100 1:36 : deux pistes ajoutées.
- 126 1:31 : 27 points au lieu de 32 et une piste ajoutée à 1:32.

## Référence v0.2.0

Cette référence tient ce que la v0.2.0 en Rust détecte et suit.
`record` l'a enregistrée le 17 octobre 2026 sur lumiel depuis un clone propre du commit 1e4cf6b (#250).
Format, emplacements, provenance, preuve et recette sont dans le README de `tests/fixtures/reference-v0.2.0/`.
Le dépôt contient l'extrait, la 092, la 1440, les 16 réglages et les deux bancs de la 092 (5,5 Mo).
Les 26 vidéos sont hors dépôt sous le préfixe `reference-v0.2.0/` du seau (31 Mo).
Elles passent par `BATDETECT_REFERENCE_V020_DIR` et `rust:data`.

- La preuve vient du binaire de `v0.2.0` construit à part :
  - Les 19 cas du dépôt ont les mêmes `frames.jsonl` à l'octet.
  - Les 26 vidéos ont les mêmes entrées et les mêmes détections hors plages abîmées (`--same-as`).
  - Les pistes de l'extrait et de la 092 sont par valeurs celles du CSV de `batdetect` de `v0.2.0`.
  - Les deux bancs donnent les mêmes fichiers. Pour `bench.json`, ce sont les mêmes valeurs hors `code` et durées.
- Elle ne se reproduit que hors des plages abîmées (#176). `SHA256SUMS` change à chaque enregistrement des 26 vidéos.
- Les gardes de `compare` portent sur elle (#275).
  Celles de la détection (#276) et du suivi aussi (#277 : `committed_tracking`, `committed_exclusion`, `local_tracking`, `local_exclusion`, `committed_metrics`).

### Gardes de détection

Chaque garde compare à la v0.2.0 par la règle de `compare` et fige ses écarts **par vidéo et par cas** (`Comparison::gaps`, #276).
Ces écarts sont les images qui diffèrent et les détections perdues et ajoutées, puis les pistes appariées, concordantes, perdues et ajoutées.
Les détections faibles encadrées ne font qu'ajouter des détections et aucune n'est perdue (#21, [10](10-logique-de-detection.md#5-ter-garder-les-taches-faibles-qui-comblent-un-vol)).
Les écarts sont alors `Gaps::added_only(images, ajoutées, [appariées, concordantes, perdues, ajoutées])`, mesurés sur lumiel le 17 octobre 2026.
Avant #21, ils valaient tous 0 sur lumiel (`Gaps::agreeing(n)`).
Une PR qui change la détection les met à jour avec leur cause ([05](05-outillage.md)).
Les gardes `local_*` lisent la v0.2.0 des 26 vidéos par `BATDETECT_REFERENCE_V020_DIR` et les vidéos par `BATDETECT_VIDEOS_DIR`.
`local_command` lit aussi les zooms et la 103 annotée de la v0.1.1 par `BATDETECT_REFERENCE_DIR`.

- `compare/tests/slow_extract` : l'extrait, 6 détections faibles ajoutées et ses 4 pistes.
  Il garde aussi le bit près de `tests/fixtures/rust/` et `--same-as`.
- `compare/tests/settings_agreement` : les 16 cas. Les détections faibles en ajoutent 755 en tout, dont 664 sans filtre (`sigma_0`).
  Ailleurs, elles ajoutent au plus 2 pistes (`width_720`).
  Elles font une piste perdue et une ajoutée à `sigma_3` et une ajoutée à `max_area_400`. `threshold_25` ne change pas.
  La portée (#326) change deux cas. `sigma_0` a 15 pistes appariées, 11 concordantes, 15 perdues et 9 ajoutées.
  Les 4 chauves-souris de l'extrait restent et le bruit perd des pistes, comme dans `committed_tracking`.
  `width_720` n'a plus que 2 pistes concordantes sur 5. Les deux jumelles de la chauve-souris des images 705-722 se la partagent autrement.
  L'écart de `sigma_0` et `window_2s` à la v0.1.1 dans la bande d'affichage s'explique par la table `GRAY` (« Gris de la détection » dans `engine`).
- `compare/tests/local_agreement` fige une ligne par vidéo : 26 lignes et 972 pistes.
  Les détections faibles en ajoutent 236 622 (155 968 sur la 103) et aucune n'est perdue.
  Pistes : 842 appariées, 694 concordantes, 130 perdues et 79 ajoutées.
  C'est l'effet de #21, du recollage (#24) et de la portée (#326) ensemble.
  Cet effet se remesure sur l'arbre qui les a tous et ne s'additionne pas.
  Une piste de la v0.2.0 réunie à une autre par le recollage compte pour perdue car une piste ne s'apparie qu'à une seule.
  Le recollage fait 81 réunions sur les 26 vidéos. 67 se font sur les seules détections de la v0.2.0.
  13 de plus réunissent des bouts que les détections faibles changent (cause non classée une à une). Une de plus vient de la portée.
  La portée coupe des pistes qui sautaient vers une autre tache. Leurs bouts comptent comme perdus ou ajoutés.
  Les endroits changés ont été jugés à l'œil ([10](10-logique-de-detection.md#7-relier-les-taches-dans-le-temps--le-suivi)).
  La 092 ne bouge pas.
- `cli/tests/reference_092` :
  - La 092 : 14 pistes appariées et concordantes, 20 détections faibles ajoutées sur 19 images.
  - La 092 à 1440 pixels : 9 détections faibles ajoutées, 14 pistes appariées et concordantes.
    Un second morceau du passage de 1:48.93 se trouve à 27-36 px de lui.
    Il atteindrait 6 points grâce à la détection faible de 1:49.83 (image 3298) et ferait une piste de plus.
    Avec la portée, cette détection est trop loin pour lui. Il s'arrête à 2 points et se fond dans la piste 6 comme une jumelle.
  - La commande sur la 092 : 14 pistes appariées et concordantes sans écart, ses 15 extraits figés et son zoom.
  - Les faits de la naturaliste sont les seuls affirmés sans référence.
    Les 13 chauves-souris confirmées le 9 octobre ont chacune leur piste ([03](03-resultats-092.md)).
    Chaque piste commence à ±5 images de leur début (temps aux `pts` de la lecture du calcul).
    À côté d'elles, le test tolère au plus une piste à chacun de deux endroits.
    Le premier est l'objet lent de 3:29, qui n'est pas une chauve-souris.
    Le second est la cible connue de 3:42.10 ([03](03-resultats-092.md#cibles-connues-pour-les-issues-suivantes)).
    C'est un objet réel que le réglage par défaut manque aujourd'hui et que la naturaliste n'a pas encore vu.
    Toute autre piste nouvelle fait échouer le test : c'est une question pour elle.
    Le nombre de pistes n'est affirmé que par les écarts figés parce qu'un recollage peut passer de 14 à 13 pistes sans perdre un passage.
- `cli/tests/slow_command` : les 4 pistes de l'extrait sans écart.
  La colonne des détections du CSV est celle de `tests/fixtures/rust/…/tracks.csv`, en-tête exclu.
- `cli/tests/local_command` : les pistes de la commande sur les 26 vidéos, avec une ligne figée par vidéo.
  Ces lignes sont `[référence, appariées, concordantes, perdues, ajoutées]`, les mêmes que `local_agreement`.
  Une piste recollée y est comparée à tous ses bouts.
  Le test compare le zoom de chaque piste à celui de toutes les pistes de la v0.2.0 qu'elle contient (premier point compris dans les siens).
  Il exige que chaque piste de la v0.2.0 soit contenue dans une piste et une seule. Deux listes figées tiennent les exceptions.
  - `UNHELD` liste les pistes de la v0.2.0 dont aucune piste de la commande ne tient le premier point. Elle en a 11.
    Quatre étaient déjà perdues avec les seules détections faibles et sans recollage : 090 #14 et #17, 096 #44, 097 #4.
    Sept viennent de la portée, à des endroits jugés à l'œil ([10](10-logique-de-detection.md#7-relier-les-taches-dans-le-temps--le-suivi)) :
    - 090 #5 (2:09.80, trois pistes devenues deux) et #7 (2:12.33, la piste qui perd deux points, seul de ces endroits jugé pire).
    - 094 #7 (2:28.11, trop courte une fois le saut refusé).
    - 099 #60 (1:32.09, la piste perd 4 points) et #90 (3:22.07, trois pistes devenues deux).
    - 100 #36 (1:36.56, trois pistes devenues deux) et #43 (1:55.58, des points changent de tache).

    La 098 #61 est tenue : le morceau de #311 qui prolonge la #59 contient son premier point. Le suivi rejoué sans portée la tient aussi.
  - `ZOOM_SWITCHES` liste les pistes dont le zoom automatique diffère de celui des pistes de la v0.2.0 dont elles tiennent le premier point.
    Elle a 31 lignes : 30 zooms gagnés et un perdu, la 098 #114.
    Parmi les zooms gagnés, 25 viennent de pistes qui ne tiennent aucun début de la v0.2.0.
    La piste qui tient le début de la 098 #114 suit la seconde chauve-souris jusqu'à l'image 7713.
    Sa tache y atteint 164 px² et une intensité de 65, au-dessus des seuils du zoom (100 et 40).
    Coupée de la seconde, la première chauve-souris gagne son zoom (#112, 76 px² au plus, et #113).
    Les cinq autres zooms gagnés viennent de pistes qui tiennent un début de la v0.2.0 : 094 #2, 096 #89, 097 #85, 098 #113 et 098 #126.
    Quatre causes ont été relevées avant la portée, qui a pu les changer :
    - La 094 #2 : une petite tache repasse là où était la grande, une image plus tard et à moins de 8 px.
      Leurs deux pistes échangent leur seconde moitié après deux détections faibles.
    - La 097 #85 : la tache unique de deux chauves-souris qui se croisent (2:41.72-2:41.76) va à l'autre piste.
      Celle-ci est mieux prédite grâce à une détection faible.
    - Deux pistes recollées prennent le zoom d'un bout ajouté par les détections faibles.
      La 096 #89 de la v0.2.0 est réunie au bout de 6 points des images 5523-5530. La 098 #126 est réunie à celui des images 8452-8459.

    La cause de la 098 #113 n'est pas relevée.
    Avant la portée, la 099 #90 gagnait un zoom (le début de 7 points qui restait après la réunion juste de #88 et #90).
    Elle est maintenant dans `UNHELD`.
  - Les zooms enregistrés par la v0.1.1 sont décidés sur ses propres pistes.
    Ils se vérifient à part et sans décodage (`the_zooms_recorded_with_the_v011_are_those_its_own_tracks_ask_for`).
- `cli/tests/local_clips` exige toutes les fenêtres des extraits des 26 vidéos.
  Il les compare vidéo par vidéo à celles des pistes de la v0.2.0.
  17 vidéos ont des fenêtres qui changent avec leurs pistes : 180 fenêtres propres à la commande et 234 propres à la v0.2.0.
  Le nombre de fenêtres de chaque côté et de celles propres à chacun est figé par vidéo.
- `process` et `tracked` n'écartent que les plages du décodeur courant.
  `reference_092`, `slow_command`, `local_command` et `local_clips` dépendent donc du poste sur les 8 vidéos où lumiel voit 14 images en erreur de plus que le Mac.
  Leurs lignes sont figées sur lumiel et restent à vérifier sur le Mac. `compare` réunit les deux lectures (#275) et n'en dépend pas.
- Restent sur la v0.1.1 :
  - Les entrées, avec ffprobe pour témoin.
    Ce sont `slow_extract` pour les entrées de l'extrait, `reference_092` pour les 9 011 images de la 092 et `local_command` pour les `pts` de la 103 annotée.
    Dans `engine`, ce sont `slow_input`, `local_input` et `local_damage`, dont la règle de lumiel reste celle de #250.
  - L'aval nourri des pistes de la v0.1.1 (`summary`, `files`, `render`).
  - Le format v1, les sorties (`local_outputs`) et le banc (`local_guard`).
