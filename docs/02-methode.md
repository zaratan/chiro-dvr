# Méthode

Le traitement d'une vidéo se fait en deux passes. La première lit la vidéo en petit,
détecte et suit. La seconde relit la vidéo en pleine résolution, dessine les boîtes et
encode directement chaque extrait, l'extrait zoomé et ralenti des pistes petites ou
faibles, et la vidéo annotée complète avec `--annotated`.

**Règle de réglage** (décision du 9 octobre 2026, après le verdict de Manon sur la 092) : mieux
vaut un faux positif facile à écarter à l'extrait qu'un passage manqué. Quand un seuil
hésite, il penche vers la sensibilité ; l'extrait, zoomé et ralenti s'il le faut, sert à
écarter. La piste lente de 3:29 sur la 092 reste produite pour cette raison.

Toutes les distances et surfaces des réglages, des pistes et du CSV sont en **pixels de
la vidéo d'origine**. Pour détecter, la vidéo est réduite à `work_width` px de large
(960 par défaut depuis le 6 octobre 2026, 480 auparavant ; jamais plus que la vidéo
elle-même) ; les détections sont reconverties en pixels d'origine dès leur création.
Changer `work_width` ne change donc pas le sens des autres réglages, **à l'arrondi
près** : à 960 px de large sur une vidéo 1440×1080, un pixel de travail vaut 1,5 px
d'origine et 2,25 px² ; à 480 px, 3 px et 9 px². `merge_radius` est arrondi au pixel de
travail. Les pixels d'origine ne sont pas une unité physique : une autre caméra, avec une
autre résolution, demandera peut-être d'autres valeurs. Les valeurs par défaut ont été
choisies sur sept vidéos 1440×1080 de la falaise (092, 089, 090, 091, 125, 126, 127 ;
[08](08-banc-de-mesure.md#sur-six-autres-vidéos)).

Deux modes : `--mode normal` (défaut) prend les valeurs décrites ici ; `--mode quick`
reprend celles d'avant le seuil par pixel (#7 : `work_width` 480, seuil fixe 25, sans
filtre à la taille de la cible ni seuil par pixel). Une option donnée explicitement
l'emporte sur le mode.

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
  lentement. Un `bg_step` qui laisse moins de 3 images dans la fenêtre (16 et plus à
  30 i/s) est refusé : le fond ne serait plus une médiane mais une ou deux images
  (moyennées).
- La médiane est calculée par un tri par comparaisons partiel (`median.py`) : des
  `minimum` et `maximum` sur des images entières, jusqu'à fixer la ou les valeurs
  centrales. Résultat identique au bit près à `np.median` (sur des flottants, la moyenne
  des deux valeurs centrales se fait en double précision). Sans filtre, la fenêtre est
  gardée en uint8 : 0,7 ms par image à 480 px sur un cœur, contre 12,6 à 18 ms avec
  `np.median`. Avec le filtre à la taille de la cible (défaut), elle est en float32 et
  la médiane coûte plus cher ([09](09-profilage.md)).
- Aux bords de la vidéo, la fenêtre est tronquée : la première et la dernière demi-seconde
  sont analysées avec moins d'images.

## 2. Compensation du gain automatique

Les jumelles ajustent leur gain : toute l'image peut s'éclaircir d'un coup. On retire
au résidu l'écart entre la luminosité moyenne de l'image et celle des images du fond.
Le fond médian, lui, n'est pas corrigé : pendant la demi-seconde où la fenêtre mélange
des images avant et après un saut, le résidu garde une partie du saut. Avec l'ancien
seuil fixe de 25, un saut global de 30 niveaux ne produisait aucune détection. Avec les
réglages par défaut (seuil 12, filtre, seuil par pixel), mesuré sur des images
fabriquées : un saut de 10 donne au plus une tache, un saut de 20 jusqu'à 10 taches par
image sur 11 images, un saut de 30 à 40 des centaines de taches par image. Aucune piste
n'en sort (testé de 10 à 40) : pour 30 et 40, le filtre des images saturées
(§ 4 bis) ignore 2,6 à 2,8 s autour du saut.

## 3. Seuil dans les deux sens

Un pixel est retenu si |image − fond − écart de gain| > max(`threshold`, `noise_factor` × σ du
pixel), sur l'image filtrée.

- `threshold` = 12 niveaux de gris sur 255, mesurés sur l'image filtrée : c'est le
  **plancher** du seuil par pixel ci-dessous. Seul, sans seuil par pixel, il laisse
  passer le bruit de fond de certaines vidéos (089, 090, 091 : des centaines de pistes) ;
  avant le 6 octobre 2026, le seuil était fixe à 25 sur l'image non filtrée à 480 px.
- La valeur absolue rend la détection indifférente au signe : tache sombre sur roche
  chaude ou tache claire sur ciel froid.
- **Filtre à la taille de la cible** (`--target-sigma s`, en pixels d'origine, 1,5 par
  défaut, 0 pour le désactiver) : chaque image réduite passe par un flou gaussien
  d'écart-type `s` avant le fond et la compensation du gain. Le filtre étant linéaire, le
  fond, le résidu, le bruit du seuil par pixel et `peak_amplitude` sont tous mesurés sur
  l'image filtrée. C'est le filtre adapté à une tache gaussienne : il moyenne le bruit
  d'un pixel à l'autre et garde une tache de la même taille. Il remplace le débruitage
  qu'apporte la réduction à 480 px, et rend utilisables des largeurs de travail plus
  grandes. Les images filtrées sont gardées en flottants, ce qui rend la médiane plus
  lente. Une zone `--osd-region` est élargie de 3 `s` pour couvrir l'étalement de
  l'affichage par le flou. `min_area` et `max_area` s'appliquent à la tache filtrée,
  plus large que la tache brute. Mesures : [08](08-banc-de-mesure.md#filtre-à-la-taille-de-la-cible-b2).
- **Seuil par pixel** (`--noise-factor k`, 8 par défaut, 0 pour revenir au seuil fixe) : le
  seuil d'un pixel devient max(`threshold`, k × σ), où σ = 1,4826 × la MAD des écarts au
  fond des images de la fenêtre (celles de la médiane, 11 à 30 i/s), chacune corrigée de son
  écart de gain et centrée sur leur médiane. Elle n'est calculée que sur les pixels dont
  le résidu dépasse déjà `threshold`, seuls à pouvoir être retenus. `threshold` devient
  le plancher : on l'abaisse pour gagner en sensibilité sur la roche calme, et la MAD
  relève le seuil là où l'image s'agite. C'est lui qui empêche le bruit de fond des vidéos
  agitées de devenir des pistes. La carte ne dépend que de la fenêtre : la détection parallèle reste
  identique au bit près. Aux bords de la vidéo, quand la fenêtre tronquée compte moins de
  7 images, le seuil fixe s'applique seul. Pour la même raison, le seuil par pixel est
  **désactivé sur toute la vidéo** sous 17,5 images par seconde avec `bg_step` = 3, ou
  dès `bg_step` = 6 à 30 i/s : il ne reste que le plancher de 12, qui laisse passer le
  bruit de fond des vidéos agitées. Toutes nos vidéos sont à 30 i/s. Le ciel saturé à 0 a une MAD nulle et aucun
  résidu sombre possible : le seuil par pixel n'y gagne rien. Mesures et coût :
  [08](08-banc-de-mesure.md#seuil-par-pixel-d1).
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
  au-dessus du seuil, pas ceux ajoutés par la fermeture, sur l'image filtrée. **À 960 px
  de large, un pixel de travail vaut 2,25 px² : `min_area` 4 demande donc deux pixels de
  travail.** Une tache d'un seul pixel est rejetée ; c'est ce qui fait perdre une image du
  passage de 3:42.10 sur la 092 à 960 px sans seuil par pixel
  ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)). À 480 px, 4 px² laissait passer
  un seul pixel de travail. Mesuré au banc : 9 ou 18 px² perdent les petites cibles
  faibles sans retirer de bruit.

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
dessous ; la règle a été réglée à l'ancien seuil fixe de 25. Avec les réglages par
défaut actuels (seuil par pixel), la médiane reste à 0 tache par image sur 089, 090 et 091
et la limite à 20. Secondes ignorées : 089 8,3 s contre 19,5 avec l'ancien réglage,
091 20,6 s contre 22,3 ; le glissement de la 089 produit moins de taches au-dessus du
seuil par pixel. Les périodes ignorées sont ces images
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
cibles visibles celles qui tombent dans une période ignorée. Les images illisibles (4 ter)
sont retirées avant le comptage : elles ne déclenchent plus de période et ne pèsent pas
sur la médiane des taches.

## 4 ter. Images illisibles

Une passe `ffprobe` sur un seul fil tourne pendant la détection et signale les images
que le décodeur a dû réparer (message d'erreur rattaché à l'image), ainsi que l'image qui
suit un trou dans les horodatages de plus d'un pas et demi. Chacune est écartée jusqu'à
l'image qui précède la prochaine image-clé, puis la plage est élargie de la demi-fenêtre
du fond de chaque côté (`half_window`, 15 images à 30 i/s, déduite de `bg_window_s`),
avant la fusion des plages, la stabilité et le suivi. Pas d'option pour désactiver.

Mesuré : la 092 (aucune erreur) est inchangée au bit près. La 093 (11 erreurs) a 14,0 s
illisibles en 8 plages, une seule période instable au lieu de quatre, et 3 pistes au lieu
de 4. La 122 (195 erreurs, 13 images jamais décodées) a 201,2 s illisibles en 79 plages,
plus aucune période instable, et plus aucune piste au lieu de 25. Sans la marge, il
restait sur la 122 50,3 s instables et 4 pistes, toutes collées aux plages. Les raisons, la
preuve de la correspondance entre ffprobe et OpenCV et les limites sont dans
[10](10-logique-de-detection.md). La passe coûte 55 à 75 s de CPU pour 5 min de vidéo,
mais elle finit avant la détection à `--workers 1` et n'allonge pas la commande
([09](09-profilage.md#avec-les-réglages-du-6-octobre-issue-36)).

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

### Extrait zoomé et ralenti

Une cible de quelques pixels qui traverse le champ en 0,2 s ne se voit pas sur l'extrait
normal (passage de 3:42.10 de la 092, jugé faux puis reconnu réel en zoomant et en
ralentissant dans le lecteur, #37). Pour ces pistes, un second fichier
`split/<nom>_zoom.mp4` est écrit à côté de l'extrait normal.

- **Quelles pistes** (`--zoom auto`, par défaut) : celles dont la plus grande tache fait
  moins de 100 px² (`max_area_px`) **ou** dont le plus fort écart au fond est sous 40
  (`peak_amplitude`), comparaisons strictes, mêmes grandeurs que le CSV. Sur la 092,
  seule la piste 0:51.62 (36 px², 34,7) a son zoom ; le passage de 3:42.10, quand un
  réglage le trouve (1440 px sans seuil par pixel : 70 px², 22,0), l'aurait aussi. Sur
  le balayage du 7 octobre (14 vidéos, 848 pistes), 384 pistes ont leur zoom (45 %),
  de 0 sur la 089 à 83 sur 136 pour la 096 ; sur 095 à 099, c'est surtout l'aire qui
  déclenche. `--zoom all` en fait un pour chaque piste, `--zoom none` aucun. Les seuils
  (`zoom_below_area`, `zoom_below_amplitude`) sont dans `params.json` mais pas en option.
  **L'amplitude dépend du filtre `--target-sigma`, du seuil et de `--work-width`** : le
  seuil de 40 se remesure quand ces réglages changent.
- **Cadre** : fixe pour tout l'extrait. Boîte englobante des détections de la piste,
  plus 40 px (pixels d'origine) de chaque côté, élargie au rapport de l'image source
  (4:3 pour les jumelles) en unités entières, au moins un quart de l'image par côté
  (360 × 270 sur 1440 × 1080), puis décalée pour rester dans l'image ; si elle dépasse
  l'image, c'est l'image entière. Le rapport exact passe avant l'agrandissement : pour des
  dimensions sans diviseur commun (1442 × 1081), le cadre est toujours l'image entière. L'agrandissement va donc de ×1 à ×4 : ×4 pour la piste
  0:51.62 de la 092, ×1,29 à ×4 (médiane ×2,77) pour les 39 zooms de la 125. Une
  traversée rapide du champ donne un cadre presque entier : le ralenti fait alors
  l'essentiel.
- **Agrandissement** au plus proche voisin : chaque pixel source devient un carré,
  rien n'est lissé. La sortie a la taille de la vidéo source.
- **Ralenti ×0,25** : les mêmes images que l'extrait normal (même fenêtre, marges
  comprises), aucune dupliquée ni interpolée, déclarées à un quart de la cadence
  (7,5 i/s pour 30 i/s). La piste 0:51.62 dure 3,3 s en extrait normal, 13,2 s en zoom.
- **Dessin** : seulement la trace de la piste du fichier (`--trail`, trait de 1 px après
  agrandissement), sans boîte ni numéro, et rien des autres pistes. Chaque segment de la
  trace est coupé à 16 px (pixels d'origine) de la boîte de la cible, pour ne pas
  recouvrir une cible ténue, même quand la piste revient sur elle-même. Le recadrage se
  fait sur l'image brute, avant le dessin de l'extrait normal, qui reste identique.
- **Coût** : chaque zoom a son propre encodeur, compté dans les 6 de `MAX_WRITERS`. Sur
  la 092, rien de mesurable ; sur la 125 (39 zooms), une seconde passe et environ 42 s
  de CPU de plus ([09](09-profilage.md#extraits-zoomés-issue-37)). En `libx264`, la
  qualité `--crf` dépend de la cadence déclarée : un zoom pèse plus qu'un extrait aux
  mêmes images.

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

Garde-fou : au-delà de `max_tracks` pistes (300 par défaut, 0 désactive), les extraits
et la vidéo annotée ne sont pas produits, et la vidéo compte en échec. Le CSV,
`params.json` (avec le seuil, dans `RenderConfig`) et l'image résumé restent écrits. 300
est au-dessus de la vidéo la plus chargée sans explosion (148 pistes, 097) et en dessous
de la plus petite explosion connue (374 pistes, seuil 12 sans facteur de bruit). Le
seuil est fixe, pas un taux par minute : il suppose des clips d'environ 5 min, comme ceux
des jumelles, et se monte avec `--max-tracks` pour une vidéo plus longue. Au-delà, la
liste des pistes n'est pas affichée en console.

Colonnes du CSV : `id`, `start`, `end`, `start_s` (début en secondes), `duration_s`, `hits` (nombre de détections),
`chord_px` (distance de bout en bout), `path_px` (chemin parcouru), `speed_px_s`
(chemin ÷ durée), `max_area_px` (plus grande tache, en pixels d'origine),
`peak_amplitude` (plus fort écart au fond, en niveaux de gris ; avec `--target-sigma`,
écart mesuré sur l'image filtrée, donc plus bas que l'écart brut pour une petite tache),
`filled_frames` (images
comblées par interpolation à l'affichage).
