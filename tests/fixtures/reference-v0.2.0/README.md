# Référence v0.2.0

Ce que la chaîne de détection et de suivi en Rust de la v0.2.0 (tag `v0.2.0`, ea4d789) détecte et suit, enregistré une fois par le binaire `record` de la crate `compare` (#250), depuis un clone propre du commit 1e4cf6b, sur lumiel (Linux x86_64).
Elle sert de référence aux gardes de détection et de suivi ; la [référence v0.1.1](../reference/README.md) reste le témoin indépendant (entrées vues par ffprobe, aval du Python).

## Emplacements

- **Dans le dépôt**, `tests/fixtures/reference-v0.2.0/` : l'extrait (`video_092_original_3m24-4m05/`), la 092 entière (`video_092_original/`), la 092 à 1440 pixels de travail (`video_092_original_width_1440/`), les 16 cas de réglages (`settings/<cas>/`), mêmes dossiers et mêmes réglages que la v0.1.1, et les deux bancs de la 092 (`bench/transit/`, `bench/hunt/`, 2,7 Mo).
  5,5 Mo en tout. `SHA256SUMS` : voir [Récupération](#récupération-sur-une-autre-machine).
- **Hors dépôt**, dans le dossier nommé par la variable facultative `BATDETECT_REFERENCE_V020_DIR` (sur lumiel : `~/Projects/dvr-wt/reference-v0.2.0`) : `data/<vidéo>/`, les données des 26 vidéos de la v0.1.1, 31 Mo.
  Les vidéos restent celles de `BATDETECT_VIDEOS_DIR`.

### Récupération sur une autre machine

Préfixe `reference-v0.2.0/` du seau `chiro-dvr-videos`, à côté de ceux de la v0.1.1 : `data/` seulement.
`SHA256SUMS`, à côté de ce fichier, donne la somme des 78 fichiers de ce préfixe.

```bash
BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=… BATDETECT_REFERENCE_V020_DIR=… mise run rust:data
```

Sans `BATDETECT_REFERENCE_V020_DIR`, la tâche ne récupère que la v0.1.1, comme avant.
Avec elle, le dossier passe par les mêmes contrôles que les deux autres : chemin absolu, aucun recouvrement avec `in/`, `~/dossier sans titre/img_0000` ou les deux autres dossiers, aucun fichier absent de `SHA256SUMS`, hormis un `README.md` à sa racine.

## Format

Le format de la v0.1.1, en version 2 : mêmes dossiers, mêmes `frames.jsonl` et `tracks.jsonl`, mêmes conventions (numéros d'image, plages bornes comprises, coordonnées en pixels de la vidéo d'origine, gzip sans nom ni date).
Pas de `tracks.csv` : le CSV du Rust s'écrit par `batdetect`, et celui de l'extrait est `tests/fixtures/rust/video_092_original_3m24-4m05/tracks.csv`.
Les flottants sont écrits par `serde_json` : la plus courte écriture qui redonne le même `f64`, `12.0` pour un réglage entier.

### `video.json`

| Champ | Contenu |
| --- | --- |
| `format_version` | 2 |
| `reference` | `v0.2.0`, le tag de version qui précède le commit du binaire |
| `video`, `stream` | recopiés du `video.json` de la v0.1.1 du même cas ; `record` refuse une vidéo dont `media` lit une autre taille, une autre cadence ou une autre base de temps |
| `decoder` | à la place de `ffprobe` : `error_frames` (images que le décodeur de `media` signale), `gap_frames` (images après un saut de `pts`), `losses` (`before_first`, `after_last` : images perdues avant la première et après la dernière), `threads` (`{"kind": "single"}`, ou `frame` ou `slice` avec `count`) |
| `background_half_window` | demi-fenêtre du fond en images, marge des plages abîmées |
| `damaged_spans` | plages abîmées exactes du décodeur courant (`engine::damage::Damage`), marge comprise |
| `unstable_spans` | plages instables du suivi, hors plages abîmées |
| `counts` | `frames`, `detections` (brutes), `detections_kept` (après exclusion), `damaged_frames_reported`, `tracks`, `track_points` |
| `config` | `DetectConfig` (réglages avec lesquels le moteur a tourné : ceux de la v0.1.1 du même cas), `TrackConfig`, `StabilityConfig` (défauts) |
| `versions` | `batdetect` (`git describe`), `commit`, `rustc`, `ffmpeg`, `system` |
| `files` | `frames` (`frames.jsonl`, ou `frames.jsonl.gz` comme dans la v0.1.1) et `tracks` |

`frames.jsonl` : une ligne par image, images vides comprises ; `pts`, `key_frame` et `decode_error` vus par le décodeur de `media` ; détections brutes dans l'ordre du moteur.
`tracks.jsonl` : toutes les pistes du suivi par défaut (`exclude` avec les plages abîmées, puis `track_detections`, comme `batdetect`), avec tous leurs points.

## Bancs

`bench/<nom>/video_092_original/` : un run de `batdetect-bench tests/fixtures/video_092_original.mp4 --targets <v0.1.1>/bench/<nom>/bats.jsonl.gz` depuis le clone de l'enregistrement, avec les cibles des deux bancs de la v0.1.1 (`transit` : 270 cibles, `hunt` : 720), réglages par défaut.
`run.json` (vidéo, cadence, plages, réglages, chemin des cibles rejouées sur lumiel), `reference.jsonl.gz` et `injected.jsonl.gz` (détections des deux runs), `bats.jsonl.gz` (cibles avec le contraste mesuré par le Rust), `bench.json` (réglages et évaluation, `code` : `1e4cf6b`).
`reference::location::committed_bench(nom)` donne `bench/<nom>/`.

## Provenance

- Le binaire `record` refuse de tourner s'il est `-dirty`, fichiers non suivis compris, ou construit hors de git (docs/developpement.md, crate `compare`).
  L'enregistrement s'est fait depuis `git clone --branch rust/250-record` au commit 1e4cf6b, `git status` vide, `git describe` à `v0.2.0-28-g1e4cf6b`.
- Chaîne de détection depuis `v0.2.0` (`git diff --stat v0.2.0 1e4cf6b -- crates/{engine,media,tracking,types} Cargo.lock rust-toolchain.toml packaging/ffmpeg-* .cargo`) : #172 (`FrameDetections` rempli par `push` dans les tests seuls), #245 (noms uniques des fichiers `.partial`, côté encodage), #227 (ouverture par `avformat_open_input` puis `avformat_find_stream_info`, comme le faisait `ffmpeg::format::input`, pour les chemins non UTF-8), #274 et #250 (tests et dépendance de test).
  Aucun ne change ce que le décodeur rend ni ce que le moteur détecte, ce que la preuve ci-dessous mesure.
- `cargo tree -p engine -p media -p tracking -p types -e normal` est le même qu'à `v0.2.0`, aux chemins près.
  `Cargo.lock` ne gagne que `serde` et `serde_json` dans les dépendances de `compare`, et `serde_json` dans celles d'`engine`, comme dépendance de test (#274) : elle n'entre pas dans le binaire.

## Preuve par le binaire de v0.2.0

`v0.2.0` construit à part (`git archive v0.2.0`, vidéos LFS copiées, `~/Projects/dvr-wt/v0.2.0`), puis `compare <cible> --frames` sur l'extrait, la 092, la 1440, les 16 réglages et les 26 vidéos.

- Détections, 19 cas du dépôt : chaque `frames.jsonl` décompressé est identique à l'octet à celui de `v0.2.0`. Aucun n'a de plage abîmée.
- Détections, 26 vidéos : `record local --same-as` donne « identical » pour les 26, entrées comprises (nombre d'images, `pts`, images-clés, erreurs de décodage) ; hors des plages abîmées, les détections sont égales.
  Dans les plages, 15 vidéos et 1 754 images diffèrent : les pixels que FFmpeg y dissimule varient d'une lecture à l'autre (#159, #176).
- Pistes : `tracks.jsonl` de l'extrait et de la 092 contre le CSV de `batdetect` de `v0.2.0` : mêmes numéros, nombres de points, débuts (temps des `pts`), cordes, chemins, surfaces et amplitudes, 4 et 14 pistes, 0 écart.
- Bancs : les deux runs de `batdetect-bench` de `v0.2.0` donnent les mêmes `.gz` décompressés, le même `run.json` et le même `bench.json` par valeurs, hors `code` (`unknown` depuis une extraction sans git) et durées.

## Reproduction

Le tag `reference-v0.2.0` se pose sur le commit qui ajoute ces fichiers.
Réenregistrer depuis ce tag donne les mêmes détections **hors des plages abîmées**, pas les mêmes fichiers à l'octet pour les 26 vidéos : dans les plages, les détections changent d'une lecture à l'autre.
`SHA256SUMS` change donc.
Les 19 cas du dépôt n'ont pas de plage abîmée : leurs `frames.jsonl` et `tracks.jsonl` se refont à l'octet, mais leur `video.json` change par `versions.batdetect` et `versions.commit`, puisque le commit du tag n'est pas 1e4cf6b.

Le décodeur de lumiel signale 14 images en erreur de plus que celui du Mac, sur 8 vidéos (docs/developpement.md, `local_damage`) : les plages de ces vidéos sont celles de lumiel.
Un enregistrement sur le Mac ne les signalerait pas.

```bash
git clone --branch reference-v0.2.0 file://$(git rev-parse --path-format=absolute --git-common-dir) ../record   # LFS récupéré, git status vide
cd ../record && mise trust && mise run rust:link
export BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=…
for target in extract 092 092-1440 settings local; do
  cargo run --profile ci -p compare --bin record -- $target --out DOSSIER   # hors de tests/fixtures/ et des dossiers de référence
done
for name in transit hunt; do
  cargo run --release -p bench --bin batdetect-bench -- tests/fixtures/video_092_original.mp4 --targets $BATDETECT_REFERENCE_DIR/bench/$name/bats.jsonl.gz --out-dir DOSSIER/bench/$name
done
```

## Enregistrement du 17 octobre 2026

Clone du commit 1e4cf6b sur lumiel (Linux x86_64, 12 cœurs) : rustc 1.99.0, FFmpeg 7.1.1 statique (`7.1.1-e864b95369435048`), décodeur en fils automatiques (`frame`, 13).
Les temps ne sont pas publiés : le passage n'était pas sous `verrou chrono`, et les durées que portent les `bench.json` (`seconds`) ne sont pas des mesures.

| Vidéo | Images | Détections brutes | Gardées | Pistes (v0.1.1) | Points | Abîmées (plages / images) | Instables (plages / images) | Fils du décodeur |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| video_092_original_3m24-4m05 | 1230 | 403 | 403 | 4 (4) | 358 | 0 / 0 | 0 / 0 | frame 13 |
| video_092_original | 9011 | 781 | 781 | 14 (14) | 610 | 0 / 0 | 0 / 0 | frame 13 |
| video_092_original_width_1440 | 9011 | 753 | 753 | 14 (14) | 617 | 0 / 0 | 0 / 0 | frame 13 |
| video_089 | 9011 | 4784 | 1817 | 1 (1) | 9 | 0 / 0 | 4 / 248 | frame 13 |
| video_090 | 9011 | 1976 | 1976 | 20 (20) | 401 | 0 / 0 | 0 / 0 | frame 13 |
| video_091 | 9011 | 34504 | 1894 | 21 (21) | 538 | 0 / 0 | 4 / 620 | frame 13 |
| video_092 | 9011 | 781 | 781 | 14 (14) | 610 | 0 / 0 | 0 / 0 | frame 13 |
| video_093 | 9011 | 2667 | 209 | 6 (6) | 83 | 8 / 449 | 1 / 153 | frame 13 |
| video_094 | 9011 | 155189 | 542 | 14 (14) | 204 | 4 / 394 | 8 / 1773 | frame 13 |
| video_095 | 9011 | 4905 | 3122 | 78 (78) | 2688 | 5 / 365 | 1 / 63 | frame 13 |
| video_096 | 9011 | 22500 | 4926 | 136 (136) | 4209 | 8 / 526 | 8 / 581 | frame 13 |
| video_097 | 9011 | 17851 | 6208 | 149 (148) | 5021 | 9 / 535 | 8 / 513 | frame 13 |
| video_098 | 9011 | 15490 | 5512 | 131 (131) | 4681 | 8 / 433 | 7 / 488 | frame 13 |
| video_099 | 9011 | 13214 | 5830 | 117 (115) | 5042 | 8 / 674 | 1 / 69 | frame 13 |
| video_100 | 9011 | 7491 | 6069 | 98 (96) | 5260 | 4 / 217 | 1 / 80 | frame 13 |
| video_101 | 9011 | 5609 | 1782 | 32 (32) | 1226 | 10 / 551 | 3 / 341 | frame 13 |
| video_102 | 9011 | 4364 | 1844 | 36 (36) | 1294 | 11 / 594 | 1 / 68 | frame 13 |
| video_103 | 8998 | 548393 | 1338 | 10 (10) | 258 | 8 / 376 | 16 / 6637 | frame 13 |
| video_120 | 8996 | 14638 | 411 | 0 (0) | 0 | 44 / 2215 | 0 / 0 | frame 13 |
| video_121 | 9011 | 28979 | 1148 | 2 (2) | 12 | 87 / 4912 | 0 / 0 | frame 13 |
| video_122 | 8998 | 31406 | 760 | 0 (0) | 0 | 79 / 6073 | 0 / 0 | frame 13 |
| video_123 | 9011 | 29215 | 499 | 8 (8) | 83 | 66 / 6627 | 0 / 0 | frame 13 |
| video_124 | 9011 | 31264 | 211 | 5 (5) | 104 | 65 / 7458 | 0 / 0 | frame 13 |
| video_125 | 9011 | 1262 | 1262 | 47 (47) | 1069 | 0 / 0 | 0 / 0 | frame 13 |
| video_126 | 9011 | 324 | 324 | 13 (12) | 252 | 0 / 0 | 0 / 0 | frame 13 |
| video_127 | 9011 | 177 | 177 | 7 (7) | 152 | 0 / 0 | 0 / 0 | frame 13 |
| video_128 | 9011 | 4170 | 198 | 5 (5) | 127 | 13 / 1006 | 0 / 0 | frame 13 |
| video_129 | 9011 | 1669 | 551 | 16 (16) | 434 | 3 / 423 | 0 / 0 | frame 13 |
| video_130 | 5241 | 31241 | 155 | 6 (6) | 124 | 4 / 260 | 1 / 526 | frame 13 |

Total des 26 vidéos : 972 pistes (966 pour la v0.1.1).

Les 16 cas de réglages : mêmes dossiers que la v0.1.1, aucun n'a de plage abîmée.
