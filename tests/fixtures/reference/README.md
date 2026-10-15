# Référence v0.1.1

Ce que la v0.1.1 de batdetect (tag `v0.1.1`, commit c9948d2) détecte, enregistré une fois par
`tools/reference_v011.py` (au tag `python-final`), pour que la réécriture en Rust s'y compare sans relancer le Python. Le
script refuse de tourner si `src/` diffère du tag (`git diff --quiet v0.1.1 -- src`).

La référence sert à comparer (aussi bien ou mieux), pas à exiger l'identité : les fichiers vidéo
produits (`.mp4`) sont hors comparaison, et sur les images réparées au décodage OpenCV ne rend pas
toujours les mêmes pixels d'une lecture à l'autre.

## Emplacements

- **Dans le dépôt**, `tests/fixtures/reference/` : les données de l'extrait
  (`video_092_original_3m24-4m05/`) et de la 092 entière (`video_092_original/`), la 092 à 1440
  pixels de travail (`video_092_original_width_1440/`, voir [Réglages](#réglages)), et dans
  `settings/` les 16 cas enregistrés à d'autres réglages de détection (voir
  [Réglages](#réglages)). Dans `zooms/`, la liste triée des noms des `*_zoom.mp4` que la
  v0.1.1 a écrits pour la 097 et la 125 (`video_097.txt`, 74 noms ; `video_125.txt`, 39), tirée
  une fois de `outputs/fr/<vidéo>/extraits/` ; `local_command` la lit à la place de ces dossiers.
  `SHA256SUMS` : voir [Récupération](#récupération-sur-une-autre-machine). Fichiers git
  ordinaires, sans vidéo ni image.
- **Hors dépôt**, dans le dossier nommé par la variable `BATDETECT_REFERENCE_DIR` (sur le Mac de
  l'enregistrement : `~/Projects/dvr-wt/reference-v0.1.1`) :
  - `data/<vidéo>/` : les données des 26 vidéos de `~/dossier sans titre/img_0000/` ;
  - `bench/transit/`, `bench/hunt/` : les deux bancs de la 092 ;
  - `summary/<vidéo>.json` : les attendus de l'image résumé ;
  - `outputs/fr/<vidéo>/`, `outputs/en/<vidéo>/` : les sorties complètes de la commande ;
  - `videos/video_089-091.mp4` : la 089, la 090 et la 091 concaténées sans réencodage ;
  - `logs/` : journal de chaque lancement, avec son temps de calcul.
- **Les 26 vidéos**, dans le dossier nommé par la variable `BATDETECT_VIDEOS_DIR` (sur le Mac
  de l'enregistrement : `~/dossier sans titre/img_0000`).

### Récupération sur une autre machine

Le seau Scaleway `chiro-dvr-videos` (projet `chiro-dvr`), lu par rclone sous le remote
`chiro-dvr` (clé de l'utilisateur dans sa configuration rclone), porte deux préfixes :

- `reference-v0.1.1/` : `data/`, `bench/`, `summary/`, `logs/`, les sorties de l'extrait
  (`outputs/{fr,en}/video_092_original_3m24-4m05/`, lues par `local_outputs`) et un `README.md`
  (provenance, recette pour réenregistrer). Les autres sorties et `videos/` n'y sont pas : aucun
  test ne les lit, et s'ils sont refaits depuis le tag, c'est hors de `BATDETECT_REFERENCE_DIR`,
  que la tâche refuserait ensuite ;
- `img_0000/` : les 26 vidéos.

`SHA256SUMS`, à côté de ce fichier, donne la somme (`shasum -a 256`) de chacun des 199
fichiers de la référence et des 26 vidéos, sous ces deux préfixes ; le `README.md` du seau n'y
est pas, et n'est pas récupéré.

Contrôles du seau après envoi, depuis le Mac de l'enregistrement, par `rclone check` du dossier
local contre le préfixe :

| Préfixe | Source | Date | Résultat |
| --- | --- | --- | --- |
| `img_0000/` (`--include "*.mp4"`) | `~/dossier sans titre/img_0000` | 15 octobre 2026, 20 h 41 | `0 differences found`, `26 matching files` |
| `reference-v0.1.1/` | `~/Projects/dvr-wt/seau/reference-v0.1.1` | 16 octobre 2026, 0 h 26 | `0 differences found`, `200 matching files` |

```bash
BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=… mise run rust:data
```

Les deux variables sont obligatoires et leurs chemins absolus, comme les lisent les tests.
Avant tout téléchargement, la tâche refuse un dossier qui recouvre `in/` ou
`~/dossier sans titre/img_0000` (comparés comme fichiers, quelle que soit la casse écrite), les
deux variables sur le même dossier, un dossier de référence qui contient un fichier absent de
`SHA256SUMS` et un dossier de vidéos qui contient un mp4 absent de `SHA256SUMS` (`.DS_Store` et
`._*` ignorés), ou un mp4 au nom d'une des 26 vidéos mais à la somme différente : ce peut être
une vidéo d'une autre soirée, les jumelles reprenant leur numérotation, et la tâche n'écrit
jamais par-dessus. Un `.partial` laissé par rclone n'est pas concerné. Elle copie ensuite par
`rclone copy` les seuls fichiers nommés dans `SHA256SUMS` ; rclone ne télécharge que ce qui
manque ou diffère. Puis elle vérifie chaque somme et s'arrête sur la
première différence, avec le nom des fichiers. Un fichier abîmé se supprime à la main, puis la
tâche se relance. Elle finit par une ligne de bilan par emplacement. Relancée
aussitôt, elle ne télécharge rien.

## Conventions

- **Numéros d'image** : à partir de 0, dans l'ordre où le décodeur rend les images (ordre
  d'affichage), les mêmes pour OpenCV et ffprobe (le script échoue si les deux ne comptent pas le
  même nombre d'images).
- **Plages** : `[première, dernière]`, bornes comprises.
- **Coordonnées, tailles et surfaces** en pixels de la vidéo d'origine.
- **Flottants** écrits par `repr` de Python : la plus courte écriture qui redonne le même `f64`.
  Les relire sans perte (`serde_json` avec `float_roundtrip`, ou `str::parse::<f64>`) et comparer
  des nombres, pas des octets. Un flottant entier s'écrit `4.0` ; les réglages gardent leur type
  Python (`"threshold": 12`).
- **JSON Lines** : un objet JSON par ligne, UTF-8, fin de ligne `\n`. Les fichiers `.gz` sont des
  gzip sans nom ni date (`mtime` 0) ; comparer leur contenu décompressé, les octets compressés
  dépendent de la version de zlib.

## Données d'une vidéo

Quatre fichiers par dossier.

### `video.json`

| Champ | Contenu |
| --- | --- |
| `format_version` | 1 |
| `reference` | `v0.1.1` |
| `video` | `name` (nom du fichier), `bytes`, `sha256` |
| `stream` | en-tête du flux vu par ffprobe : `width`, `height`, `avg_frame_rate`, `r_frame_rate`, `time_base` (chaînes rationnelles telles que ffprobe les écrit), `nb_frames` (chaîne aussi, peut manquer selon le conteneur) |
| `opencv` | `fps` (`CAP_PROP_FPS`), `frame_count_property` (`CAP_PROP_FRAME_COUNT`), `frames_read` (images réellement lues), `width`, `height`, `work_width`, `work_height` (taille de détection) |
| `ffprobe` | `frame_count`, `error_frames` (images avec un message d'erreur de décodage), `gap_frames` (images après un saut de `pts`), `decode_messages` (`frame` et `messages` de chaque image en erreur, sans l'adresse mémoire du décodeur, `[h264 @ 0x…]` devenu `[h264]`, qui change à chaque lancement) |
| `background_half_window` | demi-fenêtre du fond en images, marge des plages abîmées |
| `damaged_spans` | plages abîmées (`damaged_spans` de `damage.py`) |
| `unstable_spans` | plages instables (`unstable_spans` de `stability.py`, calculées hors plages abîmées) |
| `counts` | `frames`, `detections` (brutes), `detections_kept` (après exclusion), `damaged_frames_reported` (images d'erreur ou de saut), `tracks`, `track_points` |
| `config` | `DetectConfig`, `TrackConfig`, `StabilityConfig` : les réglages par défaut de la v0.1.1, sauf `DetectConfig` dans `settings/` et `video_092_original_width_1440/`, qui porte ceux de `--detect` |
| `versions` | `batdetect`, `python`, `numpy`, `opencv`, `opencv_ffmpeg` (bibliothèques FFmpeg d'OpenCV), `ffprobe`, `platform` |
| `files` | noms des trois autres fichiers |

### `frames.jsonl` (`frames.jsonl.gz` pour les 26 vidéos)

Une ligne par image, **images vides comprises** :

```json
{"frame":0,"pts":0,"key_frame":true,"decode_error":false,"detections":[{"x":1400.5,"y":246.25,"left":1399.5,"top":246.0,"width":3.0,"height":1.5,"area":4.5,"amplitude":13.370320638020829},{"x":1435.0,"y":1057.75,"left":1434.0,"top":1057.5,"width":3.0,"height":1.5,"area":4.5,"amplitude":12.623766581217453}]}
{"frame":1,"pts":333,"key_frame":false,"decode_error":false,"detections":[]}
```

- `pts` : entier dans la base de temps du flux, `null` si ffprobe n'en donne pas ;
- `key_frame`, `decode_error` : vus par ffprobe, lancé avec les arguments de `probe.py`
  (`-threads 1`, `-show_log 16`) ;
- `detections` : les détections **brutes**, avant l'exclusion des plages abîmées et instables,
  **dans l'ordre** où `find_blobs` (`detect.py` de la v0.1.1) les rend :
  - `x`, `y` : barycentre de la composante après fermeture (`merge_radius`), ramené à l'image
    d'origine par `(c + 0,5) × échelle − 0,5` ;
  - `left`, `top`, `width`, `height` : boîte de cette composante, multipliée par l'échelle ;
  - `area` : nombre de pixels au-dessus du seuil, multiplié par le carré de l'échelle (ni la
    boîte, ni la composante fermée) ;
  - `amplitude` : plus grand écart au fond (`|résidu|`) de ces pixels, en niveaux de gris de
    l'image réduite et filtrée (`target_blur`), corrigée du niveau moyen.

### `tracks.jsonl`

Une ligne par piste, dans l'ordre et avec les numéros de la v0.1.1, avec **tous ses points**
(mêmes champs qu'une détection, plus `frame`). Quand deux pistes jumelles sont fusionnées, leurs
points d'une même image sont combinés en un seul (`combine` de `track.py`), qui n'est **pas** dans
`frames.jsonl` : 17 points sur 358 dans l'extrait, 63 sur 610 dans la 092.

### `tracks.csv`

Le CSV des pistes écrit par `write_tracks_csv`. La v0.1.1 l'écrit avec des fins de ligne CRLF ;
les copies du dépôt sont en LF, git les ayant normalisées au commit. Les valeurs sont les mêmes :
comparer le CSV par ses valeurs, pas à l'octet.

## Bancs

`bench/transit/` : banc par défaut de la 092, graine 1 (9 classes × 30 cibles). `bench/hunt/` :
graine 1, `--motions hunt circle --amplitudes -60 -30 --per-class 60` (12 classes × 60).

- `bench.json` : le rapport de `batdetect-bench`, avec `code` mis à `v0.1.1` et `video` réduit au
  nom du fichier.
- `bench-run.json` : arguments, nombres, versions, sommes.
- `bats.jsonl.gz` : une ligne par cible tirée : `id`, `motion`, `amplitude`, `sigma`,
  `start_frame`, `positions` (`[x, y]` par image depuis `start_frame`), `amplitudes` (amplitude de
  chaque image, battements compris), `effective` (`[image, contraste effectif]` mesuré sur la vraie
  image après écrêtage), `patch`.
- `patch` : le patch de gris de la cible à l'image du milieu de sa trajectoire, injecté par
  `inject` dans une image uniforme de niveau `base` (128, sans écrêtage pour une amplitude
  inférieure à 128 en valeur absolue) : `frame`, `left`, `top` (coin du patch dans l'image
  d'origine), `values` (lignes de différences entières au niveau de base). `null` si la cible est
  hors champ à cette image. La conversion en `uint8` de `inject` sur un fond entier donne un
  **plancher** : `values = ⌊amplitude_at(frame) · exp(−d² / 2σ²)⌋`, arrondi vers −∞ (une amplitude
  négative donne −1 jusqu'au bord, jamais 0), sur la fenêtre `[⌊x⌋ − ⌈3σ⌉, ⌊x⌋ + ⌈3σ⌉]` de
  chaque axe, bornée à l'image.
- `injected.jsonl.gz` : une ligne par image du run injecté, `frame` et `detections` comme
  `frames.jsonl`. Le run de référence du banc n'est pas réécrit : ses détections sont celles de
  `video_092_original/frames.jsonl`, vérifié à l'enregistrement pour les deux bancs.

Le tirage (`SeedSequence.spawn`, `default_rng`) ne se reproduit pas en Rust : rejouer les cibles
de `bats.jsonl.gz`.

En Rust (#100), `reference::bench` lit et écrit `bats.jsonl.gz` (`TargetLine`), les lignes
`{frame, detections}` d'`injected.jsonl.gz` et `bench.json` (`BenchReport`) ;
`reference::location::recorded_bench(nom)` donne `$BATDETECT_REFERENCE_DIR/bench/<nom>/`. Les tests
`local_injection`, `local_metrics` et `local_guard` de `bench` lisent les deux bancs, et
`batdetect-bench VIDÉO --against DOSSIER` rejoue un banc pour la garde (docs/11, « Banc de
mesure »). `bench-run.json` n'est pas relu.

## Sorties complètes

Commande `batdetect` par défaut (encodeur VideoToolbox), une fois en français
(`LANG=fr_FR.UTF-8`) dans `outputs/fr/`, une fois en anglais (sans variable de langue) dans
`outputs/en/` : extrait, 092, 097, 125 et `video_089-091`. Les extraits, l'image résumé, le CSV et
`params.json` servent de modèle ; les `.mp4` ne se comparent pas octet par octet (VideoToolbox
n'est pas déterministe), et `params.json` garde le chemin de la vidéo tel qu'il a été passé à
la commande.

## Relancer

Seulement depuis un checkout du tag `python-final` (un worktree, par exemple) : le script et le
Python ont quitté `main` à #199, et le script exige un `src/` identique à `v0.1.1`, ce qui est le
cas à ce tag. L'environnement s'installe comme le dit le `README.md` du dépôt à ce tag
(section « Depuis les sources ») ; les deux commandes, `data` et `bench`, sont dans la section
« Relancer » de ce fichier-ci (`tests/fixtures/reference/README.md`) à ce tag.

`bench` refuse un dossier de cache qui contient déjà la vidéo : la clé du cache du banc ignore les
réglages de suivi, qui guident pourtant le placement des cibles.

`--detect` remplace les champs de `DetectConfig` qu'il nomme, par un objet JSON
(`'{"threshold": 25}'`) ; `osd_regions` s'y écrit en listes de quatre bornes
(`[[0, 0, 1, 0.07]]`). Un champ inconnu ou une valeur que la v0.1.1 refuse arrête le script avant
toute détection. Suivi et stabilité restent aux défauts.

## Réglages

`settings/<cas>/` : 16 enregistrements de la v0.1.1 à d'autres réglages de détection (#149),
sur l'extrait et sur sa réduction en 720×540 (`video_092_original_3m24-4m05_720x540.mp4`, sous
LFS), avec les mêmes versions que ci-dessous. Mêmes quatre fichiers par dossier, `frames.jsonl.gz`
compressé, 1,6 Mo en tout. Liste des cas, réglages, commande de la 720×540 et chiffres :
[docs/11](../../../docs/11-migration-rust.md#réglages).

`video_092_original_width_1440/` : la 092 entière à `--detect '{"work_width": 1440}'` (#151),
enregistrée le 12 octobre 2026 sous le verrou, mêmes versions, mêmes quatre fichiers
(`frames.jsonl.gz` compressé, 0,2 Mo en tout). 751 détections, 14 pistes. La v0.1.1 y a pris
277 s réelles, 521 s de CPU et 0,54 Go au plus (`/usr/bin/time -l`), contre 0,53 Go pour
`width_1440` sur l'extrait. `compare 092-1440` la relit.

## Enregistrement du 10 octobre 2026

Code au tag `v0.1.1` (c9948d2), sur un Mac arm64 (macOS 27.0) : Python 3.14.8, numpy 2.5.3,
OpenCV 5.0.0 (avcodec 61.19.101, avformat 61.7.100, avutil 59.39.100, swscale 8.3.100),
ffprobe 9.0.2 de Homebrew. Chaque `video.json` porte ces versions.

Relancé sur l'extrait et la 092 : les quatre fichiers sont identiques à l'octet près. Relancé
sur la 103 (images réparées, 10 erreurs de décodage, 2 sauts de `pts`) : les quatre fichiers sont
identiques aussi, détections des 8 998 images comprises, plages abîmées incluses. La 092 des 26 vidéos (`data/video_092/`) est le même fichier que la 092 du dépôt :
données identiques.

La concaténation `videos/video_089-091.mp4` (1 383 717 488 octets, sha256
`74efde9d353ae2602f3f212e48ee8a9e42c625c96721684e0c491fd3ad3d5575`) est faite sans réencodage par
le démultiplexeur `concat` de ffmpeg, vidéo seule (`-map 0:v:0 -c copy`), avec la durée vidéo de
chaque fichier (`duration 300.0663`, 9 011 images de 333/10 000 s). Sans cette durée, ffmpeg décale
chaque fichier de la durée du conteneur, plus longue à cause de l'audio, et laisse un saut de
`pts` à chaque jonction. Vérifiée par ffprobe : 27 033 images, pas de `pts` constant de 333, aucune
erreur, images-clés aux deux jonctions (9 011 et 18 022). Son `avg_frame_rate` vaut
1126375/37509, d'où un `CAP_PROP_FPS` de 30,0295 au lieu de 30,03.

### Vidéos et sommes

Les 26 vidéos font 11 565 045 370 octets.

| Vidéo | Octets | sha256 |
| --- | ---: | --- |
| video_092_original_3m24-4m05.mp4 | 60240812 | `ff70c89fc9a8c0d6ae6728c1a5684287149433831ab75be2e7c59de3c4ebee17` |
| video_092_original.mp4 | 454104980 | `64d692966a662933d45d37f9c8de3b852765a93f17adafb5548b8b155b0a0af9` |
| video_089.mp4 | 467164569 | `d340c737fb6b11cc2448bd3f64839a30f80c2777c20399a25d6a7ddbc82dffd5` |
| video_090.mp4 | 466531225 | `ba3e001b89b800d46769105fe84efc9bb504dfdf600d4f7a9432b34471efcfb0` |
| video_091.mp4 | 451092465 | `a5611dd7620210958381c0472e30123500fd4b5f0e201e987949b73c2053b85e` |
| video_092.mp4 | 454104980 | `64d692966a662933d45d37f9c8de3b852765a93f17adafb5548b8b155b0a0af9` |
| video_093.mp4 | 457984457 | `b74dfdf04e11119cbd71df36848f05a5a796b0270e6a49d16437a737b49e74c6` |
| video_094.mp4 | 461239090 | `87693bda1f30ebc42cc901f6d8eb97663d63def924eb69371e3d1f0e6c033e4a` |
| video_095.mp4 | 447669802 | `0fb4a523a82196dbd630c22f0c76b59315d87ef7446fefaa67faba58e2f3bd68` |
| video_096.mp4 | 443195749 | `730808c15b42226a23467c34e3045b02692ef83486dcb526470f083aef848dd1` |
| video_097.mp4 | 434382065 | `f96b1f41632cfdbdccc798050309ae23634027a980e871995fd534459c3f3f19` |
| video_098.mp4 | 450091569 | `5c54514a29fe3654b0cea4e513a437ca222048ebfa0c15b69637254848ec484d` |
| video_099.mp4 | 474772642 | `dd216392af228664aebb42a63585eae9d492e604a37d094f5601d9e03c596050` |
| video_100.mp4 | 446657024 | `808c757720b0375dc1a76ffd11c63e17ba1d09b8a46c40a0ff12e97b59ce4b9c` |
| video_101.mp4 | 449503751 | `9a7486801bc68d2964b62fcac236dc82cc34cb264265098086064e463a036074` |
| video_102.mp4 | 460728653 | `5fc3765c21502c79ba377c2b1ab04f5645a41d5ac9780b6c91e924a59bfb3646` |
| video_103.mp4 | 463283902 | `72652e0751d68538bb6d0a74ab99b4dacf18eaf2c053146a1bbf3480c8a108d7` |
| video_120.mp4 | 444944696 | `11c4c2277e31d9fad1e86408c8dee7c587819296d9f16856be619e2fdb077e95` |
| video_121.mp4 | 468700455 | `4ddb4afc9bf32f319d60373f1b36766b79a0145e31d84230d410bdd098e5566b` |
| video_122.mp4 | 449841923 | `3e0cef6979ed0bb69d314cfea9cf711798d29bb184ac6ccbba390993b54b75f7` |
| video_123.mp4 | 447891000 | `748d046e8a11d5f0336eeec7775a574e2effe9e4b66f88fc25a93f897c2db11b` |
| video_124.mp4 | 457638457 | `503d1ebeef11a7571bb704c123f331394d6a86209291db4e70e1b838dd23cd6a` |
| video_125.mp4 | 457975195 | `7ea5c5cb99e2d123b4e6c40a40c934c96b1586f9b0bd4db63685df445a1fb605` |
| video_126.mp4 | 465163753 | `1b073bbb57afef779e7699ea4498f344a05371bc13caa81bf2a4dc5e069c7434` |
| video_127.mp4 | 437283145 | `92b35c9eaabf2d5cbd6a71a2855d85b520c9d95f288413e3882aad57e79da503` |
| video_128.mp4 | 432475884 | `32a0b94683b7e19e80d9996c5f49fd06f202cf471cffca380d01d08aa2297bf9` |
| video_129.mp4 | 426511175 | `9c2e2a37e75a35cfc720c90cc57cc77b578c182ba91511267557fa8280fb2861` |
| video_130.mp4 | 248217744 | `1748593d8e6cff8f61d9cb83ad63564a38976de7fb349932d1309fe922c7fe74` |

### Ce que la v0.1.1 en tire

Réglages par défaut, un processus. Plages et images comptées en images ; temps de la commande `data` sous `/usr/bin/time -l` (CPU = utilisateur + système). Total des 26 vidéos : 966 pistes.

| Vidéo | Images | Détections brutes | Gardées | Pistes | Points | Abîmées (plages / images) | Instables (plages / images) | Temps réel | Temps CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| video_092_original_3m24-4m05 | 1230 | 402 | 402 | 4 | 358 | 0 / 0 | 0 / 0 | 18 s | 52 s |
| video_092_original | 9011 | 780 | 780 | 14 | 610 | 0 / 0 | 0 / 0 | 132 s | 380 s |
| video_089 | 9011 | 4791 | 1822 | 1 | 9 | 0 / 0 | 4 / 248 | 129 s | 385 s |
| video_090 | 9011 | 1972 | 1972 | 20 | 402 | 0 / 0 | 0 / 0 | 130 s | 387 s |
| video_091 | 9011 | 34524 | 1905 | 21 | 540 | 0 / 0 | 4 / 620 | 130 s | 385 s |
| video_092 | 9011 | 780 | 780 | 14 | 610 | 0 / 0 | 0 / 0 | 122 s | 375 s |
| video_093 | 9011 | 2611 | 210 | 6 | 83 | 8 / 419 | 1 / 153 | 121 s | 374 s |
| video_094 | 9011 | 155182 | 541 | 14 | 208 | 4 / 394 | 8 / 1773 | 131 s | 386 s |
| video_095 | 9011 | 5111 | 3119 | 78 | 2686 | 5 / 365 | 1 / 63 | 126 s | 380 s |
| video_096 | 9011 | 22603 | 4926 | 136 | 4209 | 8 / 526 | 8 / 581 | 125 s | 379 s |
| video_097 | 9011 | 17678 | 6211 | 148 | 5018 | 9 / 521 | 8 / 513 | 124 s | 378 s |
| video_098 | 9011 | 15428 | 5513 | 131 | 4697 | 8 / 433 | 7 / 488 | 124 s | 379 s |
| video_099 | 9011 | 13518 | 5833 | 115 | 5036 | 8 / 674 | 1 / 69 | 124 s | 385 s |
| video_100 | 9011 | 7430 | 6075 | 96 | 5239 | 4 / 217 | 1 / 80 | 125 s | 380 s |
| video_101 | 9011 | 5619 | 1785 | 32 | 1226 | 10 / 551 | 3 / 342 | 122 s | 380 s |
| video_102 | 9011 | 4398 | 1851 | 36 | 1295 | 11 / 594 | 1 / 68 | 123 s | 384 s |
| video_103 | 8998 | 548329 | 1344 | 10 | 258 | 8 / 376 | 16 / 6637 | 179 s | 432 s |
| video_120 | 8996 | 15633 | 410 | 0 | 0 | 44 / 2185 | 0 / 0 | 121 s | 369 s |
| video_121 | 9011 | 28596 | 1146 | 2 | 12 | 87 / 4912 | 0 / 0 | 125 s | 382 s |
| video_122 | 8998 | 30496 | 762 | 0 | 0 | 79 / 6043 | 0 / 0 | 127 s | 381 s |
| video_123 | 9011 | 28811 | 502 | 8 | 83 | 66 / 6494 | 0 / 0 | 126 s | 382 s |
| video_124 | 9011 | 30750 | 210 | 5 | 104 | 64 / 7415 | 1 / 71 | 125 s | 384 s |
| video_125 | 9011 | 1263 | 1263 | 47 | 1069 | 0 / 0 | 0 / 0 | 126 s | 383 s |
| video_126 | 9011 | 323 | 323 | 12 | 245 | 0 / 0 | 0 / 0 | 120 s | 379 s |
| video_127 | 9011 | 176 | 176 | 7 | 151 | 0 / 0 | 0 / 0 | 127 s | 379 s |
| video_128 | 9011 | 3655 | 199 | 5 | 127 | 13 / 991 | 0 / 0 | 122 s | 376 s |
| video_129 | 9011 | 1629 | 551 | 16 | 434 | 3 / 423 | 0 / 0 | 120 s | 373 s |
| video_130 | 5241 | 31282 | 155 | 6 | 124 | 4 / 260 | 1 / 526 | 74 s | 222 s |

### Sorties et bancs

| Lancement | Pistes (run injecté pour un banc) | Extraits (dont zooms), ou cibles | Temps réel fr / en | Temps CPU fr / en |
| --- | ---: | ---: | ---: | ---: |
| sorties video_092_original_3m24-4m05 | 4 | 4 (0) | 22 s / 21 s | 68 s / 67 s |
| sorties video_092_original | 14 | 15 (1) | 135 s / 134 s | 437 s / 433 s |
| sorties video_097 | 148 | 222 (74) | 234 s / 232 s | 625 s / 624 s |
| sorties video_125 | 47 | 86 (39) | 166 s / 165 s | 518 s / 516 s |
| sorties video_089-091 | 42 | 53 (11) | 427 s / 428 s | 1433 s / 1437 s |
| banc transit | 163 | 253 cibles évaluées, 148 trouvées | 243 s | 689 s |
| banc hunt | 515 | 584 cibles évaluées, 456 trouvées | 244 s | 691 s |
