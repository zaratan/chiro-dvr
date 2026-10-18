# Rapport #347 — faisabilité d'une version web en WebAssembly (méta #346)

Branche `rust/347-wasm` (depuis `origin/main`, 9172af1), worktree `/home/zaratan/Projects/dvr-wt/lancement`. Exploration seule : rien dans `crates/`, rien de commité, `git status` propre. Tout le prototype vit dans `out/347/` du worktree.

**Verdict : faisable, sans obstacle bloquant dans Chrome.** La détection et le suivi tournent dans le navigateur et donnent **au bit près** les détections, les pistes et le fond du natif : extrait et 092 entière, sur un fil comme sur 12 threads. WebCodecs décode l'extrait image pour image comme FFmpeg. Il encode des extraits mp4 que FFmpeg et la page de résultats lisent. Le zip fait dans le navigateur s'ouvre et se juge comme un dossier de l'outil. Le coût est surtout un chantier de découpage du code natif (environ 50 points, plus bas). Safari et Firefox ne sont pas éprouvés.

Machine : lumiel, Ryzen 5 3600X (6 cœurs, 12 fils), sans GPU utilisable par Chrome. Navigateur : celui de Playwright 1.64.1, « Google Chrome for Testing » 156.0.8078.4 en mode headless, une version de marque avec ses codecs propriétaires. Je n'ai pas essayé de Chromium entièrement libre : aucun n'est installé, et je n'en ai pas installé.

## Fait

### 1. Ce qui compile en `wasm32-unknown-unknown`

Rust 1.99.0, `cargo build --release --target wasm32-unknown-unknown -p <crate>`, sur une copie du dépôt (`out/347/ws/`).

| Crate | Compile tel quel | Ce qui casse à l'exécution, ou à changer |
|---|---|---|
| `types` | oui | rien |
| `tracking` | oui | rien |
| `draw` | oui | rien vu (police Roboto embarquée, 489 Ko) |
| `page` | oui | rien : `page` ne lit aucun disque |
| `files` | oui | `std::fs` rend `Unsupported` sans paniquer. `jiff` : `Zoned::now()` panique sans sa feature `js`, qui ajoute environ 854 Ko |
| `summary` | oui | il écrit son PNG par `std::fs` : sortir une fonction qui rend des octets |
| `engine` | **non** : `ffmpeg-sys-next` | il ne prend à `media` que `frame::YuvFrame` et `decode::{Threads, ActiveThreads}` |
| `render` | **non** : `media` | il compile une fois fermés 7 modules liés à FFmpeg (1 248 lignes). Les 1 094 lignes pures restent |
| `cli` | non, et ne compilera jamais | clap, rustix, ctrlc, FFmpeg |

- **engine** : dans la copie, j'ai mis `ffmpeg-next` et `tempfile` de `media` derrière une feature `native`, par défaut. `frame` et `planes` restent hors de la feature. Dans `engine`, `input`, `detection` et `Damage::of` passent aussi derrière `native`.
  - J'ai ajouté une entrée `engine::web::Streamed` (87 lignes), qui reçoit les plans Y, U et V déjà décodés.
  - Le diff est de 22 lignes dans `media` et 87 dans `engine`, hors du nouveau module. `engine` compile alors, avec des avertissements de code mort seulement.
- **render** : voir les notes du fork, `out/347/notes-io.md` §3.
  - `Region` est à sortir de `media::convert`, et `Marks` de `picture`.
  - `thread::scope` panique sans atomics. Le décodage et l'encodage sont synchrones en Rust, alors que WebCodecs est asynchrone : il faut inverser le contrôle, c'est-à-dire piloter render image par image.
  - La conversion BT.709 de swscale ne sert que sur les zones dessinées. En pur Rust, elle ne serait pas identique au bit près : la version web aurait ses propres empreintes figées. Les détections ne sont pas concernées.
