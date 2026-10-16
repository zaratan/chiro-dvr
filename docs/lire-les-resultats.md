# Lire les résultats

Ce document s'adresse à qui relit les résultats de `batdetect` : l'image résumé, le
tableau des pistes et les extraits. Il définit les mots qu'on y croise ; la façon dont
l'outil trouve les pistes est expliquée sans jargon dans
[docs/10](10-logique-de-detection.md#les-techniques-en-simple).

L'outil ne sait pas reconnaître une chauve-souris : c'est à Manon, la naturaliste, de
trancher, et la façon dont l'outil est réglé pour l'aider est dite dans
[Ce que l'outil ne voit pas](#ce-que-loutil-ne-voit-pas).

D'où deux mots distincts. Une **piste** est ce que l'outil a trouvé : quelque chose qui
s'est déplacé dans l'image. Un **passage** est le vol d'une chauve-souris que Manon a
confirmé sur l'extrait. L'outil compte des pistes ; seuls les verdicts comptent des
passages.

## Ce que l'outil ne voit pas

L'outil dit « quelque chose a bougé ici », jamais « c'est une chauve-souris ». Ses réglages
préfèrent une piste de trop à un passage manqué : c'est à l'œil, sur l'extrait, que vous
l'écartez. La piste lente de 3:29 sur la 092 n'est pas une chauve-souris, et l'outil la sort
quand même : c'est voulu. Il peut aussi manquer un passage, en couper un en deux ou en
réunir deux : un nombre de pistes n'est pas un nombre de chauves-souris, et une zone sans
trajectoire n'est pas une zone sans passage.

Le tableau dit ce que vous pouvez voir, ce qui a pu se passer et comment le vérifier. Le
« pourquoi » renvoie aux [techniques en simple][techniques]. Le numéro entre parenthèses
renvoie au suivi du défaut ; une ligne sans numéro est une limite de principe.

| Ce que vous voyez | Ce qui a pu se passer | Quoi faire |
| --- | --- | --- |
| Une piste s'arrête net dans le ciel, ou une partie du ciel n'a aucune piste | Le ciel est d'une seule teinte uniforme, tout noir sur vos vidéos : une chauve-souris sombre s'y confond, et rien ne signale cette zone ([« Un seuil propre à chaque endroit »][techniques]) | Revoir la vidéo d'origine là où la piste s'arrête, et ne pas lire ce ciel comme vide ([#9]) |
| Deux pistes qui se suivent, la seconde repartant là où la première s'arrête | La cible a disparu trop longtemps et le vol a été coupé en deux ([« Le suivi image par image »][techniques]) | Regarder les deux extraits : si c'est le même animal, c'est un seul passage ([#24]) |
| Une piste qui commence au tout début ou finit à la toute fin d'un clip | Le vol continue dans le clip voisin, traité séparément ; et la scène sans animal y est estimée sur moins d'images, donc la détection y est moins sûre ([« La scène sans animal »][techniques]) | Ouvrir la fin du clip précédent ou le début du suivant : un vol à cheval sur deux clips est un seul passage ([#11]) |
| Un animal posé, en vol stationnaire, très lent ou qui tourne sur place, sans piste | Il s'est fondu dans le décor, ou son trajet n'a pas été reconnu comme un vol ([« La scène sans animal »][techniques], [« Ressembler à un vol »][techniques]) | Revoir la vidéo d'origine : l'outil voit mal ces animaux ([#22]) |
| Une cible faible, petite ou lointaine, ou qui passe devant de la végétation qui bouge, que vous suivez à l'œil, sans piste | Sur aucune image, ou sur trop peu, elle ne se détache assez du fond : votre œil additionne plusieurs images, l'outil décide image par image ([« Le filtre à la taille de la cible »][techniques], [« Un seuil propre à chaque endroit »][techniques], [« Les taches »][techniques]) | Noter son temps et la compter vous-même : c'est un passage que l'outil a manqué ([#23]) |
| Dans l'extrait, la boîte passe en trait fin sur quelques images | La piste n'a pas de point sur ces images ; la boîte y est à une position estimée ([« Le suivi image par image »][techniques]) | Regarder l'extrait zoomé, s'il existe, ou la vidéo d'origine pour voir où est vraiment l'animal ([#21]) |
| L'animal est déjà visible dans l'extrait avant que la boîte n'apparaisse | La piste ne commence qu'une fois que la cible se détache assez du fond ; sur des vols de chasse fabriqués, ce retard dépasse parfois quelques images ([« Le suivi image par image »][techniques]) | Regarder l'extrait depuis son tout début : la durée et la vitesse du tableau des pistes ne couvrent alors qu'une partie du vol ([#42]) |
| L'animal vole encore dans l'extrait après que la boîte a disparu | La piste finit à la dernière image où la cible se détache assez du fond ; sur des vols de chasse fabriqués, cette fin arrive parfois quelques images trop tôt ([« Le suivi image par image »][techniques]) | Regarder l'extrait jusqu'à sa toute fin : la durée et la vitesse du tableau des pistes ne couvrent alors qu'une partie du vol ([#42]) |
| Une seule piste là où vous voyez deux animaux | Deux animaux restés tout près l'un de l'autre sont réunis en une piste ; cette règle n'a jamais été éprouvée sur deux vraies chauves-souris ([« La fusion des morceaux »][techniques]) | Compter les animaux à l'œil dans l'extrait, et noter ce moment : il servira à éprouver l'outil ([#26]) |
| Une piste très lente au milieu de vols rapides | L'outil ne distingue pas un vol d'un déplacement lent : c'est le cas de la piste de 3:29 sur la 092 ([verdict](03-resultats-092.md#verdict-de-manon-9-octobre-2026), [« Ressembler à un vol »][techniques]) | Comparer sa vitesse dans le tableau des pistes, puis trancher à l'extrait ([#28]) |
| Une chauve-souris plus claire que le ciel derrière elle | L'outil cherche dans les deux sens, plus clair ou plus sombre, mais n'a jamais été essayé sur une vraie vidéo de ce genre ([« Un seuil propre à chaque endroit »][techniques]) | Revoir la vidéo d'origine, et garder la vidéo : elle servira à éprouver l'outil ([#27]) |
| Une période hors analyse ou illisible sur l'image résumé | Les jumelles ont bougé, la luminosité a sauté d'un coup, un essaim a rempli l'image, ou le fichier est abîmé : rien n'y a été cherché ([« Les périodes ignorées »][techniques]) | Revoir la vidéo d'origine aux moments donnés par le bandeau (les trois premiers de chaque sorte) et par la frise du temps sous l'image, qui les montre tous |
| Une piste sur une image visiblement abîmée (blocs, morceaux d'image décalés), hors des périodes illisibles | Le fichier a été réparé à la lecture sans que rien ne le signale, et l'image passe pour une vraie scène ([« Les périodes ignorées »][techniques]) | L'écarter à l'extrait |
| Un nombre de passages à la fin de la relecture | L'outil manque des passages, pour les raisons ci-dessus, et aucun comptage de référence ne dit encore dans quelle proportion | Le présenter comme un minimum, et ne pas le comparer tel quel à un comptage à l'œil ou à une autre soirée ([#25]) |

### Prise de vue

Une règle est vérifiée sur vos vidéos : les jumelles doivent rester fixes, sur trépied,
sinon le moment est hors analyse ([docs/01](01-contexte.md#ce-que-la-prise-de-vue-impose)).
Les conseils suivants viennent de la littérature sur l'observation en caméra thermique
([docs/04](04-outils-existants.md)) et n'ont pas encore été essayés sur vos jumelles
([#15]) :

- palette et contraste fixes ;
- calibrage (NUC) manuel ou semi-automatique pendant l'enregistrement ;
- débit maximal ;
- pas de zoom numérique.

## Où sont les résultats

Pour chaque vidéo, un dossier à son nom, dans `batdetect/` à côté des vidéos, ou dans le
dossier choisi au lancement (`-o`). Sous une locale française : l'image résumé
`<vidéo>_resume.png`, le tableau des pistes `<vidéo>_pistes.csv`, les extraits dans
`extraits/` (`piste_01_0m04s.mp4`, et `piste_01_0m04s_zoom.mp4` pour l'extrait zoomé), et
`params.json`, les réglages du calcul. Sous une autre locale, les mêmes fichiers en anglais
(`_summary.png`, `_tracks.csv`, `clips/track_01_0m04s.mp4`). La liste complète est dans le
[README](../README.md#fichiers-produits).

À la fin du calcul, la commande écrit un bilan de la soirée : pour chaque vidéo sa durée, son
nombre de pistes, le temps non analysé et le temps de calcul, puis à part les vidéos qui ont
trop de pistes, celles en échec avec ce qu'il faut faire, et les entrées refusées. Sous macOS,
lancée dans un terminal, elle annonce aussi la fin par une notification et ouvre chaque dossier de
résultats dans le Finder (`--no-notify` pour s'en passer).

[techniques]: 10-logique-de-detection.md#les-techniques-en-simple
[#9]: https://github.com/zaratan/chiro-dvr/issues/9
[#11]: https://github.com/zaratan/chiro-dvr/issues/11
[#15]: https://github.com/zaratan/chiro-dvr/issues/15
[#21]: https://github.com/zaratan/chiro-dvr/issues/21
[#22]: https://github.com/zaratan/chiro-dvr/issues/22
[#23]: https://github.com/zaratan/chiro-dvr/issues/23
[#24]: https://github.com/zaratan/chiro-dvr/issues/24
[#25]: https://github.com/zaratan/chiro-dvr/issues/25
[#26]: https://github.com/zaratan/chiro-dvr/issues/26
[#27]: https://github.com/zaratan/chiro-dvr/issues/27
[#28]: https://github.com/zaratan/chiro-dvr/issues/28
[#42]: https://github.com/zaratan/chiro-dvr/issues/42

## Glossaire

### Amplitude

Le plus fort écart au fond médian atteint par la piste, en niveaux de gris, mesuré sur
l'image filtrée par l'outil et non sur l'image brute
([docs/10](10-logique-de-detection.md#4-décider-pixel-par-pixel--le-seuil)). On la lit dans
le tableau des pistes : plus elle est faible, plus la cible se voit mal à l'œil.

### Corde

La distance en ligne droite entre le début et la fin de la piste, en pixels de la vidéo
d'origine, dans le tableau des pistes (`chord_px`). Une piste longue à petite corde a tourné
ou est revenue sur ses pas.

### Extrait

Une courte vidéo par piste, de peu avant son début à peu après sa fin, avec une boîte
autour de la cible, son numéro et la trace de son vol juste derrière elle ; les autres
pistes visibles au même moment y sont dessinées aussi. C'est sur l'extrait que se donne le
verdict.

### Extrait zoomé

Un second extrait, rangé à côté du premier, pour les pistes petites ou faibles : la même
scène recadrée sur la trajectoire, agrandie jusqu'à quatre fois et montrée à la taille de la
vidéo, et ralentie quatre fois. Seule la trace du vol y est dessinée, sans boîte, sans numéro ni autres pistes,
pour ne pas cacher une cible ténue.

### Fond médian

La scène sans animal : pour chaque pixel, la valeur médiane des images voisines dans le
temps, qu'un animal qui ne fait que traverser ne modifie pas
([docs/10](10-logique-de-detection.md#2-estimer-le-fond--la-médiane-temporelle)). Un objet
qui reste immobile plus d'un instant entre dans ce fond et n'est plus vu ; le décor assombri
de l'image résumé est obtenu de la même façon, mais sur toute la vidéo.

### Garde-fou du nombre de pistes

Au-delà d'un certain nombre de pistes, elles sont presque sûrement du bruit d'image :
l'outil écrit l'image résumé et le tableau des pistes, mais aucun extrait. Le bilan de la
soirée compte cette vidéo à part, avec son nombre de pistes, et les autres vidéos du lot
sont traitées normalement.

### Image résumé

Une image qui montre les pistes sur le fond médian, chacune avec une couleur qui la
distingue de ses voisines, une flèche dans le sens du vol et sa pastille. Le bandeau à droite
donne le nom de la vidéo, le nombre de pistes, le début et la fin de la tranche couverte, les
périodes hors analyse ou illisibles, puis la légende (numéro et début de chaque piste, à la
seconde). Une frise du temps sous l'image montre la tranche, les départs des pistes et toutes
les périodes ignorées. Une vidéo de plus de 10 min donne une image par tranche de 10 min ;
une tranche de plus de 20 pistes est coupée en parties (`_1_sur_3`, « partie 1 / 3 » dans le
bandeau), et la frise d'une partie montre aussi, en gris, les départs des autres parties.

### Images estimées

Les images d'une piste sans point, entre deux points : la boîte de l'extrait y passe en trait
fin, à une position estimée. Le tableau des pistes en donne le nombre (`filled_frames`).

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
Le bandeau de l'image résumé en donne le début et la fin (les trois premières plages, puis
le nombre des autres et la durée illisible en tout) ; la frise du temps les montre toutes.
Comme une période hors analyse, rien n'y a été cherché.

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
