# #347 — notes d'E/S, de page, de render et de zip (fork « lecture de code »)

Lu sur `rust/347-wasm` (9172af1). Les essais de compilation sont faits dans `out/347/ws-render/` (copie de `out/347/ws/`, cible `out/347/target-render/`) et `out/347/zipprobe/` (projets autonomes, cible `out/347/target-zip/`). Rust 1.99.0, `wasm32-unknown-unknown`.

Incident : `cargo new` dans `out/347/zipprobe/` a ajouté les sondes aux `members` du `Cargo.toml` du dépôt, et les premières compilations ont touché `Cargo.lock` (7 paquets). J'ai rétabli les deux fichiers à l'identique depuis `git show HEAD:…` ; `git status` est propre. Chaque sonde porte maintenant son propre `[workspace]`.

## 1. Écrire un dossier de sortie dans une destination en mémoire

### Inventaire des accès disque (hors tests)

Lignes qui touchent le disque (`std::fs`, `File`, `tempfile`, `persist`, `rename`, `create_dir`, `read_dir`, `metadata`, `OpenOptions`, `remove_*`) : 56 dans `files` (10 fichiers sur 13), 3 dans `summary` (`image_file.rs`), 32 dans `cli` (9 fichiers).

Écritures du dossier d'une vidéo, dans l'ordre de `cli::process::processed_since` (`process.rs`, 451 lignes) :