- **Horloge** : `Instant::now()` et `Zoned::now()` paniquent. La date et la durée viendront de JS (`new Date()`, `performance.now()`), et jiff n'entrera pas dans le wasm.

### 2. Détection et suivi en wasm, comparés au natif

- **Prototype** : la crate `out/347/ws/crates/webproto`, avec un `Session` (push d'images I420, puis `finish`) lié par wasm-bindgen 0.2.129, et un binaire natif qui lit les mêmes images brutes. Dans le navigateur, un Worker module démultiplexe le mp4 par Mediabunny 1.61.6 (MPL-2.0) et décode par `VideoDecoder`. Il copie chaque `VideoFrame` en I420 et le pousse dans le wasm.
- **Identité** :
  - extrait, 1 230 images : 403 détections sur 403 identiques au bit près à `tests/fixtures/rust/…/frames.jsonl` ; les 4 pistes ont les mêmes débuts, fins et nombres de points (290, 17, 24 et 27) ; même somme du fond (190 073 144). Vrai pour le build sans SIMD, avec SIMD sur un fil, et sur 6 et 12 threads ;
  - 092 entière, 9 011 images, sur 12 threads : 781 détections sur 781 identiques à la référence v0.2.0, et 14 pistes sur 14 avec les mêmes débuts, fins et points.
  - Le natif sur des images brutes décodées par le FFmpeg 8.1 du système redonne aussi les 403 détections au bit près.
- **Temps sur l'extrait**, sous `verrou chrono`, deux passages par cas, qui concordent à 3 % près (sauf le natif sur un fil, 7,7 et 9,6 s) :

  | Cas | Bout en bout | dont détection | Mémoire wasm | Chrome au pic |
  |---|---:|---:|---:|---:|
  | natif, images brutes, 1 fil | 7,7 à 9,6 s | — | — | — |
  | natif, images brutes, 12 fils | 2,7 s | — | — | — |
  | navigateur, décodage seul | 4,1 s | — | — | 620 Mo |
  | navigateur, wasm sans SIMD, 1 fil | 84,2 s (un passage) | 78,2 s | 208 Mo | — |
  | navigateur, wasm `simd128`, 1 fil | 13,7 s | 9,7 s | 208 Mo | 777 Mo |
  | navigateur, threads wasm, 6 | 7,3 à 7,6 s | 3,2 à 3,5 s | 264 Mo | 829 Mo |
  | navigateur, threads wasm, 12 | 7,2 s | 3,1 s | 327 Mo | 893 Mo |

  - **`simd128` est indispensable** : la détection va 8 fois plus vite et reste au bit près. Avec lui, le wasm sur un fil est au niveau du natif sur un fil.
  - Sur 12 threads, la détection (3,1 s) rejoint le natif (2,7 s) ; **le goulot devient le décodage**.
- **092 entière**, 12 threads : 48,3 s de bout en bout, dont 47,8 s jusqu'à la dernière image décodée, soit environ 188 i/s pour le décodeur logiciel de Chrome. La mémoire wasm est de 327 Mo, Chrome culmine à 1,33 Go. Pour comparer, la détection native de la 092 prend 18,5 à 20,9 s sur lumiel (#214).
- **Threads** :
  - il faut nightly (1.101, déjà installé avec `rust-src`) avec `-Z build-std=panic_abort,std`, `+atomics,+bulk-memory,+simd128` et des options d'édition de liens explicites (`--shared-memory`, `--import-memory`, `--max-memory=4294967296`, exports TLS) ;
  - il faut aussi `wasm-bindgen-rayon` 1.3 avec sa feature `no-bundler` : sans elle, le pool ne démarre jamais, car le module principal est chargé par `../../..` ;
  - servi sans COOP/COEP, le build à threads échoue tout de suite (`SharedArrayBuffer transfer requires self.crossOriginIsolated`). Le build sur un fil marche avec ou sans ces en-têtes.
  - **La page charge donc l'un ou l'autre selon `crossOriginIsolated`.**
- **Taille** : build sur un fil, 343 Ko (111 Ko en gzip) ; build à threads, 402 Ko (127 Ko en gzip). Sans render, summary ni draw.

### 3. WebCodecs : décodage, placement à l'image, encodage, mp4

- **Décodage** : `avc1.640028` (High 4.0) est pris en charge. Les 1 230 images sortent en `I420`, BT.709, plage limitée, et leurs empreintes sont identiques image par image à celles de FFmpeg. `copyTo` vers un autre format que RGB est refusé : on copie au format natif.
- **Placement à l'image** : Mediabunny trouve l'image-clé qui précède (`getKeyPacket`), puis on décode jusqu'à l'image visée. Sur 12 images visées (débuts et fins des pistes, première, dernière et images-clés), les 12 sont identiques à FFmpeg, en 15 à 65 ms chacune, pour une image-clé toutes les 15 images. Le lecteur `<video>` n'est pas en jeu : la précision vient du démultiplexage.
- **Encodeurs disponibles** dans ce Chrome, par `VideoEncoder.isConfigSupported` en 1 440 × 1 080 :
  - H.264 Baseline 4.2, Main 4.0, High 4.0 et High 5.0 en logiciel, **en débit variable seulement** : le mode à quantificateur fixe est refusé (« Unsupported bitrate mode ») ;
  - Baseline 3.1 refusé, trop petit pour cette taille ;
  - VP9 et AV1 en logiciel, quantificateur compris ;
  - pas de HEVC, et rien en matériel, puisqu'il n'y a pas de GPU.
- **Vitesse d'encodage** : 3 extraits de 120 images, avec un cadre dessiné dans le plan Y, en High 4.0 à 24 Mb/s, le débit mesuré des extraits natifs :
  - au calme, 0,69 à 0,72 s par extrait, soit environ 170 i/s pour un encodeur ;
  - 3 encodeurs en parallèle : 1,34 s pour les trois ;
  - un premier passage pendant une forte charge d'autres sessions (charge moyenne de 16,7 sur 15 minutes) avait donné 4,1 à 4,7 s par extrait. Il est écarté.
- **Mp4** : WebCodecs ne rend que des images encodées. Mediabunny (`Output`, `Mp4OutputFormat({ fastStart: "in-memory" })`, `EncodedVideoPacketSource("avc")`, `BufferTarget`) emballe les paquets dans un mp4, entièrement en mémoire. `mp4-muxer` est déprécié au profit de Mediabunny.
- **Téléchargement et lecture hors du navigateur** : l'extrait est téléchargé par un lien `blob:` avec `download` (événement de téléchargement de Playwright), puis vérifié par `ffprobe` et `ffmpeg` :
  - conteneur `isom`, marques `isomavc1mp41`, et boîtes dans l'ordre `ftyp`, `moov` (1 203 octets), `mdat` : la lecture peut commencer avant la fin du fichier ;
  - H.264 High 4.0, 1 440 × 1 080, `yuv420p`, BT.709 en plage limitée ;
  - 120 images comptées une à une, à 10000/333 i/s, soit 3,996 s, des temps réguliers de 0,0333 s, une seule image-clé en tête ;
  - `ffmpeg -f null` décode sans erreur ;
  - le même fichier, mis à la place d'un extrait natif dans un dossier dézippé, se lit dans la page de résultats.
  - **QuickTime et VLC ne sont pas sur lumiel** : le fichier est dans `/home/zaratan/Projects/dvr-wt/rapports/347/browser-clip.mp4` pour le Mac.

### 4. Mémoire

- **Détection** : la mémoire wasm ne dépend pas de la longueur de la vidéo. La fenêtre du fond est fixée en secondes. La 092 (9 011 images) culmine à 327 Mo sur 12 threads, comme l'extrait (1 230 images), et à 208 Mo sur un fil. On est loin des 4 Go de wasm32.
- **Budget du fond** : `BACKGROUND_BUDGET` vaut 4 Gio, soit tout l'espace de wasm32. Il ne protège plus rien en wasm et doit y être abaissé (par exemple à 1 Gio). Avec les réglages par défaut, la fenêtre fait environ 21 Mo.
- **Chrome entier** : il culmine à 1,33 Go sur la 092. Ce chiffre inclut la vidéo de 454 Mo, que le banc lit par `fetch().blob()`. Un `File` choisi par la page n'est pas mesuré : il devrait rester sur le disque et coûter moins.
- **Extraits gardés en mémoire, projetés sur la 092** : 14 extraits et un zoom font 194,6 Mo au débit des extraits natifs (dossier natif réel). Au même débit dans le navigateur (24 Mb/s mesurés), comptez environ 195 Mo de `Blob` dans Chrome.
- **Zip de la 092** : environ 197 Mo, soit 196,7 Mo de fichiers plus environ 200 octets par entrée. Assemblé en `Blob` à partir de morceaux, il double la place des extraits le temps du téléchargement, pour un pic projeté d'environ 400 Mo de plus. `showSaveFilePicker` permettrait d'écrire en flux sans copie, mais il n'existe que dans Chromium.
- **Temps projeté du rendu sur la 092** : 1 896 images d'extraits et 99 de zoom, à 170 i/s pour un encodeur, font environ 12 s. Il faut aussi redécoder environ 2 100 images à 188 i/s, soit environ 11 s. La 092 complète prendrait donc environ 70 s dans le navigateur, contre 28,7 s pour le binaire sur lumiel (docs/09). C'est déduit, pas mesuré de bout en bout.

### 5. Destination en mémoire et « Tout télécharger »

Le détail est dans `out/347/notes-io.md`, écrit par le fork « lecture de code ».

- **Accès disque** (hors tests) : 56 lignes dans `files`, 3 dans `summary` et 32 dans `cli`.
  - Le CSV, `params.json`, les PNG et la page sont déjà faits en mémoire, puis écrits par `partial` et `persist`.
  - À faire :
    - sortir les fonctions qui rendent les octets (environ 60 lignes déplacées) ;
    - un trait `Destination`, avec `put(chemin, octets)` et `stream(chemin)`, et son implémentation `Disk` (environ 80 lignes) ;
    - surtout, sortir l'orchestration d'une vidéo de `cli::process` vers une crate sans FFmpeg (environ 300 des 451 lignes, plus 100 à 150 d'adaptation).
  - Au total, 500 à 700 lignes touchées ou déplacées.
- **Sans objet dans le navigateur** : `exclusive`, `partial_file`, `previous`, `reuse`, `writable`, `disk`, `space`, `inventory` et `jobs`. Avec une seule vidéo, `ledger` et `refusals` aussi : un échec s'affiche au lieu de s'écrire.
- **Page** :
  - `page_of` relit le dossier (`read_results`, `png_size`, verdicts) : il faut le couper en une partie pure et un adaptateur disque, environ 120 lignes ;
  - `cli::sheet` (Fluent, qui compile en wasm) doit sortir de `cli` ;
  - la page web sans jugement demande un drapeau `judging` dans le modèle et des URL `blob:`, 30 à 60 lignes. Le zip garde la page d'aujourd'hui telle quelle.
- **Zip** : `zip` 9.0.5 sans ses features par défaut, `ZipWriter::new_stream`, entrées Stored. Il compile en wasm.
  - Mon zippeur (`out/347/zipweb`, wasm-bindgen) pèse 115 Ko. Le fork a éprouvé le ZIP64 en natif sur 4,5 Go.
  - Essai réel : le dossier natif de l'extrait (9 fichiers) est relu par `fetch`, zippé en flux dans la page en 0,31 s (75,4 Mo, morceaux d'au plus 2 Mo), puis téléchargé.
  - `unzip -lv` voit 9 entrées Stored, `zipfile.testzip()` ne trouve rien, et les SHA-256 sont identiques au dossier natif.
  - À régler : la date des entrées vaut le 1er janvier 1980 (à prendre de JS), et `large_file` seulement au-delà de 4 Gio.
- **Dossier dézippé ouvert dans Chromium** (`file://…/resultats.html`) :
  - aucune erreur JS, et les 4 pastilles « Juger la piste » ;
  - un clic sur la piste 1 charge `extraits/piste_01_0m04s.mp4` (1 440 × 1 080, 12,65 s, sans erreur) ;
  - « o » au clavier coche « Oui », et « Télécharger les verdicts » rend un `verdicts.csv` à la ligne juste (`…;1;0:00:04;4,80;oui;;80;843`) ;
  - même essai, et même résultat, avec l'extrait encodé par le navigateur à la place de l'extrait natif.
  - Non éprouvé : l'Utilitaire d'archive de macOS (le zip est dans `/home/zaratan/Projects/dvr-wt/rapports/347/batdetect.zip`).

### 6. Hébergement

- **`vercel.json` minimal** : `/home/zaratan/Projects/dvr-wt/lancement/out/347/vercel.json`, avec une copie dans `rapports/347/`. Il pose `Cross-Origin-Opener-Policy: same-origin` et `Cross-Origin-Embedder-Policy: require-corp` sur `/(.*)`. Rien n'est déployé.
- **Banc local** : `out/347/web/server.mjs`, un serveur statique Node qui pose les deux en-têtes ou aucun. Comparaison : 12 threads isolés, 7,2 s ; un fil, 13,7 s ; threads sans isolation, refus immédiat.
- Avec `require-corp`, toute ressource d'une autre origine doit porter un CORP. La page embarque tout, donc rien ne change pour elle.

### Safari et Firefox (non éprouvés, d'après ce que je sais)

Tout ceci est à confirmer sur les vrais navigateurs.

- **Safari** : WebCodecs vidéo depuis 16.4, avec le décodage et l'encodage H.264 par VideoToolbox. Wasm SIMD depuis 16.4. Threads wasm avec COOP/COEP depuis 15.2. Le plus probable est un décodage matériel, donc plus rapide que le logiciel mesuré ici. Les limites de mémoire, surtout sur iPad, sont plus serrées.
- **Firefox** : WebCodecs vidéo sur ordinateur depuis la version 130. Wasm SIMD depuis 89 et `SharedArrayBuffer` isolé depuis 79. Le H.264 passe par les décodeurs du système (FFmpeg sous Linux). **L'encodeur H.264 de WebCodecs dépend de la plateforme : c'est le seul point qui pourrait bloquer**, pour ce navigateur seulement.
- `showSaveFilePicker` n'existe ni dans Safari ni dans Firefox : le zip y passe par un `Blob`.

## Obstacles classés

- **Bloquant** : aucun dans Chrome. Seul candidat à éprouver : l'encodeur H.264 de WebCodecs dans Firefox, et dans Safari dans une moindre mesure.
- **Contournable** :
  - `engine` lié à FFmpeg par `media` : feature `native` et entrée en flux d'images (environ 110 lignes) ;
  - `render` : modules liés à FFmpeg, `thread::scope`, swscale et contrôle synchrone. Il faut une inversion de contrôle et une conversion BT.709 en pur Rust, avec des empreintes figées propres au web ;
  - l'orchestration d'une vidéo enfermée dans `cli::process` : une crate sans FFmpeg, paramétrée par la source d'images, l'encodeur, la destination et l'horloge ;
  - écriture par `std::fs` : fonctions qui rendent des octets, et trait `Destination` ;
  - `Instant` et `Zoned` paniquent en wasm : date et durée fournies par JS ;
  - les threads demandent nightly, `build-std`, des options d'édition de liens et `wasm-bindgen-rayon` avec `no-bundler` ; sans COOP/COEP, il faut le build sur un fil, 1,9 fois plus lent ;
  - pas de quantificateur fixe en H.264 : débit variable de 24 Mb/s, donc des extraits de qualité voisine mais pas identique à openh264 QP 23 ;
  - `BACKGROUND_BUDGET` de 4 Gio en wasm32 : l'abaisser en wasm ;
  - date de 1980 dans le zip.
- **Confort** :
  - le décodage logiciel de Chrome (188 i/s) fait de la 092 un calcul de 48 s, contre environ 20 s en natif ; environ 70 s projetées avec les extraits ;
  - taille du wasm : 343 à 402 Ko aujourd'hui, avec render, summary et la police (489 Ko) à venir ;
  - environ 195 Mo d'extraits en mémoire sur la 092, plus environ 197 Mo le temps du zip ;
  - jiff et sa base de fuseaux (854 Ko), à éviter.

## Découpage proposé pour la méta #346

Échelle des étiquettes : 1, 2, 3, 5, 8 et 13. Aucune issue au-delà de 8.

1. **`media` sans FFmpeg et entrée en flux d'images dans `engine`** (feature `native`, module de flux, `compare local` sans écart) — 3.
2. **Construction wasm et garde de détection dans le navigateur** : build sur un fil (`simd128`) et build à threads (nightly), test Playwright de l'extrait contre `frames.jsonl` au bit près, tâche de CI — 5.
3. **Fonctions d'octets et trait `Destination`** dans `files` et `summary`, avec `Disk`, `cli` passant par lui, sans effet sur les sorties — 5.
4. **Crate d'orchestration d'une vidéo sans FFmpeg**, sortie de `cli::process` (source d'images, encodeur, destination et horloge en paramètres) — 8, conception nécessaire.
5. **`render` sans encodeur natif** : feature `native`, `Region` et `Marks` déplacés, BT.709 en pur Rust sur les zones, contrôle image par image, empreintes figées web — 8.
6. **Libellés Fluent hors de `cli`**, dans une crate partagée par le binaire et le web — 2.
7. **`page`** : drapeau `judging` et table chemin → URL — 3.
8. **Application web, calcul** : page de chargement, page « calcul en cours » avec l'avancement, Worker, démultiplexage par Mediabunny, décodage WebCodecs, choix du build selon `crossOriginIsolated`, budget du fond abaissé — 5.
9. **Application web, extraits et zooms** : encodeur WebCodecs, mp4 par Mediabunny, extraits gardés en `Blob`, lus par la page et téléchargeables un par un — 5.
10. **« Tout télécharger »** : zip en flux par la crate `zip` (Stored, date, ZIP64), assemblé en `Blob`, ou `showSaveFilePicker` quand il existe — 3.
11. **Hébergement Vercel** : `vercel.json`, construction statique, essai de déploiement — 2.
12. **Essais Safari, Firefox et Mac** : décodage et encodage H.264, threads, SIMD, QuickTime, VLC, Utilitaire d'archive — 3.

Total : 52 points. 1, 3 et 6 sont des refactorisations du binaire seul, vérifiables par `compare local` : elles peuvent partir avant toute décision sur le web. 4 et 5 sont le cœur du chantier.

## Fichiers

Tous hors du suivi git :

- `out/347/ws/` : copie du dépôt, avec la séparation de `media` et d'`engine` (`crates/engine/src/web.rs`) et la crate `crates/webproto` (bibliothèque wasm et binaires `native` et `hash`). `out/347/ws-render/` : la copie du fork pour `render`.
- `out/347/web/` :
  - le serveur `server.mjs` ;
  - les pilotes Playwright `run.mjs` et `opened.mjs` ;
  - les Workers `worker.js` (décodage et détection), `encode.js` (encodage et mp4) et `seek.js` (placement) ;
  - les pages `page.html`, `encode.html`, `seek.html` et `zip.html` ;
  - les dépendances npm `playwright-core@1.64.1` et `mediabunny@1.61.6`.
- `out/347/zipweb/` : le zippeur wasm. `out/347/zipprobe/` : les sondes du fork (zip, jiff, Fluent).
- `out/347/pkg-single`, `pkg-simd`, `pkg-threads` et `pkg-zip` : les sorties de wasm-bindgen. `out/347/tools/` : `wasm-bindgen-cli` 0.2.129 (`cargo install --root`).
- `out/347/raw/extract.yuv` : l'extrait en I420 brut, 2,87 Go, à effacer quand tu veux. `out/347/runs/` : tous les journaux, JSON et fichiers produits. `out/347/video/` : des liens vers les deux vidéos de `tests/fixtures/`, en lecture seule.
- `out/347/notes-io.md` : notes du fork. `out/347/vercel.json`.
- `/home/zaratan/Projects/dvr-wt/rapports/347/` : `browser-clip.mp4`, `batdetect.zip` et `vercel.json`, pour le Mac.

## Tests ajoutés

Aucun dans le dépôt. Les comparaisons au bit près sont les scripts `out/347/scripts/same.py` (détections) et `table.py` (série chronométrée).

## Vérifications

- Toutes les comparaisons de détections et de pistes sont exactes (voir §2). Les empreintes des images décodées par WebCodecs sont identiques à FFmpeg sur 1 230 images sur 1 230.
- Les temps publiés passent par `verrou chrono` ; les compilations, par `verrou lourd`.
- Le calme n'est pas garanti : lumiel porte d'autres sessions, et 19 processus Chrome orphelins d'autres sessions tournaient. Je ne les ai pas touchés, et mon relevé de mémoire ne compte que mes propres processus. Le passage d'encodage faussé par la charge est écarté.
- Rien de lourd du dépôt n'a été lancé (pas de `check`, de `compare` ni de `slow`) : rien n'a changé dans le dépôt.

## Diff

Aucun dans le dépôt : `git status` propre et `git diff --stat` vide.

## Décisions prises seul

- **Mediabunny plutôt que mp4box et mp4-muxer** : une seule bibliothèque maintenue pour lire et écrire le mp4. `mp4-muxer` se dit lui-même remplacé par elle.
- **Débit variable de 24 Mb/s pour les extraits**, le débit mesuré des extraits natifs, puisque le quantificateur fixe est refusé.
- **Mesure de la 092 entière** plutôt qu'une projection, puisqu'elle ne prenait qu'une minute.
- **Un fork pour la lecture de code** (sorties disque, page, `render`, zip), en parallèle du prototype.
  - Incident : son `cargo new` dans `out/347/zipprobe/` a ajouté les sondes aux `members` du `Cargo.toml` du worktree, et une compilation a touché `Cargo.lock`.
  - Il a rétabli les deux fichiers à l'identique depuis `git show HEAD:…`, sans commande git qui écrit. J'ai vérifié : `git status` est propre.
- **Pas de test du service worker** : ta consigne a changé pour Vercel.

## Questions ouvertes

- **Sur le Mac**, à faire, mais rien de bloquant pour ce rapport :
  - ouvrir `rapports/347/browser-clip.mp4` dans QuickTime et VLC (image par image) ;
  - dézipper `rapports/347/batdetect.zip` par l'Utilitaire d'archive ;
  - rejouer le banc dans Safari et dans Chrome avec VideoToolbox. Le banc se relance par `node web/run.mjs '<json>'` depuis `out/347/`, ce qui demande de copier `out/347/` sur le Mac.
- **Firefox** : à éprouver avant d'écrire l'issue 8, à cause de l'encodeur H.264.
- **Qualité des extraits** : le débit variable de WebCodecs ne vaut pas le QP 23 d'openh264. Faut-il juger à l'œil un extrait web à côté de l'extrait natif ?

## Hors périmètre repéré

- `BACKGROUND_BUDGET` (4 Gio) ne vaut que pour 64 bits. C'est noté dans le découpage (issue 8), pas fait.
- `out/347/raw/extract.yuv` (2,87 Go) et `out/347/runs/` sont à effacer quand tu veux.
