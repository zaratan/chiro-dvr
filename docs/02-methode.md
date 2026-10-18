# Méthode

Le traitement d'une vidéo se fait en deux passes. La première lit la vidéo en petit, détecte et suit.
La seconde relit la vidéo en pleine résolution, seulement autour des pistes, dessine les boîtes et encode directement chaque extrait, l'extrait zoomé et ralenti des pistes petites ou faibles, et la vidéo annotée complète avec `--annotated` ([Rendu](#rendu)).

**Règle de réglage** (décision du 9 octobre 2026, après le verdict de Manon sur la 092) : mieux vaut un faux positif facile à écarter à l'extrait qu'un passage manqué.
Quand un seuil hésite, il penche vers la sensibilité ; l'extrait, zoomé et ralenti s'il le faut, sert à écarter.
La piste lente de 3:29 sur la 092 reste produite pour cette raison.

Toutes les distances et surfaces des réglages, des pistes et du CSV sont en **pixels de la vidéo d'origine**.
Pour détecter, la vidéo est réduite à `work_width` px de large (960 par défaut depuis le 6 octobre 2026, 480 auparavant ; jamais plus que la vidéo elle-même) ; les détections sont reconverties en pixels d'origine dès leur création.
Changer `work_width` ne change donc pas le sens des autres réglages, **à l'arrondi près** : à 960 px de large sur une vidéo 1440×1080, un pixel de travail vaut 1,5 px d'origine et 2,25 px² ; à 480 px, 3 px et 9 px².
`merge_radius` est arrondi au pixel de travail.
Les pixels d'origine ne sont pas une unité physique : une autre caméra, avec une autre résolution, demandera peut-être d'autres valeurs.
Les valeurs par défaut ont été choisies sur sept vidéos 1440×1080 de la falaise (092, 089, 090, 091, 125, 126, 127 ; [08](08-banc-de-mesure.md#sur-six-autres-vidéos)).

Ce document tient la valeur de chaque réglage et sa justification ; la logique de chaque étape et son module sont dans [10](10-logique-de-detection.md).
Les réglages portent ici leur nom de `params.json`.
Seuls quatre sont des options de la commande : `--threshold`, `--min-hits`, `--max-blobs` et `--work-width` ; `batdetect --help` affiche leurs défauts.
Les autres ont une valeur fixe, écrite dans `params.json`, et restent réglables par les options du banc ([08](08-banc-de-mesure.md)) : `--target-sigma`, `--noise-factor`, `--unstable-pad`, `--twin-distance` et les autres options citées ci-dessous en sont.
La v0.1.1 avait en plus deux modes (`--mode quick` : 480 px, seuil fixe 25, sans filtre ni seuil par pixel) ; le Rust n'en a plus.

La détection se partage entre tous les cœurs sans changer le résultat, au bit près ([10](10-logique-de-detection.md#10-aller-vite-sans-changer-le-résultat)).

## 1. Fond par médiane glissante

Pour chaque image *t*, le fond est la médiane, pixel par pixel, des images de *t − ½ fenêtre* à *t + ½ fenêtre*, une sur `bg_step`.

- `bg_window_s` = 1,0 s.
  Une chauve-souris traverse le champ en 0,3 à 1,3 s (mesuré sur la 092) : sur une seconde, elle n'occupe un pixel donné que sur quelques images, donc la médiane l'efface.
  Une fenêtre plus longue suivrait mal les changements de lumière ; plus courte, elle garderait l'animal dans le fond.
- `bg_step` = 3 : 11 images au lieu de 31 pour la médiane, puisque le fond change lentement.
  Un `bg_step` qui laisse moins de 3 images dans la fenêtre (16 et plus à 30 i/s) est refusé : le fond ne serait plus une médiane mais une ou deux images (moyennées).
- La médiane est calculée par un réseau de tri sur des valeurs entières (au 1/64 de niveau), qui ne fixe que la ou les deux valeurs du milieu ; pour un nombre pair de valeurs, le fond est leur demi-somme ([10](10-logique-de-detection.md#2-estimer-le-fond--la-médiane-temporelle)).
  Une fenêtre dont les images dépasseraient 4 Gio de mémoire est refusée.
- Aux bords de la vidéo, la fenêtre est tronquée : la première et la dernière demi-seconde sont analysées avec moins d'images.

## 2. Compensation du gain automatique

Les jumelles ajustent leur gain : toute l'image peut s'éclaircir d'un coup.
On retire au résidu l'écart entre la luminosité moyenne de l'image et celle des images du fond.
Le fond médian, lui, n'est pas corrigé : pendant la demi-seconde où la fenêtre mélange des images avant et après un saut, le résidu garde une partie du saut.
Avec l'ancien seuil fixe de 25, un saut global de 30 niveaux ne produisait aucune détection.
Avec les réglages par défaut (seuil 12, filtre, seuil par pixel), mesuré avec la v0.1.1 sur des images fabriquées : un saut de 10 donne au plus une tache, un saut de 20 jusqu'à 10 taches par image sur 11 images, un saut de 30 à 40 des centaines de taches par image.
Aucune piste n'en sort (testé de 10 à 40) : pour 30 et 40, le filtre des images saturées (§ 4 bis) ignore 2,6 à 2,8 s autour du saut.

## 3. Seuil dans les deux sens

Un pixel est retenu si |image − fond − écart de gain| > max(`threshold`, `noise_factor` × σ du pixel), sur l'image filtrée.

- `threshold` = 12 niveaux de gris sur 255, mesurés sur l'image filtrée : c'est le **plancher** du seuil par pixel ci-dessous.
  Seul, sans seuil par pixel, il laisse passer le bruit de fond de certaines vidéos (089, 090, 091 : des centaines de pistes) ; avant le 6 octobre 2026, le seuil était fixe à 25 sur l'image non filtrée à 480 px.
- La valeur absolue rend la détection indifférente au signe : tache sombre sur roche chaude ou tache claire sur ciel froid.
- **Filtre à la taille de la cible** (`--target-sigma s`, en pixels d'origine, 1,5 par défaut, 0 pour le désactiver) : chaque image réduite passe par un flou gaussien d'écart-type `s` avant le fond et la compensation du gain.
  Le filtre étant linéaire, le fond, le résidu, le bruit du seuil par pixel et `peak_amplitude` sont tous mesurés sur l'image filtrée.
  C'est le filtre adapté à une tache gaussienne : il moyenne le bruit d'un pixel à l'autre et garde une tache de la même taille.
  Il remplace le débruitage qu'apporte la réduction à 480 px, et rend utilisables des largeurs de travail plus grandes.
  Les images filtrées sont gardées en entiers, au 1/64 de niveau.
  Une zone `--osd-region` est élargie de la portée du flou, ⌈4 σ⌉ pixels de travail (σ = `s` ramené à la largeur de travail : 4 px à 960 sur 1440), pour couvrir l'étalement de l'affichage.
  `min_area` et `max_area` s'appliquent à la tache filtrée, plus large que la tache brute.
  Mesures : [08](08-banc-de-mesure.md#filtre-à-la-taille-de-la-cible-b2).
- **Seuil par pixel** (`--noise-factor k`, 8 par défaut, 0 pour revenir au seuil fixe) : le seuil d'un pixel devient max(`threshold`, k × σ), où σ = 1,4826 × la MAD des écarts au fond des images de la fenêtre (celles de la médiane, 11 à 30 i/s), chacune corrigée de son écart de gain et centrée sur leur médiane.
  Elle n'est calculée que sur les pixels dont le résidu dépasse déjà `threshold`, seuls à pouvoir être retenus.
  `threshold` devient le plancher : on l'abaisse pour gagner en sensibilité sur la roche calme, et la MAD relève le seuil là où l'image s'agite.
  C'est lui qui empêche le bruit de fond des vidéos agitées de devenir des pistes.
  La carte ne dépend que de la fenêtre : le résultat ne dépend pas du découpage du travail entre les fils.
  Aux bords de la vidéo, quand la fenêtre tronquée compte moins de 7 échantillons (une image sur `bg_step`), le seuil fixe s'applique seul.
  Pour la même raison, le seuil par pixel est **désactivé sur toute la vidéo** sous 17 images par seconde avec `bg_step` = 3, ou dès `bg_step` = 6 à 30 i/s : il ne reste que le plancher de 12, qui laisse passer le bruit de fond des vidéos agitées.
  Toutes nos vidéos sont à 30 i/s. Le ciel saturé à 0 a une MAD nulle et aucun résidu sombre possible : le seuil par pixel n'y gagne rien.
  Mesures et coût : [08](08-banc-de-mesure.md#seuil-par-pixel-d1).
- Aucun masque par défaut (voir [01](01-contexte.md)).
  `--osd-region x0,y0,x1,y1`, répétable, en fractions de l'image, met à zéro une zone d'affichage qui bougerait, et la masque aussi sur le fond de l'image résumé.
  Pour les Symbion, `0,0,1,0.07` cache l'heure, la date, la batterie et le chrono ; mesuré sur l'original de la 092 : mêmes 14 pistes, mêmes débuts et mêmes fins que sans masque.
  Les anciennes bandes du haut et du bas (7 et 10 %) coupaient 5 pistes sur 14.

## 4. Taches

- `merge_radius` = 6 px : une fermeture morphologique recolle les fragments d'un même animal séparés de moins de ~12 px.
  Sur la 092 à 3:51, une seule chauve-souris sortait en deux taches distantes de 12 à 18 px.
- `min_area` = 4 et `max_area` = 2 700 px².
  L'aire compte les pixels réellement au-dessus du seuil, pas ceux ajoutés par la fermeture, sur l'image filtrée.
  **À 960 px de large, un pixel de travail vaut 2,25 px² : `min_area` 4 demande donc deux pixels de travail.**
  Une tache d'un seul pixel est rejetée ; c'est ce qui fait perdre une image du passage de 3:42.10 sur la 092 à 960 px sans seuil par pixel ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)).
  À 480 px, 4 px² laissait passer un seul pixel de travail.
  Mesuré au banc : 9 ou 18 px² perdent les petites cibles faibles sans retirer de bruit.

## 4 bis. Images saturées de taches

Quand les jumelles bougent, même d'une fraction de pixel par image, les images de la fenêtre du fond ne sont plus alignées : chaque contour contrasté laisse un résidu, et une seule image compte des centaines de taches.
Sur la 089 (cadre qui glisse d'environ 4 px entre 4,5 et 19 s), cela donnait 1 452 fausses pistes en 20 s ; sur la 091 (deux gros mouvements vers 3:57 et 4:06), 4 270.
Un décalage figé, lui, est rattrapé par la fenêtre de 1 s : c'est le glissement qui crée les pistes.

Une image compte comme **saturée** si elle a au moins `max_blobs` = 20 taches de plus que 6 fois la médiane des taches par image de la vidéo.
La médiane fait monter la limite avec un seuil de détection bas, où même une vidéo stable compte des dizaines de taches par image : sur l'extrait de la 092 à `--threshold 15`, une limite fixe de 20 excluait toute la vidéo, la limite relative n'exclut rien.
Non mesuré : avec un seuil bas, la limite monte (872 taches à `--threshold 12`) et un vrai mouvement pourrait passer dessous ; la règle a été réglée à l'ancien seuil fixe de 25.
Avec les réglages par défaut actuels (seuil par pixel), la médiane reste à 0 tache par image sur 089, 090 et 091 et la limite à 20.
Secondes ignorées : 089 8,3 s contre 19,5 avec l'ancien réglage, 091 20,6 s contre 22,3 ; le glissement de la 089 produit moins de taches au-dessus du seuil par pixel (mesures de la v0.1.1).
Les périodes ignorées sont ces images élargies de `pad_s` = 1 s (`--unstable-pad` dans le banc) de chaque côté (la fenêtre du fond déborde de ½ s, et les images calmes au milieu d'un mouvement restent suspectes), fusionnées quand au plus 1 s les sépare.
Leurs détections sont retirées avant le suivi ; la détection elle-même ne change pas. Une piste qui traverse une période est coupée en deux.

Mesuré sur les détections réelles : cadre stable, au plus 12 taches par image (089, 091) et au plus 4 sur toute la 092 ; cadre qui bouge, plus de 270 en médiane.
Résultat : 089 1 546 → 96 pistes (19,5 s ignorées), 091 4 327 → 53 (22,3 s), 092 inchangée (14 pistes, rien d'ignoré).
Le nombre de taches sert d'indicateur parce qu'il est gratuit : mesurer le décalage du cadre par corrélation de phase coûterait 3,2 ms par image, environ 29 s par vidéo de 5 min (mesuré avec la v0.1.1).

La règle repère des images saturées, pas seulement des mouvements : sur la 089, entre 7,5 et 10,7 s, des images à plus de 20 taches apparaissent sans mouvement mesurable.
Un essaim d'au moins 20 chauves-souris dans la même image serait aussi ignoré : les périodes sont toujours signalées (bandeau de l'image résumé, « hors analyse 0:00 - 0:09 », trois lignes au plus puis « + N autres » ; frise du temps ; temps non analysé dans le bilan de la soirée ; `params.json` sous `ignored_s`), et `--max-blobs 0` désactive le filtre.
Le banc applique le même filtre et retire des cibles visibles celles qui tombent dans une période ignorée.
Les images illisibles (4 ter) sont retirées avant le comptage : elles ne déclenchent plus de période et ne pèsent pas sur la médiane des taches.

## 4 ter. Images illisibles

La lecture de la détection signale les images que le décodeur a dû réparer, et l'image qui suit un trou dans les horodatages (plus d'une image manquante au pas habituel).
Chacune est écartée jusqu'à l'image qui précède la prochaine image-clé, puis la plage est élargie de la demi-fenêtre du fond de chaque côté (`half_window`, 15 images à 30 i/s, déduite de `bg_window_s`), avant la fusion des plages, la stabilité et le suivi.
Pas d'option pour désactiver. Une vidéo dont il manque des images sans explication est comptée en échec.

Mesuré avec la v0.1.1 (le Rust donne les mêmes plages sur 25 des 26 vidéos de la référence, [10](10-logique-de-detection.md#5-bis-écarter-les-images-que-le-décodeur-a-réparées)) : la 092 (aucune erreur) est inchangée au bit près.
La 093 (11 erreurs) a 14,0 s illisibles en 8 plages, une seule période instable au lieu de quatre, et 3 pistes au lieu de 4.
La 122 (195 erreurs, 13 images jamais décodées) a 201,2 s illisibles en 79 plages, plus aucune période instable, et plus aucune piste au lieu de 25.
Sans la marge, il restait sur la 122 50,3 s instables et 4 pistes, toutes collées aux plages.
Les raisons et les limites sont dans [10](10-logique-de-detection.md).
Ces images sortent de la lecture même de la détection : les repérer ne coûte rien de plus.

## 5. Suivi

Chaque image, les détections sont attribuées aux pistes ouvertes.

- **Prédiction** à vitesse constante, la vitesse étant mesurée entre le dernier point et le plus récent situé au moins 3 images avant (une période de la saccade des jumelles).
  Avec les deux derniers points seulement, l'erreur atteignait un facteur 2 une image sur trois ; sur la 092, la piste la plus rapide (3:48) décrochait 2 images trop tôt.
  Une piste de deux points garde la prédiction sur ces deux points.
- **Appariement** au plus proche dans un rayon `max_jump` = 120 px, mais les pistes qui ont déjà une vitesse passent avant celles d'un seul point.
  Sans cette priorité, une piste naissante (un fragment) volait le point de la vraie piste : le passage de 3:58 sortait coupé en deux.
- **Portée** : une piste qui a une vitesse ne prend une tache qu'à moins de `reach_margin` = 50 px plus `reach_travel` = 1 fois le chemin prévu (sa vitesse fois les images depuis son dernier point) de l'endroit prévu ; `reach_margin` à 0 coupe la portée.
  Sur les 26 vidéos, avant recollage, 40 px change 74 pistes, 50 px 56, et 60 px, comme 2 fois le chemin, laisse une piste de la 090 sauter sur une autre chauve-souris (109 px pour 110 permis).
  Ne sont pas des réglages : la marge de 30 px au bord, celle du recollage, où la portée ne joue pas ; et la vitesse effondrée, sous 0,4 fois la vitesse des 5 derniers points, remplacée par celle-ci (même résultat de 0,25 à 0,6 sur les cas jugés).
  Logique, mesures et cas jugés : [10 § 7](10-logique-de-detection.md#7-relier-les-taches-dans-le-temps--le-suivi).
- **Trous** : une piste survit à `max_gap` = 6 images consécutives sans détection (0,2 s), le temps qu'une chauve-souris peu contrastée réapparaisse ; elle est close à la septième.
  Les tests de borne vérifient exactement ces deux cas.

## 6. Fusion des pistes jumelles

Deux pistes qui coexistent et restent à moins de `twin_distance` = 36 px l'une de l'autre sur **toutes** leurs images communes sont un seul animal fragmenté : elles sont fusionnées (boîte englobante, centre pondéré par l'aire).
Deux vraies chauves-souris s'écartent au moins une fois et restent séparées.
Sur la 092 à 2:44, une chauve-souris peu contrastée sortait en fragments écartés de 18 à 24 px, trop pour `merge_radius`.

`twin_distance` = 0 désactive la fusion (`--twin-distance 0` dans le banc).

## Rendu

Entre deux détections d'une même piste, la boîte est interpolée linéairement à chaque image et dessinée en trait fin, pour qu'elle ne clignote pas ; la trace de la dernière seconde la suit jusqu'à cette position.
Ces positions servent uniquement à l'affichage : le CSV et le banc ne comptent que les vraies détections, et la colonne `images comblées` indique combien d'images ont été comblées.

Boîte, numéro et trace prennent la couleur de la piste sur l'image résumé, dans les extraits, les zooms et la vidéo annotée (#13).
La boîte (2 px, 1 px interpolée) est bordée d'un liseré noir de 1 px de chaque côté, et le numéro, 6 px au-dessus, d'un liseré noir de 1 px : la couleur se détache de la roche claire comme du ciel sombre, même la plus claire de la palette (235, 235, 235).
Le numéro garde sa place au-dessus de la boîte (choix de l'utilisateur, 16 octobre 2026).
L'autre place essayée, à 24 px en diagonale de la boîte, ne cachait aucun pixel à moins de 30 px de la cible sur trois images sur quatre, et 16 pixels sur la quatrième ; à sa place, le numéro en cache de 13 à 277 sur ces quatre images.
La trace n'a pas de liseré.
Une piste absente de l'image résumé garderait la boîte rouge (255, 0, 0) et la trace jaune orangé (255, 200, 0) sans liseré ; cela n'arrive pas aujourd'hui, chaque piste étant dessinée sur une image résumé.

Chaque côté de la boîte dessinée est borné à 4 fois la médiane du même côté sur les points détectés de la piste, et la boîte bornée est recentrée sur le barycentre : une tache étalée (339 × 235,5 px pour 353 px de surface sur la 092 à 3:51) ne dessine plus une boîte autour de rien.
Cette borne ne sert qu'à l'affichage ; la détection, le suivi et le CSV ne changent pas ([11](11-migration-rust.md#crates), `render`).

Sous macOS, les extraits et la vidéo annotée sont encodés par le moteur multimédia de la puce (VideoToolbox) quand un essai d'encodage au démarrage réussit, sinon par openh264 ; ailleurs, par openh264.
Un groupe d'extraits dont VideoToolbox échoue en cours de route est réécrit par openh264, et une ligne le dit.
Les zooms sont toujours encodés par openh264. `params.json` enregistre les encodeurs retenus (`encoders`).
Qualités, essai et mesures : [11](11-migration-rust.md#encodage).

La seconde passe ne relit que les fenêtres des extraits (début de la piste moins `--clip-margin`, fin plus `--clip-margin`).
Les fenêtres qui se chevauchent forment un groupe ; chaque groupe se lit depuis une image-clé saine qui le précède, avec la même numérotation des images que la détection, et un seul encodeur écrit tous les extraits du groupe.
Les zooms et la vidéo annotée sont des travaux à part. Les travaux se partagent la moitié des cœurs.
Un fichier s'écrit sous un nom provisoire et ne prend son nom qu'une fois complet : un extrait qui échoue ou une interruption ne laissent pas de fichier tronqué.

### Extrait zoomé et ralenti

Une cible de quelques pixels qui traverse le champ en 0,2 s ne se voit pas sur l'extrait normal (passage de 3:42.10 de la 092, jugé faux puis reconnu réel en zoomant et en ralentissant dans le lecteur, #37).
Pour ces pistes, un second fichier `extraits/<nom>_zoom.mp4` (`clips/` en anglais) est écrit à côté de l'extrait normal.

- **Quelles pistes** (`--zoom auto`, par défaut) : celles dont la plus grande tache fait moins de 100 px² (`surface max (px)`) **ou** dont le plus fort écart au fond est sous 40 (`amplitude (niveaux de gris)`), comparaisons strictes, mêmes grandeurs que le CSV.
  Sur la 092, seule la piste 0:51.62 (36 px², 34,7) a son zoom ; le passage de 3:42.10, quand un réglage le trouve (1440 px sans seuil par pixel : 70 px², 22,0), l'aurait aussi.
  Sur le balayage du 7 octobre (14 vidéos, 848 pistes), 384 pistes ont leur zoom (45 %), de 0 sur la 089 à 83 sur 136 pour la 096 ; sur 095 à 099, c'est surtout l'aire qui déclenche.
  Une piste recollée ([10 § 9 bis](10-logique-de-detection.md#9-bis-recoller-un-vol-caché-derrière-la-végétation)) a son zoom si l'un de ses bouts l'aurait eu seul.
  `--zoom all` en fait un pour chaque piste, `--zoom none` aucun.
  Les seuils (`zoom_below_area`, `zoom_below_amplitude`) sont dans `params.json` mais pas en option.
  **L'amplitude dépend du filtre (`target_sigma`), du seuil et de `--work-width`** : le seuil de 40 se remesure quand ces réglages changent.
- **Cadre** : fixe pour tout l'extrait.
  Boîte englobante des détections de la piste, plus 40 px (pixels d'origine) de chaque côté, élargie au rapport de l'image source (4:3 pour les jumelles) en unités entières, au moins un quart de l'image par côté (360 × 270 sur 1440 × 1080), puis décalée pour rester dans l'image ; si elle dépasse l'image, c'est l'image entière.
  Le rapport exact passe avant l'agrandissement : pour des dimensions sans diviseur commun (1442 × 1081), le cadre est toujours l'image entière.
  L'agrandissement va donc de ×1 à ×4 : ×4 pour la piste 0:51.62 de la 092, ×1,29 à ×4 (médiane ×2,77) pour les 39 zooms de la 125.
  Une traversée rapide du champ donne un cadre presque entier : le ralenti fait alors l'essentiel.
- **Agrandissement** au plus proche voisin : chaque pixel source devient un carré, rien n'est lissé.
  La sortie a la taille de la vidéo source.
- **Ralenti ×0,25** : les mêmes images que l'extrait normal (même fenêtre, marges comprises), aucune dupliquée ni interpolée, déclarées à un quart de la cadence (7,5 i/s pour 30 i/s).
  La piste 0:51.62 dure 3,3 s en extrait normal, 13,2 s en zoom.
- **Dessin** : seulement la trace de la piste du fichier (`trail_s`, 1 s, trait de 1 px après agrandissement), sans boîte ni numéro, et rien des autres pistes.
  Chaque segment de la trace est coupé à 16 px (pixels d'origine) de la boîte de la cible, pour ne pas recouvrir une cible ténue, même quand la piste revient sur elle-même.
  Le recadrage se fait sur l'image brute, avant le dessin de l'extrait normal, qui reste identique.
- **Coût** : chaque zoom est un travail à part, encodé par openh264.
  Coût mesuré : [11](11-migration-rust.md#encodage) ; celui de la v0.1.1 : [09](09-profilage.md#extraits-zoomés-issue-37).

## 7. Filtres finaux

- `min_hits` = 6 détections (depuis le 4 octobre ; 5 avant) : élimine les étincelles de quelques images.
  Le passage valide le plus court de la 092 (2:37) en a 7 : ne pas dépasser 7.
- `min_travel` = 45 px de bout en bout : élimine un point qui scintille sur place.
- `max_median_turn` = 0,8 rad : élimine les pistes qui zigzaguent.
  Le virage d'une piste est l'angle entre deux pas successifs ; on en prend la médiane, qui tolère un virage brusque isolé.
  Un pas nul (même pixel rallumé) est ignoré ; une piste trop courte pour avoir un virage est gardée. 3,15 ou plus désactive le filtre.

Ces deux derniers réglages visent le scintillement de pixels isolés que le suivi enchaîne (`max_jump` 120 px) sur les vidéos bruitées : taches d'un pixel de travail, contraste juste au-dessus du seuil, 5 à 7 détections, directions au hasard (virage médian 1,5 rad, contre au plus 0,35 pour les vraies pistes de la 092).
Mesuré, après le filtre des images saturées : 089 96 → 4 pistes, 091 53 → 23, 092 inchangée (14).
Banc : transit 115 trouvées sur 253 visibles comme avant, chasse 357 sur 584 au lieu de 371 sur 592 ; le virage ne coûte aucune cible, même en chasse ou en tournoiement ([08](08-banc-de-mesure.md#vols-de-chasse-4-octobre-2026)), les 14 pertes viennent de `min_hits` 6 (cibles peu contrastées détectées sur 5 images seulement ; le banc se sert aussi de `min_hits` pour dire qu'une cible est trouvée, 11 des 14 viennent du suivi).

Limite connue : une chauve-souris lente avance de quelques pixels par image, et l'arrondi des positions fait tourner sa direction.
La piste lente de 3:29 a un virage médian de 0,22 rad sur toute sa longueur, mais 12 fenêtres de 7 points sur 280 dépassent 0,8 : un fragment court d'un vol lent (coupé par un trou ou une période ignorée) serait rejeté.
Le banc ne simule pas de vol lent. Piste : ignorer les pas trop courts, ou mesurer la direction sur 3 images comme la vitesse.
Avec 5 détections, environ 16 % des pistes de bruit de 5 points passent le virage par hasard (3 virages seulement).

## 7 bis. Recollage

Deux pistes gardées sont un même vol caché un moment quand la seconde démarre après un trou de plus de `max_gap` images et d'au plus `rejoin_hidden` images (qui doit donc dépasser `max_gap`), à au plus `rejoin_distance` px du point attendu : la fin de la première, prolongée sur le trou à la moyenne des vitesses des deux bouts (5 points chacun), comme si l'animal virait au milieu du trou.
Jamais à travers une période ignorée, jamais quand un des deux bouts est à moins de 30 px d'un bord.
Logique et mesures : [10 § 9 bis](10-logique-de-detection.md#9-bis-recoller-un-vol-caché-derrière-la-végétation).

- `rejoin_hidden` = 19 images (0,63 s ; depuis le 17 octobre 2026) : couvre la traversée des deux zones de végétation de la 095 à la 100 (8 images en médiane, 12 à 18 pour une traversée sur dix) et les 8 passages jugés « même passage » sur la planche (trous de 8 à 13 images).
  0 désactive le recollage. Un trou de 5,9 s jugé « non » est hors de portée.
- `rejoin_distance` = 60 px : le plus grand écart des 8 passages jugés est de 45 px, donc 40 px en perdrait un ; sur les 26 vidéos, 60 px soude 67 paires pour 2,7 au hasard (pistes décalées de 10 ou 20 s), 80 px 74 pour 3,7.
- La marge de 30 px au bord n'est pas un réglage : c'est là que l'excès de fins de piste près des bords retombe au niveau du reste de l'image.
- `rejoin_piece_points` = 2 : les morceaux écartés par `min_hits` qui ont au moins 2 détections relient ou prolongent des pistes gardées, jamais seuls.
  À 1, 35 prolongements de plus, d'une seule image chacun, dont le début de la piste 5 de la 092 avancé de 8 images ; à 3, 1 pont au lieu de 3 et 52 prolongements au lieu de 79.
  0 n'en prend aucun.
- `rejoin_junction_pace` = 0,25 et `rejoin_piece_pace` = 0,45 : un morceau ne relie ou ne prolonge une piste que si le saut par-dessus le trou va au moins à 0,25 fois la vitesse de la piste à son bout, et le morceau lui-même (s'il a 2 points ou plus) au moins à 0,45 fois.
  Sur une planche de 10 morceaux, les 2 jugés douteux allaient à 0,02 à 0,10 (saut) et 0,29 à 0,40 (morceau), les 8 bons à 0,33 au moins (saut) et 0,49 au moins (morceau).
  Le saut vaut aussi entre deux pistes gardées, contre la vitesse de chacune : il refuse 2 des 67 soudures, où la seconde piste repart presque là où la première s'arrête, jugées très douteuses à l'œil.

Les pistes restantes sont numérotées par ordre chronologique, après le recollage.
Le numéro change dès qu'une piste apparaît ou disparaît plus tôt dans la vidéo : pour désigner un passage, utiliser son temps de début.

## Sorties

La liste des fichiers produits est dans [lire les résultats](lire-les-resultats.md#où-sont-les-résultats).
Ce qui suit ne sert qu'au développeur.

`params.json` s'écrit en dernier, une fois les extraits finis.
Il donne les réglages employés, la version et le FFmpeg lié, l'état (`done`, ou `too_many_tracks` au-delà de `--max-tracks`), le nombre de pistes, la durée, le temps non analysé et le temps de calcul (`track_count`, `length_s`, `not_analysed_s`, `elapsed_s`, en secondes non arrondies), la taille de la vidéo en octets (`video_bytes`), la langue des noms (`language`, `fr` ou `en`) et l'heure locale de la fin du calcul (`date`).
`tracks` liste les pistes : numéro (`id`), début et durée non arrondis (`start_s`, `duration_s`), couleur de la trajectoire sur l'image résumé en RVB (`color`, `null` au-delà du garde-fou, qui n'a pas d'image résumé), image résumé qui la dessine (`summary`, par exemple `video_097_resume_3_sur_8.png`), et chemins de l'extrait et du zoom écrits, relatifs au dossier de la vidéo (`null` quand il n'y en a pas), et point de départ de la piste en pixels entiers de la vidéo d'origine (`start_xy`, absent avant #228), qui aide la page à rattacher un verdict à sa piste.
Pour que la page redessine l'image résumé sans rien recalculer (#259), `params.json` donne aussi le fond, la scène sans trace (`background`, par exemple `video_097_fond.png`), la mise en page de chaque image résumé (`summaries` : `file`, la tranche `window_s`, la partie `part` en `[n, total]` ou `null`, les périodes ignorées et illisibles coupées à la tranche `ignored_s` et `damaged_s`, les débuts des pistes des autres parties `other_starts_s`, au centième, et le rayon des pastilles `radius` en pixels de la vidéo), et pour chaque piste dessinée sa trace (`trace` : `points` et `head`, la trajectoire et les trois points de la flèche en chaîne `"x,y x,y …"` de pixels entiers de la vidéo, sans point répété ; `dot`, le centre de la pastille ; `anchor`, le départ du trait de rappel quand la pastille est écartée de la trajectoire, sinon `null`).
Une piste non dessinée a `trace` à `null`, et un dossier écrit avant #259 n'a ni `background` ni `summaries` : la page montre alors l'image résumé.
Les périodes ignorées sont dans `ignored_s`, les illisibles dans `damaged_s`, avec `damaged_frames` (nombre d'images signalées par le décodeur ou qui suivent un trou dans les horodatages, pas le nombre d'images écartées).

Une vidéo déjà faite est sautée : son dossier a un `params.json` terminé, et la vidéo, la version de batdetect, la langue des noms et les réglages (sauf la liste des encodeurs) n'ont pas changé.
Rien n'est écrit ni effacé dans son dossier ; le bilan reprend sa durée, ses pistes et son temps non analysé, affiche « — » en calcul et compte « N déjà faites ».
Une vidéo laissée avec trop de pistes reste comptée ainsi.
Après un Ctrl-C, relancer la même commande reprend là où le lot s'est arrêté ; `--force` refait toutes les vidéos.

Une fois la détection finie, chaque calcul (pas une vidéo sautée parce que déjà faite) efface, dans le dossier de la vidéo, `params.json` d'abord, puis les sorties des deux langues et celles des versions précédentes (`split/`, `<nom>.tracks.*`, `<nom>_boxes.mp4`).
Les sorties de la v0.1.1, écrites dans `out/` du dossier où la commande était lancée, restent où elles sont.

Garde-fou : au-delà de `max_tracks` pistes (300 par défaut, 0 désactive), ni les extraits, ni l'image résumé, ni son fond, ni la vidéo annotée ne sont produits (`background` à `null`, `summaries` vide, `trace` à `null`), et la vidéo est comptée à part au bilan.
Le CSV et `params.json` (avec le seuil, dans `RenderConfig`, et `"state": "too_many_tracks"`) restent écrits ; l'image résumé n'est plus écrite depuis #221, car sur une vidéo de bruit elle se coupait en dizaines de parties.
300 est au-dessus de la vidéo la plus chargée sans explosion (148 pistes, 097) et en dessous de la plus petite explosion connue (374 pistes, seuil 12 sans facteur de bruit).
Le seuil est fixe, pas un taux par minute : il suppose des clips d'environ 5 min, comme ceux des jumelles, et se monte avec `--max-tracks` pour une vidéo plus longue.
Le bilan de la soirée compte ces vidéos à part, avec leur nombre de pistes ; une relance identique ne les refait pas et les compte encore ainsi.

Colonnes du CSV, dans cet ordre : `piste` (numéro), `début` et `fin` (`h:mm:ss`, tronqués à la seconde comme le nom de l'extrait), `durée (s)`, `points` (nombre de détections), `corde (px)` (distance de bout en bout), `chemin (px)` (chemin parcouru), `vitesse (px/s)` (chemin ÷ durée), `surface max (px)` (plus grande tache, en pixels d'origine), `amplitude (niveaux de gris)` (plus fort écart au fond ; avec le filtre à la taille de la cible, écart mesuré sur l'image filtrée, donc plus bas que l'écart brut pour une petite tache), `images comblées` (images comblées par interpolation à l'affichage) et `début (s)` (début en secondes, au centième).
Un début dans le dernier centième d'une seconde s'écrit `0:00:04` d'un côté et `5,0` de l'autre : `début` tronque, `début (s)` arrondit.

Le tableau appartient à l'outil : un recalcul le remplace sans le lire.
Les verdicts n'y sont pas : ils se notent dans la page et vivent dans `verdicts.csv`, à côté d'elle, que l'outil lit sans jamais l'écrire (docs/11, crate `page`).

Le CSV suit la langue des messages.
En français, `_pistes.csv` commence par un BOM UTF-8 (pour qu'Excel lise les accents), sépare par un point-virgule et écrit la virgule décimale.
En anglais, `_tracks.csv` n'a pas de BOM, sépare par une virgule et écrit le point, avec les en-têtes `track`, `start`, `end`, `duration (s)`, `points`, `chord (px)`, `path (px)`, `speed (px/s)`, `max area (px)`, `amplitude (grey levels)`, `filled frames`, `start (s)`.
Fins de ligne CRLF dans les deux langues. `0:04` serait lu par LibreOffice comme 0 h 04 min : d'où `0:00:04`.

### Place prise

Mesurée le 16 octobre 2026 sur lumiel (binaire release 0.2.0-19-g709ee52, openh264, réglages par défaut, `--annotated` sur 6 vidéos), sur les 26 vidéos de `BATDETECT_VIDEOS_DIR`, une par une, le dossier de résultats relevé puis effacé.
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

- Les sorties vont de 0,002 à 6,9 fois la vidéo, médiane 0,48 : elles suivent le nombre et la durée des passages, inconnus avant la détection.
  Les soirées chargées (095 à 100, 78 à 149 pistes) dépassent 3 fois, les calmes restent sous 1 fois.
- La vidéo annotée pèse de 0,64 fois la vidéo (120, sans piste) à 2,72 fois (090).
- Par seconde de fenêtre d'extrait (piste et marges de 1,5 s), un extrait pèse de 0,81 à 3,17 fois le débit de la vidéo d'entrée, un zoom de 0,29 à 2,63 fois ; 089 à 103 sont nettement au-dessus de 121 à 130, à débit d'entrée égal.

Coefficients retenus (`space`, décision de l'utilisateur du 16 octobre) :

- au lancement, 1 fois la vidéo : 18 soirées sur 26 tiennent dessous ; une soirée chargée le dépasse, et c'est le contrôle par vidéo qui la protège alors ;
- vidéo annotée, 2,8 fois la vidéo, le plus prudent mesuré arrondi au-dessus ;
- par vidéo, avant les extraits : 3,2 fois le débit d'entrée par seconde d'extrait et 2,7 par seconde de zoom, les plus prudents mesurés arrondis au-dessus.

VideoToolbox écrit environ 0,77 fois ce qu'écrit openh264 (docs/11) : sur le Mac, ces coefficients sont donc plus prudents encore.
Le comportement des deux contrôles est en docs/11, section cli.
