# Méthode

Ce document tient la valeur de chaque réglage et la mesure qui la justifie.
La logique de chaque étape, son module et ses limites sont dans [10](10-logique-de-detection.md).

Une vidéo se traite en deux passes.
La première lit la vidéo en petit pour détecter et suivre.
La seconde relit la vidéo en pleine résolution, mais seulement autour des pistes.
Elle y dessine les boîtes et encode les extraits ([Rendu](#rendu)).
Les pistes petites ou faibles ont en plus un extrait zoomé et ralenti, et `--annotated` demande la vidéo annotée complète.

**Règle de réglage** (9 octobre 2026, après le verdict de Manon sur la 092) : mieux vaut un faux positif facile à écarter à l'extrait qu'un passage manqué.
Quand un seuil hésite, il penche vers la sensibilité. C'est l'extrait qui sert à écarter, zoomé et ralenti s'il le faut.
C'est pour cela que la piste lente de 3:29 sur la 092 reste produite.

Les réglages portent ici leur nom de `params.json`.
Seuls quatre réglages de la détection et du suivi sont des options de la commande : `--threshold`, `--min-hits`, `--max-blobs` et `--work-width`.
S'y ajoute le masque `--osd-region`. `batdetect --help` affiche leurs défauts.
Les autres ont une valeur fixe écrite dans `params.json`.
Ils se règlent par les options du banc ([08](08-banc-de-mesure.md)) : `--target-sigma`, `--noise-factor`, `--unstable-pad`, `--twin-distance` et les autres citées plus bas.

Toutes les distances et surfaces des réglages, des pistes et du CSV sont en **pixels de la vidéo d'origine**.
Seule la détection travaille plus petit, à `work_width` px de large. Chaque détection est reconvertie en pixels d'origine dès sa création.
Changer `work_width` ne change donc pas le sens des autres réglages, **à l'arrondi près**.
Sur une vidéo 1440×1080 réduite à 960 px, un pixel de travail vaut 1,5 px d'origine et 2,25 px².
Les pixels d'origine ne sont pas une unité physique : une caméra d'une autre résolution demandera peut-être d'autres valeurs.
Les valeurs par défaut ont été choisies sur sept vidéos 1440×1080 de la falaise (092, 089, 090, 091, 125, 126, 127).
Les mesures sont dans [08](08-banc-de-mesure.md#sur-six-autres-vidéos).

- `work_width` = 960 px (6 octobre 2026). Il vaut au moins 16 et jamais plus que la largeur de la vidéo.
  Sur ces sept vidéos, la pleine résolution donne presque les mêmes pistes pour 2,4 fois le temps de calcul.
  À 480 px, la réduction faisait perdre 30 à 50 % de leur contraste aux cibles de moins de 3 px ([07](07-ameliorer-la-detection.md)).
  Le filtre à la taille de la cible (§ 3) débruite à sa place.

## 1. Fond par médiane glissante

Le fond d'une image est la médiane pixel par pixel des images voisines sur `bg_window_s`.
La médiane prend une image sur `bg_step` ([10 § 2](10-logique-de-detection.md#2-estimer-le-fond--la-médiane-temporelle)).

- `bg_window_s` = 1,0 s.
  Une chauve-souris traverse le champ en 0,3 à 1,3 s (mesuré sur la 092).
  Sur une seconde, elle n'occupe un pixel donné que quelques images et la médiane l'efface.
  Une fenêtre plus longue suivrait mal les changements de lumière. Plus courte, elle garderait l'animal dans le fond.
- `bg_step` = 3 : la médiane porte sur 11 images au lieu de 31. Le fond change assez lentement pour cela.
  Un `bg_step` qui laisse moins de 3 images dans la fenêtre (16 et plus à 30 i/s) est refusé.
  Le fond serait alors la moyenne d'une ou deux images et non plus une médiane.
  Un pas plus grand désactive aussi le seuil par pixel (§ 3).

## 2. Compensation du gain automatique

Cette étape n'a pas de réglage. Sa logique est en [10 § 3](10-logique-de-detection.md#3-compenser-le-gain-automatique).
On l'a mesurée avec la v0.1.1 sur des images fabriquées, avec les réglages par défaut : seuil 12, filtre et seuil par pixel.
Un saut de gain de 10 niveaux donne au plus une tache.
Un saut de 20 donne jusqu'à 10 taches par image sur 11 images, et un saut de 30 à 40 des centaines de taches par image.
Aucune piste n'en sort, de 10 à 40.
Pour 30 et 40, le filtre des images saturées (§ 4 bis) ignore 2,6 à 2,8 s autour du saut.

## 3. Seuil dans les deux sens

Sur l'image filtrée, un pixel est retenu si |image − fond − écart de gain| > max(`threshold`, `noise_factor` × σ du pixel) ([10 § 4](10-logique-de-detection.md#4-décider-pixel-par-pixel--le-seuil)).

- `threshold` = 12 niveaux de gris sur 255, mesurés sur l'image filtrée.
  C'est le **plancher** du seuil par pixel.
  On l'abaisse pour gagner en sensibilité sur la roche calme, et le seuil par pixel le relève là où l'image s'agite.
  Sans seuil par pixel, il laisse passer le bruit de fond de la 089, de la 090 et de la 091 : des centaines de pistes.
- **Filtre à la taille de la cible** (`target_sigma`, `--target-sigma s` dans le banc) = 1,5 px d'origine, la taille d'une petite chauve-souris lointaine.
  0 le désactive. Mesures : [08](08-banc-de-mesure.md#filtre-à-la-taille-de-la-cible-b2).
- **Seuil par pixel** (`noise_factor`, `--noise-factor k` dans le banc) = 8. À 0, on revient au seuil fixe.
  C'est lui qui empêche le bruit de fond des vidéos agitées de devenir des pistes.
  Mesures et coût : [08](08-banc-de-mesure.md#seuil-par-pixel-d1).
  σ s'estime sur les images de la fenêtre du fond. Il en faut au moins 7.
  Le seuil par pixel est donc **désactivé sur toute la vidéo** sous 17 images par seconde avec `bg_step` = 3, ou dès `bg_step` = 6 à 30 i/s.
  Il ne reste alors que le plancher de 12, qui laisse passer le bruit de fond des vidéos agitées.
  Toutes nos vidéos sont à 30 i/s.
- **Masque** : aucun par défaut (voir [01](01-contexte.md)).
  L'option `--osd-region x0,y0,x1,y1` se répète et prend des fractions de l'image.
  Elle met à zéro une zone d'affichage qui bougerait et la masque aussi sur le fond de l'image résumé.
  Pour couvrir l'étalement de l'affichage, la zone est élargie de la portée du flou : ⌈4 σ⌉ pixels de travail (4 px à 960 sur 1440).
  Pour les Symbion, `0,0,1,0.07` cache l'heure, la date, la batterie et le chrono.
  Sur l'original de la 092, on obtient alors les mêmes 14 pistes que sans masque, avec les mêmes débuts et les mêmes fins.
  Des bandes fixes en haut et en bas (7 et 10 % de l'image) coupaient 5 pistes sur 14.

## 4. Taches

- `merge_radius` = 6 px arrondi au pixel de travail.
  La fermeture morphologique recolle les fragments d'un même animal séparés de moins de ~12 px.
  Sur la 092 à 3:51, une seule chauve-souris sortait en deux taches distantes de 12 à 18 px.
- `min_area` = 4 et `max_area` = 2 700 px². Ils portent sur la tache filtrée, plus large que la tache brute.
  **À 960 px de large, un pixel de travail vaut 2,25 px² : `min_area` 4 demande donc deux pixels de travail.**
  Une tache d'un seul pixel est rejetée.
  À 960 px sans seuil par pixel, c'est ce qui fait perdre une image du passage de 3:42.10 sur la 092 ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)).
  Mesuré au banc avec la v0.1.1 : 9 ou 18 px² perdent les petites cibles faibles sans retirer de bruit.

## 4 bis. Images saturées de taches

Quand les jumelles bougent, même d'une fraction de pixel par image, chaque contour contrasté laisse un résidu.
Une image compte alors des centaines de taches ([10 § 6](10-logique-de-detection.md#6-écarter-les-images-où-les-jumelles-bougent)).
Sur la 089, le cadre glisse d'environ 4 px entre 4,5 et 19 s. La 091 a deux gros mouvements, vers 3:57 et 4:06.
Quand le cadre est stable, on compte au plus 12 taches par image sur la 089 et la 091, et au plus 4 sur toute la 092.
Quand il bouge, on en compte plus de 270 en médiane.

Une image est **saturée** si elle a au moins `max_blobs` = 20 taches de plus que 6 fois la médiane des taches par image de la vidéo.
`--max-blobs 0` désactive le filtre.

- La médiane fait monter la limite quand le seuil de détection est bas. Même une vidéo stable compte alors des dizaines de taches par image.
  Sur l'extrait de la 092 à `--threshold 15`, une limite fixe de 20 excluait toute la vidéo. La limite relative n'exclut rien.
- La règle a été réglée avec l'ancien seuil fixe de 25.
  Non mesuré : avec un seuil bas, la limite monte (872 taches à `--threshold 12`) et un vrai mouvement pourrait passer dessous.
  Avec les réglages par défaut et leur seuil par pixel, la médiane reste à 0 tache par image sur 089, 090 et 091. La limite reste donc à 20.
- `pad_s` = 1 s (`--unstable-pad` dans le banc) de chaque côté de ces images.
  La fenêtre du fond déborde de ½ s. Les images calmes au milieu d'un mouvement restent aussi suspectes.
  Deux périodes séparées d'au plus 1 s sont fusionnées.

Mesuré avec la v0.1.1 au seuil fixe de 25, le filtre fait passer la 089 de 1 546 à 96 pistes en ignorant 19,5 s.
La 091 passe de 4 327 à 53 pistes (22,3 s ignorées). La 092 reste à 14 pistes et rien n'y est ignoré.
Avec les réglages par défaut, le filtre ignore 8,3 s sur la 089 et 20,6 s sur la 091.
Le glissement de la 089 produit moins de taches au-dessus du seuil par pixel.
Les périodes ignorées sont toujours signalées : bandeau et frise de l'image résumé, bilan de la soirée, `ignored_s` dans `params.json`.

## 4 ter. Images illisibles

Cette étape n'a ni réglage ni option pour la désactiver.
Une image que le décodeur a dû réparer est écartée jusqu'à la prochaine image-clé.
La plage est ensuite élargie de `half_window` de chaque côté.
C'est la demi-fenêtre du fond, déduite de `bg_window_s` : 15 images à 30 i/s.
Logique, mesures et limites : [10 § 5 bis](10-logique-de-detection.md#5-bis-écarter-les-images-que-le-décodeur-a-réparées).

## 5. Suivi

Logique, mesures et cas jugés : [10 § 7](10-logique-de-detection.md#7-relier-les-taches-dans-le-temps--le-suivi).

- `max_jump` = 120 px : une piste prend la tache la plus proche de sa prédiction dans ce rayon.
  Un animal rapide parcourt 60 px par image et peut manquer plusieurs images.
- **Portée** : `reach_margin` = 50 px et `reach_travel` = 1.
  Une piste qui a une vitesse ne prend une tache qu'à moins de `reach_margin` plus `reach_travel` fois le chemin prévu.
  Avec `reach_margin` à 0, la portée est coupée.
  Sur les 26 vidéos avant recollage, une marge de 40 px change 74 pistes et une marge de 50 px en change 56.
  À 60 px, une piste de la 090 saute sur une autre chauve-souris (109 px pour 110 permis). Il en va de même avec 2 fois le chemin.
- `max_gap` = 6 images consécutives sans détection (0,2 s) : le temps qu'une chauve-souris peu contrastée réapparaisse.
  La piste est close à la septième.

## 6. Fusion des pistes jumelles

- `twin_distance` = 36 px : deux pistes qui restent à moins de cette distance sur toutes leurs images communes sont fusionnées ([10 § 8](10-logique-de-detection.md#8-fusionner-les-pistes-jumelles)).
  À 2:44 sur la 092, les fragments d'une chauve-souris peu contrastée étaient écartés de 18 à 24 px.
  `merge_radius` ne pouvait pas les recoller.
  0 désactive la fusion (`--twin-distance 0` dans le banc).

## Rendu

Entre deux détections d'une même piste, la boîte est interpolée linéairement à chaque image et dessinée en trait fin pour qu'elle ne clignote pas.
La trace de la dernière seconde la suit jusque-là.
Ces positions ne servent qu'à l'affichage : le CSV et le banc ne comptent que les vraies détections.
La colonne `images comblées` dit combien d'images ont été comblées.

Boîte, numéro et trace prennent la couleur de la piste, sur l'image résumé comme dans les extraits, les zooms et la vidéo annotée.
La boîte (2 px, 1 px quand elle est interpolée) est bordée d'un liseré noir de 1 px de chaque côté.
Le numéro, placé 6 px au-dessus, a lui aussi un liseré noir de 1 px.
La couleur se détache ainsi de la roche claire comme du ciel sombre, même la plus claire de la palette (235, 235, 235).
La trace n'a pas de liseré.

Le numéro reste au-dessus de la boîte (choix de l'utilisateur, 16 octobre 2026).
On a essayé une autre place, à 24 px en diagonale de la boîte.
Elle ne cachait aucun pixel à moins de 30 px de la cible sur trois images sur quatre, et 16 pixels sur la quatrième.
Au-dessus de la boîte, le numéro en cache de 13 à 277 sur ces quatre images.
Une piste absente de toute image résumé aurait une boîte rouge (255, 0, 0) et une trace jaune orangé (255, 200, 0) sans liseré.
Cela n'arrive pas : chaque piste est dessinée sur une image résumé.

Chaque côté de la boîte dessinée est borné à 4 fois la médiane du même côté sur les points détectés de la piste.
La boîte bornée est ensuite recentrée sur le barycentre.
Une tache étalée (339 × 235,5 px pour 353 px de surface sur la 092 à 3:51) ne dessine donc pas une boîte autour de rien.
Cette borne ne sert qu'à l'affichage : la détection, le suivi et le CSV n'en dépendent pas ([11](11-migration-rust.md#crates), `render`).

Sous macOS, les extraits et la vidéo annotée sont encodés par le moteur multimédia de la puce (VideoToolbox) si un essai d'encodage au démarrage réussit.
Sinon, et sur les autres systèmes, c'est openh264 qui les encode.
Un groupe d'extraits dont VideoToolbox échoue en cours de route est réécrit par openh264. Une ligne le dit.
Les zooms sont toujours encodés par openh264. `params.json` enregistre les encodeurs retenus (`encoders`).
Qualités, essai et mesures : [11](11-migration-rust.md#encodage).

La seconde passe ne relit que les fenêtres des extraits : du début de la piste moins `--clip-margin` à sa fin plus `--clip-margin`.
Les fenêtres qui se chevauchent forment un groupe.
Chaque groupe se lit depuis une image-clé saine qui le précède, avec la même numérotation des images que la détection.
Un seul encodeur écrit tous les extraits du groupe.
Les zooms et la vidéo annotée sont des travaux à part. L'ensemble des travaux se partage la moitié des cœurs.
Un fichier s'écrit sous un nom provisoire et ne prend son nom qu'une fois complet : un extrait qui échoue ou une interruption ne laissent pas de fichier tronqué.

### Extrait zoomé et ralenti

Une cible de quelques pixels qui traverse le champ en 0,2 s ne se voit pas sur l'extrait normal.
Le passage de 3:42.10 de la 092 a été jugé faux, puis reconnu réel en zoomant et en ralentissant dans le lecteur.
Pour ces pistes, un second fichier `extraits/<nom>_zoom.mp4` (`clips/` en anglais) est écrit à côté de l'extrait normal.

- **Quelles pistes** (`--zoom auto`, par défaut) : celles dont la plus grande tache fait moins de 100 px² (`surface max (px)`) **ou** dont le plus fort écart au fond est sous 40 (`amplitude (niveaux de gris)`).
  Les comparaisons sont strictes et portent sur les mêmes grandeurs que le CSV.
  Sur la 092, seule la piste 0:51.62 (36 px², 34,7) a son zoom.
  Le passage de 3:42.10 l'aurait aussi quand un réglage le trouve (1440 px sans seuil par pixel : 70 px², 22,0).
  Le balayage du 7 octobre porte sur 14 vidéos et 848 pistes : 384 ont leur zoom, soit 45 %.
  Cela va de 0 sur la 089 à 83 sur 136 pour la 096. Sur 095 à 099, c'est surtout l'aire qui déclenche.
  `--zoom all` en fait un pour chaque piste et `--zoom none` aucun.
  Les seuils (`zoom_below_area`, `zoom_below_amplitude`) sont dans `params.json` mais pas en option.
  **L'amplitude dépend du seuil, de `--work-width` et du filtre (`target_sigma`)** : le seuil de 40 se remesure quand ces réglages changent.
- **Piste recollée** ([10 § 9 bis](10-logique-de-detection.md#9-bis-recoller-un-vol-caché-derrière-la-végétation)) : elle a son zoom si l'un de ses bouts l'aurait eu seul.
  Les points des morceaux trop courts (§ 7 bis) ne comptent pas.
  Sinon, le bout net ferait perdre son zoom au bout faible.
  Sans la première règle, 10 pistes des 26 vidéos le perdraient. Sans la seconde, 6 le perdraient.
- **Cadre** : fixe pour tout l'extrait.
  C'est la boîte englobante des détections de la piste plus 40 px (pixels d'origine) de chaque côté.
  Elle est élargie au rapport de l'image source (4:3 pour les jumelles) en unités entières.
  Il fait au moins un quart de l'image par côté (360 × 270 sur 1440 × 1080) puis se décale pour rester dans l'image.
  S'il la dépasse, c'est l'image entière.
  Le rapport exact passe avant l'agrandissement : le cadre d'une image sans diviseur commun à ses dimensions (1442 × 1081) est toujours l'image entière.
  L'agrandissement va donc de ×1 à ×4.
  Il vaut ×4 pour la piste 0:51.62 de la 092, et de ×1,29 à ×4 (médiane ×2,77) pour les 39 zooms de la 125.
  Une traversée rapide du champ donne un cadre presque entier : c'est alors le ralenti qui aide.
- **Agrandissement** au plus proche voisin : chaque pixel source devient un carré et rien n'est lissé.
  La sortie a la taille de la vidéo source.
- **Ralenti ×0,25** : le zoom reprend les images de l'extrait normal, sur la même fenêtre et marges comprises.
  Aucune n'est dupliquée ni interpolée. Elles sont déclarées à un quart de la cadence (7,5 i/s pour 30 i/s).
  La piste 0:51.62 dure 3,3 s en extrait normal et 13,2 s en zoom.
- **Dessin** : seulement la trace de la piste du fichier (`trail_s`, 1 s, trait de 1 px après agrandissement).
  Le zoom n'a ni boîte ni numéro et ne montre rien des autres pistes.
  Chaque segment de la trace est coupé à 16 px (pixels d'origine) de la boîte de la cible.
  La trace ne recouvre ainsi jamais une cible ténue, même quand la piste revient sur elle-même.
  Le recadrage se fait sur l'image brute avant le dessin de l'extrait normal. Celui-ci reste identique.
- **Coût** : chaque zoom est un travail à part encodé par openh264. Mesures en [11](11-migration-rust.md#encodage).

## 7. Filtres finaux

- `min_hits` = 6 détections (4 octobre 2026) : élimine les étincelles de quelques images.
  Le passage valide le plus court de la 092 (2:37) en a 7 : ne pas dépasser 7.
  À 5, environ 16 % des pistes de bruit passent par hasard le filtre du virage : elles n'ont que 3 virages.
- `min_travel` = 45 px de bout en bout : élimine un point qui scintille sur place.
- `max_median_turn` = 0,8 rad : élimine les pistes qui zigzaguent ([10 § 9](10-logique-de-detection.md#9-garder-ce-qui-ressemble-à-un-vol)).
  3,15 ou plus désactive le filtre.

`min_travel` et `max_median_turn` visent le scintillement de pixels isolés que le suivi enchaîne (`max_jump` 120 px) sur les vidéos bruitées.
Ce sont des taches d'un pixel de travail, d'un contraste juste au-dessus du seuil.
Elles font 5 à 7 détections et prennent leurs directions au hasard.
Leur virage médian est de 1,5 rad, contre 0,35 au plus pour les vraies pistes de la 092.
Mesurés après le filtre des images saturées, ces deux filtres font passer la 089 de 96 à 4 pistes et la 091 de 53 à 23.
La 092 garde ses 14 pistes.
Au banc, le virage ne coûte aucune cible, même en chasse ou en tournoiement ([08](08-banc-de-mesure.md#vols-de-chasse-4-octobre-2026)).
En transit, 115 cibles sont trouvées sur 253 visibles, comme sans ces filtres. En chasse, on en trouve 357 sur 584 au lieu de 371 sur 592.
Les 14 pertes de la chasse viennent de `min_hits` 6 : des cibles peu contrastées détectées sur 5 images seulement.
Le banc se sert aussi de `min_hits` pour dire qu'une cible est trouvée. Sur les 14, 11 viennent du suivi.

Limite connue du virage : une chauve-souris lente avance de quelques pixels par image.
L'arrondi des positions fait alors tourner sa direction.
La piste lente de 3:29 a un virage médian de 0,22 rad sur toute sa longueur, mais 12 fenêtres de 7 points sur 280 dépassent 0,8.
Un fragment court d'un vol lent serait donc rejeté s'il était coupé par un trou ou une période ignorée.
Le banc ne simule pas de vol lent.
Deux pistes pour y remédier : ignorer les pas trop courts ou mesurer la direction sur 3 images comme la vitesse.

## 7 bis. Recollage

La règle et ses mesures d'ensemble sont en [10 § 9 bis](10-logique-de-detection.md#9-bis-recoller-un-vol-caché-derrière-la-végétation).
Les mesures ci-dessous datent du 17 octobre 2026. Elles portent sur les 26 vidéos de la référence, sans détections faibles ni portée.

- `rejoin_hidden` = 19 images (0,63 s) : le plus long trou recollé.
  Il doit dépasser `max_gap`. À 0, le recollage est désactivé.
  Il couvre la traversée des deux zones de végétation de la 095 à la 100 (8 images en médiane, 12 à 18 pour une traversée sur dix) et les 8 passages jugés « même passage » sur la planche (trous de 8 à 13 images).
  Au-delà, le prolongement ne vaut plus grand-chose.
  Sur des vols continus, prolonger une piste de 20 images manque déjà sa vraie position de 114 px une fois sur deux.
  À 30 images, il la manque de 216 px. Un trou de 5,9 s jugé « non » est hors de portée.
- `rejoin_distance` = 60 px entre le point attendu et le début de la seconde piste.
  Le plus grand écart des 8 passages jugés est de 45 px : 40 px en perdrait un.
  60 px soude 67 paires pour 2,7 au hasard (pistes décalées de 10 ou 20 s). 80 px en soude 74 pour 3,7.
- `rejoin_piece_points` = 2 : les morceaux écartés par `min_hits` qui ont au moins 2 détections relient ou prolongent des pistes gardées.
  Ils ne sont jamais gardés seuls. À 0, aucun morceau n'est pris.
  À 2, on compte 3 ponts et 79 prolongements.
  À 1, on a 35 prolongements de plus, d'une seule image chacun.
  L'un d'eux avance de 8 images le début de la piste 5 de la 092 (1:12.60, une chauve-souris confirmée) sur la foi d'un seul point.
  À 3, 1 pont et 52 prolongements.
- `rejoin_junction_pace` = 0,25 et `rejoin_piece_pace` = 0,45.
  Un morceau ne relie ou ne prolonge une piste que si le saut par-dessus le trou va au moins à 0,25 fois la vitesse de la piste à son bout.
  Le morceau lui-même (2 points ou plus) doit aller au moins à 0,45 fois cette vitesse.
  Sur une planche de 10 morceaux, les 2 jugés douteux allaient à 0,02 à 0,10 (saut) et 0,29 à 0,40 (morceau).
  Les 8 bons allaient au moins à 0,33 (saut) et 0,49 (morceau).
  Les seuils se placent entre les deux groupes. L'écart est mince et ne repose que sur 10 cas.
  Sans ces deux seuils, on compterait 4 ponts et 88 prolongements, dont les deux douteux de la planche.

Les pistes restantes sont numérotées dans l'ordre chronologique, après le recollage.
Le numéro change dès qu'une piste apparaît ou disparaît plus tôt dans la vidéo : pour désigner un passage, on utilise son temps de début.

## Sorties

La liste des fichiers produits est dans [lire les résultats](lire-les-resultats.md#où-sont-les-résultats).
Ce qui suit ne sert qu'au développeur.

`params.json` s'écrit en dernier, une fois les extraits finis.
Il donne :

- les réglages employés, la version et le FFmpeg lié.
- l'état : `done`, ou `too_many_tracks` au-delà de `--max-tracks`.
- le nombre de pistes, la durée, le temps non analysé et le temps de calcul (`track_count`, `length_s`, `not_analysed_s`, `elapsed_s`, en secondes non arrondies).
- la taille de la vidéo en octets (`video_bytes`).
- la langue des noms (`language`, `fr` ou `en`).
- l'heure locale de la fin du calcul (`date`).
- les périodes ignorées (`ignored_s`) et illisibles (`damaged_s`).
- `damaged_frames` : le nombre d'images signalées par le décodeur ou qui suivent un trou dans les horodatages.
  Ce n'est pas le nombre d'images écartées.

`tracks` liste les pistes :

- numéro (`id`).
- début et durée non arrondis (`start_s`, `duration_s`).
- couleur de la trajectoire sur l'image résumé en RVB (`color`, `null` au-delà du garde-fou, qui n'a pas d'image résumé).
- image résumé qui la dessine (`summary`, par exemple `video_097_resume_3_sur_8.png`).
- chemins de l'extrait et du zoom écrits, relatifs au dossier de la vidéo (`null` quand il n'y en a pas).
- point de départ en pixels entiers de la vidéo d'origine (`start_xy`) : il aide la page à rattacher un verdict à sa piste et manque aux dossiers écrits avant #228.

Pour que la page redessine l'image résumé sans rien recalculer, `params.json` donne aussi :

- le fond, la scène sans trace (`background`, par exemple `video_097_fond.png`).
- la mise en page de chaque image résumé (`summaries`) :
  - `file`.
  - `window_s` : la tranche.
  - `part` : la partie, en `[n, total]` ou `null`.
  - `ignored_s` et `damaged_s` : les périodes ignorées et illisibles, coupées à la tranche.
  - `other_starts_s` : les débuts des pistes des autres parties, au centième.
  - `radius` : le rayon des pastilles, en pixels de la vidéo.
- pour chaque piste dessinée, sa trace (`trace`) :
  - `points` et `head` : la trajectoire et les trois points de la flèche, en chaîne `"x,y x,y …"` de pixels entiers de la vidéo, sans point répété.
  - `dot` : le centre de la pastille.
  - `anchor` : le départ du trait de rappel quand la pastille est écartée de la trajectoire, sinon `null`.

Une piste non dessinée a `trace` à `null`.
Un dossier écrit avant #259 n'a ni `background` ni `summaries` : la page montre alors l'image résumé.

Une vidéo déjà faite est sautée.
C'est le cas quand son dossier a un `params.json` terminé et que ni la vidéo, ni la version de batdetect, ni la langue des noms, ni les réglages (sauf la liste des encodeurs) n'ont changé.
Rien n'est écrit ni effacé dans son dossier.
Le bilan reprend sa durée, ses pistes et son temps non analysé. Il affiche « — » en calcul et compte « N déjà faites ».
Après un Ctrl-C, relancer la même commande reprend là où le lot s'est arrêté. `--force` refait toutes les vidéos.

Une fois la détection finie, chaque calcul efface d'abord `params.json` dans le dossier de la vidéo.
Il efface ensuite les sorties des deux langues et celles des versions précédentes (`split/`, `<nom>.tracks.*`, `<nom>_boxes.mp4`).
Une vidéo sautée parce que déjà faite n'est pas concernée.
Les sorties de la v0.1.1 restent où elles sont, dans le `out/` du dossier où la commande était lancée.

Garde-fou : au-delà de `max_tracks` pistes, ni les extraits, ni l'image résumé, ni son fond, ni la vidéo annotée ne sont produits.
`max_tracks` vaut 300 par défaut et 0 le désactive.
Dans `params.json`, `background` est alors à `null`, `summaries` vide et `trace` à `null`.
L'image résumé ne servirait à rien : sur une vidéo de bruit, elle se coupait en dizaines de parties.
Le CSV et `params.json` restent écrits, avec le seuil (dans `RenderConfig`) et `"state": "too_many_tracks"`.
300 est au-dessus de la vidéo la plus chargée sans explosion (148 pistes, 097) et en dessous de la plus petite explosion connue (374 pistes, seuil 12 sans facteur de bruit).
Le seuil est fixe, pas un taux par minute. Il suppose des clips d'environ 5 min comme ceux des jumelles.
Pour une vidéo plus longue, on le monte avec `--max-tracks`.
Le bilan de la soirée compte ces vidéos à part, avec leur nombre de pistes.
Une relance identique ne les refait pas et les compte encore ainsi.

Colonnes du CSV, dans cet ordre :

- `piste` : le numéro.
- `début` et `fin` : en `h:mm:ss`, tronqués à la seconde comme le nom de l'extrait.
- `durée (s)`.
- `points` : le nombre de détections.
- `corde (px)` : la distance de bout en bout.
- `chemin (px)` : le chemin parcouru.
- `vitesse (px/s)` : chemin ÷ durée.
- `surface max (px)` : la plus grande tache, en pixels d'origine.
- `amplitude (niveaux de gris)` : le plus fort écart au fond.
  Avec le filtre à la taille de la cible, il est mesuré sur l'image filtrée et reste donc plus bas que l'écart brut pour une petite tache.
- `images comblées` : les images comblées par interpolation à l'affichage.
- `début (s)` : le début en secondes, au centième.

Un début dans le dernier centième d'une seconde s'écrit `0:00:04` d'un côté et `5,0` de l'autre : `début` tronque, `début (s)` arrondit.
`0:04` serait lu par LibreOffice comme 0 h 04 min : d'où `0:00:04`.

Le tableau appartient à l'outil : un recalcul le remplace sans le lire.
Les verdicts n'y sont pas. Ils se notent dans la page et vivent à côté d'elle dans `verdicts.csv`.
L'outil lit ce fichier sans jamais l'écrire ([11](11-migration-rust.md#crates), crate `page`).

Le CSV suit la langue des messages.
En français, `_pistes.csv` commence par un BOM UTF-8 pour qu'Excel lise les accents.
Il sépare par un point-virgule et écrit la virgule décimale.
En anglais, `_tracks.csv` n'a pas de BOM, sépare par une virgule et écrit le point.
Ses en-têtes sont `track`, `start`, `end`, `duration (s)`, `points`, `chord (px)`, `path (px)`, `speed (px/s)`, `max area (px)`, `amplitude (grey levels)`, `filled frames`, `start (s)`.
Les fins de ligne sont en CRLF dans les deux langues.

### Place prise

On l'a mesurée le 16 octobre 2026 sur lumiel, avec le binaire release 0.2.0-19-g709ee52, openh264 et les réglages par défaut.
Les 26 vidéos de `BATDETECT_VIDEOS_DIR` sont passées une par une, avec `--annotated` sur 6 d'entre elles.
Chaque dossier de résultats a été relevé puis effacé.
« Sorties » compte tout le dossier de la vidéo sauf la vidéo annotée : extraits, zooms, images résumé, tableau, `params.json`.

| Vidéo | Vidéo (Mo) | Pistes | Sorties (Mo) | Sorties ÷ vidéo | Vidéo annotée ÷ vidéo |
|---|---:|---:|---:|---:|---:|
| video_089 | 467,2 | 1 | 15,2 | 0,032 | 2,71 |
| video_090 | 466,5 | 20 | 440,7 | 0,945 | 2,72 |
| video_091 | 451,1 | 21 | 357,7 | 0,793 | 2,61 |
| video_092 | 454,1 | 14 | 195,6 | 0,431 | 1,97 |
| video_093 | 458,0 | 6 | 89,9 | 0,196 |  |
| video_094 | 461,2 | 14 | 245,7 | 0,533 |  |
| video_095 | 447,7 | 78 | 1455,0 | 3,250 |  |
| video_096 | 443,2 | 136 | 2787,9 | 6,291 |  |
| video_097 | 434,4 | 149 | 2995,6 | 6,896 | 2,45 |
| video_098 | 450,1 | 131 | 2872,0 | 6,381 |  |
| video_099 | 474,8 | 117 | 2614,7 | 5,507 |  |
| video_100 | 446,7 | 98 | 2212,5 | 4,954 |  |
| video_101 | 449,5 | 32 | 821,1 | 1,827 |  |
| video_102 | 460,7 | 36 | 1025,8 | 2,227 |  |
| video_103 | 463,3 | 10 | 256,3 | 0,553 |  |
| video_120 | 444,9 | 0 | 0,9 | 0,002 | 0,64 |
| video_121 | 468,7 | 2 | 16,1 | 0,034 |  |
| video_122 | 449,8 | 0 | 1,1 | 0,002 |  |
| video_123 | 447,9 | 8 | 48,7 | 0,109 |  |
| video_124 | 457,6 | 5 | 35,5 | 0,078 |  |
| video_125 | 458,0 | 47 | 327,8 | 0,716 |  |
| video_126 | 465,2 | 13 | 111,3 | 0,239 |  |
| video_127 | 437,3 | 7 | 66,5 | 0,152 |  |
| video_128 | 432,5 | 5 | 59,1 | 0,137 |  |
| video_129 | 426,5 | 16 | 157,4 | 0,369 |  |
| video_130 | 248,2 | 6 | 49,7 | 0,200 |  |

- Les sorties vont de 0,002 à 6,9 fois la vidéo, avec une médiane de 0,48.
  Elles suivent le nombre et la durée des passages, qu'on ne connaît pas avant la détection.
  Les soirées chargées (095 à 100, 78 à 149 pistes) dépassent 3 fois la vidéo. Les calmes restent sous 1 fois.
- La vidéo annotée pèse de 0,64 fois la vidéo (120, sans piste) à 2,72 fois (090).
- Un extrait pèse de 0,81 à 3,17 fois le débit de la vidéo d'entrée par seconde de fenêtre (piste et marges de 1,5 s).
  Un zoom pèse de 0,29 à 2,63 fois ce débit.
  À débit d'entrée égal, 089 à 103 sont nettement au-dessus de 121 à 130.

Coefficients retenus (`space`, décision de l'utilisateur du 16 octobre) :

- au lancement, 1 fois la vidéo : 18 soirées sur 26 tiennent dessous.
  Une soirée chargée le dépasse et c'est alors le contrôle par vidéo qui la protège.
- vidéo annotée : 2,8 fois la vidéo, le plus prudent mesuré arrondi au-dessus.
- par vidéo, avant les extraits : 3,2 fois le débit d'entrée par seconde d'extrait et 2,7 par seconde de zoom, les plus prudents mesurés arrondis au-dessus.

VideoToolbox écrit environ 0,77 fois ce qu'écrit openh264 ([11](11-migration-rust.md#encodage)) : sur le Mac, ces coefficients sont donc plus prudents encore.
Le comportement des deux contrôles est dans [11](11-migration-rust.md#crates), crate `cli`.
