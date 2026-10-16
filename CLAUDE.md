# batdetect — guide pour Claude

Détection et suivi de chauves-souris dans des vidéos de jumelles thermiques. Rust, FFmpeg
lié en statique. Installation et commandes : [README.md](README.md). Base de
connaissances : [docs/](docs/README.md), à lire avant de toucher à un réglage
([docs/02-methode.md](docs/02-methode.md)).

```bash
mise run check                    # rust:static et rust:test, 20 s à 1 min : vert avant de rendre la main
cargo run --release -p batdetect -- in/video_092_original.mp4 -o out # référence : 5 min, 14 pistes, ~20 s de calcul (docs/09)
```

## Rôles

Manon, naturaliste, confirme les passages à l'œil. Son verdict sur la 092 (9 octobre 2026 :
13 chauves-souris sur 14 pistes, seule la piste lente 3:29 n'en est pas une) est la vérité
terrain du test `slow` ([docs/03](docs/03-resultats-092.md)). Mieux vaut un faux positif
facile à écarter qu'un passage manqué. Une question sur ce qui est ou n'est pas une
chauve-souris lui remonte ; elle ne se tranche pas dans le code.

## Règles

- **Jamais d'écriture dans git** : ni `add`, ni `commit`, ni `stash`, ni `push`.
  L'utilisateur construit l'historique lui-même.
- **Tout changement de détection ou de suivi se mesure sur `video_092_original.mp4`** avant d'être rendu :
  nombre de pistes, temps de début, et mise à jour de [docs/03-resultats-092.md](docs/03-resultats-092.md).
- **`in/` et `out/` sont hors git.** `in/` contient les vidéos de l'utilisateur : ne rien
  y supprimer ni modifier.
- **L'extrait `tests/fixtures/video_092_original_3m24-4m05.mp4` est coupé sans réencodage**
  sur une image-clé de l'original (image source 6150, 30,03 i/s). Le réencoder changerait
  les détections.
