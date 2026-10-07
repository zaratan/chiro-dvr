# La logique de détection, expliquée

Ce document explique **comment l'outil raisonne** : quel outil mathématique ou technique
sert à chaque étape, pourquoi celui-là plutôt qu'un autre, et ce qu'il ne sait pas faire.
Les valeurs des réglages et leurs mesures sont dans [02-methode.md](02-methode.md) ; les
pistes d'amélioration dans [07](07-ameliorer-la-detection.md).

## L'idée en une phrase

Une chauve-souris est **une petite tache qui s'écarte du fond pendant quelques images et
qui se déplace de façon régulière**. Tout le traitement découle de ces trois mots : fond,
tache, régularité.

```
vidéo ─► image grise réduite et filtrée ─► fond (médiane) ─► résidu ─► seuil par pixel ─► taches
                                                                        │
      pistes gardées ◄─ filtres ◄─ fusion des jumelles ◄─ suivi ◄─ images instables retirées
                                                                        ▲
vidéo ─► ffprobe, un fil (pendant la détection) ─────────────────► images illisibles retirées
```

| Étape | Module | Outil | Question posée |
| --- | --- | --- | --- |
| Lecture | `video.py`, `prefetch.py`, `detect.py` | OpenCV, moyenne de surface, flou gaussien | Que voit le capteur, en plus petit, à la taille d'une chauve-souris ? |
| Fond | `detect.py`, `median.py` | médiane temporelle | À quoi ressemble la scène sans animal ? |
| Gain | `detect.py` | différence de moyennes | L'image entière a-t-elle changé de luminosité ? |
| Seuil | `detect.py`, `noise.py` | valeur absolue, plancher et bruit du pixel (MAD) | Ce pixel s'écarte-t-il assez du fond, compte tenu de son propre bruit ? |
| Taches | `detect.py` | fermeture morphologique, composantes connexes | Quels pixels forment un même objet ? |
| Illisible | `probe.py`, `damage.py`, `exclusion.py` | journal de décodage d'ffprobe, images-clés | Le décodeur a-t-il dû réparer l'image ? |
| Stabilité | `stability.py` | comptage, médiane | Les jumelles ont-elles bougé ? |
| Suivi | `track.py` | prédiction linéaire, plus proche voisin | Cette tache prolonge-t-elle une piste ? |
| Jumelles | `track.py` | union-find | Deux pistes sont-elles un seul animal ? |
| Filtres | `track.py` | comptage, distance, angle médian | Cette piste ressemble-t-elle à un vol ? |

## 1. Lire et réduire l'image

**Outil.** OpenCV décode la vidéo (par ffmpeg), convertit en niveaux de gris de 0 à 255,
et réduit l'image à 960 px de large par **moyenne de surface** (`INTER_AREA`) : sur une
vidéo de 1440 px, chaque pixel réduit moyenne 1,5 × 1,5 pixels d'origine. Puis un **flou
gaussien** d'écart-type 1,5 px d'origine, la taille d'une petite chauve-souris lointaine,
est appliqué à chaque image réduite, avant tout le reste.

**Pourquoi le flou.** C'est le **filtre adapté** (*matched filter*) d'une tache gaussienne
dans du bruit, le meilleur rapport signal sur bruit possible pour une cible de cette
taille : il moyenne le bruit d'un pixel à ses voisins sans écraser une tache de quelques
pixels. Il remplace le débruitage qu'apportait jusqu'au 6 octobre 2026 la réduction à
480 px, qui faisait perdre 30 à 50 % de leur contraste aux cibles de moins de 3 px
([07](07-ameliorer-la-detection.md)).

**Choix.** Filtrer l'image avant le fond plutôt que le résidu après, pour que le fond, le
résidu et le bruit de chaque pixel soient mesurés dans le même espace. 960 px plutôt que
1440 : sur sept vidéos, la pleine résolution donne presque les mêmes pistes pour 2,4 fois
le temps de calcul ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)).

