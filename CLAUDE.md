# batdetect — guide pour Claude

Détection et suivi de chauves-souris dans des vidéos de jumelles thermiques. Python 3.14,
uv, OpenCV, numpy, ffmpeg. Installation et commandes : [README.md](README.md). Base de
connaissances : [docs/](docs/README.md), à lire avant de toucher à un réglage
([docs/02-methode.md](docs/02-methode.md)).

```bash
mise run check                    # lint, format, types, tests : vert avant de rendre la main
uv run batdetect in/video_092_original.mp4 # référence : 5 min, 14 pistes, ~45 s de calcul
```

## Rôles

Manon, naturaliste, confirme les passages à l'œil. Ses confirmations sur la 092 (3:36 et
3:58) sont la vérité terrain du test `slow`. Une question sur ce qui est ou n'est pas une
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
- **Les vidéos se copient depuis les jumelles en USB**, jamais par l'export Stream Vision 2,
  qui divise le débit par 12 ([docs/01-contexte.md](docs/01-contexte.md)).
- **Docs synchronisées** : un changement de comportement met à jour `docs/` dans la même
  série de changements.

## Code

**Un module, une responsabilité, son fichier de tests** (`tests/test_<module>.py`, ou
`test_<paquet>_<module>.py`). Dépendances à sens unique : `track` → `detect` → `video`, `detect` → `median`.
Mieux vaut beaucoup de petits fichiers clairs qu'un gros fichier à plusieurs rôles.

- `video.py` : ouverture, lecture et réduction des images (`open_video`, `read_frames`,
  `to_work_gray`). Seul endroit qui lit la vidéo pour la détection.
- `detect.py` : `DetectConfig`, `Detection`, masques, fond médian et taches
  (`detect_frames`, `find_blobs`). Fonctions pures testables sur des tableaux numpy.
- `median.py` : `temporal_median`, médiane d'une pile d'images par tri par comparaisons,
  identique au bit près à `np.median`, environ 10 fois plus rapide.
- `track.py` : `TrackConfig`, `Track`, appariement, fusion des jumelles
  (`track_detections`). Ne dépend que de `Detection`.
- `output/` : `config.py` (`RenderConfig`), `timefmt.py` (temps et noms d'extraits),
  `overlay.py` (boîtes, trace, interpolation, `track_overlay`), `render.py` (une lecture
  de l'original qui alimente les extraits et la vidéo annotée optionnelle ; au plus
  `MAX_WRITERS` encodeurs à la fois, sinon plusieurs passes), `writer.py` (`FrameWriter`,
  un ffmpeg qui reçoit des images brutes ; supprime le fichier partiel s'il est
  abandonné), `clips.py` (fenêtres d'extraits, répartition en passes), `tables.py`
  (CSV, `params.json`), `encoder.py`
  (moteur multimédia ou x264, choisi au démarrage par un essai d'encodage). Image résumé :
  `summary.py` (assemblage, écriture), `background.py` (fond médian, zones d'affichage masquées), `style.py`
  (palette, tailles selon la largeur), `geometry.py` (longueur d'arc), `colors.py`,
  `placement.py` (pastilles), `arrows.py`, `marker.py`, `legend.py`, `periods.py`
  (tranches de 10 min).
- `jobs.py` (vidéos à traiter, dossiers de sortie), `arguments.py` (options partagées et
  construction des configs), `cli.py` (commande `batdetect`).
- `parallel.py` : détection découpée en tranches de temps, une par processus (`--workers`,
  1 par défaut : plus de tranches ajoutent surtout de la relecture, voir
  [docs/09](docs/09-profilage.md)). Chaque tranche lit la vidéo depuis le début et saute
  jusqu'à elle : jamais `CAP_PROP_POS_FRAMES`, imprécis d'une image en mp4. Le résultat
  doit rester identique au bit près au traitement séquentiel. Seul le test `slow` sur le
  vrai extrait détecte un positionnement imprécis : le lancer après toute modification de
  `parallel.py`. Les processus de détection s'arrêtent si le parent meurt. Dans chaque
  tranche, un fil lecteur (`prefetch.py`, file bornée à `READ_AHEAD` images) décode,
  applique le crochet et réduit pendant que le fil principal détecte ; il est arrêté
  et attendu avant la libération de la vidéo.
- `synthetic/` (`trajectory.py`, `sampling.py`, `injection.py`) : fausses chauves-souris
  injectées en pleine résolution. `bench/` (`config`, `collect`, `cache`, `matching`,
  `metrics`, `stats`, `report`, `cli`) : banc de mesure (`batdetect-bench`,
  [docs/08-banc-de-mesure.md](docs/08-banc-de-mesure.md)).
  Toute amélioration de la détection ou du suivi se mesure au banc avant et après.
- Toutes les coordonnées, distances et surfaces hors de `find_blobs` sont en pixels de la
  vidéo d'origine ; seule la détection travaille à `work_width`.
- Configs en dataclasses gelées qui valident dans `__post_init__` ; pas de `Namespace`
  au-delà de `cli.py` et `arguments.py`.
- Chaque retour de `cv2` passe par `np.asarray(..., dtype=...)` : les stubs d'OpenCV
  sont trop larges pour basedpyright strict.
- Code et identifiants en anglais, docs en français. Zéro commentaire, zéro `noqa` ou
  `type: ignore` : une valeur magique devient une constante nommée.
- Tests sans commentaire, le nom du test porte le pourquoi. Pas de mock d'OpenCV ni de
  subprocess : images fabriquées (`tests/helpers.py`) ou vraie vidéo.
