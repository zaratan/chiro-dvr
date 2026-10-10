# Changelog

Les versions suivent [SemVer](https://semver.org/lang/fr/). Chaque version publiée a sa section
ici : la release GitHub en reprend le texte, et le workflow de release échoue sans elle.

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

[0.1.1]: https://github.com/zaratan/chiro-dvr/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zaratan/chiro-dvr/releases/tag/v0.1.0