**Limites.** Une tache plus grande que le filtre est moins rehaussée, une plus petite
perd du contraste. À 960 px, un pixel de travail vaut 2,25 px² : `min_area` 4 exige deux
pixels, et une cible qui n'en allume qu'un sur une image perd cette image. La médiane
sur des images en flottants coûte plus cher qu'en entiers de 8 bits : la 092 se traite
en 135 à 151 s au lieu de 30 ([09](09-profilage.md)). `--target-sigma 0 --work-width 480`
revient à l'ancienne lecture.

**Détail technique.** Un fil d'exécution décode et réduit en avance (4 images) pendant
que le fil principal détecte. Toutes les coordonnées ressortent en pixels d'origine : le
centre d'un pixel réduit `c` vaut `(c + 0,5) × échelle − 0,5`.

## 2. Estimer le fond : la médiane temporelle

**Outil.** Pour chaque pixel, la **médiane** de ses valeurs sur une seconde centrée sur
l'image étudiée, en prenant une image sur 3 (11 valeurs).

**Pourquoi la médiane.** C'est une statistique *robuste* : tant que l'animal occupe le
pixel sur moins de la moitié des images, la médiane l'ignore complètement. Une moyenne,
elle, garderait une traînée de l'animal dans le fond. Une chauve-souris en transit ne
reste sur un pixel que quelques images sur 30 : elle disparaît du fond.

**Pourquoi une fenêtre centrée.** La fenêtre regarde une demi-seconde avant et une
demi-seconde après. Le traitement n'est pas en direct, donc on peut utiliser le futur :
le fond suit ainsi les changements lents de lumière sans retard.

**Pourquoi pas autre chose.**

- *Différence entre deux images* : simple, mais voit chaque objet deux fois (où il était,
  où il est) et rate les objets lents.
- *Mélanges de gaussiennes* (MOG2, utilisé par ThruTracker) : la sortie est binaire. On
  perd le signe et l'amplitude de l'écart, dont le reste du traitement a besoin.
- *Fond fixe sur toute la vidéo* : ne suit ni la lumière ni le gain des jumelles.

**Limite.** Un objet qui reste plus d'une demi-seconde au même endroit entre dans son
propre fond et s'efface (issue #22).

**Technique de calcul.** La médiane de 11 images coûtait 78 % du temps. Elle est
calculée par un **tri par comparaisons partiel** : des `minimum` et `maximum` appliqués
à des images entières font remonter les plus grandes valeurs, comme un tri à bulles
arrêté dès que la valeur centrale est fixée. Le résultat est identique au bit près à
`np.median`, environ 10 fois plus vite ([09](09-profilage.md)).

## 3. Compenser le gain automatique

**Outil.** On retire au résidu la différence entre la luminosité moyenne de l'image et
celle des images du fond.

**Pourquoi.** Les jumelles ajustent leur gain : toute l'image s'éclaircit d'un coup. Sans
correction, chaque pixel dépasserait le seuil. Le modèle est le plus simple possible : un
décalage identique pour tous les pixels.

**Limite.** Un changement de contraste, qui étire les valeurs au lieu de les décaler,
n'est pas corrigé.

## 4. Décider pixel par pixel : le seuil

**Outil.** Le **résidu** est `image − fond − décalage de gain`, sur l'image filtrée. Un
pixel est retenu si la valeur absolue du résidu dépasse **son propre seuil** :
max(12, 8 × bruit du pixel).

**Pourquoi la valeur absolue.** L'animal peut être plus sombre que le fond (devant la
roche chaude) ou plus clair (devant un ciel froid). Le signe n'est pas supposé.

**Pourquoi pas un seuil fixe.** Jusqu'au 6 octobre 2026, le seuil était fixe à 25 : sur la
vidéo de référence, le bruit restait dessous. Mais un seuil unique traite pareil la roche
calme et la végétation qui bouge, et surtout il ne s'adapte pas d'une vidéo à l'autre.
Mesuré sur sept vidéos : un seuil fixe abaissé à 12, bon sur la 092, laisse des centaines
de pistes de bruit sur la 089, la 090 et la 091, dont le bruit de fond dépasse 12 en
quelques points de chaque image ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)).