- `fs::create_dir_all(dest)` (`process.rs:201`), puis `files::previous::remove_previous_outputs` (`previous.rs`, 126 lignes, 7 accès : `read_dir`, `remove_file`). C'est le ménage d'un passage précédent, sans objet dans le navigateur : la destination est neuve à chaque calcul.
- CSV des pistes : `files::table::write_tracks_csv` (`table.rs:56`). Il formate ligne par ligne dans un `BufWriter<File>` sur un `partial_file`, puis passe par `exclusive::rename` (`persist_noclobber`, sinon `reserve` + `persist`, `exclusive.rs`, 30 lignes). Le contenu ne dépend que de `header()` et `row()`, qui sont purs : une fonction `tracks_csv(…) -> String` suffit, et l'écriture disque n'en garde que l'emballage.
- Image résumé et fond : `cli::overview::write_overview` (`overview.rs:79`) → `summary::image_file::write_png` (`image_file.rs`, 77 lignes). Le PNG est encodé dans un `File` sur un `partial_file`, puis `persist`. `png::Encoder` accepte n'importe quel `Write` : `png_bytes(canvas) -> Vec<u8>` est un changement d'environ 15 lignes.
- Extraits et zooms : `fs::create_dir_all(extraits)` (`process.rs:444`), puis `render::clips::render`, qui écrit par `media::encode::Encoder::create(path)` (FFmpeg ouvre le fichier lui-même, avec `media::encode::partial_file`). Voir le §3 : la destination d'un extrait est un encodeur, pas un fichier.
- `params.json` : `files::params::write_params` (`params.rs:114`) fait `serde_json::to_string_pretty` en mémoire, puis `partial` + `persist`. Une fonction `params_json(…) -> Vec<u8>` le sépare. `params.rs:102` `read_params` relit par `fs::read_to_string`.
- Page `resultats.html` : `cli::written::write_page` (`written.rs:60`) relit tout le dossier (§2), rend la page par `page::render`, et la remplace par `partial` + `persist` seulement si `ours()` (lecture de l'en-tête du fichier existant). `create_reached_root` fait `create_dir_all`.
- `.batdetect/echecs/` : `cli::ledger` → `files::failures::{record_failure, clear_failure}` (`failures.rs`, 80 lignes, 7 accès). `.batdetect/refus.json` : `cli::refusals` → `files::refusals::{record_refusals, clear_refusals}` (`refusals.rs`, 65 lignes, 6 accès). Sans objet dans la page d'essai, qui n'a qu'une vidéo : un échec s'y affiche au lieu de s'écrire. Pour le zip identique, ils ne sont écrits que s'il y a un échec : avec une seule vidéo, un échec veut dire pas de dossier à zipper.
- Autres lectures de `process.rs` : `video_bytes` = `fs::metadata(video).len()` (`process.rs:327`), qui devient `File.size` côté navigateur ; `started.elapsed()` sur un `std::time::Instant` (panique en `wasm32-unknown-unknown`, voir plus bas) ; `Zoned::now()` pour la date.

Sans objet dans le navigateur, à laisser au binaire : `exclusive` (noclobber, réservation), `partial_file` (préfixe et suffixe `.partial`, `tempfile`), `previous`, `cli::reuse` (sauter une vidéo faite, 64 lignes), `cli::writable` (sonde d'écriture, 59 lignes), `cli::disk` (`statvfs`, 35 lignes), `cli::space` et `cli::inventory` (place et inventaire du lot), `cli::jobs` (5 accès), et l'écriture conditionnelle `ours()` de la page.

À l'exécution en `wasm32-unknown-unknown`, `std::fs` compile mais rend `Err(Unsupported)` sans paniquer. `files` et `summary` compilent donc (essai du parent), mais n'écrivent rien.

### Forme proposée

- Un trait de destination, en chemins relatifs à la racine de la soirée :

  ```rust
  trait Destination {
      fn put(&mut self, path: &RelativePath, bytes: Vec<u8>) -> Result<(), Error>;
      fn stream(&mut self, path: &RelativePath) -> Result<Box<dyn Write + '_>, Error>;
  }
  ```

  `put` sert à tout ce qui est fait en mémoire de toute façon (CSV d'environ 1 Ko, `params.json` de 13 Ko, PNG d'environ 1 Mo, page d'environ 107 Ko sur la 092). `stream` sert aux extraits, qui sortent paquet par paquet de l'encodeur.
- `Disk` : implémentation native avec `partial` + `persist` / `exclusive::rename`, et le comportement « taken » actuel.
- `Memory` : `BTreeMap<RelativePath, Blob>` côté Rust pour les petits fichiers, et pour les extraits un handle opaque vers un `Blob` JS, pour que les 194 Mo d'extraits de la 092 ne vivent pas dans la mémoire linéaire du wasm.
- Effort :
  - `files` : sortir `tracks_csv`, `params_json` et `png_bytes` en fonctions pures, environ 60 lignes déplacées, et la même chose dans `summary::image_file`. Le trait et `Disk` font environ 80 lignes neuves. `failures` et `refusals` restent sur disque.
  - `cli::process` : l'orchestration d'une vidéo (`processed_since`, `listed`, `summaries`, `Settings::of`, `tracked`) dépend de `render::clips`/`source` (FFmpeg), de `Instant` et de `Zoned`. Il faut la sortir de `cli`, qui ne compilera jamais en wasm (clap, rustix, ctrlc, FFmpeg), vers une crate de bibliothèque sans FFmpeg, paramétrée par la destination, la source d'images et l'encodeur (§3). Environ 300 des 451 lignes déménagent, plus 100 à 150 lignes d'adaptation.
  - Total estimé : 500 à 700 lignes touchées ou déplacées, sans changer le binaire.
- Découpage : (a) fonctions pures d'octets dans `files`/`summary`, refactor sans effet ; (b) trait `Destination` et `Disk`, avec `cli` qui passe par lui ; (c) crate d'orchestration d'une vidéo sans FFmpeg, extraite de `cli::process` ; (d) `Memory` et liaison JS. (a) et (b) sont indépendants du wasm et se vérifient par `compare local` sans écart.

### jiff en `wasm32-unknown-unknown`

- Sans la feature `js`, `Zoned::now()` et `Timestamp::now()` paniquent. Source `jiff-0.2.38/src/now.rs:27-39` : « getting the current time in wasm32-unknown-unknown is not possible with just the standard library, enable Jiff's `js` feature ».
- Avec `js` (`wasm-bindgen` et `js-sys`), l'heure vient de `js_sys::Date` et le fuseau de `Intl.DateTimeFormat().resolvedOptions().timeZone` (`tz/system/wasm_js.rs`). La base des fuseaux est embarquée (`tzdb-bundle-platform`).
- Taille mesurée : un cdylib `opt-level = "s"`, LTO et `strip` qui appelle `Zoned::now().strftime` pèse 867 671 octets, contre 13 791 pour une base vide. Jiff ajoute donc environ 854 Ko, presque tout pour la base des fuseaux.
- `std::time::Instant::now()`, dont dépend `elapsed_s` dans `process.rs:180-273`, panique aussi sur cette cible.
- Recommandation : `files` n'appelle pas l'horloge, seul `cli` le fait. La crate d'orchestration reçoit la date déjà formatée (`local_date`) et la durée de l'appelant. La page les tire de `new Date()` et `performance.now()`, et jiff n'entre pas dans le wasm. `files` garde jiff pour `verdicts.rs` (`Timestamp::try_from(SystemTime)`, lecture disque seulement).

## 2. La page de résultats

- Construction : `cli::written::write_page` → `page_of(root, jobs, run)` (`written.rs:114-165`) → `page::model::Page` → `page::render(&page, labels)` (`page/src/document.rs`). La crate `page` ne lit aucun disque : 1 770 lignes de Rust, et 75 820 octets de CSS et JS embarqués par `include_str!`. Elle compile en wasm (essai du parent).
- Ce que `page_of` lit du disque :
  - `files::results::read_results(root)` (`results.rs:72-190`) : `read_dir` de la racine, puis pour chaque dossier `fs::read(params.json)` et `serde_json::from_slice::<Written>`. Il teste ensuite la présence du CSV, de l'image résumé et de la vidéo annotée par `is_file`, et la taille des PNG par `png_size` (`picture.rs`, lecture de l'en-tête IHDR). Il lit aussi `read_failures` dans `.batdetect/echecs/`.
  - `files::refusals::read_refusals(root)`.
  - `files::verdicts::read_verdicts_file(root)` : `read_dir`, `metadata().modified()` et `fs::read` du `verdicts*.csv` le plus récent, embarqué en octets dans la page.
- Depuis la mémoire, rien n'est à relire : on a déjà `Written`, en désérialisant les octets de `params.json` qu'on vient de produire (le même chemin `serde_json::from_slice`), les tailles des PNG (celles du canevas) et la liste des fichiers produits. Il suffit de couper `results::finished(dest, written)` en deux : une partie pure, `finished_from(written, present: impl Fn(&str) -> bool, size: impl Fn(&str) -> Option<(u32, u32)>)`, et l'adaptateur disque actuel. Même chose pour `page_of`, coupé en « dossiers → `Page` » pur et lecture. Il faut environ 80 lignes déplacées dans `files::results` et 40 dans `written`, plus `video()`, `refusals()`, `failed()`, `date()` et `title()` (`written.rs:167-288`), à sortir de `cli` avec l'orchestration.
- Libellés : `page::labels::Labels` est implémenté par `cli::sheet` en Fluent (`locales/{fr,en}.ftl`). Fluent compile en wasm (sonde `fluentprobe` : `fluent-bundle` 0.16.0 et `unic-langid` 0.9.6, environ 17 Ko de plus que jiff seul dans le même cdylib). `sheet` doit sortir de `cli` vers une crate de libellés, avec les `.ftl`.
- Zip (avec zone de jugement) : c'est la page d'aujourd'hui, à l'identique. `page::document` rend la barre d'enregistrement et la zone de jugement dès qu'une entrée a une section (`judged = page.entries.iter().any(has_section)`, `document.rs:62`). Les médias y sont des chemins relatifs (`clip`, `zoom`, `summary`, `background`, `table`) : dézippée, la page les trouve comme dans un dossier de l'outil. `verdicts` vaut `None` à la première génération.
- Page web (sans jugement) : il faut un drapeau du modèle, par exemple `Page::judging: bool`, qui ne rend ni `judge.js`, ni `store.js`, ni la barre, ni `csv.js` pour la lecture des verdicts, et qui garde la frise, la liste et le lecteur. Les chemins relatifs y deviennent des URL `blob:`. Soit la page reçoit une table chemin → URL et le modèle porte les URL à la place des noms de fichier, soit un petit JS réécrit les `src` et les `href` au chargement. La première forme est plus simple à tester par les tests figés de `page`. Effort estimé : 30 à 60 lignes dans `page`, avec ses tests.

## 3. render sans son encodeur natif

### Essai de compilation

Dans `out/347/ws-render/` :

- `media` : feature `native` par défaut (FFmpeg, `tempfile`). `frame`, `planes` et un nouveau `region` restent hors feature : j'ai sorti la structure `Region` de `convert/region.rs`, qui y était mêlée aux contextes swscale, vers `media/src/region.rs`, et `convert::region` la réexporte.
- `render` : feature `native` (→ `media/native`). Sont fermés `clips` (228 lignes), `encoders` (170), `group` (274), `picture` (66), `source` (251), `schedule` (168) et `ahead` (94), soit 1 248 lignes.
- Deux retouches pour délier ce qui reste :
  - le trait `Marks` passe de `picture` (qui tient le `Converter` swscale) à un module `marks` ;
  - `zoom` a sa propre erreur (`ZoomSize`, `Frame`) au lieu de `clips::Error`. Dans le vrai changement, `clips::Error` l'envelopperait par `#[from]`.
- Résultat : `cargo build --release --target wasm32-unknown-unknown -p render --no-default-features` passe, avec 62 avertissements de code mort, puisque les points d'entrée sont fermés. Restent 1 094 lignes : `colored`, `config`, `ink`, `marks`, `overlay`, `positions`, `windows`, `work`, `zones` et `zoom`.
- Ce qui reste est pur : dessin des boîtes, `#N` et traces (`overlay`, `ink`, par `draw`/tiny-skia), géométrie des zones, fenêtres et groupes d'extraits (`windows`), recadrage et agrandissement du zoom au plus proche voisin, en pur Rust, sur `YuvFrame` (`zoom::Magnifier::magnify`, `sample`).
- Non vérifié : la compilation native de cette copie (`ffmpeg-sys-next` y échoue faute de `.ffmpeg/` dans la copie). Les retouches ne font que déplacer des définitions.

### Ce qui casse à l'exécution, et non à la compilation

- Fils : `schedule::run` (`thread::scope`, `schedule.rs:120`), `ahead::serve_ahead` (`thread::scope`, `ahead.rs:36`) et `clips` (`thread::available_parallelism`, `clips.rs:119`). En `wasm32-unknown-unknown` sans atomics, `thread::scope` compile mais le `spawn` panique. Il faut un chemin à un seul fil (un `Alone` existe déjà dans `schedule.rs:30`), ou des workers JS.
- Asynchrone : le décodage (`source`, `ahead`) et l'encodage (`encoders::Sink::push`) sont synchrones en Rust, alors que WebCodecs est asynchrone (`VideoDecoder.decode` → callback `output`, `VideoEncoder.encode` → callback `output`, contre-pression par `encodeQueueSize`). Un wasm ne peut pas attendre une promesse au milieu d'un appel. Il faut soit inverser le contrôle (render piloté image par image : « voici l'image n, rends les images dessinées »), soit faire tourner render dans un Worker qui bloque sur `Atomics.wait` pendant que le thread principal sert WebCodecs (avec `SharedArrayBuffer`, donc COOP/COEP), soit utiliser JSPI. Je recommande l'inversion de contrôle : la boucle des groupes (`group.rs`) est déjà un flux d'images.

### Conversion YUV ↔ RGBA (swscale BT.709)

Usage dans render : `picture::Picture::draw` (`picture.rs:43-63`).

1. `Converter::display(source, drawn)` : niveaux de luma « noir à 16 » (`black_at_16`, table de 256) et chroma ramenée en plage limitée (`chroma_squeeze`, table de 256). Ce sont deux tables, en pur Rust immédiat.
2. Pour chaque zone touchée par un dessin seulement (`Marks::zones`), `yuv_to_rgba_region` en BT.709, puis le dessin tiny-skia sur le RGBA, puis `rgba_to_yuv_region` en retour. Seules ces petites zones passent par swscale, pas l'image entière.

- `convert.rs` règle swscale par `sws_setColorspaceDetails` (`SWS_CS_ITU709`, plage source et destination, `convert.rs:301-…`).
- En pur Rust, il faut une conversion BT.709 en virgule fixe sur les zones. Elle ne sera pas identique au bit près à swscale : arrondis et filtres de chroma différents. Les empreintes figées de `render::drawn` (tests `slow_frozen`) n'en vaudraient plus pour la version web, qui aurait les siennes. Les détections ne sont pas concernées.
- Côté navigateur, on peut aussi dessiner sur un `OffscreenCanvas` au-dessus d'un `VideoFrame` et donner au `VideoEncoder` un `VideoFrame` RGBA, que le navigateur convertit lui-même. Mais le dessin quitterait alors `draw` (Roboto, palette) pour le Canvas 2D, et deux rendus seraient à maintenir. Je recommande la conversion en pur Rust, sur les zones.
- `ColorRange` (`frame.rs`) et la plage de la source se lisent dans `VideoFrame.colorSpace.fullRange`.

### Forme d'un encodeur abstrait que WebCodecs peut remplir

Le trait existe déjà, privé : `render::encoders::Sink` (`encoders.rs:17-22`) :

```rust
fn open(&mut self, path: &Path, frames: usize) -> Result<(), encode::Error>;
fn open_stamped(&mut self, path: &Path, stamps: Stamps) -> Result<(), encode::Error>;
fn push(&mut self, frame: &YuvFrame) -> Result<(), encode::Error>;
fn finish(self) -> Result<(), encode::Error>;
```

Il est implémenté par `media::encode::fanout::Fanout` (un encodeur par extrait d'un groupe). Pour WebCodecs :

- Le chemin devient une clé de la destination (§1). `Stamps` (pts et time base de la source, `encode.rs:232`) donne les `timestamp` en microseconds des `VideoFrame`. `Settings` (taille, cadence, `Codec`, intervalle d'images-clés, `encode.rs:178-206`) donne le `VideoEncoderConfig` : `codec: "avc1.42…"` ou `"avc1.64…"`, `width`, `height`, `framerate`, `bitrate`, `avc: { format: "avc" }`.
- `push(&YuvFrame)` livre trois plans I420, que JS emballe en `new VideoFrame(buffer, { format: "I420", codedWidth, codedHeight, timestamp, layout })`. C'est une copie de la mémoire wasm vers le navigateur, et la sortie est un `EncodedVideoChunk`, plus `decoderConfig.description` (avcC) au premier paquet.
- L'emballage mp4 (moov, trak, stts et ctts aux pts de la source) est à faire à part, en JS ou en Rust, puisque WebCodecs ne rend que des paquets. Il est hors de mon périmètre (parent).
- `encode::Error` porte des causes FFmpeg (`LibraryError`) : il faut une erreur de sink propre à `render`, ou générique.

## 4. Zip sans recompression, écrit par morceaux

Sonde : `out/347/zipprobe/zipstream`, avec `zip = { version = "9.0.5", default-features = false }`.

- Dépendances en wasm : `crc32fast`, `indexmap` (et `hashbrown`, `equivalent`), `memchr`, `typed-path`. Pas de deflate, pas de C, pas de `getrandom`.
- Compile en `wasm32-unknown-unknown` : oui. Un cdylib de 93 249 octets (`opt-level = "s"`, LTO, `strip`, `panic = "abort"`), contre 13 791 pour une base vide : zip ajoute environ 79 Ko.
- Écriture par morceaux : `ZipWriter::new_stream(W: Write)`, sans `Seek`, avec des en-têtes locaux suivis d'un descripteur de données (bit 3, « extended local header »). Chaque `write` est remis à un rappel, qui sera une `WritableStream` ou une liste de parts de `Blob` côté JS. La plus grande écriture vaut la taille du morceau donné (65 536 octets). L'archive n'est jamais tenue en mémoire : sur une archive de 4,50 Go, le pic de mémoire résidente du processus natif est de 1 896 Ko.
- ZIP64 : géré par `large_file(true)`. Essai natif : 4 entrées, dont une de 4 500 000 000 octets, soit 4 501 804 796 octets écrits en 4,1 s. `unzip -l` liste les 4 tailles justes, et `zipfile.testzip()` de Python ne trouve aucune erreur (CRC vérifiés). Toutes les entrées sont « stored », version requise 4.5. Fichier effacé après l'essai.
- Réglages à prévoir :
  - `large_file(true)` seulement pour une entrée de plus de 4 Gio, pour garder la version 2.0 et l'en-tête court sur les autres. On connaît la taille de chaque fichier avant de l'écrire, puisqu'il est déjà produit.
  - La date des entrées vaut 01-01-1980 par défaut : il faut la donner par `last_modified_time`, depuis la date JS.
- À vérifier sur le Mac, non fait ici : l'Utilitaire d'archive de macOS sur des entrées « stored » avec descripteur de données, et sur du ZIP64 en flux. Il passe par le répertoire central, mais je ne l'ai pas éprouvé. Si l'Utilitaire refuse, on peut écrire les tailles et le CRC dans l'en-tête local, puisqu'ils sont connus avant l'écriture : il faut alors calculer le CRC d'abord, soit une lecture de plus du `Blob`. `rawzip` (0.5.2) et `async_zip` (0.0.19) existent et n'ont pas été essayés.

## Taille projetée du zip de la 092

Mesurée sur un dossier de sortie réel de la 092 sur lumiel (`out/214/b-092-1/`, réglages par défaut) : 20 fichiers, 196 666 471 octets.

| Contenu | Taille |
|---|---:|
| 14 extraits | 192,6 Mo (de 10,1 à 38,8 Mo) |
| 1 zoom (piste 3) | 2,0 Mo |
| image résumé | 1,07 Mo |
| fond | 0,87 Mo |
| `resultats.html` | 107 Ko |
| `params.json` | 13,5 Ko |
| CSV | 1 Ko |

- Le zip « stored » ajoute environ 200 octets par entrée (en-tête local, descripteur et entrée du répertoire central), soit environ 4 Ko pour 20 entrées : environ 197 Mo au total.
- L'extrait (`out/214/a-extract-1/`) : 9 fichiers, 75 362 351 octets, dont 73,4 Mo pour ses 4 extraits.
- La taille dépend du débit de l'encodeur WebCodecs, qui ne sera pas celui d'openh264 en natif : le chiffre ne vaut que pour un débit égal.
- Mémoire : si les extraits restent des `Blob` JS, gardés hors du tas wasm, et si le zip est écrit par morceaux vers `showSaveFilePicker` (Chromium) ou assemblé en `Blob` de parts, le tas wasm ne tient que les petits fichiers (environ 2 Mo) et un morceau de 64 Ko. Le navigateur, lui, garde les 197 Mo de `Blob` des extraits, en mémoire ou sur disque selon le moteur.
