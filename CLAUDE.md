# batdetect — guide pour Claude

Détection et suivi de chauves-souris dans des vidéos de jumelles thermiques. Python 3.14,
uv, OpenCV, numpy, ffmpeg. Installation et commandes : [README.md](README.md). Base de
connaissances : [docs/](docs/README.md), à lire avant de toucher à un réglage
([docs/02-methode.md](docs/02-methode.md)).

```bash
mise run check                    # lint, format, types, tests : vert avant de rendre la main
uv run batdetect in/video_092_original.mp4 # référence : 5 min, ~4 min 30 de calcul, 14 pistes
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

- `pipeline.py` : lecture vidéo, détection, suivi. Fonctions pures testables sur des
  tableaux numpy, OpenCV isolé dans `open_video`, `read_gray_frames`, `find_blobs`.
- `output.py` : vidéo annotée, PNG, CSV, `params.json`, extraits (ffmpeg en sous-processus).
- `cli.py` : arguments, construction des configs, boucle sur les vidéos.
- Configs en dataclasses gelées qui valident dans `__post_init__` ; pas de `Namespace`
  au-delà de `cli.py`.
- Chaque retour de `cv2` passe par `np.asarray(..., dtype=...)` : les stubs d'OpenCV
  sont trop larges pour basedpyright strict.
- Code et identifiants en anglais, docs en français. Zéro commentaire, zéro `noqa` ou
  `type: ignore` : une valeur magique devient une constante nommée.
- Tests sans commentaire, le nom du test porte le pourquoi. Pas de mock d'OpenCV ni de
  subprocess : images fabriquées (`tests/helpers.py`) ou vraie vidéo.