**Le seuil par pixel** (`--noise-factor`, 8 par défaut, 0 pour revenir au seuil fixe).
*Outil* : pour chaque pixel, on mesure son bruit sur les images de la fenêtre du fond
(11 à 30 i/s) par la **MAD** (médiane des écarts absolus à la médiane, multipliée par 1,4826 pour
valoir un écart-type sur un bruit gaussien), et le seuil devient
max(seuil fixe, k × bruit). *Choix* : la MAD plutôt que l'écart-type, parce qu'un passage
d'animal sur une ou deux images de la fenêtre ne la fait presque pas monter ; la fenêtre
plutôt qu'une moyenne qui s'accumule depuis le début, pour que chaque tranche de la
détection parallèle donne le même résultat ; le seuil fixe gardé comme plancher, parce
que la MAD sur 11 images est souvent très faible : sur l'extrait de test à 480 px,
descendre le plancher de 4 à 1 niveau multiplie par 60 les pixels au-dessus de 6 fois le
bruit (2,6 → 160 par image). *Limites* : 11 images estiment mal le bruit d'un pixel, si
bien qu'une feuille agitée passe encore parfois. Une cible faible qui passe sur des
pixels agités peut être perdue : sur la 092, le passage réel de 3:42.10 (pic de 13 à 22
après filtre) tombe sur deux de ses six images sous un seuil relevé à 22 à 36, et n'a
plus assez de détections pour faire une piste. Le calcul coûte deux médianes de plus par
image, limitées aux pixels dont le résidu dépasse déjà le plancher (les autres ne
peuvent pas être retenus) : rien de mesurable à 480 et 960 px, environ 1,6 ms par image
à 1440 px. Un saut de gain de 30 niveaux, qu'absorbait l'ancien seuil de 25, fait
désormais ignorer 2,6 à 2,8 s autour du saut par le filtre des images saturées (§ 6),
sans créer de piste.

## 5. Regrouper les pixels en taches

**Outils.** Deux opérations classiques de morphologie mathématique.

- **Fermeture** (une dilatation puis une érosion, avec un disque de 6 px de rayon) : elle
  bouche les trous et recolle les fragments proches d'un même animal, sans grossir
  l'objet.
- **Composantes connexes** : chaque groupe de pixels qui se touchent reçoit un numéro.
  On en tire la boîte englobante, le centre de gravité, l'aire et l'amplitude maximale.

**Choix.** L'aire compte seulement les pixels réellement au-dessus du seuil, pas ceux
ajoutés par la fermeture : la fermeture sert à regrouper, pas à mesurer. Une tache est
gardée entre 4 et 2 700 px² d'origine. À 960 px, un pixel réduit vaut 2,25 px² : il faut
donc deux pixels au-dessus du seuil, et une tache d'un seul pixel est rejetée (à 480 px,
un seul suffisait). La position d'une tache filtrée reste à 2 px d'origine près
(testé). Plus haut, 9 ou 18 px² perdent les petites cibles faibles sans retirer de
bruit.

## 5 bis. Écarter les images que le décodeur a réparées

**Le problème.** Sur 17 des 22 vidéos des sessions du 2 septembre et du 2 octobre 2026,
il manque quelques octets à la fin de certaines images. Le décodeur bouche le trou par
*dissimulation d'erreur* : il recopie un morceau d'image voisine. OpenCV rend cette image
réparée sans rien dire. Une image P réparée sert de référence aux suivantes, donc le
dégât dure jusqu'à l'image-clé suivante (8 à 16 images). Sur la 122, cela donnait des
centaines de fausses pistes ; une fois le filtre de stabilité en place, les deux tiers de
la vidéo ignorés comme « jumelles qui bougent ».

**Outil.** Une passe **ffprobe** lit tout le fichier pendant que la détection tourne
(`-show_log 16`). Elle range chaque message d'erreur du décodeur sous l'image qui l'a
produit, et donne les images-clés et les horodatages (`pts`). Une image est abîmée si elle
porte un message de niveau erreur, ou si elle suit un trou dans les `pts` de plus d'un pas
et demi (13 images jamais décodées sur la 122, sans aucun message). La plage écartée va
de l'image abîmée à l'image qui précède la prochaine image-clé.

