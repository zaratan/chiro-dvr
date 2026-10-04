# Méthode

Le traitement d'une vidéo se fait en deux passes. La première lit la vidéo en petit,
détecte et suit. La seconde relit la vidéo en pleine résolution, dessine les boîtes et
encode directement chaque extrait (et la vidéo annotée complète avec `--annotated`).

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

La détection tourne par défaut dans un seul processus. Avec `--workers` > 1, elle peut
être découpée en tranches de temps traitées en parallèle, si la vidéo est assez longue,
avec une demi-fenêtre de recouvrement : le résultat est identique au traitement d'un seul
tenant. Chaque tranche supplémentaire ajoute surtout de la relecture ([09](09-profilage.md)).

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

## 4 bis. Images saturées de taches

Quand les jumelles bougent, même d'une fraction de pixel par image, les images de la
fenêtre du fond ne sont plus alignées : chaque contour contrasté laisse un résidu, et
une seule image compte des centaines de taches. Sur la 089 (cadre qui glisse d'environ
4 px entre 4,5 et 19 s), cela donnait 1 452 fausses pistes en 20 s ; sur la 091 (deux
gros mouvements vers 3:57 et 4:06), 4 270. Un décalage figé, lui, est rattrapé par la
fenêtre de 1 s : c'est le glissement qui crée les pistes.

Une image compte comme **saturée** si elle a au moins `max_blobs` = 20 taches de plus
que 6 fois la médiane des taches par image de la vidéo. La médiane fait monter la limite
avec un seuil de détection bas, où même une vidéo stable compte des dizaines de taches
par image : sur l'extrait de la 092 à `--threshold 15`, une limite fixe de 20 excluait
toute la vidéo, la limite relative n'exclut rien. Non mesuré : avec un seuil bas, la
limite monte (872 taches à `--threshold 12`) et un vrai mouvement pourrait passer
dessous ; la règle a été réglée au seuil par défaut. Les périodes ignorées sont ces images
élargies de `pad_s` = 1 s (`--unstable-pad`) de chaque côté (la fenêtre du fond déborde de ½ s, et
les images calmes au milieu d'un mouvement restent suspectes), fusionnées quand au plus
1 s les sépare. Leurs détections sont retirées avant le suivi ; la détection elle-même
ne change pas. Une piste qui traverse une période est coupée en deux.

Mesuré sur les détections réelles : cadre stable, au plus 12 taches par image (089,
091) et au plus 4 sur toute la 092 ; cadre qui bouge, plus de 270 en médiane. Résultat :
089 1 546 → 96 pistes (19,5 s ignorées), 091 4 327 → 53 (22,3 s), 092 inchangée
(14 pistes, rien d'ignoré). Le nombre de taches sert d'indicateur parce qu'il est
gratuit : mesurer le décalage du cadre par corrélation de phase coûterait 3,2 ms par
image, environ 29 s par vidéo de 5 min.

La règle repère des images saturées, pas seulement des mouvements : sur la 089, entre
7,5 et 10,7 s, des images à plus de 20 taches apparaissent sans mouvement mesurable.
Un essaim d'au moins 20 chauves-souris dans la même image serait aussi ignoré : les
périodes sont toujours signalées (en-tête de l'image résumé, « hors analyse » sans accent
parce que la police ne dessine que l'ASCII, trois lignes au plus puis « + N autres » ;
console ; `params.json` sous `ignored_s`), et
`--max-blobs 0` désactive le filtre. Le banc applique le même filtre et retire des
cibles visibles celles qui tombent dans une période ignorée.

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
image et dessinée en trait fin, pour qu'elle ne clignote pas ; la trace jaune de la
dernière seconde la suit jusqu'à cette position. Ces positions servent
uniquement à l'affichage : le CSV et le banc ne comptent que les vraies détections, et la
colonne `filled_frames` indique combien d'images ont été comblées.

La vidéo annotée et les extraits sont encodés par le moteur multimédia des puces Apple
(`h264_videotoolbox`, qualité `--vt-quality` 65) quand un essai d'encodage de quelques
images (0,2 s) au démarrage réussit, sinon par `libx264` (`--crf` 20), par exemple sous Linux.
`--encoder videotoolbox|x264` force le choix, et un encodeur forcé inutilisable arrête la
commande avec le message d'ffmpeg ; `params.json` enregistre l'encodeur réellement
utilisé. Mesuré sur la 092 : vidéo annotée en 19,8 s au lieu de 118,5 s, fidélité
presque égale (SSIM 0,977 contre 0,982, [09](09-profilage.md)).

La seconde passe lit l'original dans l'ordre, avec la même numérotation des images que
la détection, une seule fois quand 6 encodeurs suffisent. Chaque extrait a son encodeur, ouvert à la première image de
sa fenêtre (début de la piste moins `--clip-margin`, fin plus `--clip-margin`) et fermé à
la dernière ; les images que personne n'attend sont sautées sans être converties. Au plus
6 encodeurs tournent en même temps (`MAX_WRITERS`) : au-delà, les extraits sont répartis
en plusieurs passes, chacune relisant la vidéo. Un extrait qui échoue ou une interruption
ne laissent pas de fichier tronqué.

## 7. Filtres finaux

- `min_hits` = 6 détections (depuis le 4 octobre ; 5 avant) : élimine les étincelles
  de quelques images. Le passage valide le plus court de la 092 (2:37) en a 7 : ne pas
  dépasser 7.
- `min_travel` = 45 px de bout en bout : élimine un point qui scintille sur place.
- `max_median_turn` = 0,8 rad : élimine les pistes qui zigzaguent. Le virage d'une piste
  est l'angle entre deux pas successifs ; on en prend la médiane, qui tolère un virage
  brusque isolé. Un pas nul (même pixel rallumé) est ignoré ; une piste trop courte pour
  avoir un virage est gardée. 3,15 ou plus désactive le filtre.

Ces deux derniers réglages visent le scintillement de pixels isolés que le suivi enchaîne
(`max_jump` 120 px) sur les vidéos bruitées : taches d'un pixel de travail, contraste
juste au-dessus du seuil, 5 à 7 détections, directions au hasard (virage médian 1,5 rad,
contre au plus 0,35 pour les vraies pistes de la 092). Mesuré, après le filtre des images
saturées : 089 96 → 4 pistes, 091 53 → 23, 092 inchangée (14). Banc : transit 115
trouvées sur 253 visibles comme avant, chasse 357 sur 584 au lieu de 371 sur 592 ; le
virage ne coûte aucune cible, même en chasse ou en tournoiement
([08](08-banc-de-mesure.md#vols-de-chasse-4-octobre-2026)), les 14 pertes viennent de
`min_hits` 6 (cibles peu contrastées détectées sur 5 images seulement ; le banc se sert
aussi de `min_hits` pour dire qu'une cible est trouvée, 11 des 14 viennent du suivi).

Limite connue : une chauve-souris lente avance de quelques pixels par image, et
l'arrondi des positions fait tourner sa direction. La piste lente de 3:29 a un virage
médian de 0,22 rad sur toute sa longueur, mais 12 fenêtres de 7 points sur 280 dépassent
0,8 : un fragment court d'un vol lent (coupé par un trou ou une période ignorée) serait
rejeté. Le banc ne simule pas de vol lent. Piste : ignorer les pas trop courts, ou
mesurer la direction sur 3 images comme la vitesse. Avec
5 détections, environ 16 % des pistes de bruit de 5 points passent le virage par hasard
(3 virages seulement).

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
