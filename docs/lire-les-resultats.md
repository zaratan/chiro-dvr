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
| Deux pistes qui se suivent, la seconde repartant là où la première s'arrête | La chauve-souris a disparu plusieurs secondes, elle est sortie de l'image puis revenue, ou elle a beaucoup changé de direction pendant qu'elle était cachée, ou elle semble repartir presque sur place, bien plus lentement qu'elle ne volait : l'outil ne recolle que les disparitions de moins d'une seconde, derrière un buisson par exemple, quand le vol reprend là où on l'attend ([« Le recollage »][techniques]) | Regarder les deux extraits : si c'est le même animal, c'est un seul passage |
| Une piste qui commence au tout début ou finit à la toute fin d'un clip | Le vol continue dans le clip voisin, traité séparément ; et la scène sans animal y est estimée sur moins d'images, donc la détection y est moins sûre ([« La scène sans animal »][techniques]) | Ouvrir la fin du clip précédent ou le début du suivant : un vol à cheval sur deux clips est un seul passage ([#11]) |
| Un animal posé, en vol stationnaire, très lent ou qui tourne sur place, sans piste | Il s'est fondu dans le décor, ou son trajet n'a pas été reconnu comme un vol ([« La scène sans animal »][techniques], [« Ressembler à un vol »][techniques]) | Revoir la vidéo d'origine : l'outil voit mal ces animaux ([#22]) |
| Une cible faible, petite ou lointaine, ou qui passe devant de la végétation qui bouge, que vous suivez à l'œil, sans piste | Sur aucune image, ou sur trop peu, elle ne se détache assez du fond : votre œil additionne plusieurs images, l'outil décide image par image ([« Le filtre à la taille de la cible »][techniques], [« Un seuil propre à chaque endroit »][techniques], [« Les taches »][techniques]) | Noter son temps et la compter vous-même : c'est un passage que l'outil a manqué ([#23]) |
| Dans l'extrait, la boîte passe en trait fin sur quelques images | La piste n'a pas de point sur ces images, même en comptant les taches faibles qui comblent un vol ; la boîte y est à une position estimée ([« Le suivi image par image »][techniques], [« Les taches faibles encadrées »][techniques]) | Regarder l'extrait zoomé, s'il existe, ou la vidéo d'origine pour voir où est vraiment l'animal |
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

## La page des résultats

À la fin du calcul, la page des résultats s'ouvre dans le navigateur. C'est le fichier
`resultats.html`, dans le dossier `batdetect/` à côté des vidéos, ou dans le dossier choisi au
lancement. Pour la rouvrir plus tard, double-cliquer dessus. Elle marche sans internet, et
encore après avoir copié tout le dossier `batdetect/` sur une clé ou sur un autre ordinateur.

Ce qu'elle montre, de haut en bas :

- le total de la soirée : les vidéos, celles réussies, et le nombre de pistes ; quand il y en a, celles qui ont trop de pistes, celles en échec, les entrées refusées, les inachevées, celles d'une ancienne version, celles en attente et celles pas encore traitées ;
- quand il y en a eu au dernier lancement, les entrées refusées au départ, avant les vidéos : chaque vidéo, dossier ou fichier écarté avant le calcul, avec la raison et ce qu'il faut faire, les mêmes lignes que dans le bilan. Rien n'a été fait pour elles : les corriger comme indiqué, puis relancer batdetect. Le lancement suivant remplace ces lignes par les siennes, ou les retire s'il n'a rien refusé ;
- une section par vidéo, dans l'ordre alphabétique des noms de leurs dossiers (majuscules
  d'abord), avec la date du calcul. Les soirées
  précédentes rangées dans le même dossier y sont aussi. Les vidéos du dernier calcul sont
  dépliées, les autres repliées : un clic sur le nom ouvre ou ferme la section.

En haut de la page, une barre reste visible quand on fait défiler : le décompte des verdicts de la soirée (oui, non, doute, à voir), l'état des verdicts, et les boutons « Télécharger les verdicts » et « Recharger un fichier de verdicts… » ([Noter le verdict](#noter-le-verdict)).

Dans une section, l'image résumé et la liste des pistes. Chaque piste a son numéro, de la couleur de sa trajectoire sur l'image, son début, écrit partout dans la page comme dans le tableau des pistes (« 0:00:51 »), sa durée, son verdict (« à voir » tant qu'il n'est pas donné) et l'espèce notée, et une étiquette « Zoom » quand l'extrait zoomé existe. L'en-tête de la vidéo dit combien de ses pistes sont jugées. Une vidéo qui n'a ni extrait ni image résumé, comme une vidéo à trop de pistes, ne se juge pas dans la page : ses pistes sont listées avec leur numéro, leur début et leur durée seulement, et ne comptent pas dans « à voir » ; un verdict noté sur elles lors d'un calcul précédent n'y est pas montré, mais il compte dans le décompte du haut et reste dans le fichier de verdicts.

L'image de la page est redessinée par la page elle-même : sur la scène de la vidéo, sans aucune trace, elle trace les trajectoires, les flèches et les pastilles numérotées aux places que l'outil a choisies pour l'image résumé. Les traits gardent la même épaisseur quelle que soit la taille de l'image, et les numéros sont écrits dans la police de l'ordinateur ; l'image redessinée peut donc différer très légèrement de l'image résumé.

- Au-dessus de l'image, une ligne dit l'intervalle montré, le nombre de pistes, et les périodes hors analyse et illisibles (« rien hors analyse, rien d'illisible » quand il n'y en a pas ; au-delà de trois périodes, « + 4 autres »). Le lien « Ouvrir l'image complète » ouvre l'image résumé, avec son bandeau et sa frise : c'est le fichier à garder ou à envoyer.
- Sous l'image, la frise du temps : un trait par début de piste, de la couleur de sa pastille (en gris, les pistes des autres parties de la tranche), les minutes, et les bandes « hors analyse » et « illisible » seulement quand il y en a. Le survol d'un trait donne la piste et son début.
- La légende du bandeau de l'image résumé n'est pas redessinée : la liste des pistes, avec leurs pastilles et leurs débuts, en tient lieu.
- Une vidéo calculée par une version précédente de l'outil garde son image résumé, sans frise sous l'image ; relancer la même commande la refait avec l'image redessinée.

- Un clic sur une piste (sa ligne ou son numéro) ouvre le lecteur à la place de l'image : l'extrait de la piste passe en boucle, en grand, avec le verdict et l'espèce dessous. À droite, l'image en petit, avec toutes les trajectoires mais la seule pastille de la piste jugée (un clic ouvre l'image complète), et la liste, où la piste jugée est en évidence.
- Un clic sur une trajectoire de l'image, ou sur sa pastille, ouvre aussi le lecteur sur cette piste ; au survol, elle ressort et les autres pâlissent. Quand deux trajectoires se touchent, le clic prend la plus proche, et un second clic au même endroit prend l'autre. Dans l'image en petit du lecteur, une trajectoire ressort aussi au survol, et un clic sur elle passe à cette piste ; ailleurs, il ouvre l'image complète. Au clavier, la touche Tab mène aux pastilles de l'image, les flèches passent d'une piste à l'autre et Entrée ouvre le lecteur. En revenant à l'image, le clavier reprend sur la pastille de la piste jugée.
- Un clic sur un début de piste de la frise ouvre lui aussi le lecteur sur cette piste ; au survol, sa trajectoire ressort sur l'image. Quand deux débuts se touchent, le plus proche du pointeur l'emporte.
- Au-dessus de l'image, et au-dessus de l'image en petit pendant le jugement, des boutons choisissent les pistes montrées : « Toutes », « À voir », « Oui », « Non », « Doute », chacun avec son nombre, et « Autre » quand une piste est notée d'un autre mot. Le choix vaut pour l'image, la frise et la liste de la vidéo ; la piste en cours de jugement reste toujours montrée. Sur la grande image, chaque piste jugée porte un petit signe près de sa pastille : une coche verte pour oui, une croix rouge pour non, un point d'interrogation jaune pour doute ; une piste notée d'un autre mot n'a pas de signe, la liste et le bouton « Autre » le disent. L'image en petit du lecteur n'a pas de signe : la piste regardée y ressort, les autres sont voilées. Les pistes jugées « non » sont grisées sur l'image et pâlies dans la frise. Pendant le jugement, les flèches ‹ et › ne passent que par les pistes montrées.
- « Extrait » et « Zoom » choisissent la vidéo ; « Image résumé » ferme le lecteur ; ‹ et › passent à la piste précédente ou suivante ; « À voir » saute à la prochaine piste sans verdict. Sur un écran étroit, l'étiquette « Zoom » de la liste laisse sa place au verdict pendant le jugement : le bouton « Zoom », ou « pas de zoom », dit si la piste jugée en a un.
- Le clavier : `O` oui, `N` non, `D` (ou `?`) doute, `Suppr` efface le verdict, `E` va au champ espèce, `Z` passe de l'extrait au zoom, les flèches ← et → changent de piste, Maj et → va à la prochaine piste à voir, Espace lit ou arrête. Un second appui sur le même verdict l'efface. Les lettres tapées dans le champ espèce restent du texte.
- « Enchaîner », cochée d'origine : après « Oui », le curseur va dans le champ espèce, et Entrée passe à la piste suivante ; après « Non » ou « Doute », la page passe directement à la suivante. Décochée, rien ne bouge seul. On ne passe jamais d'une vidéo à l'autre.
- Une vidéo chargée a plusieurs images résumé : des onglets « Partie » les séparent, et chaque onglet dit quelles pistes il montre.

Une vidéo en échec garde sa place dans la liste, marquée en rouge, avec la raison et ce qu'il
faut faire, comme dans le bilan. Si les résultats d'un calcul précédent sont encore là, un lien
y mène ; ce n'est pas toujours le cas, car un nouveau calcul commence par les effacer. Une vidéo « inachevée » est une vidéo dont le calcul s'est arrêté
avant la fin : relancer la même commande la refait. Pendant le calcul, les vidéos pas encore traitées sont « en attente ». Si le calcul est interrompu avant elles, elles restent sur la page, marquées « pas encore traitée » : relancer la même commande les traite. Une vidéo calculée par une version plus ancienne de l'outil est marquée « ancienne version » et nommée à part dans le total : relancer la même commande la refait.

Limite : les résultats d'une vidéo sont rangés sous son nom. Si deux soirées sont envoyées dans
le même dossier choisi au lancement et ont des vidéos de même nom, la seconde remplace la
première, et la page ne montre que la dernière. Gardez un dossier par soirée.

## Où sont les résultats

Pour chaque vidéo, un dossier à son nom, dans `batdetect/` à côté des vidéos, ou dans le
dossier choisi au lancement (`-o`). Les noms des fichiers sont en français sur un Mac réglé en
français, en anglais sinon ; les mots ajoutés par l'outil ne portent jamais d'accent. Un nouveau
calcul efface les fichiers de l'autre langue.

| Fichier | Français | Anglais |
| --- | --- | --- |
| Image résumé | `<vidéo>_resume.png` | `<vidéo>_summary.png` |
| Scène seule, sans trace, pour la page | `<vidéo>_fond.png` | `<vidéo>_background.png` |
| Tableau des pistes | `<vidéo>_pistes.csv` | `<vidéo>_tracks.csv` |
| Extrait | `extraits/piste_01_0m04s.mp4` | `clips/track_01_0m04s.mp4` |
| Extrait zoomé | `extraits/piste_01_0m04s_zoom.mp4` | `clips/track_01_0m04s_zoom.mp4` |
| Vidéo annotée | `<vidéo>_annotee.mp4` | `<vidéo>_annotated.mp4` |
| Réglages et bilan du calcul | `params.json` | `params.json` |

À côté de la page, dans le dossier `batdetect/` ou le dossier choisi au lancement, `verdicts.csv` garde les verdicts de toute la page ([Noter le verdict](#noter-le-verdict)) : c'est la page qui le télécharge, et l'outil le lit sans jamais l'écrire.

- Une vidéo de plus de 10 min a une image résumé par tranche de 10 min
  (`<vidéo>_resume_000m-010m.png`, …), et une image qui aurait plus de 20 pistes est coupée en
  parties (`<vidéo>_resume_1_sur_3.png`, …).
- Un extrait est nommé par le numéro de sa piste et son début à la seconde. Le numéro départage
  deux pistes de la même seconde ; il a deux chiffres, trois dès 100 pistes, pour que le Finder
  range les extraits dans l'ordre. Par défaut, l'extrait zoomé n'existe que pour les pistes
  petites ou faibles ([Juger un extrait](#juger-un-extrait)).
- La vidéo annotée, la vidéo entière avec toutes les pistes dessinées, n'est écrite qu'avec
  `--annotated` ; sans l'option, celle d'un calcul précédent est effacée. Au-delà du
  [garde-fou](#garde-fou-du-nombre-de-pistes), il n'y a ni extraits, ni image résumé, ni fond, ni vidéo annotée.
- `params.json` s'écrit en dernier : un dossier sans lui n'est pas terminé (calcul interrompu,
  ou vidéo en échec autrement qu'au-delà du garde-fou), et la vidéo est refaite à la relance.
- Le tableau français s'ouvre dans un tableur réglé en français (point-virgule, virgule
  décimale), le tableau anglais dans un tableur réglé en anglais. Un tableur réglé en anglais affiche le début `0:00:04` comme `12:00:04 AM` : la
  valeur est juste (4 s), et un format Durée la montre en `0:00:04`.
- Deux vidéos du même nom, aux majuscules près, qui vont dans le même
  dossier de résultats prennent leur extension dans le nom de leur dossier, `video_092_mp4/` et
  `video_092_mov/`, et, avec `-o`, aussi le nom de leur dossier d'origine,
  `nuit-12-oct_video_092_mp4/` ; deux vidéos qui tomberaient encore sur le même dossier sont
  refusées avant tout calcul, et il faut renommer l'une d'elles. Si la seconde arrive plus tard
  (`video_092.mov` à côté de `video_092.mp4`), les deux sont recalculées dans leurs dossiers
  renommés, et l'ancien dossier `video_092/` reste là, orphelin, à supprimer à la main.
- À côté des dossiers des vidéos : la [page des résultats](#la-page-des-résultats),
  `resultats.html` (`results.html` en anglais), et un dossier caché, `.batdetect/`, où l'outil
  garde la raison des vidéos en échec pour la page. Les laisser avec les dossiers des vidéos :
  la page en a besoin.

À la fin du calcul, la commande écrit un bilan de la soirée : pour chaque vidéo sa durée, son
nombre de pistes, le temps non analysé et le temps de calcul, puis à part les vidéos qui ont
trop de pistes, celles en échec et les entrées refusées, chacune avec ce qu'il faut faire. Sous
macOS, lancée dans un terminal, elle annonce aussi la fin par une notification et ouvre la page
des résultats de chaque dossier (`--no-notify` pour s'en passer).

Relancer la même commande ne refait pas une vidéo déjà faite avec les mêmes réglages : son
dossier reste tel quel, et le bilan la compte « déjà faite » avec ses chiffres d'origine
(`--force` refait tout).

[techniques]: 10-logique-de-detection.md#les-techniques-en-simple
[#9]: https://github.com/zaratan/chiro-dvr/issues/9
[#11]: https://github.com/zaratan/chiro-dvr/issues/11
[#15]: https://github.com/zaratan/chiro-dvr/issues/15
[#22]: https://github.com/zaratan/chiro-dvr/issues/22
[#23]: https://github.com/zaratan/chiro-dvr/issues/23
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
   l'extrait (`piste_01_0m04s.mp4`) ; le début au centième est dans la colonne `début (s)` du
   tableau.

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

La couleur sert seulement à distinguer une piste de ses voisines sur la même image. Il y en a 12 : chaque piste prend la couleur la moins présente parmi les traits proches déjà dessinés, et à égalité la moins employée. Les 12 premières pistes d'une image ont chacune la sienne ; à partir de la 13ᵉ, une couleur revient, sur une piste éloignée si possible. Deux traits de même couleur sont donc deux pistes, et la même couleur sur deux images ne désigne pas la même piste. Elle ne dit rien de l'animal, de sa vitesse ni de la confiance de l'outil. Les extraits, les zooms et la vidéo entière reprennent la couleur que chaque piste a sur l'image résumé : sa boîte, son numéro et la trace de son vol sont de cette couleur. Seul le numéro identifie une piste.

## Juger un extrait

L'image résumé dit où regarder ; le verdict se donne sur l'extrait, à l'œil. L'outil ne juge
pas : il montre ce qui a bougé, et Manon décide si c'est une chauve-souris.

### Regarder l'extrait, puis le zoom

Chaque piste a son extrait, `extraits/piste_01_0m04s.mp4`, sauf au-delà du
[garde-fou](#garde-fou-du-nombre-de-pistes) : la vidéo en pleine taille et à vitesse réelle,
par défaut de 1,5 s avant le début de la piste à 1,5 s après sa fin. Il se regarde dans la
[page des résultats](#la-page-des-résultats), ou dans un lecteur vidéo pour avancer image par
image. Le regarder d'abord en entier, puis image par image là où la cible passe.

Si la cible est petite ou faible, un second fichier l'accompagne, `piste_01_0m04s_zoom.mp4` :
la même scène recadrée autour du trajet, agrandie jusqu'à quatre fois et ralentie quatre fois.
Par défaut, il n'existe que pour les pistes dont la plus grande tache fait moins de 100 px
ou dont l'amplitude est sous 40. Quand l'outil a recollé une piste après un passage derrière la végétation, elle a son zoom si l'une de ses parties, seule, l'aurait eu : la partie nette ne fait pas perdre le zoom à la partie faible. `--zoom all` en fait un pour chaque piste, `--zoom none` aucun
([docs/02](02-methode.md#extrait-zoomé-et-ralenti)). Le regarder après l'extrait normal : une
cible de 3:42 sur la 092, jugée fausse à vitesse normale, s'est révélée un objet réel une fois
agrandie et ralentie dans un lecteur ; c'est de là que vient le zoom.

### Ce que l'extrait dessine

![Extrait de la piste 2, une chauve-souris, avec les repères 1 à 4](images/extrait-chauve-souris.jpg)

*Extrait de la piste 2 de l'extrait de test (0:00:23), à 1,9 s du début de l'extrait : une
chauve-souris qui descend de droite à gauche.*

1. **La boîte** entoure la cible, avec par défaut une marge de 12 pixels de chaque côté. Elle a la couleur de la piste sur l'image résumé, bordée d'un liseré noir qui la détache de la roche claire comme du ciel sombre.
2. **Le numéro** `#2`, au-dessus de la boîte et de la même couleur, est celui de la piste : dans le tableau, la légende de l'image résumé et le nom du fichier.
3. **La trace**, de la même couleur, suit le trajet de la dernière seconde (par défaut), jusqu'à la cible. Pour un vol rapide comme celui-ci, elle montre presque tout le passage.
4. **L'affichage des jumelles** (chrono d'enregistrement en rouge, heure, date, batterie) reste tel qu'il a été filmé : ce rouge-là ne vient pas de l'outil.

Les autres pistes présentes au même moment sont dessinées aussi, chacune avec sa boîte, son
numéro et sa trace. Les réglages du dessin sont dans [docs/02](02-methode.md#rendu).

![Trois images successives de la piste 2 agrandies trois fois : boîte épaisse, boîte fine, boîte épaisse](images/extrait-boites.png)

*Trois images successives du même extrait, vers 1,8 s, agrandies trois fois.*

**Boîte épaisse ou fine.** La boîte est épaisse (2 pixels) sur une image où l'outil a vu la
cible, fine (1 pixel) sur une image où il ne l'a pas vue : sa place y est estimée entre les
deux images les plus proches où l'outil l'a vue, et l'animal peut ne pas y être tout à fait.
L'image du milieu est de celles-là ; le tableau des pistes en donne le nombre par piste
(`images comblées`).

![Extrait zoomé de la piste 1 : la trace seule, sans boîte ni numéro](images/extrait-zoom.jpg)

*Zoom de la piste 1 (0:00:04), au même moment, à 8,3 s du début de l'extrait normal, produit
avec `--zoom all` : cette piste est assez grande et contrastée pour ne pas en avoir par défaut.
Agrandie environ ×1,6 ici, parce que son trajet couvre une grande partie de l'image.*

**Sur le zoom**, seule la trace est dessinée, en trait fin et de la couleur de la piste : ni boîte, ni numéro, ni autres pistes, pour ne rien cacher d'une cible ténue. La trace s'arrête un peu avant la cible. Le cadre ne bouge pas pendant tout le zoom.

### Désigner une piste

Le numéro d'une piste change quand un réglage ou une nouvelle version ajoute ou retire une
piste plus tôt dans la vidéo. Pour parler d'une piste, dans une note ou un message, donner
**la vidéo, le début et le numéro** : « video_092_original_3m24-4m05, 0:00:04, piste 1 ». Le
début est celui de la colonne `début` du tableau des pistes ; le numéro départage deux pistes
qui commencent dans la même seconde.

### Noter le verdict

Le verdict se note dans la page, piste par piste, dans le lecteur : « Oui », « Non » ou « Doute », et l'espèce, en texte libre (la page propose les espèces déjà tapées). L'outil ne tient jamais compte de ce qui y est noté : les verdicts servent à compter les passages.

La page ne peut pas écrire sur le disque : elle garde les verdicts dans un fichier, `verdicts.csv`, qu'on télécharge par « Télécharger les verdicts ». Le fichier arrive dans Téléchargements, peut-être sous le nom `verdicts (1).csv` ou `verdicts-2.csv` si le navigateur en a déjà un ; le glisser dans le dossier de la page, à côté de `resultats.html`. S'il y en a plusieurs, la page lit le plus récent et dit lequel. Il couvre toute la page, toutes les vidéos de la soirée, et s'ouvre dans un tableur pour compter : une ligne par piste jugée, avec la vidéo, le numéro, le début, le verdict et l'espèce.

Tant que le fichier n'est pas téléchargé, la barre le dit (« Changements non téléchargés »), et le navigateur demande confirmation avant de quitter la page. Le navigateur garde aussi les verdicts non téléchargés : en rouvrant la page, on les retrouve, et la page le signale. « Recharger un fichier de verdicts… » relit un fichier choisi, par exemple celui resté dans Téléchargements.

Un nouveau calcul ne touche jamais au fichier de verdicts. Chaque verdict retrouve sa piste par son début (à un cinquième de seconde près) et son point de départ sur l'image, même si les numéros ont changé. Un verdict dont la piste n'existe plus est montré au-dessus de la liste de sa vidéo, « Verdicts sans piste », avec la piste la plus proche à vérifier sur l'extrait, ou dans la vidéo elle-même quand elle n'a pas d'extrait ; il reste dans le fichier. Le tableau des pistes, `<vidéo>_pistes.csv`, appartient à l'outil : un recalcul le remplace, et ce qu'on y aurait écrit est perdu.

### Une piste de trop vaut mieux qu'un passage manqué

L'outil est réglé ainsi depuis le verdict de Manon sur la 092 (9 octobre 2026) : mieux vaut un
faux positif facile à écarter à l'extrait qu'un passage manqué. Quand il hésite, l'outil penche
vers la sensibilité ; c'est à l'œil, sur l'extrait, qu'on écarte.

### Les faux positifs déjà vus

Les cas ci-dessous ont été vus sur de vraies vidéos ; chacun renvoie à l'endroit où il est
décrit. La liste vient des relectures, pas d'un jugement de l'outil, et elle est soumise à la
relecture de Manon.

| Ce que vous voyez | Où ça a été vu | Source |
| --- | --- | --- |
| Un objet qui se déplace lentement et régulièrement, sur plusieurs secondes | La piste de 3:29 sur la 092, que Manon a écartée. C'est la piste 1 de l'extrait de test (0:00:04, 95 px/s, contre 1 383 à 1 926 px/s pour les trois chauves-souris) : voir l'image ci-dessous | [docs/03](03-resultats-092.md#verdict-de-manon-9-octobre-2026) |
| Une piste courte qui change de direction au hasard, sur une vidéo au grain fort | Le scintillement de pixels isolés que le suivi enchaîne : 96 pistes sur la 089 avant les deux filtres qui le visent, qui en écartent 92 | [docs/02](02-methode.md#7-filtres-finaux) |
| Une piste sur une image abîmée (blocs, morceaux d'image décalés) | Sur la 122, avant la marge autour des images abîmées : 4 pistes collées aux plages illisibles | [docs/10](10-logique-de-detection.md#5-bis-écarter-les-images-que-le-décodeur-a-réparées), [Ce que l'outil ne voit pas](#ce-que-loutil-ne-voit-pas) |
| Une piste avec un trou, qui repart ailleurs : peut-être deux animaux | Pas encore vu sur une vraie vidéo. Possible quand deux animaux se suivent de près, au même endroit et à la même vitesse : l'outil ne sépare que ceux dont la seconde piste repart presque là où la première s'est arrêtée | [docs/10](10-logique-de-detection.md#9-bis-recoller-un-vol-caché-derrière-la-végétation) |

D'autres cas sont possibles, sans qu'aucun ait encore été relevé sur ces vidéos : un insecte,
un oiseau, ou une feuille agitée par le vent, dont un pixel passe encore parfois le seuil
([docs/10](10-logique-de-detection.md#4-décider-pixel-par-pixel--le-seuil)).

![Extrait de la piste 1, la piste lente de 3:29 sur la 092](images/extrait-lente.jpg)

*Extrait de la piste 1 de l'extrait de test (0:00:04), à 8,3 s du début de l'extrait : la
piste lente, qui n'est pas une chauve-souris. La trace d'une seconde y est courte : l'objet
avance peu.*

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

Au-delà d'un certain nombre de pistes, elles sont presque sûrement du bruit d'image : l'outil écrit le tableau des pistes, mais ni extraits ni image résumé. Pour en juger, ouvrir le tableau et regarder la vidéo à quelques-uns de ses temps ; si ce sont de vrais passages, relancer avec un `--max-tracks` plus grand. Le bilan de la soirée compte cette vidéo à part, avec son nombre de pistes, et les autres vidéos du lot sont traitées normalement.

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
pistes ; chaque piste a sa ligne dans le tableau des pistes et, sauf au-delà du garde-fou, sa pastille sur l'image résumé et son extrait.

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
distance et du contraste de l'animal autant que de sa taille. Une piste qui a été cachée en cours de vol, par exemple derrière la végétation, garde son extrait zoomé si une de ses parties est petite ou pâle, même si cette colonne montre une grande tache.

### Verdict

Le jugement de Manon, à l'œil sur l'extrait : cette piste est-elle une chauve-souris ? L'outil ne s'en sert pas ; il se note dans la page ([Noter le verdict](#noter-le-verdict)) et se garde dans le fichier de verdicts de la soirée. Ailleurs, on désigne la piste par la vidéo, son début et son numéro, car le numéro seul peut changer (ceux de la 092 sont dans [docs/03](03-resultats-092.md#verdict-de-manon-9-octobre-2026)).

### Vitesse

Le chemin parcouru par la piste divisé par sa durée, en pixels par seconde dans l'image,
dans le tableau des pistes. Ce n'est pas une vitesse en mètres par seconde : la distance
entre l'animal et les jumelles est inconnue.
