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
[README](../README.md#fichiers-produits). Un tableur réglé en anglais affiche le début
`0:00:04` comme `12:00:04 AM` : la valeur est juste (4 s), et un format Durée la montre en
`0:00:04`.

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

## Lire l'image résumé

L'image résumé montre d'un coup d'œil toutes les pistes d'une vidéo sur la scène sans animal :
où elles passent, dans quel sens, quand elles commencent, et quels moments l'outil n'a pas
regardés. Elle sert à choisir les extraits à ouvrir ; elle ne dit pas ce qu'est une piste.
Une zone sans trait n'est pas une zone sans passage : voir
[Ce que l'outil ne voit pas](#ce-que-loutil-ne-voit-pas).

Les repères numérotés (carrés blancs) ont été ajoutés pour ce document : l'outil ne les
dessine pas.

![Image résumé de l'extrait de test de la 092, avec les repères 1 à 9](images/resume-extrait.jpg)

*L'extrait de test de la 092 (40 s, de 3:24 à 4:05), affichage des jumelles masqué : 4 pistes.*

### Le bandeau, à droite

1. **Le nom de la vidéo**, terminé par « ... » s'il est trop long.
2. **Le nombre de pistes de l'image.** Au-delà de 20 pistes dans une même tranche de 10 min,
   l'image est coupée en parties d'au plus 20 pistes : une ligne `partie 1 / 3` s'ajoute alors
   entre le nom et ce nombre, qui est celui de la partie.
3. **L'intervalle couvert**, compté depuis le début de la vidéo, à la seconde inférieure :
   la vidéo entière jusqu'à 10 min, sinon la tranche (`10:00 - 20:00`) ; pour une partie, du
   début de sa première piste à la fin de la dernière à finir, arrondie au-dessus. Une piste
   à cheval sur deux tranches n'est dessinée que sur l'image de celle où elle commence : son
   trait peut continuer au-delà de l'intervalle.

   Viennent ensuite, quand il y en a, les périodes où rien n'a été cherché dans la tranche,
   arrondies vers l'extérieur à la seconde, celles qui se touchent réunies ; chaque partie
   de la tranche les répète. On lit jusqu'à trois lignes `hors analyse 1:02 - 1:15`, puis
   `+ 4 autres`, et jusqu'à trois lignes `illisible 2:10 - 2:12`, puis
   `+ 4 autres, 45 s illisibles en tout`, la durée comptant toutes les périodes illisibles de
   la tranche. L'extrait n'en a aucune : ces lignes manquent sur la capture.
4. **La légende** : la pastille de chaque piste et son début, à la seconde inférieure, sur
   plusieurs colonnes s'il le faut. Le numéro est celui du tableau des pistes et du nom de
   l'extrait (`piste_01_0m04s.mp4`) ; le début au centième est dans le tableau.

### L'image

5. **Le trait** : le trajet de la piste, qui relie ses points, bordé de noir pour rester
   visible sur un fond clair.
6. **La flèche** : posée au bout du trait, à la fin de la piste, elle pointe dans le sens du
   vol à ce moment-là. Le départ est l'autre bout du trait, sans marque.
7. **La pastille** : le disque numéroté de la piste. Elle est posée vers le quart du trajet,
   puis glissée le long du trait pour s'écarter des autres pastilles, des pointes de flèche et
   des endroits où un autre trait passe tout près. Elle ne marque donc **pas le départ** :
   pour le trouver, suivre le trait à l'opposé de la flèche.

   Quand aucune place sur le trait n'évite une autre pastille, la pastille est décalée à
   côté ; si elle ne touche plus son trait, un **trait de rappel** de sa couleur la relie au
   point le plus proche de son trajet. Les quatre pastilles de l'extrait sont sur leur trait :
   il n'y en a aucun.
8. **La zone masquée** par `--osd-region` : un gris très foncé uni, plus sombre que le
   bandeau, à la place de l'affichage des jumelles (heure, date, batterie, chrono ;
   `--osd-region 0,0,1,0.07` pour les Symbion). L'outil n'y cherche rien. Sans cette
   option, l'affichage reste visible dans le fond, et les inscriptions fixes que l'option ne
   couvre pas restent aussi : ici les grossissements « 2.0x » et « 6.5x », à peine visibles
   en bas de l'image.

Le fond est la scène sans animal de toute la vidéo ([fond médian](#fond-médian)), assombrie
pour que les traits ressortent.

### La frise du temps, sous l'image

9. **La frise** couvre toute la vidéo, ou toute la tranche de 10 min, même sur une partie.

![Gros plan de la frise du temps de l'extrait, avec les repères 10 à 13](images/resume-frise.png)

10. **La bande « hors analyse »** : un rectangle gris par période hors analyse, toutes et pas
    seulement les trois du bandeau, large d'au moins 2 pixels même pour un instant. Vide sur
    l'extrait : aucune image n'y a été écartée.
11. **La bande « illisible »** : de même, un rectangle blanc cassé par période illisible.
    Vide aussi sur l'extrait.
12. **Les départs des pistes** : un petit trait de la couleur de la piste, au-dessus de
    l'axe, à son début. Sur une partie, les départs des autres parties de la même tranche y
    sont aussi, en gris.
13. **La graduation** : les minutes, tous les 1, 2, 5 ou 10 min selon la place ; une seule
    ici, 0:00, puisque l'extrait dure 40 s.

### La couleur ne veut rien dire

La couleur sert seulement à distinguer une piste de ses voisines sur la même image. Il y en a
12 : chaque piste prend la couleur la moins présente parmi les traits proches déjà dessinés,
et à égalité la moins employée. Les 12 premières pistes d'une image ont chacune la sienne ; à partir de la
13ᵉ, une couleur revient, sur une piste éloignée si possible. Deux traits de même couleur
sont donc deux pistes, et la même couleur sur deux images ne désigne pas la même piste. Elle ne
dit rien de l'animal, de sa vitesse ni de la confiance de l'outil, et les extraits ne la
reprennent pas : leur boîte est toujours rouge et la trace du vol toujours jaune. Seul le numéro identifie une piste.

### Refaire les captures

<details>
<summary>Commandes, depuis la racine du dépôt</summary>

Les coordonnées des repères valent pour l'image résumé de l'extrait, 1879 × 1215 pixels :
si elle change, les replacer.

```bash
LANGUAGE=fr cargo run --release -p batdetect -- \
  tests/fixtures/video_092_original_3m24-4m05.mp4 --osd-region 0,0,1,0.07 -o out/resume
resume=out/resume/video_092_original_3m24-4m05/video_092_original_3m24-4m05_resume.png
reperes=()
repere() {
  local n=$1 x=$2 y=$3 tx=$4 ty=$5
  reperes+=(-stroke black -strokewidth 8 -draw "line $x,$y $tx,$ty"
    -stroke white -strokewidth 3 -draw "line $x,$y $tx,$ty"
    -stroke black -strokewidth 3 -fill white
    -draw "roundrectangle $((x - 22)),$((y - 22)) $((x + 22)),$((y + 22)) 6,6"
    -stroke none -fill black -gravity Center
    -annotate "$(printf '%+d%+d' $((x - width / 2)) $((y - height / 2)))" "$n" -gravity NorthWest)
}
width=1949 height=1215
repere 1 1915 44 1866 44
repere 2 1635 82 1562 82
repere 3 1650 120 1594 120
repere 4 1660 250 1612 250
repere 5 560 560 600 628
repere 6 170 700 106 772
repere 7 990 375 918 428
repere 8 720 38 720 38
repere 9 1500 1140 1426 1130
magick "$resume" -background 'rgb(32,32,32)' -gravity East -splice 70x0 -gravity NorthWest \
  -stroke white -strokewidth 3 -draw 'line 1612,148 1612,352' \
  -font crates/draw/fonts/Roboto.ttf -pointsize 32 "${reperes[@]}" \
  -resize 1200x -quality 80 docs/images/resume-extrait.jpg
reperes=() width=1520 height=215
repere 10 800 30 800 76
repere 11 1480 115 1424 115
repere 12 1040 172 977 137
repere 13 290 170 226 168
magick "$resume" -crop 1440x135+0+1080 +repage -background 'rgb(32,32,32)' \
  -gravity North -splice 0x60 -gravity South -splice 0x20 -gravity East -splice 80x0 \
  -gravity NorthWest -font crates/draw/fonts/Roboto.ttf -pointsize 32 "${reperes[@]}" \
  docs/images/resume-frise.png
```

</details>

## Glossaire

### Amplitude

Le plus fort écart au fond médian atteint par la piste, en niveaux de gris, mesuré sur
l'image filtrée par l'outil et non sur l'image brute
([docs/10](10-logique-de-detection.md#4-décider-pixel-par-pixel--le-seuil)). On la lit dans
le tableau des pistes : plus elle est faible, plus la cible se voit mal à l'œil.

### Corde

La distance en ligne droite entre le début et la fin de la piste, en pixels de la vidéo
d'origine, dans le tableau des pistes (`corde (px)`). Une piste longue à petite corde a tourné
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

Une image qui montre toutes les pistes sur le fond médian : un trait, une flèche au bout
(sens du vol) et une pastille numérotée par piste, un bandeau à droite et une frise du temps
dessous. Une vidéo de plus de 10 min donne une image par tranche de 10 min, et une tranche de
plus de 20 pistes est coupée en parties. Comment la lire, repère par repère :
[Lire l'image résumé](#lire-limage-résumé).

### Images estimées

Les images d'une piste sans point, entre deux points : la boîte de l'extrait y passe en trait
fin, à une position estimée. Le tableau des pistes en donne le nombre (`images comblées`).

### Passage

Le vol d'une chauve-souris que Manon a confirmé en regardant son extrait ; si l'outil l'a
coupé en deux pistes, c'est un seul passage. L'outil ne produit que des pistes : les
passages viennent des verdicts.

### Pastille

Le disque numéroté posé sur chaque trajectoire de l'image résumé, vers le quart du trajet et
à l'écart des croisements : elle ne marque pas le départ de la piste. Quand elle a dû être
décalée au point de ne plus toucher son trait, un trait de rappel de sa couleur l'y relie
([repère 7](#limage)). Son numéro est celui de la piste dans la légende, dans le tableau des
pistes et dans le nom de l'extrait. Les numéros suivent l'ordre chronologique et changent dès
qu'un réglage ajoute ou retire une piste plus tôt dans la vidéo.

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
L'outil n'en tire rien ; il se note dans la colonne `verdict` du tableau des pistes, sur
la ligne de la piste. Un verdict écrit garde le tableau tel quel à la relance, qui n'écrit
alors pas de nouveau tableau ; seule cette colonne le protège : un tableau trié, ou dont des
lignes ont été retirées, sans aucun verdict, est remplacé. Hors du tableau, on le note avec le temps de début de la piste plutôt
qu'avec son numéro, qui peut changer (ceux de la 092 sont dans
[docs/03](03-resultats-092.md#verdict-de-manon-9-octobre-2026)).

### Vitesse

Le chemin parcouru par la piste divisé par sa durée, en pixels par seconde dans l'image,
dans le tableau des pistes. Ce n'est pas une vitesse en mètres par seconde : la distance
entre l'animal et les jumelles est inconnue.
