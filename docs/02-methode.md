# Méthode

Le traitement d'une vidéo se fait en deux passes. La première lit la vidéo en petit,
détecte et suit. La seconde relit la vidéo en pleine résolution pour dessiner les
boîtes, puis découpe les extraits.

Les distances des paramètres sont en **pixels de travail** : la vidéo est réduite à
`work_width` = 480 px de large (hauteur proportionnelle). Pour une vidéo 1440×1080, un
pixel de travail vaut 3 pixels d'origine. Le CSV, lui, donne les distances en pixels
d'origine.

## 1. Fond par médiane glissante

Pour chaque image *t*, le fond est la médiane, pixel par pixel, des images de
*t − ½ fenêtre* à *t + ½ fenêtre*, une sur `bg_step`.

- `bg_window_s` = 1,0 s. Une chauve-souris traverse le champ en 0,3 à 1,3 s (mesuré
  sur la 092) : sur une seconde, elle n'occupe un pixel donné que sur quelques images,
  donc la médiane l'efface. Une fenêtre plus longue suivrait mal les changements de
  lumière ; plus courte, elle garderait l'animal dans le fond.
- `bg_step` = 3 : 11 images au lieu de 31 pour la médiane. La médiane est 85 % du temps
  de détection (9,3 ms par image, mesuré), le fond change lentement, donc on
  sous-échantillonne.
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
- Les bandes `osd_top` (7 %) et `osd_bottom` (10 %) sont mises à zéro pour ignorer
  l'affichage incrusté.

## 4. Taches

- `merge_radius` = 2 : une fermeture morphologique (disque de 5 px) recolle les
  fragments d'un même animal séparés de moins de ~4 px. Sur la 092 à 3:51, une seule
  chauve-souris sortait en deux taches distantes de 4 à 6 px.
- `min_area` = 2 et `max_area` = 300 pixels de travail. L'aire compte les pixels
  réellement au-dessus du seuil, pas ceux ajoutés par la fermeture. Sous 2, c'est du
  bruit ; au-dessus de 300, ce n'est plus un petit animal.

## 5. Suivi

Chaque image, les détections sont attribuées aux pistes ouvertes.

- **Prédiction** à vitesse constante depuis les deux derniers points.
- **Appariement** au plus proche dans un rayon `max_jump` = 40 px, mais les pistes qui
  ont déjà une vitesse passent avant celles d'un seul point. Sans cette priorité, une
  piste naissante (un fragment) volait le point de la vraie piste : le passage de 3:58
  sortait coupé en deux.
- **Trous** : une piste survit à `max_gap` = 6 images consécutives sans détection
  (0,2 s), le temps qu'une chauve-souris peu contrastée réapparaisse ; elle est close à
  la septième. Les tests de borne vérifient exactement ces deux cas.

## 6. Fusion des pistes jumelles

Deux pistes qui coexistent et restent à moins de `twin_distance` = 12 px l'une de
l'autre sur **toutes** leurs images communes sont un seul animal fragmenté : elles sont
fusionnées (boîte englobante, centre pondéré par l'aire). Deux vraies chauves-souris
s'écartent au moins une fois et restent séparées. Sur la 092 à 2:44, une chauve-souris
peu contrastée sortait en fragments écartés de 6 à 8 px, trop pour `merge_radius`.

`twin_distance` = 0 désactive la fusion.

## 7. Filtres finaux

- `min_hits` = 5 détections : élimine les étincelles d'une ou deux images.
- `min_travel` = 15 px de bout en bout : élimine un point qui scintille sur place.

Les pistes restantes sont numérotées par ordre chronologique. Le numéro change dès
qu'une piste apparaît ou disparaît plus tôt dans la vidéo : pour désigner un passage,
utiliser son temps de début.

## Sorties

La liste des fichiers produits est dans le [README](../README.md#utilisation).

Colonnes du CSV : `id`, `start`, `end`, `start_s` (début en secondes), `duration_s`, `hits` (nombre de détections),
`chord_px` (distance de bout en bout), `path_px` (chemin parcouru), `speed_px_s`
(chemin ÷ durée), `max_area_px` (plus grande tache, en pixels d'origine),
`peak_amplitude` (plus fort écart au fond, en niveaux de gris).