- **Seules trois vidéos de `tests/fixtures/` sont suivies par Git LFS** (`.gitattributes`) :
  les deux de la 092 et `video_092_original_3m24-4m05_720x540.mp4`, l'extrait réduit une
  fois par la commande de [docs/11](docs/11-migration-rust.md#réglages), jamais refait.
  `git lfs pull` après un clone, sinon les tests `slow` et `reference` échouent sur un
  pointeur (garde dans la crate `reference`). Les cinq petites
  fixtures du Rust (`full_range.mp4`, `full_range_midway.mp4`, `two_gops.mp4`, `spot.mp4`
  et son remux en plage pleine `spot_full_range.mp4`, quelques Ko) sont des fichiers git
  ordinaires, faits une fois par la commande de
  [docs/11](docs/11-migration-rust.md#tests). Les détections de l'extrait par le Rust
  (`tests/fixtures/rust/`) aussi, écrites par `compare --frames` et redemandées au bit
  près sur chaque système. La 092 entière `video_092_original.mp4` est
  une copie exacte de `in/`. Ses tests (`tests/reference_*.rs`) tournent par
  `mise run rust:reference`, dans leur propre tâche de CI `rust-reference`. La tâche
  `rust-windows-slow` ne récupère que l'extrait ; les tâches
  `rust-slow` et `rust-settings`, l'extrait et sa 720×540.
  Tolérances du test de référence, sur les deux vidéos : nombre de pistes exact, début ±5
  images, points ±3, qui couvrent l'écart mesuré entre Mac et Linux (4 images, 2 points).
- **Les vidéos se copient depuis les jumelles en USB**, jamais par l'export Stream Vision 2,
  qui divise le débit par 12 ([docs/01-contexte.md](docs/01-contexte.md)).
- **Docs synchronisées** : un changement de comportement met à jour `docs/` dans la même
  série de changements, dans le document qui possède cette vérité (table de
  [docs/README.md](docs/README.md#qui-tient-quoi)), et seulement là.

## Code

**Un module, une responsabilité, ses tests.** Mieux vaut beaucoup de petits fichiers clairs
qu'un gros fichier à plusieurs rôles.

- Code et identifiants en anglais, docs en français. Une valeur magique devient une constante
  nommée.
- Tests sans commentaire, le nom du test porte le pourquoi. Pas de mock de FFmpeg ni d'un
  processus : images fabriquées ou vraie vidéo.
- `tests/fixtures/reference/` : la référence v0.1.1 de l'extrait et de la 092, la 092 à
  1440 pixels de travail (`video_092_original_width_1440/`, #151), et dans `settings/` ses
  16 cas à d'autres réglages de détection (`--detect`, docs/11), fichiers git ordinaires ;
  format, dossier des 26 vidéos (`BATDETECT_VIDEOS_DIR`) et versions dans son
  `README.md`. Elle ne se réenregistre que depuis le tag `python-final` (`tools/reference_v011.py`, qui
  exige `src/` identique à `v0.1.1`).
- `packaging/` : `NOTICE` et `GPL-3.0.txt` décrivent le binaire v0.1.1 publié, jusqu'à #93.
  `release.yml` publie sur un tag `vX.Y.Z`, derrière les gardes de `release-guards.sh` (docs/05).
- Toutes les coordonnées, distances et surfaces hors de la détection sont en pixels de la
  vidéo d'origine ; seule la détection travaille à `work_width`.

## Rust

Le Python est retiré depuis #199 (dernier état au tag `python-final`). La chaîne de
construction, la charte, les tests et l'histoire de la réécriture sont dans
[docs/11](docs/11-migration-rust.md).

```bash
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:ffmpeg  # FFmpeg 7.1.1 + openh264, une fois par poste
mise run rust:static      # fmt, clippy, règles de la charte, cargo deny, cargo shear
mise run rust:test        # tests rapides, sans couverture (dans check)
mise run rust:coverage    # tests rapides, couverture ≥ 95 % (dans check:ci)
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run check:ci  # rust:coverage, rust:slow, rust:settings : joué par la CI
# git push : le hook pre-push de lefthook lance mise run check (pre-commit : rustfmt seul)
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:slow       # l'extrait LFS, le test de mémoire (hors couverture)
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:settings   # les 16 cas de réglages, sous Linux seulement en CI
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:reference  # la 092 entière
/Users/zaratan/Projects/dvr-wt/verrou lourd cargo run --profile ci -p compare -- local  # écarts à la référence (extract, 092, 092-1440, settings, local)
/Users/zaratan/Projects/dvr-wt/verrou lourd cargo run --release -p bench -- VIDÉO --against DOSSIER  # garde du banc contre un banc de la v0.1.1
BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=… mise run rust:data  # référence et 26 vidéos sur une autre machine (seau, sommes dans tests/fixtures/reference/SHA256SUMS)
```

Avant de rendre : `mise run check`. Le lourd (`check:ci`, `rust:reference`, `local_*`) une seule
fois, seulement si le diff peut changer son résultat : la CI en joue les morceaux sur chaque PR.
Un temps ou une mémoire publiés passent par `verrou chrono /usr/bin/time -l` au calme
(`/Users/zaratan/Projects/dvr-wt/MESURES.md`).
Le hook pre-push de `lefthook.yml` lance `mise run check` (pre-commit : rustfmt seul) ; sans cache
FFmpeg pour la branche, `LEFTHOOK=0 git push` le saute. Ne pas lancer `lefthook run` ni
`lefthook install` : ils réécrivent les hooks du dossier git commun à tous les worktrees.

Windows (MSVC, `/MT`, `+crt-static`) ne se construit que dans les tâches CI
`rust-windows` et `rust-windows-slow`, par `packaging/ffmpeg-windows.sh` sous MSYS2
([docs/11](docs/11-migration-rust.md#windows)).

Workspace virtuel, `members = ["crates/*"]`, toolchain de `rust-toolchain.toml`. Douze
crates depuis #100, dépendances à sens unique :

- `types` : vocabulaire commun (`Rational`, `FrameIndex`, `Pts`, `FrameRate`, `Timeline`,
  `Detection`, `Measures`, `FrameDetections`, `Span`, `Track`, `Language`, `SourceSize`, `WorkScale`).
  Ne dépend d'aucune autre crate du workspace.
- `media` → `types` : seule frontière avec FFmpeg et seule crate avec `unsafe`
  (`frame`, `decode`, `encode`, `convert`, `library`) ; `convert` passe du YUV au RGBA, et retour, en BT.709 ;
  `decode::placed` se place sur une image-clé par `pts` ; `payload` (feature `testing`) abîme un mp4 pour les tests ;
  `fingerprint` (même feature) est la seule empreinte d'image des tests, stable d'un système et d'une
  version de Rust à l'autre : les tests figés l'enregistrent.
- `reference` → `types` : chemins des fixtures et garde LFS, emplacements hors dépôt
  (`location`, dont `reference_video` pour les vidéos de la référence), lecture et écriture de la
  référence v0.1.1 et de ses bancs (`bench`), appariement des détections et des pistes.
- `engine` → `media`, `types` (#116), avec la couture du banc (`detect_injected`) ;
  `tracking` → `types` (#120) ; `files` → `types` (#122).
- `draw` : dessin pur sur une toile RGBA opaque, police Roboto embarquée, palette et tailles ; ne dépend d'aucune crate du workspace.
- `render` → `media`, `draw`, `types` (#110, #112) : images des extraits choisies par `pts`
  (`source`), reprise sur la dernière image-clé saine, un seul fil de décodage sur une plage abîmée ;
  fenêtres et groupes (`windows`), boîtes, `#N` et traces (`overlay`, `picture`),
  un encodeur par groupe (`media::encode::fanout`), groupes en parallèle (`clips`,
  `schedule`, `group`, `ahead`, `encoders`) ; zooms ralentis en openh264 (`zoom`, `work`, #103) ; vidéo annotée aux `pts` de la source, lue par tronçons (#103) ;
  `drawn` (feature `testing`) rend les images dessinées sans encodeur, pour les tests figés (#198).
- `summary` → `draw`, `types`, `files` (#104) : l'image résumé (tranches, couleurs,
  flèches, pastilles, bandeau, frise du temps, PNG), sans FFmpeg.
- `compare` → `engine`, `media`, `tracking`, `reference`, `types` (#119) : outil de
  développement, binaire `compare`, non livré ;
  `bench` → `engine`, `tracking`, `reference`, `media`, `types` (#100) : banc de mesure,
  binaire `batdetect-bench` (`--targets`, `--replay`, `--against`) ;
  sa feature `testing` porte la vidéo bruitée et la cible lente de ses tests (`noisy`) ;
  `cli` (paquet `batdetect`, binaire `batdetect`) → `engine`, `tracking`, `files`,
  `summary`, `render`, `draw`, `media`, `types`, et `compare` pour ses tests (#184) ;
  `options` (réglages de la commande), `process` (une vidéo de bout en bout ; `tracked` en
  est le suivi, partagé avec les tests figés), `consent` (question avant de traiter les vidéos
  valides quand des entrées sont refusées, `--skip-refused`), `language`, `overview` (image
  résumé), `settings` (réglages de `params.json`),
  `outcome` (boucle des vidéos, seul `catch_unwind` du code livré, #113), `writable`
  (sonde d'écriture de chaque dossier de sortie avant tout calcul, #86), `access` (table
  fermée des raisons d'un chemin refusé, sans texte, #55), `jobs` (vidéos et dossiers de
  sortie, toutes les entrées fautives refusées ensemble avant tout calcul, #55), `progress` (étapes
  d'une vidéo annoncées par `process`, #63), `failure` (cause d'un échec, sans texte, #64), `tally` (modèle du bilan de fin de soirée, #65), dans le binaire `status` (vidéo et étape en
  cours sur la sortie d'erreur, redessinée sur un terminal, #63), `board` (mise en forme du bilan, #65), `style` (couleurs et nom de l'option d'aide, #56) et `parsing` (lecture
  des valeurs d'options, erreur typée avec l'option et sa plage, #55) ; usages courants en tête de l'aide et sans argument (#56), version gravée par
  `build.rs` ; textes en Fluent dans `locales/{fr,en}.ftl`, langue de l'aide, des erreurs,
  des lignes et du bandeau (#114).

Charte (23 règles numérotées, lints et raisons dans
[docs/11](docs/11-migration-rust.md#charte)) :

- Rust idiomatique, sans imiter ni Python, ni numpy, ni OpenCV. Un écart de détection
  se mesure par appariement, puis se juge.
- Lints dans `[workspace.lints]` seulement ; zéro `#[allow]` ou `#[expect]`. Un lint
  gênant se résout en réécrivant le code.
- Erreurs typées par `thiserror`, avec leur cause. Ni `unwrap`, ni `expect`, ni
  `panic!`, ni `println!` hors des tests ; aucun résultat jeté.
- Aucun type de `ffmpeg_next` hors de `media`. Configurations validées à la
  construction.
- Un module, ses tests dans `src/<module>/tests.rs`. Le niveau d'un test se choisit par
  le nom de son fichier : `tests/slow_*.rs`, `tests/settings_*.rs`, `tests/reference_*.rs`,
  `tests/local_*.rs` (les 26 vidéos, jamais en CI), `tests/hardware_*.rs` (VideoToolbox).
- Une dépendance se déclare une fois dans `[workspace.dependencies]`.
- **Commentaires** : la règle du zéro commentaire vaut aussi en Rust, avec deux
  exceptions. Chaque bloc `unsafe` contient une seule opération et porte une ligne
  `// SAFETY:` qui dit pourquoi le contrat tient. Un `///` d'une ligne est permis sur un
  élément public dont le nom ne dit pas le contrat externe. Au plus 2 %.
