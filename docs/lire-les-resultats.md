# Lire les résultats

Ce document s'adresse à qui relit les résultats de `batdetect` : l'image résumé, le
tableau des pistes et les extraits. Il définit les mots qu'on y croise ; la façon dont
l'outil trouve les pistes est expliquée sans jargon dans
[docs/10](10-logique-de-detection.md#les-techniques-en-simple).

L'outil est réglé pour manquer le moins possible : mieux vaut une fausse piste qu'on écarte
en quelques secondes à l'extrait qu'une chauve-souris manquée. Il sort donc des pistes qui
ne sont pas des chauves-souris, et c'est voulu : il ne sait pas reconnaître une
chauve-souris, c'est à Manon, la naturaliste, de trancher.

D'où deux mots distincts. Une **piste** est ce que l'outil a trouvé : quelque chose qui
s'est déplacé dans l'image. Un **passage** est le vol d'une chauve-souris que Manon a
confirmé sur l'extrait. L'outil compte des pistes ; seuls les verdicts comptent des
passages.

## Glossaire

### Amplitude

Le plus fort écart au fond médian atteint par la piste, en niveaux de gris, mesuré sur
l'image filtrée par l'outil et non sur l'image brute
([docs/10](10-logique-de-detection.md#4-décider-pixel-par-pixel--le-seuil)). On la lit dans
le tableau des pistes : plus elle est faible, plus la cible se voit mal à l'œil.

### Extrait

Une courte vidéo par piste, de peu avant son début à peu après sa fin, avec une boîte
autour de la cible, son numéro et la trace de son vol juste derrière elle ; les autres
pistes visibles au même moment y sont dessinées aussi. C'est sur l'extrait que se donne le
verdict.

### Extrait zoomé

Un second extrait, rangé à côté du premier, pour les pistes petites ou faibles : la même
scène recadrée sur la trajectoire, agrandie autant que la trajectoire le permet, et
ralentie. Seule la trace du vol y est dessinée, sans boîte, sans numéro ni autres pistes,
pour ne pas cacher une cible ténue.

### Fond médian

La scène sans animal : pour chaque pixel, la valeur médiane des images voisines dans le
temps, qu'un animal qui ne fait que traverser ne modifie pas
([docs/10](10-logique-de-detection.md#2-estimer-le-fond--la-médiane-temporelle)). Un objet
qui reste immobile plus d'un instant entre dans ce fond et n'est plus vu ; le décor assombri
de l'image résumé est obtenu de la même façon, mais sur toute la vidéo.

### Garde-fou du nombre de pistes

Au-delà d'un certain nombre de pistes, elles sont presque sûrement du bruit d'image :
l'outil écrit l'image résumé et le tableau des pistes, mais aucun extrait. La vidéo est
signalée en échec à la fin du calcul, et les autres vidéos du lot sont traitées
normalement.

### Image résumé

Une image par vidéo qui montre toutes les pistes sur le fond médian, chacune avec une
couleur qui la distingue de ses voisines, une flèche dans le sens du vol et sa pastille.
Le bandeau à droite donne le nom de la vidéo, le nombre de pistes, le début et la fin de la
tranche couverte, les périodes hors analyse ou illisibles, puis la légende (numéro et début
de chaque piste, à la seconde) ; une longue vidéo en donne plusieurs, une par tranche de
temps.

### Passage

Le vol d'une chauve-souris que Manon a confirmé en regardant son extrait ; si l'outil l'a
coupé en deux pistes, c'est un seul passage. L'outil ne produit que des pistes : les
passages viennent des verdicts.

### Pastille

Le disque numéroté posé sur chaque trajectoire de l'image résumé ; son numéro est celui de
la piste dans la légende, dans le tableau des pistes et dans le nom de l'extrait. Les
numéros suivent l'ordre chronologique et changent dès qu'un réglage ajoute ou retire une
piste plus tôt dans la vidéo.

### Période hors analyse

Un moment où l'image se remplit soudain de taches, le plus souvent parce que les jumelles
ont bougé : l'outil ignore ces images et leurs abords
([docs/10](10-logique-de-detection.md#6-écarter-les-images-où-les-jumelles-bougent)). Le
bandeau de l'image résumé en donne le début et la fin, comptés depuis le début de la vidéo
(les premières seulement s'il y en a beaucoup) : rien n'y a été cherché, ces moments sont à
revoir à l'œil.

### Période illisible

Un moment où le fichier vidéo est abîmé, avec des images manquantes ou réparées à la
lecture, que l'outil ignore avec une marge de chaque côté
([docs/10](10-logique-de-detection.md#5-bis-écarter-les-images-que-le-décodeur-a-réparées)).
Le bandeau de l'image résumé en donne seulement la durée totale et le nombre de plages, pas
leurs moments ; comme une période hors analyse, rien n'y a été cherché.

### Piste

Ce que l'outil trouve : une suite de points sur des images proches, reliés comme le vol
d'un même objet, et dont le trajet ressemble à un vol
([docs/10](10-logique-de-detection.md#7-relier-les-taches-dans-le-temps--le-suivi),
[§ 9](10-logique-de-detection.md#9-garder-ce-qui-ressemble-à-un-vol)). Ce peut être une
chauve-souris, un autre animal ou du bruit d'image, et un même vol peut être coupé en deux
pistes ; chaque piste a sa pastille sur l'image résumé, sa ligne dans le tableau des pistes
et, sauf au-delà du garde-fou, son extrait.

### Point

Une détection : la tache (groupe de pixels voisins qui se détachent du fond) retenue pour
une piste sur une image
([docs/10](10-logique-de-detection.md#5-regrouper-les-pixels-en-taches)). Le tableau des
pistes donne le nombre de points de chaque piste ; dans l'extrait, la boîte suit aussi les
images sans point, en trait plus fin, à une position estimée entre deux points, pour ne pas
clignoter.

### Réglage

Une valeur qui guide le calcul, comme le contraste minimal d'une cible ou le nombre minimal
de points d'une piste ; chaque calcul enregistre les siens à côté des résultats. Changer un
réglage de détection ou de suivi peut changer les pistes trouvées et leurs numéros.

### Surface

La surface de la plus grande tache de la piste, comptée en pixels de la vidéo d'origine qui
se détachent du fond, dans le tableau des pistes
([docs/10](10-logique-de-detection.md#5-regrouper-les-pixels-en-taches)). Elle dépend de la
distance et du contraste de l'animal autant que de sa taille.

### Verdict

Le jugement de Manon, à l'œil sur l'extrait : cette piste est-elle une chauve-souris ?
L'outil ne s'en sert pas ; on note chaque verdict avec le temps de début de la piste, au
centième de seconde comme dans le tableau des pistes, car son numéro peut changer (ceux de
la 092 sont dans [docs/03](03-resultats-092.md#verdict-de-manon-9-octobre-2026)).

### Vitesse

Le chemin parcouru par la piste divisé par sa durée, en pixels par seconde dans l'image,
dans le tableau des pistes. Ce n'est pas une vitesse en mètres par seconde : la distance
entre l'animal et les jumelles est inconnue.