**Pourquoi une marge.** Le fond d'une image est la médiane de ±½ s d'images
(§ 2). Une image saine à moins d'une demi-fenêtre d'une image réparée a donc un fond
faux. Mesuré sur la 122 sans marge : les 81 images qui déclenchaient encore le filtre de
stabilité étaient toutes à 9 images au plus d'une plage abîmée, avant comme après, et les
4 pistes restantes finissaient toutes 1 à 6 images avant une plage. Chaque plage est donc
élargie de la demi-fenêtre du fond de chaque côté (`half_window`, 15 images à 30 i/s),
avant la fusion des plages, avant la stabilité et avant le suivi.

**Pourquoi ffprobe et pas OpenCV.** OpenCV ne signale rien : `cap.read()` répond « vrai ».
Changer de décodeur pour la détection est exclu, parce que la conversion en gris d'ffmpeg
diffère de celle d'OpenCV ([09](09-profilage.md)). Il faut donc relier chaque message
d'ffprobe à l'image d'OpenCV de même numéro. C'est vérifié sur la 122 : horodatages
identiques sur les 8 998 images, et contenu de même numéro dans 8 993 cas sur 8 993
comparés. L'ordre des
messages sur la sortie d'erreur d'`ffmpeg`, lui, est décalé d'une à deux images, et
ffprobe sur plusieurs fils rattache mal les messages : la passe tourne sur un seul fil.

