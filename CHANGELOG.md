# Changelog

Les versions suivent [SemVer](https://semver.org/lang/fr/). Chaque version publiée a sa section
ici ; à partir de #93, la release GitHub en reprendra le texte (`packaging/release-notes.sh`), et
le workflow de release échouera sans elle.

## [Non publiée]

### Ajouté

- Usages courants en tête de l'aide ; `-h` donne l'essentiel, `--help` toutes les options (#56).
- Toutes les entrées sont contrôlées avant le calcul, les refusées listées ensemble ;
  `--skip-refused` traite les vidéos valides sans poser la question (#55).
- Une ligne par étape pendant le calcul, et un bilan de la soirée à la fin (#63, #65).
- Sous macOS, dans un terminal : notification de fin de lot et dossiers de résultats ouverts dans
  le Finder ; `--no-notify` les éteint (#69).
- Image résumé : frise du temps et plages illisibles datées (#127), coupée en parties au-delà
  de 20 pistes (#157). Extraits zoomés en pleine taille (#103).
- Archives du binaire pour macOS arm64, Linux x86_64 (glibc 2.35 au moins) et Windows
  x86_64, avec `SHA256SUMS` et les sources de FFmpeg et d'openh264 jointes à la release.
  Chaque archive est contrôlée avant publication : liens dynamiques, système minimal,
  version et tableau de l'extrait (#223).

### Modifié

- `batdetect` et `batdetect-bench` sont réécrits en Rust, FFmpeg lié en statique (#106). Sur la
  092, la commande prend 19 s au lieu de 135 s. Sur 26 vidéos, 963 des 966 pistes de la 0.1.1 se
  retrouvent, 3 sont perdues et 9 apparaissent ; les écarts sont listés dans docs/11.
- Le tableau des pistes se lit dans un tableur : en français, BOM UTF-8, point-virgule, virgule
  décimale et en-têtes français ; en anglais, virgule et point. Une colonne `verdict` vide suit
  la durée, début et fin s'écrivent `0:00:04`, le début en secondes passe en dernière colonne
  (#84).
- Au recalcul, un tableau qui porte des verdicts, ou qui n'a plus la forme écrite par l'outil,
  est mis de côté tel quel sous un nom daté (`<nom>_pistes_2026-10-16_1430.csv`) et le nouveau
  tableau s'écrit à côté ; le bilan le dit (#83).
- Sans `-o`/`--out-dir`, les résultats vont dans `batdetect/` à côté de chaque vidéo, et non plus
  dans `out/` du dossier de lancement (#86).
- Encodage par VideoToolbox puis openh264 sous macOS, openh264 ailleurs, au lieu de libx264 ;
  ffmpeg et ffprobe ne sont plus nécessaires. Le binaire liera FFmpeg sous LGPL et openh264 (BSD),
  sa licence se décidera à #93.

### Retiré

- Le code Python. Son dernier état est au tag `python-final` (#199).
- Les options `--mode`, `--workers`, `--encoder`, `--crf`, `--vt-quality`, `--box-pad`, `--trail`
  et les réglages fins de détection et de suivi : la commande garde `--threshold`, `--min-hits`,
  `--max-blobs` et `--work-width`, les autres valeurs sont écrites dans `params.json`.

## [0.1.1] - 2026-10-10

### Ajouté

- Aide, erreurs d'arguments, lignes de fin de traitement et bandeau de l'image résumé en français
  sous une locale française (`LANG`, `LC_MESSAGES`, `LC_ALL` ou `LANGUAGE`), en anglais sinon (#57).
- Chaque option de `batdetect --help` a une description et affiche sa valeur par défaut, avec la
  valeur de chaque mode pour les réglages qui en dépendent (#54).

### Modifié

- Les fichiers produits suivent la langue : en français `extraits/piste_01_0m04s.mp4`,
  `<vidéo>_pistes.csv`, `<vidéo>_resume.png`, `<vidéo>_annotee.mp4` ; en anglais `clips/track_01_0m04s.mp4`,
  `<vidéo>_tracks.csv`, `<vidéo>_summary.png`, `<vidéo>_annotated.mp4`. Les anciens noms (`split/`,
  `.tracks.csv`, `.tracks.png`, `_boxes.mp4`) sont effacés au prochain calcul de la vidéo ; le contenu
  du CSV ne change pas (#85).
- Le bandeau de l'image résumé compte des « pistes » et non des « passages » : un passage est une
  piste confirmée par la naturaliste (#87).
- Le temps dans le nom des extraits ne garde plus les centièmes ; le numéro de piste prend trois
  chiffres dès 100 pistes, pour que le Finder les range dans l'ordre (#85).

## [0.1.0] - 2026-10-08

Première version publiée : binaire macOS arm64 (macOS 14 ou plus récent) installé par
`brew install zaratan/bat-tools/batdetect`, sous GPL v3 pour le binaire et MIT pour le code (#30).

- Détection et suivi des chauves-souris dans les vidéos de jumelles thermiques, une image résumé, un
  CSV et un extrait vidéo par piste.
- `--mode normal|quick` pour choisir entre finesse de détection et vitesse (#38).
- Extrait zoomé et ralenti pour les pistes petites ou faibles (`--zoom`, #37).
- `--max-tracks` (300 par défaut) arrête le rendu d'une vidéo qui sort trop de pistes (#3).
- Images abîmées au décodage repérées et sorties du calcul (#1), réglages invalides refusés (#4).

[Non publiée]: https://github.com/zaratan/chiro-dvr/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/zaratan/chiro-dvr/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zaratan/chiro-dvr/releases/tag/v0.1.0
