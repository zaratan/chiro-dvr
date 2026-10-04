# Méthode

Le traitement d'une vidéo se fait en deux passes. La première lit la vidéo en petit,
détecte et suit. La seconde relit la vidéo en pleine résolution pour dessiner les
boîtes, puis découpe les extraits.

Toutes les distances et surfaces des réglages, des pistes et du CSV sont en **pixels de
la vidéo d'origine**. Pour détecter, la vidéo est réduite à `work_width` px de large
(480 par défaut, jamais plus que la vidéo elle-même) ; les détections sont reconverties
en pixels d'origine dès leur création. Changer `work_width` ne change donc pas le sens
des autres réglages, **à l'arrondi près** : à 480 px de large sur une vidéo 1440×1080, un
pixel de travail vaut 3 px d'origine et 9 px². Toute valeur de `min_area` jusqu'à 9 px²
revient à « un pixel de travail suffit », et `merge_radius` est arrondi au pixel de
travail. Les pixels d'origine ne sont pas une unité physique : une autre caméra, avec une
autre résolution, demandera peut-être d'autres valeurs. Les valeurs par défaut sont celles
qui donnaient les résultats documentés à 480 px sur une vidéo 1440×1080.

La détection est découpée en tranches de temps traitées en parallèle, avec une
demi-fenêtre de recouvrement : le résultat est identique au traitement d'un seul tenant.

## 1. Fond par médiane glissante

Pour chaque image *t*, le fond est la médiane, pixel par pixel, des images de
*t − ½ fenêtre* à *t + ½ fenêtre*, une sur `bg_step`.

- `bg_window_s` = 1,0 s. Une chauve-souris traverse le champ en 0,3 à 1,3 s (mesuré
  sur la 092) : sur une seconde, elle n'occupe un pixel donné que sur quelques images,
  donc la médiane l'efface. Une fenêtre plus longue suivrait mal les changements de
  lumière ; plus courte, elle garderait l'animal dans le fond.
- `bg_step` = 3 : 11 images au lieu de 31 pour la médiane, puisque le fond change
  lentement.
- La médiane est calculée par un tri par comparaisons partiel (`median.py`) : des
  `minimum` et `maximum` sur des images entières, jusqu'à fixer la ou les valeurs
  centrales, sur la fenêtre gardée en uint8. Résultat identique au bit près à
  `np.median`, en 0,7 ms par image sur un cœur, contre 12,6 à 18 ms avec `np.median`
  ([09](09-profilage.md)).
- Aux bords de la vidéo, la fenêtre est tronquée : la première et la dernière demi-seconde
  sont analysées avec moins d'images.

## 2. Compensation du gain automatique

Les jumelles ajustent leur gain : toute l'image peut s'éclaircir d'un coup. On retire
au résidu l'écart entre la luminosité moyenne de l'image et celle des images du fond.
Un saut global de 30 niveaux ne produit ainsi aucune détection (testé).

## 3. Seuil dans les deux sens

Un pixel est retenu si |image − fond − écart de gain| > `threshold`.

- `threshold` = 25 niveaux de gris sur 255. Sur la 092, le bruit de fond reste sous ce
  seuil (aucun pixel retenu sur la plupart des images) et les passages montent de 49 à
  130 (`peak_amplitude` du CSV).
- La valeur absolue rend la détection indifférente au signe : tache sombre sur roche
  chaude ou tache claire sur ciel froid.
- Aucun masque par défaut (voir [01](01-contexte.md)). `--osd-region x0,y0,x1,y1`,
  répétable, en fractions de l'image, met à zéro une zone d'affichage qui bougerait,
  et la masque aussi sur le fond de l'image résumé. Pour les Symbion, `0,0,1,0.07`
  cache l'heure, la date, la batterie et le chrono ; mesuré sur l'original de la 092 :
  mêmes 14 pistes, mêmes débuts et mêmes fins que sans masque.
  Les anciennes bandes du haut et du bas (7 et 10 %) coupaient 5 pistes sur 14.

## 4. Taches

- `merge_radius` = 6 px : une fermeture morphologique recolle les fragments d'un même
  animal séparés de moins de ~12 px. Sur la 092 à 3:51, une seule chauve-souris sortait
  en deux taches distantes de 12 à 18 px.
