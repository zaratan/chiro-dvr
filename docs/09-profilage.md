# Profilage : où part le temps de calcul

Mesures du 4 octobre 2026 sur l'original de la 092 (5 min, 1440×1080, 30,03 i/s,
9 011 images, 12 Mb/s), Apple M1 Max (10 cœurs, GPU 32 cœurs, 64 Go). Scripts de
mesure hors dépôt ; les chiffres suffisent à refaire les choix.

## Étapes du traitement complet

Réglages par défaut, détection sur 10 processus, avant toute optimisation.

| Étape | Temps | Part | Cœurs occupés |
| --- | --- | --- | --- |
| Vidéo annotée | 118,5 s | 56 % | 7,8 |
| Détection | 66,6 s | 31 % | 8,0 |
| Extraits (14) | 26,4 s | 12 % | 7,3 |
| Image résumé (fond médian compris) | 1,5 s | 1 % | 4,0 |
| Suivi et CSV | < 0,1 s | 0 % | |
| **Total** | **213 s** | | |

## Détection

Un seul processus, 900 images, par image :

| Poste | Temps | Part |
| --- | --- | --- |
| Médiane `np.median` sur 11 images à 480×360 | 12,6 ms | 78 % |
| Décodage (`cap.read`) | 1,3 ms | 8 % |
| Réduction et passage en gris | 0,8 ms | 5 % |
| Résidu, taches, reste | 1,9 ms | 9 % |

La lenteur de `np.median` ne vient pas de la disposition en mémoire : avec les pixels
contigus (pile sur le dernier axe), le temps passe seulement de 18,3 à 17,1 ms. Elle
vient de `np.partition`, qui lance une sélection par pixel, soit 172 800 petites
sélections de 11 valeurs. Un **tri par comparaisons vectorisé** (des `minimum` et
`maximum` sur des images entières) donne le même résultat au bit près en 1,4 ms.

## Médiane : CPU contre GPU

Médiane de 11 images, temps par médiane, transferts vers le GPU et retour compris, sauf
la dernière ligne. Les 22 mesures donnent un résultat identique au bit près à
`np.median`. MLX 0.32.3 et PyTorch 2.14.1 (backend MPS), sous Python 3.14.

| Méthode | 480×360 | 1440×1080 |
| --- | --- | --- |
| CPU `np.median` | 18,1 ms | 162 ms |
| CPU tri par comparaisons (un cœur) | 1,3 ms | 11,8 ms |
| GPU MLX `mx.median` | 5,9 à 8,5 ms | 51 à 69 ms |
| GPU PyTorch `torch.median` | 2,5 à 3,2 ms | 23 ms |
| GPU PyTorch tri par comparaisons (lot) | 0,42 ms | 4,4 ms |
| GPU MLX tri compilé, `mx.compile` (lot) | 0,21 ms | 1,6 ms |
| GPU MLX tri compilé, données déjà sur le GPU | 0,09 ms | 0,58 ms |

- **L'algorithme compte plus que le matériel** : les médianes toutes faites des
  bibliothèques GPU trient toutes les valeurs et restent plus lentes que le tri par
  comparaisons sur un seul cœur du CPU.
- **À débit égal, CPU et GPU se valent aujourd'hui.** La détection occupe déjà environ
  8 cœurs : le CPU débite une médiane toutes les 0,16 ms environ à 480 px (1,5 ms en
  pleine résolution), contre 0,21 ms (1,6 ms) pour le GPU, unique et partagé.
- **La marge du GPU est dans les transferts** : le banc envoyait chaque image 11 fois.
  Une détection pensée pour le GPU, qui envoie chaque image une seule fois, viserait la
  dernière ligne et libérerait les cœurs pour le décodage. Non mesuré.
- Le GPU redevient intéressant pour la pleine résolution ou le filtrage à plusieurs
  échelles (B2 dans [07](07-ameliorer-la-detection.md)), avec MLX en dépendance réservée
  aux Mac.

## Encodage de la vidéo annotée

Lire et convertir une image côté Python coûte 1,2 ms, soit environ 11 s pour toute la
vidéo ; le reste du rendu est l'encodage. Sur 60 s de la 092 (SSIM : fidélité à
l'original, 1 = identique) :

