# Changelog

Les versions suivent [SemVer](https://semver.org/lang/fr/). Chaque version publiée a sa section ici.
La release GitHub en reprend le texte par `packaging/release-notes.sh`. Sans cette section, le workflow de release échoue.

## [0.3.0] - 2026-10-18

### Ajouté

- Chaque entrée refusée au départ, et chaque dossier de résultats où l'on ne peut pas écrire, est suivi d'une ligne « à faire », comme une vidéo en échec (#239, #82).
- Une ligne annonce la soirée avant la première vidéo : nombre de vidéos, durée totale, et combien sont déjà faites (#62).
- La place libre du disque des résultats est contrôlée au lancement, sur une estimation de 1 fois la taille des vidéos (2,8 fois de plus avec `--annotated`) et une marge ; un disque trop petit est refusé avant le calcul, avec la même question que les entrées refusées, et `--ignore-disk` passe outre.
  Chaque vidéo vérifie encore la place avant ses extraits et échoue en « disque plein » plutôt que de remplir le disque.
  Sous Windows, la place n'est pas contrôlée (#61).
- La page de résultats montre les entrées refusées au départ du dernier lancement, avant les vidéos, avec la raison et ce qu'il faut faire, comme le bilan (#267).
- Les verdicts se notent dans la page de résultats : un clic sur une piste ouvre son extrait en grand, en boucle, avec « Oui », « Non », « Doute » et l'espèce dessous, au clavier ou à la souris ; « Enchaîner » passe d'une piste à la suivante.
  La page les télécharge dans `verdicts.csv`, à glisser à côté d'elle ; elle le relit au calcul suivant, et « Recharger un fichier de verdicts… » relit un fichier choisi.
  Chaque verdict retrouve sa piste par son début et son point de départ, même renumérotée ; un verdict sans piste reste montré et gardé.
  Le navigateur garde les verdicts non téléchargés (#228).
- Chaque dossier de vidéo a un fond, la scène seule sans aucune trace (`<vidéo>_fond.png`, `<vidéo>_background.png` en anglais), et `params.json` donne la mise en page de chaque image résumé (`background`, `summaries`) et la trace de chaque piste dessinée (`trace`).
  La page de résultats redessine l'image sur ce fond, trajectoires, flèches et pastilles aux places de l'image résumé, avec au-dessus l'intervalle, le nombre de pistes et les périodes hors analyse et illisibles, et dessous la frise du temps ; la vignette de la zone de jugement ne garde que la pastille de la piste jugée.
  L'image résumé ne change pas : c'est le fichier qu'on ouvre seul ou qu'on envoie, par le lien « Ouvrir l'image complète ».
  Une vidéo calculée avant garde son image résumé dans la page (#259).
- Dans la page de résultats, des boutons « Toutes · À voir · Oui · Non · Doute », avec leur nombre, et « Autre » quand une piste est notée d'un autre mot, choisissent les pistes montrées sur l'image, la vignette du lecteur, la frise et la liste ; sur la grande image, chaque piste jugée porte un petit signe près de sa pastille (coche verte, croix rouge, « ? »), et les « non » sont grisés ; pendant le jugement, ‹ et › ne passent que par les pistes montrées.
  Un clic sur un début de piste de la frise ouvre le lecteur sur cette piste (#268).
- Dans la page de résultats, un clic sur une trajectoire de l'image, dans la grande image comme dans la vignette du lecteur, en fait la piste jugée et charge son extrait ; le survol la fait ressortir.
  Quand deux trajectoires se touchent, le clic prend la plus proche, un second clic au même endroit l'autre.
  Au clavier, Tab mène aux pastilles de l'image, les flèches passent d'une piste à l'autre et Entrée ouvre le lecteur (#269).
- `batdetect doctor` vérifie, avant une soirée, l'encodeur, la place libre et le droit d'écrire dans le dossier courant, avec quoi faire pour chaque défaut (#58).
- `batdetect completion zsh` écrit la complétion de la touche Tab : options, valeurs de `--zoom`, vidéos et dossiers, et les commandes `doctor` et `completion` au premier mot (#59).
- Pendant le calcul, une fois la première vidéo calculée, la ligne d'étape donne l'heure de fin estimée de la soirée (« fin vers 22:41 »), d'après le temps moyen par image des vidéos déjà calculées, et la corrige à chaque étape ; hors d'un terminal, l'heure suit la ligne de fin de chaque vidéo.
  Sur un terminal sous macOS et Linux, la ligne d'étape est coupée à la largeur de la fenêtre (#68).

### Modifié

- Un bout de vol trop court pour faire une piste, aperçu entre deux passages derrière la végétation, relie maintenant les deux pistes du même vol, ou prolonge une piste au début ou à la fin, s'il va à peu près à la même vitesse ; seul, il ne devient jamais une piste.
  Deux pistes dont la seconde repart presque là où la première s'est arrêtée, bien plus lentement que l'animal ne volait, ne sont plus réunies (#311).
- Une piste ne saute plus sur une autre chauve-souris trop loin pour sa vitesse : elle ne prend une tache qu'à 50 px de l'endroit prévu, plus le chemin qu'elle devait parcourir, sauf près du bord de l'image, où l'animal qui entre est encore coupé.
  Sur les 26 vidéos de mesure, deux chauves-souris ne donnent plus trois pistes à la 090, une piste de la 095 ne finit plus sur l'animal voisin, et 27 autres endroits changent, jugés à l'œil ; la vidéo de référence garde ses 14 pistes.
  Deux réglages de plus dans `params.json`, `reach_margin` et `reach_travel` (#326).
- Une tache un peu trop faible pour le seuil, au-dessus des trois quarts, comble le trou d'une piste quand une vraie tache la précède et la suit de près, sur le même chemin : les pistes rapides ont un cinquième de positions estimées en moins (mesuré sur 26 vidéos), et quelques pistes de plus apparaissent.
  Une tache faible n'ouvre jamais de piste et ne passe jamais avant une vraie tache, pour ne pas réunir deux chauves-souris voisines (#21).
- La détection prend environ 10 % de temps en moins sur un Mac à puce Apple et 4 % sous Linux, sans changer ses résultats (#214).
- Dans la page de résultats, le début d'une piste s'écrit partout comme dans le tableau des pistes (« 0:00:51 ») : la liste et le survol de la frise ne l'écrivent plus au centième (#304).
- Le tableau des pistes n'a plus de colonne `verdict` et appartient à l'outil : un recalcul le remplace quoi qu'il contienne.
  La mise de côté datée d'un tableau annoté (#83) est retirée, ainsi que la section « Tableau mis de côté » du bilan ; les tableaux déjà mis de côté restent où ils sont (#228).
- Les extraits, les zooms et la vidéo annotée dessinent chaque piste de la couleur qu'elle a sur l'image résumé : boîte et numéro bordés d'un liseré noir, trace de la même couleur, au lieu de la boîte rouge et de la trace jaune orangé (#13).
- Une vidéo arrêtée pour trop de pistes n'a plus ni image résumé ni fond, seulement son tableau : son « à faire » dit d'ouvrir le tableau et de regarder la vidéo à quelques-uns de ses temps (#221).
- « Réglages changés » nomme toutes les options qui font recalculer une vidéo, dont `--zoom`, `--annotated`, `--max-tracks` et `--clip-margin`, dans l'ordre de l'aide ; en français, la valeur s'écrit avec une virgule (« --threshold 11,5 ») (#221, #247).
- Une zone `--osd-region` refusée est nommée par son numéro : « la zone n° 2 de --osd-region ne vérifie pas … » (#221).
- Une entrée refusée est nommée par le dernier nom de son chemin (« nuit-13-oct : introuvable ») ; le chemin entier passe dans le « à faire » quand il sert à la retrouver : introuvable, lecture refusée, dossier sans vidéo.
  La page des résultats fait de même (#247).
- Deux vidéos du lancement qui ont le même nom sans extension sont nommées avec leur extension sur les lignes au fil de l'eau (« video_092.mov », « video_092.mp4 ») ; si le nom est le même extension comprise, dans deux dossiers, avec leur dossier (« nuit-12/video_001.mp4 », « nuit-13/video_001.mp4 ») (#247).
- Le « à faire » d'un disque plein nomme le disque (« libérer de la place sur le disque qui contient … ») au lieu du dossier (#221).
- Chaque vidéo est ouverte avant tout calcul : une vidéo illisible ou mal copiée est refusée au départ avec les autres entrées fautives, au lieu d'échouer à son tour.
  Hors d'un terminal, depuis un script par exemple, une seule vidéo mal copiée arrête donc tout le lot avant le calcul, en code 2, sauf avec `--skip-refused` ; avant, les autres vidéos étaient faites et le code valait 1 (#62).

### Corrigé

- Sous macOS, un VideoToolbox qui ne répond pas ne bloque plus le calcul sans fin : après 10 s d'attente, les extraits s'écrivent avec openh264 jusqu'à la fin du lancement, et une ligne le dit une fois (#313).
- Les fichiers cachés d'un dossier, comme les `._x.mp4` que macOS dépose sur une clé exFAT, ne sont plus pris pour des vidéos (#204).
- Sous macOS, le premier calcul n'échoue plus quand le dossier de résultats est sur une clé exFAT (#236).
- Deux lancements sur le même dossier, ou un Ctrl-C pendant l'écriture de la page de résultats, ne mélangent plus deux versions d'un même fichier (#245).
- Dans le total de la page de résultats, les vidéos calculées par une version plus ancienne de l'outil sont nommées à part (« 4 vidéos · 3 réussies · 1 d'une ancienne version ») au lieu d'être seulement comptées (#247).
- Après un Ctrl-C, les vidéos du lot pas encore commencées restent sur la page de résultats, marquées « pas encore traitée » avec ce qu'il faut faire, et la vidéo en cours y figure, « inachevée » quand rien d'un calcul précédent n'y est gardé ; avant, elles n'y apparaissaient pas.
  Un Ctrl-C pendant la première vidéo laisse aussi la page de résultats, qui montre cette vidéo « inachevée » (#247).
- Dans la page de résultats, une vidéo sans extrait ni image résumé n'ouvre plus de lecteur vide : ses pistes sont listées avec leur numéro, leur début et leur durée seulement, et ne comptent pas dans « à voir » ni dans « jugées » (#247).
- Sous Linux, une vidéo dont le nom n'est pas en UTF-8 est traitée, et une relance la reconnaît comme déjà faite.
  Dans la page de résultats, son tableau et ses verdicts s'affichent, mais ni son image résumé ni sa vidéo annotée, et les liens vers son tableau, ses extraits et son dossier sont cassés (#227).
- Dans la page de résultats, une trajectoire de la vignette du lecteur ressort au survol, comme sur la grande image : les autres pâlissent (#336).
- Dans la page de résultats, un lecteur d'écran lit le verdict d'une pastille de l'image en clair (« jugée chauve-souris », « jugée pas une chauve-souris », « doute », « notée … », « à voir ») au lieu de « Oui » ou « Non » seul (#337).
- Dans la page de résultats, un lecteur d'écran lit les temps en mots (« 3 min 27 s ») au lieu de « 0:03:27 », et une période « de 12 s à 31 s » : pastilles de l'image, colonne « début », ligne au-dessus de l'image, lecteur et verdicts sans piste ; l'affichage ne change pas, et copier une ligne ne colle que ce qui est affiché.
  Au-delà d'une heure, une durée s'écrit « 1 h 2 min 5 s », sur la page comme sur l'image résumé, et non plus « 62 min 5 s » (#338).

## [0.2.0] - 2026-10-16

### Ajouté

- Usages courants en tête de l'aide ; `-h` donne l'essentiel, `--help` toutes les options (#56).
- Toutes les entrées sont contrôlées avant le calcul, les refusées listées ensemble ; `--skip-refused` traite les vidéos valides sans poser la question (#55).
- Une ligne par étape pendant le calcul, et un bilan de la soirée à la fin (#63, #65).
- Sous macOS, dans un terminal : notification de fin de lot et page des résultats ouverte dans le navigateur ; `--no-notify` les éteint (#69, #71).
- Une page `resultats.html` (`results.html` en anglais) à la racine de chaque dossier de résultats, réécrite à chaque fin de vidéo : total, une section par vidéo, soirées précédentes comprises, image résumé, liste des pistes avec leur verdict et leurs extraits lus dans la page, vidéos en échec avec leur raison.
  Hors ligne, sans rien écrire ; une page du même nom qui n'est pas la nôtre n'est jamais remplacée (#71).
- Relancer la même commande saute les vidéos déjà faites avec la même version, la même langue et les mêmes réglages, sans toucher à leur dossier ; le bilan les compte « déjà faites ».
  `--force` refait tout (#70).
- Image résumé : frise du temps et plages illisibles datées (#127), coupée en parties au-delà de 20 pistes (#157).
  Extraits zoomés en pleine taille (#103).
- Documentation pour la naturaliste : `docs/premiere-soiree.md` (des jumelles aux résultats, pas à pas), « lire l'image résumé » et « juger un extrait » dans `docs/lire-les-resultats.md`, README réécrit pour elle ; la partie développeur est dans `CONTRIBUTING.md` (#77, #79, #80, #81).
- Archives du binaire pour macOS arm64, Linux x86_64 (glibc 2.35 au moins) et Windows x86_64, avec `SHA256SUMS` et les sources de FFmpeg et d'openh264 jointes à la release.
  Chaque archive est contrôlée avant publication : liens dynamiques, système minimal, version et tableau de l'extrait (#223).

### Modifié

- `batdetect` et `batdetect-bench` sont réécrits en Rust, FFmpeg lié en statique (#106).
  Sur la 092, la commande prend 19 s au lieu de 135 s.
  Sur 26 vidéos, 963 des 966 pistes de la 0.1.1 se retrouvent, 3 sont perdues et 9 apparaissent ; les écarts sont listés dans docs/11.
- Le tableau des pistes se lit dans un tableur : en français, BOM UTF-8, point-virgule, virgule décimale et en-têtes français ; en anglais, virgule et point.
  Une colonne `verdict` vide suit la durée, début et fin s'écrivent `0:00:04`, le début en secondes passe en dernière colonne (#84).
- Au recalcul, un tableau qui porte des verdicts, ou qui n'a plus la forme écrite par l'outil, est mis de côté tel quel sous un nom daté (`<nom>_pistes_2026-10-16_1430.csv`) et le nouveau tableau s'écrit à côté ; le bilan le dit (#83).
- Sans `-o`/`--out-dir`, les résultats vont dans `batdetect/` à côté de chaque vidéo, et non plus dans `out/` du dossier de lancement (#86).
- `params.json` s'écrit en dernier, après les extraits : un dossier sans lui n'est pas terminé.
  Il gagne l'état (`done` ou `too_many_tracks`), le nombre de pistes, la durée, le temps non analysé, le temps de calcul, la taille de la vidéo, la langue des noms, la date, et la liste des pistes avec leur couleur sur l'image résumé et leurs extraits (#66).
- Encodage par VideoToolbox puis openh264 sous macOS, openh264 ailleurs, au lieu de libx264 ; ffmpeg et ffprobe ne sont plus nécessaires.
- Le binaire n'est plus sous GPL v3 : il est sous MIT pour le code et LGPL 2.1 ou ultérieure pour FFmpeg, avec openh264 (BSD-2), Roboto (OFL-1.1) et les crates Rust.
  `NOTICE` les décrit un par un, et leurs textes de licence sont dans `licenses/` de chaque archive (#224).

### Retiré

- Le code Python. Son dernier état est au tag `python-final` (#199).
- Les options `--mode`, `--workers`, `--encoder`, `--crf`, `--vt-quality`, `--box-pad`, `--trail` et les réglages fins de détection et de suivi : la commande garde `--threshold`, `--min-hits`, `--max-blobs` et `--work-width`, les autres valeurs sont écrites dans `params.json`.

## [0.1.1] - 2026-10-10

### Ajouté

- Aide, erreurs d'arguments, lignes de fin de traitement et bandeau de l'image résumé en français sous une locale française (`LANG`, `LC_MESSAGES`, `LC_ALL` ou `LANGUAGE`), en anglais sinon (#57).
- Chaque option de `batdetect --help` a une description et affiche sa valeur par défaut, avec la valeur de chaque mode pour les réglages qui en dépendent (#54).

### Modifié

- Les fichiers produits suivent la langue : en français `extraits/piste_01_0m04s.mp4`, `<vidéo>_pistes.csv`, `<vidéo>_resume.png`, `<vidéo>_annotee.mp4` ; en anglais `clips/track_01_0m04s.mp4`, `<vidéo>_tracks.csv`, `<vidéo>_summary.png`, `<vidéo>_annotated.mp4`.
  Les anciens noms (`split/`, `.tracks.csv`, `.tracks.png`, `_boxes.mp4`) sont effacés au prochain calcul de la vidéo ; le contenu du CSV ne change pas (#85).
- Le bandeau de l'image résumé compte des « pistes » et non des « passages » : un passage est une piste confirmée par la naturaliste (#87).
- Le temps dans le nom des extraits ne garde plus les centièmes ; le numéro de piste prend trois chiffres dès 100 pistes, pour que le Finder les range dans l'ordre (#85).

## [0.1.0] - 2026-10-08

Première version publiée : binaire macOS arm64 (macOS 14 ou plus récent) installé par `brew install zaratan/bat-tools/batdetect`, sous GPL v3 pour le binaire et MIT pour le code (#30).

- Détection et suivi des chauves-souris dans les vidéos de jumelles thermiques, une image résumé, un CSV et un extrait vidéo par piste.
- `--mode normal|quick` pour choisir entre finesse de détection et vitesse (#38).
- Extrait zoomé et ralenti pour les pistes petites ou faibles (`--zoom`, #37).
- `--max-tracks` (300 par défaut) arrête le rendu d'une vidéo qui sort trop de pistes (#3).
- Images abîmées au décodage repérées et sorties du calcul (#1), réglages invalides refusés (#4).

[0.3.0]: https://github.com/zaratan/chiro-dvr/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/zaratan/chiro-dvr/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/zaratan/chiro-dvr/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zaratan/chiro-dvr/releases/tag/v0.1.0