- `min_area` = 4 et `max_area` = 2 700 px². L'aire compte les pixels réellement
  au-dessus du seuil, pas ceux ajoutés par la fermeture. À 480 px de large, 4 px²
  d'origine laisse passer une tache d'un seul pixel de travail : mesuré au banc
  ([08](08-banc-de-mesure.md)), c'est ce qui permet de trouver les petites cibles, sans
  piste de bruit en plus. L'ancienne valeur (18 px²) rejetait les taches de moins de deux
  pixels de travail. À plus haute résolution, ce réglage laisse passer le bruit.

## 5. Suivi

Chaque image, les détections sont attribuées aux pistes ouvertes.

- **Prédiction** à vitesse constante, la vitesse étant mesurée entre le dernier point et
  le plus récent situé au moins 3 images avant (une période de la saccade des jumelles).
  Avec les deux derniers points seulement, l'erreur atteignait un facteur 2 une image sur
  trois ; sur la 092, la piste la plus rapide (3:48) décrochait 2 images trop tôt. Une
  piste de deux points garde la prédiction sur ces deux points.
- **Appariement** au plus proche dans un rayon `max_jump` = 120 px, mais les pistes qui
  ont déjà une vitesse passent avant celles d'un seul point. Sans cette priorité, une
  piste naissante (un fragment) volait le point de la vraie piste : le passage de 3:58
  sortait coupé en deux.
- **Trous** : une piste survit à `max_gap` = 6 images consécutives sans détection
  (0,2 s), le temps qu'une chauve-souris peu contrastée réapparaisse ; elle est close à
  la septième. Les tests de borne vérifient exactement ces deux cas.

## 6. Fusion des pistes jumelles

Deux pistes qui coexistent et restent à moins de `twin_distance` = 36 px l'une de
l'autre sur **toutes** leurs images communes sont un seul animal fragmenté : elles sont
fusionnées (boîte englobante, centre pondéré par l'aire). Deux vraies chauves-souris
s'écartent au moins une fois et restent séparées. Sur la 092 à 2:44, une chauve-souris
peu contrastée sortait en fragments écartés de 18 à 24 px, trop pour `merge_radius`.

`twin_distance` = 0 désactive la fusion.

## Rendu

Entre deux détections d'une même piste, la boîte est interpolée linéairement à chaque
image et dessinée en trait fin, pour qu'elle ne clignote pas. Ces positions servent
uniquement à l'affichage : le CSV et le banc ne comptent que les vraies détections, et la
colonne `filled_frames` indique combien d'images ont été comblées.

La vidéo annotée et les extraits sont encodés par le moteur multimédia des puces Apple
(`h264_videotoolbox`, qualité `--vt-quality` 65) quand un essai d'encodage de quelques
images (0,2 s) au démarrage réussit, sinon par `libx264` (`--crf` 20), par exemple sous Linux.
`--encoder videotoolbox|x264` force le choix ; `params.json` enregistre l'encodeur
réellement utilisé. Mesuré sur la 092 : vidéo annotée en 19,8 s au lieu de 118,5 s,
fidélité presque égale (SSIM 0,977 contre 0,982, [09](09-profilage.md)).

## 7. Filtres finaux

- `min_hits` = 5 détections : élimine les étincelles d'une ou deux images.
- `min_travel` = 45 px de bout en bout : élimine un point qui scintille sur place.

Les pistes restantes sont numérotées par ordre chronologique. Le numéro change dès
qu'une piste apparaît ou disparaît plus tôt dans la vidéo : pour désigner un passage,
utiliser son temps de début.

## Sorties

La liste des fichiers produits est dans le [README](../README.md#utilisation).

Colonnes du CSV : `id`, `start`, `end`, `start_s` (début en secondes), `duration_s`, `hits` (nombre de détections),
`chord_px` (distance de bout en bout), `path_px` (chemin parcouru), `speed_px_s`
(chemin ÷ durée), `max_area_px` (plus grande tache, en pixels d'origine),
`peak_amplitude` (plus fort écart au fond, en niveaux de gris), `filled_frames` (images
comblées par interpolation à l'affichage).