| Encodeur | Temps | CPU | Taille | SSIM |
| --- | --- | --- | --- | --- |
| `libx264` CRF 20 (défaut) | 22,0 s | 178 s | 144 Mo | 0,982 |
| `libx264 -preset veryfast` CRF 20 | 7,7 s | 61 s | 133 Mo | 0,979 |
| `h264_videotoolbox -q:v 65` | 4,2 s | 12 s | 151 Mo | 0,977 |
| `h264_videotoolbox -b:v 12M` | 4,3 s | 12 s | 92 Mo | 0,962 |

`h264_videotoolbox` passe par le moteur multimédia des puces Apple, un circuit dédié
distinct du GPU. La vidéo annotée complète pèse 722 Mo en CRF 20, plus que l'original
(454 Mo) : le bruit thermique se compresse mal.

## Après la médiane par tri par comparaisons

Détection de la 092 avec 10 processus : 66,6 s → 49,6 s, 534 s → 406 s de CPU, pistes
identiques au bit près (une seconde exécution, dans le tableau plus bas, donne 47,3 s et
415 s : environ 5 % d'écart d'une exécution à l'autre). Le gain est plus faible qu'attendu parce que le profil ci-dessus,
fait sur un seul processus, comptait le temps réel et non le temps CPU :

| Lecture | Temps réel | CPU | Cœurs |
| --- | --- | --- | --- |
| `cap.read()` | 1,12 ms/image | 8,62 ms/image | 7,7 |
| `cap.grab()` (saut) | 0,85 ms/image | 6,62 ms/image | 7,7 |

Le décodeur d'OpenCV occupe déjà presque tous les cœurs. Avec 10 tranches, chaque
tranche relit la vidéo depuis le début pour s'y positionner : 4,5 vidéos sautées en
tout, environ 270 s de CPU, plus 80 s pour lire la vidéo elle-même. Le décodage fait
donc environ 85 % du CPU de la détection, et plus de processus coûte plus cher :

| Processus | 1 | 2 | 3 | 4 | 6 | 10 |
| --- | --- | --- | --- | --- | --- | --- |
| Temps | 31,0 s | 23,7 s | 24,7 s | 27,3 s | 31,6 s | 47,3 s |
| CPU | 114 s | 150 s | 184 s | 216 s | 282 s | 415 s |

Pistes : moins de processus par défaut ; décoder une seule fois et distribuer les
images réduites aux processus ; décodage matériel (VideoToolbox).

## Après le moteur multimédia et la fenêtre en uint8

Fenêtre de la médiane gardée en uint8 au lieu de int16 : médiane seule 1,22 → 0,73 ms
par image à 480×360, pistes identiques au bit près,
détection 31,0 → 27,2 s avec 1 processus, 23,7 → 22,2 s avec 2.

Traitement complet de la 092, détection sur 2 processus, rendu par `h264_videotoolbox`
q 65 :

| Étape | Avant | Après |
| --- | --- | --- |
| Détection | 66,6 s (10 processus) | 20,3 s (2 processus) |
| Image résumé | 1,5 s | 1,6 s |
| Vidéo annotée | 118,5 s | 19,8 s |
| Extraits | 26,4 s | 8,1 s |
| **Total** | **213 s** | **49,8 s** |

La vidéo annotée pèse 760 Mo au lieu de 722. Avec 10 processus (un par
cœur, l'ancien défaut de `--workers`), la détection reste à environ 48 s.

## Décodage : pistes éliminées

- **Décodage matériel par OpenCV** : la roue `opencv-python-headless` 5.0 accepte
  `CAP_PROP_HW_ACCELERATION` sans erreur, mais ne connaît que VAAPI (Linux) ;
  `cap.get` renvoie 0 et le décodage reste logiciel.
- **Décodage matériel par ffmpeg** (`-hwaccel videotoolbox`, sous-processus) : 5,6 s
  au lieu de 1,0 s sur l'extrait de test (environ 220 images/s), pour 0,8 s de CPU au
  lieu de 7,8. Le YUV décodé est identique au bit près, mais la conversion en BGR
  d'ffmpeg ne reproduit jamais celle d'OpenCV : jusqu'à 18 niveaux d'écart sur l'image
  grise de travail, pour un seuil de 25. Toute lecture hors d'OpenCV change les
  détections et impose une nouvelle référence validée par Manon.
- **Décoder une seule fois** (un fil lecteur alimente la détection par une file bornée) :
  prototype sur l'extrait, 3,46 s → 2,16 s, détections identiques. C'est le lecteur
  (décodage puis réduction) qui limite.

## Après le fil lecteur

Dans chaque tranche, un fil lecteur décode, applique le crochet du banc et réduit les
images pendant que le fil principal détecte (`prefetch.py`, file de 4 images). Un
troisième étage (décodage et réduction séparés) ne gagnait rien (2,12 s contre 2,10 s
sur l'extrait) et coûtait de la mémoire. Détection de la 092, pistes identiques au bit
près à la référence ; la ligne « sans fil lecteur » inclut déjà la fenêtre en uint8,
d'où l'écart avec le premier tableau par nombre de processus :

| Processus | 1 | 2 | 3 | 4 | 10 (ancien défaut) |
| --- | --- | --- | --- | --- | --- |
| Sans fil lecteur | 27,2 s | 22,2 s | | | 48,4 s |
| Avec fil lecteur | 15,1 s | 16,8 s | 20,0 s | 23,1 s | 42,3 s |
| CPU avec fil lecteur | 111 s | 146 s | 178 s | 211 s | 403 s |

Un seul processus est désormais le plus rapide sur le M1 Max : le décodeur d'OpenCV
occupe déjà les cœurs, et chaque tranche supplémentaire ajoute un saut par `grab`.
Banc sur la 092 (deux passes) : 31,6 s avec 1 processus, 47,4 s avec 4, résumés
identiques.

`--workers` vaut 1 par défaut depuis cette mesure. Commande complète par défaut sur la
092 (`uv run batdetect in/video_092_original.mp4`) : **44,8 s**, contre 213 s au début
de l'optimisation ; 14 pistes, encodeur `videotoolbox`.

## Après le rendu en une lecture

La vidéo annotée complète devient optionnelle (`--annotated`) ; les extraits sont encodés
directement depuis l'original pendant une seule lecture, au lieu d'être recoupés dans la
vidéo annotée ([02](02-methode.md#rendu)). Étape de rendu seule sur la 092 (14 extraits,
`h264_videotoolbox`), deux exécutions de chaque, machine chargée (charge 8 à 12) mais
mêmes conditions pour les trois :

| Rendu | Temps | CPU |
| --- | --- | --- |
| Avant : vidéo annotée puis découpe | 29,4 à 31,0 s | 106 s |
| Extraits seuls (défaut) | 12,0 à 12,7 s | 53 s |
| Extraits et vidéo annotée (`--annotated`) | 26,4 à 28,8 s | 89 s |

Mêmes 14 extraits, mêmes noms, même nombre d'images. Une génération d'encodage en moins :
SSIM de l'extrait 5 contre l'original 0,955 → 0,974, pour 159 Mo d'extraits au lieu de
142. Sans `--annotated`, le dossier de sortie de la 092 passe d'environ 900 Mo à 160 Mo.
La lecture saute les images hors des extraits sans les convertir et s'arrête après le
dernier (3:58 sur 5:00). Total de la commande par défaut à remesurer sur machine calme.

## Passe ffprobe des images illisibles (issue #1)

Les images que le décodeur répare sont repérées par `ffprobe -threads 1 -show_log 16`
([10](10-logique-de-detection.md)). Mesures sous verrou, une commande à la fois, second
lancement ou lancements répétés.

| | Passe seule | Commande avant | Commande après | CPU avant → après |
| --- | --- | --- | --- | --- |
| 092 (aucune erreur) | 56 à 58 s | 33,8 s | 68,3 s | 170 → 221 s |
| 093 (11 erreurs) | | 27,4 s | 65,7 s | 159 → 210 s |
| 122 (195 erreurs) | 54 à 59 s | 59,5 s | 58,6 s (5 lancements de 58,5 à 59,1 s) | 191 → 177 s |

La passe dure environ 55 s par 5 min, que la vidéo soit abîmée ou non, sur un seul cœur.
Elle tourne pendant la détection, mais devient l'étape la plus longue : le traitement
complet d'une vidéo saine double environ. La 122 ne ralentit pas parce qu'il ne reste
presque rien à analyser ni aucun extrait à encoder. Sur la même vidéo, les temps d'une
commande varient jusqu'à 30 % d'un lancement à l'autre ; le temps CPU, lui, est stable.

Éliminé pour accélérer la passe :

- **ffprobe sur plusieurs fils** (`-threads 0`) : 10 s au lieu de 56 sur la 122, mais
  192 images signalées au lieu de 195 et rattachées aux mauvaises images.
- **`-skip_loop_filter all`** : mêmes 195 images, 2 s de gagnées seulement.
- **`-flags2 fast`** : mêmes 195 images, 14 s de plus.
- **L'ordre des messages sur la sortie d'erreur d'`ffmpeg`** : décalé d'une à deux images
  avec ffmpeg 9, donc inutilisable.

### Avec les réglages du 6 octobre (issue #36)

Les 55 s de surcoût ci-dessus dataient d'une détection à 480 px, qui durait 33,8 s sur la
092 : la passe était alors l'étape la plus longue. Avec les réglages par défaut actuels
(960 px, filtre à la taille de la cible, seuil par pixel) et `--workers 1`, la détection
dure plus longtemps que la passe, qui est donc masquée. Mesures du 7 octobre 2026 sur la
092, sous verrou, sur une machine où d'autres agents mesuraient aussi : le temps réel est
bruité, le temps CPU est stable.

| | Temps réel | CPU (user + sys) |
| --- | --- | --- |
| Passe seule, un fil (`-threads 1`) | 61,0 s | 55,4 s |
| Passe seule, multifil (`-threads 0`) | 16,1 s | 62,0 s |
| Commande avec passe (4 lancements) | 140 à 590 s | 440 à 514 s, 478 s en moyenne |
| Commande sans passe (3 lancements) | 150 à 309 s | 395 à 410 s, 404 s en moyenne |

Dans deux lancements chronométrés de l'intérieur, la détection a duré 122,6 s et 199,3 s,
et la commande a attendu la fin de la passe **0,0 s** les deux fois. La passe n'allonge
donc pas la commande à `--workers 1`. Il reste environ 75 s de CPU en plus, soit 18 %,
à peser sur la machine de la naturaliste (#10). Le surcoût en temps réel ne se chiffre pas
sur une machine partagée : les écarts entre lancements dépassent largement 15 s.

Les deux pistes d'accélération n'ont pas été retenues : elles ne gagnent que du temps
réel, déjà masqué, et pas de CPU.

- **Contrôle multifil**, avec passe complète seulement en cas d'erreur : 8 à 16 s au lieu
  de 57 à 86 s, pour 58 à 62 s de CPU. Sur les cinq vidéos saines balayées (089 à 092 et
  la 092 d'origine), même nombre d'images, mêmes images-clés et mêmes trous qu'avec un
  seul fil, et aucune erreur. Balayage arrêté avant les vidéos abîmées.
- **Tranches parallèles partant d'une image-clé** (`-read_intervals`) : non mesurées, pour
  la même raison.

Si la détection redevient plus courte qu'environ 60 s (plusieurs `--workers`, réglage plus
léger), la passe redevient l'étape la plus longue, et ces deux pistes reprennent leur intérêt.

## Mesure des taches en une passe (5 octobre 2026)

`find_blobs` construisait un masque de toute l'image pour chaque tache, afin d'en compter
les pixels au-dessus du seuil et d'en prendre le pic : un coût proportionnel au nombre de
taches multiplié par le nombre de pixels. L'aire et le pic de toutes les taches viennent
désormais d'une seule passe sur les pixels au-dessus du seuil. Détection seule sur
l'extrait de test (1 230 images, un processus, réglages par défaut sauf la largeur) :

| Largeur | Taches | Avant | Après | CPU avant → après | Mémoire maximale |
| --- | --- | --- | --- | --- | --- |
| 480 px | 388 | 2,4 s | 2,2 s | 15,2 → 15,1 s | 172 → 172 Mo |
| 960 px | 3 861 | 32,3 s | 7,9 s | 58,8 → 34,6 s | 240 → 247 Mo |
| 1440 px | 342 662 | 1 221 s | 17,9 s | 1 233 → 39,7 s | 509 → 486 Mo |

Détections identiques au bit près aux trois largeurs, banc et 14 pistes de la 092
identiques. À 480 px, rien de mesurable : banc 31,7 s et 217 s de CPU avant comme
après, commande sur la 092 28,9 s contre 29,1 s. À 1440 px avec le seuil de 25,
l'extrait compte 280 taches par image : c'est probablement ce coût qui avait fait
arrêter le banc en pleine résolution après 25 minutes ([08](08-banc-de-mesure.md)),
non vérifié en le relançant.

## Carte de bruit du seuil par pixel (5 octobre 2026)

`--noise-factor` ajoute deux médianes par image, sur des écarts en float32 (le centre
des écarts, puis leur MAD). Détection seule sur l'extrait de test, seuil 18, un
processus :

| Largeur | Sans carte (k = 0) | Avec carte (k = 8) | Par image | Mémoire maximale |
| --- | --- | --- | --- | --- |
| 480 px | 2,2 s | 10,5 s | +6,7 ms | 171 → 232 Mo |
| 960 px | 9,0 s | 41,5 s | +26 ms | 348 → 339 Mo |
| 1440 px | 18,7 s | 99,9 s | +66 ms | 435 → 564 Mo |

Sur la 092 entière, la commande passe de 29 à 90 s, et le banc de 32 à 155 s. Une
médiane en float32 coûte environ 4 fois une médiane en uint8 (0,7 ms à 480 px). Non
essayé : des écarts en entiers 16 bits.

Depuis, la carte n'est calculée que sur les pixels dont le résidu dépasse déjà le
plancher : un pixel sous le plancher n'est jamais retenu, quel que soit son bruit, et
l'arithmétique reste la même pixel par pixel. Détections identiques octet par octet aux
trois largeurs. Seuil 18, k = 8, extrait de test, second lancement ; machine chargée par
d'autres travaux pendant ces mesures (charge 8 à 19), donc le CPU fait foi plus que le
temps réel :

| Largeur | Carte sur toute l'image | Carte sur les pixels candidats | CPU | Mémoire maximale |
| --- | --- | --- | --- | --- |
| 480 px | 12,5 s | 2,8 s | 25,4 → 15,6 s | 235 → 175 Mo |
| 960 px | 49,6 s | 9,8 s | 74,5 → 36,2 s | 352 → 254 Mo |
| 1440 px | 118,0 s | 21,6 s | 135,5 → 42,4 s | 578 → 370 Mo |

Mesurées ensuite en alternance avec k = 0 (deux fois chacun, charge 6 à 9), la carte ne
coûte plus rien de mesurable à 480 px (CPU 15,5 à 16,4 s contre 15,9 à 16,1 s) ni à 960 px
(36,0 à 36,1 s contre 36,1 à 36,2 s) ; à 1440 px, 21,0 à 21,2 s contre 19,0 à 19,1 s,
soit environ 1,6 ms par image, pour 0,6 s de CPU. La mémoire baisse avec la carte (moins
de taches à mesurer) : 1440 px 443 à 448 Mo sans, 371 à 386 Mo avec. Sur la 092 entière, les
deux versions alternées dans les mêmes conditions : 92,6 et 96,0 s contre 31,4 et 34,7 s,
CPU 237 et 240 s contre 178 et 180 s, mémoire 687 et 693 Mo contre 629 et 634 Mo, mêmes
pistes. Une médiane par `np.sort` sur la pile des pixels candidats, à la place du tri par
comparaisons, ne gagne rien (3,0 s contre 2,8 s à 480 px, 23,5 s contre 22,1 s à
1440 px) : écartée.

## Filtre à la taille de la cible (5 octobre 2026)

`--target-sigma` floute chaque image réduite et garde les images en float32 : la médiane
du fond travaille sur 4 fois plus d'octets qu'en uint8. Détection seule sur l'extrait de
test, σ 1,5, un processus, mesures prises avant la règle du verrou (charge non
contrôlée) :

| Largeur | Sans filtre | Avec filtre | Par image |
| --- | --- | --- | --- |
| 960 px | 8,3 s (seuil 25) | 16,3 à 16,8 s | +6,5 ms |
| 1440 px | 18,7 s (seuil 18) | 38,3 s | +16 ms |

Sur la 092 entière, sous le verrou de mesure : 30 s par défaut, 135 s à 960 px avec le
filtre (378 s de CPU, 702 Mo), 283 s à 1440 px (518 s, 838 Mo). Non essayé : garder les
images filtrées en entiers 16 bits pour accélérer la médiane.

## Réglages par défaut du 6 octobre 2026

960 px, filtre de 1,5 px, seuil par pixel. Commande complète sur la 092 (extraits
compris, sous le verrou de mesure) : 135 à 151 s au lieu de 30 à 34 s, 373 à 386 s de
CPU au lieu de 175 s, 707 Mo de mémoire au plus au lieu de 627. Sur les sept vidéos de
la comparaison (092, 089, 090, 091, 125, 126, 127), 16 min au lieu de 4, et 45 min de
CPU au lieu de 20. Le surcoût vient surtout de la médiane du fond sur des images
filtrées en float32, quatre fois plus de pixels qu'à 480 px ; la carte de bruit, limitée
aux pixels au-dessus du plancher, ne coûte presque rien. Un banc de transit prend
260 s au lieu de 32. `--work-width 480 --target-sigma 0 --threshold 25 --noise-factor 0`
retrouve l'ancien réglage et son temps.
