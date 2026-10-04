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

La vidéo annotée pèse 760 Mo au lieu de 722. Avec le réglage par défaut de
`--workers` (un processus par cœur), la détection reste à environ 48 s.

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

| Processus | 1 | 2 | 3 | 4 | 10 (défaut) |
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