**Pourquoi le journal du décodeur d'OpenCV est coupé.** À chaque lecture, le décodeur
d'OpenCV écrit sur la sortie d'erreur un message `[h264 @ …]` par image réparée : 410
lignes sur la 122 (détection, fond de l'image résumé, rendu), qui noyaient la sortie de la
commande. Ces messages ne disent pas quelle image ils concernent, et ffprobe donne déjà
chaque image abîmée, comptée et située, dans la sortie et `params.json`. Toute ouverture de
vidéo par OpenCV passe donc par `open_capture` (`video.py`), qui pose
`OPENCV_FFMPEG_LOGLEVEL=-8` (silence), même si l'utilisateur l'a posée autrement. OpenCV ne lit cette variable qu'une fois par
processus, à sa première ouverture d'une vidéo par ffmpeg : posée plus tard, elle ne change
rien. `cv2.utils.logging.setLogLevel` n'agit pas sur ce journal. Chaque processus de
détection parallèle ouvre sa vidéo par la même fonction.

**Ce que la sortie en dit.** Le temps écarté est signalé à part des mouvements :
« illisible 3 min 22 s (79 plages) » dans le bandeau, une ligne `damaged` en console,
`damaged_s` dans `params.json`. Ce temps **inclut la marge**. `damaged_frames` compte les
images signalées par le décodeur ou qui suivent un trou (196 sur la 122), pas les images
écartées (6 043). Si ffprobe et OpenCV ne voient pas le même nombre d'images, les numéros
ne se correspondent plus : la vidéo échoue, et les autres vidéos du lot continuent.

**Ce que ça coûte.**

| Vidéo | Images signalées | Illisible | Instable restant | Union | Pistes avant → après |
| --- | --- | --- | --- | --- | --- |
| 122 (195 erreurs) | 196 | 201,2 s en 79 plages | 0 s | 201,2 s sur 300 | 25 → 0 |
| 093 (11 erreurs) | 11 | 14,0 s en 8 plages | 2,1 s | 16,0 s | 4 → 3 |
| 092 (aucune) | 0 | 0 s | 0 s | 0 s | 14 → 14 |

Sur une vidéo très abîmée, la marge coûte cher : sans elle, la 122 gardait 77,8 s
illisibles, mais aussi 50,3 s instables et 4 pistes collées aux plages. Avec 15 images de
marge, il reste environ 98 s analysées sur les 300 de la vidéo. La passe ffprobe double environ le temps de
traitement de toute vidéo, abîmée ou non ([09](09-profilage.md), #36).

**Limites.**

- Une tranche très abîmée peut ne produire aucun message (vu sur une vidéo fabriquée en
  coupant 60 % d'une tranche) : l'image passe alors pour saine.
- Un nombre d'images égal ne prouve pas l'alignement : une image perdue au début et une
  image doublée plus loin passeraient le contrôle. L'écart entre les deux décodeurs dépend de
  leurs versions d'ffmpeg : sur une vidéo fabriquée dont une tranche est coupée entière,
  ffprobe 9 compte 118 images et OpenCV (libavcodec 61) 46, alors que ffmpeg 6 sous Ubuntu
  et OpenCV en comptent tous deux 119. Le contrôle protège le cas où l'écart existe.
- Le dégât s'arrête à l'image-clé parce que les images-clés des jumelles sont des IDR, sans
  image B. Un autre appareil pourrait ne pas respecter cette hypothèse.
- Après un trou d'images, les temps calculés par numéro d'image avancent sur l'horloge de
  la vidéo (0,43 s sur la 122, #35).
- La lecture d'une image réparée par OpenCV n'est pas reproductible quand la conversion
  suit la lecture de trop près : sans marge, deux traitements de la 122 ne donnaient pas
  les mêmes plages instables. La marge couvre tout l'effet mesuré (cinq traitements
  identiques).

## 6. Écarter les images où les jumelles bougent

**Outil.** Un simple **comptage**. Une image est suspecte si elle porte au moins 20
taches de plus que 6 fois le nombre médian de taches par image de la vidéo.

**Pourquoi ça marche.** Quand le cadre glisse, les images de la fenêtre du fond ne sont
plus alignées. Chaque contour laisse un résidu, et une image passe de quelques taches à
plusieurs centaines. Le nombre de taches est donc un indicateur de mouvement, et il est
gratuit : la mesure directe du décalage, par corrélation de phase, coûterait une
trentaine de secondes par vidéo.

**Pourquoi une limite relative.** La médiane de la vidéo adapte la limite au niveau de
bruit de chaque enregistrement.

**Choix.** Les détections sont retirées une seconde avant et après chaque image
suspecte, et la période est toujours signalée. L'outil préfère dire « je n'ai pas
regardé ici » plutôt que produire des centaines de fausses pistes.

**Limites.** Un essaim de 20 animaux dans une image serait pris pour un mouvement. Les
images abîmées au décodage produisaient le même symptôme ; elles sont retirées avant ce
comptage (§ 5 bis) et ne passent plus pour un mouvement.

## 7. Relier les taches dans le temps : le suivi

**Outils.** Une **prédiction à vitesse constante** et une **association gloutonne au
plus proche voisin**.

- Pour chaque piste ouverte, on prédit où elle devrait être à l'image suivante en
  prolongeant sa vitesse.
- On calcule la distance entre chaque prédiction et chaque tache, on garde les paires à
  moins de 120 px, et on les attribue de la plus proche à la plus lointaine.
- Une tache sans piste ouvre une nouvelle piste. Une piste sans tache pendant plus de 6
  images est close.

**Trois choix qui viennent des mesures.**

- *La vitesse se mesure sur 3 images au moins.* Le capteur tourne à 50 Hz et la vidéo à
  30 images par seconde : le pas de l'animal suit un motif « double, simple, simple ».
  Une vitesse mesurée sur deux images se trompe d'un facteur 2 une fois sur trois.
- *Les pistes qui ont déjà une vitesse choisissent en premier.* Sinon un fragment né à
  l'instant vole la tache de la vraie piste, qui se retrouve coupée.
- *La porte est large (120 px)* parce qu'un animal rapide parcourt 60 px par image et
  peut manquer plusieurs images.

**Pourquoi pas plus savant.** Un filtre de Kalman ou une affectation optimale
(algorithme hongrois) sont les outils habituels. Avec quelques taches par image, le
glouton donne le même résultat, et il se lit en vingt lignes. Kalman deviendra utile pour
élargir la porte pendant un trou (issue #21).

**Limites.** Une interruption de plus de 6 images coupe le passage en deux pistes (issue
#24). La piste commence quand la tache dépasse le seuil, donc après l'arrivée réelle de
l'animal (issue #8).

## 8. Fusionner les pistes jumelles

**Outil.** Une structure **union-find**, qui regroupe des éléments par paires jusqu'à
former des familles.

**Règle.** Deux pistes qui coexistent et restent à moins de 36 px l'une de l'autre sur
*toutes* leurs images communes sont un seul animal vu en deux morceaux. Elles sont
fusionnées : boîte englobante commune, centre pondéré par l'aire.

**Pourquoi « toutes ».** Deux vrais animaux qui volent ensemble s'écartent au moins une
fois ; deux fragments d'un même animal, jamais. Cette règle n'a été vérifiée que sur des
trajectoires fabriquées (issue #26).

## 9. Garder ce qui ressemble à un vol

Trois filtres, du plus simple au plus fin :

- **Au moins 6 détections** : élimine les étincelles.
- **Au moins 45 px entre le début et la fin** : élimine un point qui scintille sur place.
- **Virage médian sous 0,8 radian** : le virage est l'angle entre deux pas successifs.
  Un vol va quelque part, donc ses virages sont petits ; du bruit enchaîné par le suivi
  part dans tous les sens, avec un virage médian de 1,5 radian. La médiane tolère un
  demi-tour isolé, ce qu'une moyenne ne ferait pas.

**Limite.** Un vol très lent avance de quelques pixels par image, et l'arrondi des
positions fait tourner sa direction mesurée.

## 10. Aller vite sans changer le résultat

- **Tranches de temps** : la vidéo peut être découpée en tranches traitées par des
  processus séparés. Chaque tranche déborde d'une demi-fenêtre de fond sur ses voisines,
  ce qui rend le résultat identique au bit près au traitement d'un seul tenant.
- **Pas de saut dans le fichier** : chaque tranche relit la vidéo depuis le début. Le
  positionnement direct d'OpenCV se trompe d'une image dans un mp4, et une image de
  décalage suffit à changer les détections.
- **Un seul processus par défaut** : depuis que la médiane est rapide, une tranche de
  plus coûte plus en relecture qu'elle ne rapporte.
- **Taches mesurées en une passe** : l'aire et le pic de toutes les taches d'une image
  sont comptés en un seul parcours des pixels au-dessus du seuil, au lieu d'un masque de
  l'image par tache. Sans effet à 480 px ; à 1440 px, où le bruit fait des centaines de
  taches par image, la détection de l'extrait de test passe de 20 min à 18 s
  ([09](09-profilage.md)).

## 11. Savoir si c'est juste

Sans vérité terrain, tout réglage est un pari. Deux outils y répondent.

- **Le banc de cibles synthétiques** ([08](08-banc-de-mesure.md)) incruste de fausses
  chauves-souris dans une vraie vidéo : taches gaussiennes de taille et de contraste
  connus, sur des courbes de Bézier ou des vols de chasse simulés, avec la saccade des
  jumelles. On compte ce que l'outil retrouve, avec un **intervalle de Wilson** qui dit
  si un écart entre deux réglages est réel ou dû au hasard.
- **La vidéo de référence** : 14 pistes sur la 092, dont deux passages confirmés par la
  naturaliste, vérifiés par un test automatique.

Ce que le banc ne dit pas : si une piste réelle est une chauve-souris. Cela reste le
jugement de la naturaliste.

## Ce que cette logique suppose

| Hypothèse | Si elle est fausse |
| --- | --- |
| Les jumelles sont fixes | Les périodes de mouvement sont ignorées et signalées |
| L'animal bouge de plus que sa taille en une seconde | Il s'efface dans le fond |
| L'animal s'écarte du fond filtré de plus de 12 niveaux et de 8 fois le bruit du pixel | Il n'est pas vu ; le ciel saturé est aveugle ; devant de la végétation agitée, il faut un contraste plus fort |
| La cible a à peu près la taille du filtre gaussien (1,5 px d'origine) | Plus petite, elle perd du contraste ; plus grande, elle est moins rehaussée |
| Une cible allume au moins deux pixels de travail (960 px) | L'image est perdue (`min_area`) |
| Le gain des jumelles change peu en une demi-seconde | Autour d'un saut de 30 niveaux ou plus, quelques secondes sont ignorées |
| Le vol est régulier d'une image à la suivante | La piste se coupe ou est rejetée |
| Deux animaux proches finissent par s'écarter | Ils sont fusionnés en un seul |
| Le décodeur signale chaque image qu'il répare | Une image réparée en silence passe pour une vraie scène |
| Les images-clés arrêtent le dégât (IDR, sans image B) | Le dégât déborde au-delà de la plage écartée |
| ffprobe et OpenCV numérotent les images de la même façon | La vidéo échoue si les nombres d'images diffèrent ; sinon, non contrôlé |
