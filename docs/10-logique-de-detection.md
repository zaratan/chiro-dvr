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
vidéo ─► image grise réduite ─► fond (médiane) ─► résidu ─► seuil ─► taches
                                                                        │
      pistes gardées ◄─ filtres ◄─ fusion des jumelles ◄─ suivi ◄─ images instables retirées
```

| Étape | Module | Outil | Question posée |
| --- | --- | --- | --- |
| Lecture | `video.py`, `prefetch.py` | OpenCV, moyenne de surface | Que voit le capteur, en plus petit ? |
| Fond | `detect.py`, `median.py` | médiane temporelle | À quoi ressemble la scène sans animal ? |
| Gain | `detect.py` | différence de moyennes | L'image entière a-t-elle changé de luminosité ? |
| Seuil | `detect.py` | valeur absolue, seuil fixe | Ce pixel s'écarte-t-il assez du fond ? |
| Taches | `detect.py` | fermeture morphologique, composantes connexes | Quels pixels forment un même objet ? |
| Stabilité | `stability.py` | comptage, médiane | Les jumelles ont-elles bougé ? |
| Suivi | `track.py` | prédiction linéaire, plus proche voisin | Cette tache prolonge-t-elle une piste ? |
| Jumelles | `track.py` | union-find | Deux pistes sont-elles un seul animal ? |
| Filtres | `track.py` | comptage, distance, angle médian | Cette piste ressemble-t-elle à un vol ? |

## 1. Lire et réduire l'image

**Outil.** OpenCV décode la vidéo (par ffmpeg), convertit en niveaux de gris de 0 à 255,
et réduit l'image à 480 px de large par **moyenne de surface** (`INTER_AREA`) : chaque
pixel réduit est la moyenne des 9 pixels d'origine qu'il recouvre, sur une vidéo de
1440 px.

**Pourquoi.** Moyenner 9 pixels divise le bruit par 3 environ et le calcul par 9. La
réduction sert donc de débruitage gratuit.

**Le prix.** Une cible plus petite que 3 px se dilue dans la moyenne : elle perd 30 à
50 % de son contraste (mesuré, [07](07-ameliorer-la-detection.md)). C'est aujourd'hui le
premier frein sur les petites cibles (issue #7).

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

**Outil.** Le **résidu** est `image − fond − décalage de gain`. Un pixel est retenu si
la valeur absolue du résidu dépasse 25 niveaux.

**Pourquoi la valeur absolue.** L'animal peut être plus sombre que le fond (devant la
roche chaude) ou plus clair (devant un ciel froid). Le signe n'est pas supposé.

**Pourquoi un seuil fixe.** C'est le test le plus simple : sur la vidéo de référence, le
bruit reste sous 25 et les passages montent de 49 à 130.

**Limite, et c'est la principale.** Un seuil unique traite pareil la roche calme et la
végétation qui bouge. La bonne grandeur serait le résidu divisé par le bruit local de
chaque pixel, c'est-à-dire un *score z*. C'est le seuil par pixel de l'issue #7, avec un
filtrage à la taille de la cible qui est l'optimum théorique pour une tache dans du
bruit.

## 5. Regrouper les pixels en taches

**Outils.** Deux opérations classiques de morphologie mathématique.

- **Fermeture** (une dilatation puis une érosion, avec un disque de 6 px de rayon) : elle
  bouche les trous et recolle les fragments proches d'un même animal, sans grossir
  l'objet.
- **Composantes connexes** : chaque groupe de pixels qui se touchent reçoit un numéro.
  On en tire la boîte englobante, le centre de gravité, l'aire et l'amplitude maximale.

**Choix.** L'aire compte seulement les pixels réellement au-dessus du seuil, pas ceux
ajoutés par la fermeture : la fermeture sert à regrouper, pas à mesurer. Une tache est
gardée entre 4 et 2 700 px² d'origine. Le plancher laisse passer une tache d'un seul
pixel réduit : c'est ce qui trouve les petites cibles, au prix d'un bruit que le suivi
doit ensuite écarter.

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
erreurs de décodage du fichier produisent le même symptôme et font ignorer de longues
périodes (issue #1).

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
| L'animal s'écarte du fond de plus de 25 niveaux | Il n'est pas vu ; le ciel saturé est aveugle |
| Le vol est régulier d'une image à la suivante | La piste se coupe ou est rejetée |
| Deux animaux proches finissent par s'écarter | Ils sont fusionnés en un seul |
| Le fichier se décode sans erreur | Les erreurs passent pour un mouvement |
