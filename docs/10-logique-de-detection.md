# La logique de détection, expliquée

Ce document explique **comment l'outil raisonne**.
Pour chaque étape, il dit quel outil mathématique ou technique sert, pourquoi celui-là et ce qu'il ne sait pas faire.
Chaque étape nomme son module (chemins relatifs à `crates/`) et cite le défaut de ses réglages.
La valeur de chaque réglage et la mesure qui la justifie sont dans [02](02-methode.md).
Les temps de calcul sont dans [09](09-profilage.md) et les pistes d'amélioration dans [07](07-ameliorer-la-detection.md).

## L'idée en une phrase

Une chauve-souris est **une petite tache qui s'écarte du fond pendant quelques images et qui se déplace de façon régulière**.
Tout le traitement découle de ces trois mots : fond, tache, régularité.

## Les techniques en simple

Voici les techniques dans l'ordre où elles s'appliquent. Chacune vient avec une image de la vie courante et ce qu'elle rate.

**Le filtre à la taille de la cible.**
Chaque image est d'abord adoucie à la mesure d'une petite chauve-souris lointaine : le grain du capteur s'atténue plus qu'une tache de cette taille.
C'est le peintre qui plisse les yeux : les petits détails se fondent et les formes de la bonne taille restent.
Une cible bien plus petite que prévu y perd du contraste et peut passer inaperçue.

**La scène sans animal** ([§ 2](#2-estimer-le-fond--la-médiane-temporelle)).
Pour chaque endroit de l'image, l'outil range les valeurs des images voisines dans le temps et garde celle du milieu.
On obtient la scène telle qu'elle est quand rien ne la traverse.
C'est le photographe qui prend une dizaine de photos d'un monument envahi de touristes et garde pour chaque endroit ce que la plupart des photos montrent.
Les passants disparaissent et le monument reste. Elle rate un animal qui s'attarde au même endroit : il entre dans le décor.

**Un seuil propre à chaque endroit** ([§ 4](#4-décider-pixel-par-pixel--le-seuil)).
L'outil corrige d'abord les changements de luminosité de toute l'image ([§ 3](#3-compenser-le-gain-automatique)).
Il retient ensuite un pixel s'il s'écarte assez de la scène sans animal, en plus clair ou en plus sombre. « Assez » vaut un minimum partout.
Ce minimum est relevé là où l'endroit s'agite dans les instants qui entourent l'image.
Il faut toujours parler assez fort pour être entendu. Près d'une route, il faut crier.
Il rate une cible faible qui passe devant de la végétation agitée.

**Les taches** ([§ 5](#5-regrouper-les-pixels-en-taches)).
Les pixels retenus qui se touchent ou presque forment une tache. Une tache de taille plausible sur une image est un point.
C'est entourer d'un seul trait les gouttes d'une même éclaboussure.
Elles ratent une cible qui n'allume presque aucun pixel sur une image : elle n'y laisse pas de point.

**Les taches faibles encadrées** ([§ 5 ter](#5-ter-garder-les-taches-faibles-qui-comblent-un-vol)).
Une tache un peu trop faible pour le seuil (au-dessus des trois quarts) est gardée si une vraie tache la précède et une autre la suit de près.
Elle doit aussi tomber sur le chemin entre les deux. C'est le pointillé qu'on devine entre deux traits nets d'une même route.
Elle ne sert qu'à boucher les trous d'un vol déjà vu : elle n'ouvre jamais de piste.
Une piste ne la prend que si aucune vraie tache n'est à sa portée.
Elle rate une cible qui reste faible plusieurs images de suite sans vraie tache de part et d'autre.

**Les périodes ignorées** ([§ 6](#6-écarter-les-images-où-les-jumelles-bougent)).
L'image se remplit parfois soudain de taches, le plus souvent parce que les jumelles ont bougé. Le fichier vidéo peut aussi être abîmé.
Dans les deux cas, l'outil ne cherche rien sur ce moment et ses abords et le signale sur l'image résumé.
C'est un observateur qui note « je n'ai pas regardé » au lieu d'inventer.
Une chauve-souris qui passe pendant ces périodes est manquée, et un essaim qui remplit l'image passe pour un mouvement des jumelles.
Ces périodes sont donc à revoir à l'œil.

**Le suivi image par image** ([§ 7](#7-relier-les-taches-dans-le-temps--le-suivi)).
L'outil prolonge le trajet de chaque piste pour prédire où elle sera à l'image suivante.
Il lui donne le point le plus proche de cette prédiction s'il n'est pas trop loin. Une vraie tache passe toujours avant une tache faible.
Restée seule, elle ouvre une nouvelle piste. C'est suivre des yeux un ballon lancé en devinant sa course.
Un vol qui disparaît plus de 6 images en sort en deux pistes. Le recollage les réunit ensuite quand il le peut.
Une piste ne commence qu'une fois que la cible se détache assez du fond, donc après l'arrivée réelle de l'animal.

**La fusion des morceaux** ([§ 8](#8-fusionner-les-pistes-jumelles)).
Deux pistes qui restent tout près l'une de l'autre sur toutes leurs images communes sont réunies.
Ce sont deux morceaux du même animal, comme les deux phares d'une même voiture la nuit.
Deux animaux qui volent côte à côte et tout près du début à la fin sont comptés pour un seul.

**Ressembler à un vol** ([§ 9](#9-garder-ce-qui-ressemble-à-un-vol)).
Une piste n'est gardée que si elle a assez de points, si elle finit assez loin de son point de départ et si elle garde à peu près sa direction d'une image à l'autre.
Un oiseau traverse le jardin. Une poussière danse sur place dans un rayon de soleil.
Ce filtre peut rejeter un vol très lent, qui paraît hésitant. Une boucle qui revient à son point de départ peut aussi être rejetée.

**Le recollage** ([§ 9 bis](#9-bis-recoller-un-vol-caché-derrière-la-végétation)).
Une chauve-souris qui passe derrière un buisson disparaît de l'image quelques images puis réapparaît de l'autre côté.
Le suivi en fait deux pistes. L'outil prolonge la fin de la première et le début de la seconde.
Si les deux prolongements se rejoignent au bon moment, c'est le même vol.
C'est voir une voiture passer sous un pont et savoir que c'est la même qui en ressort.
Il ne recolle pas une piste qui finit ou commence tout près du bord de l'image : la chauve-souris a pu sortir du cadre.
Il ne recolle pas non plus une disparition de plusieurs secondes.
Un bout de vol aperçu entre deux buissons est parfois trop court pour faire une piste.
Il peut alors servir de pont entre deux pistes ou en prolonger une, mais seul il ne devient jamais une piste.
Dans tous les cas, l'animal ne doit pas avoir ralenti d'un coup pendant qu'il était caché.

## La chaîne de traitement

```
vidéo ─► image grise réduite et filtrée ─► fond (médiane) ─► résidu ─► seuil par pixel ─► taches
  │                                                                     │
  │                                                        taches faibles encadrées gardées
  │                                                                     │
  │ pistes ◄─ recollage ◄─ filtres ◄─ fusion des jumelles ◄─ suivi ◄─ images instables retirées
  │                                                                     ▲
  └─► signalements du décodeur et horodatages (même lecture) ──► images illisibles retirées
```

| Étape | Module | Outil | Question posée |
| --- | --- | --- | --- |
| Lecture | `media/src/decode.rs`, `engine/src/input.rs`, `read_ahead.rs` | FFmpeg lié dans le binaire | Que voit le capteur ? |
| Réduction | `engine/src/gray.rs`, `reduction.rs`, `blur.rs`, `prepare.rs` | gris par formule, moyenne de surface, flou gaussien en entiers | Que voit le capteur, en plus petit, à la taille d'une chauve-souris ? |
| Fond | `engine/src/window.rs`, `network.rs` | médiane temporelle, réseau de tri | À quoi ressemble la scène sans animal ? |
| Gain | `engine/src/detector.rs` | différence de moyennes | L'image entière a-t-elle changé de luminosité ? |
| Seuil | `engine/src/threshold.rs` | valeur absolue, plancher et bruit du pixel (MAD) | Ce pixel s'écarte-t-il assez du fond, compte tenu de son propre bruit ? |
| Taches | `engine/src/closing.rs`, `components.rs`, `detector.rs` | fermeture morphologique, composantes connexes, en deux passages (fortes, puis faibles) | Quels pixels forment un même objet ? |
| Faibles | `engine/src/bracketing.rs` | interpolation entre deux taches fortes | Cette tache faible comble-t-elle le trou d'un vol ? |
| Illisible | `media/src/decode_errors.rs`, `engine/src/lost.rs`, `damage.rs` | signalements du décodeur, horodatages, images-clés | Le décodeur a-t-il dû réparer l'image ? |
| Stabilité | `tracking/src/stability.rs`, `exclusion.rs` | comptage, médiane | Les jumelles ont-elles bougé ? |
| Suivi | `tracking/src/track.rs` et `track/` | prédiction linéaire, plus proche voisin, fortes d'abord | Cette tache prolonge-t-elle une piste ? |
| Jumelles | `tracking/src/track/twins.rs` | union-find | Deux pistes sont-elles un seul animal ? |
| Filtres | `tracking/src/track.rs`, `types/src/track.rs` | comptage, distance, angle médian | Cette piste ressemble-t-elle à un vol ? |
| Recollage | `tracking/src/track/rejoin.rs` | prolongement des deux bouts, paires du plus proche au plus lointain | Deux pistes sont-elles un même vol caché un moment ? |

Les réglages qui sont des options de la commande et ceux qui ne se règlent que dans le banc sont listés en tête de [02](02-methode.md).

## 1. Lire et réduire l'image

**Outil.** La vidéo est décodée par FFmpeg, lié dans le binaire (`media/src/decode.rs`).
Seule la luminance sert : chaque valeur passe au gris par une formule en entiers (`engine/src/gray.rs`). Le compilateur la vectorise (#214).
Elle redonne niveau par niveau la table tirée de la conversion d'OpenCV de la v0.1.1.
Cette table reste dans le code et les tests comparent les deux.
L'image est ensuite réduite à `work_width` (960 px) par **moyenne de surface**, avec un seul arrondi (`reduction.rs`).
Sur une vidéo de 1440 px, chaque pixel réduit moyenne 1,5 × 1,5 pixels d'origine.
Puis un **flou gaussien** d'écart-type `target_sigma` s'applique à chaque image réduite, avant tout le reste (`blur.rs`, `prepare.rs`).
Cet écart-type vaut 1,5 px d'origine, soit 1 px de travail à 960 sur 1440.

**Pourquoi le flou.**
C'est le **filtre adapté** (*matched filter*) d'une tache gaussienne dans du bruit.
Il donne le meilleur rapport signal sur bruit pour une cible de cette taille.
Il moyenne le bruit d'un pixel avec ses voisins sans écraser une tache de quelques pixels.

**Choix.**
On filtre l'image avant le fond plutôt que le résidu après.
Ainsi le fond, le résidu et le bruit de chaque pixel sont mesurés dans le même espace. Le calcul se fait en entiers.
Le noyau du flou a des poids entiers de somme 4 096 et porte à 4 écarts-types. Il replie les bords en miroir.
L'image filtrée garde ses valeurs au 1/64 de niveau.
Le calcul est exact, donc le même quel que soit le découpage du travail entre les fils (§ 10).

**Limites.** Une tache plus grande que le filtre est moins rehaussée, et une plus petite perd du contraste.

**Coordonnées.** Toutes les coordonnées ressortent en pixels d'origine : le centre d'un pixel réduit `c` vaut `(c + 0,5) × échelle − 0,5` (`types/src/scale.rs`).

## 2. Estimer le fond : la médiane temporelle

**Outil.**
Pour chaque pixel, on prend la **médiane** de ses valeurs sur une seconde centrée sur l'image étudiée (`bg_window_s`, 1,0 s).
On n'y prend qu'une image sur 3 (`bg_step`, 3). À 30 i/s, cela fait une demi-fenêtre de 15 images et 11 valeurs (`engine/src/window.rs`).
Près du début et de la fin de la vidéo, la fenêtre est tronquée : la première et la dernière demi-seconde sont analysées avec moins d'images.

**Pourquoi la médiane.**
C'est une statistique *robuste* : tant que l'animal occupe le pixel sur moins de la moitié des images, la médiane l'ignore complètement.
Une moyenne garderait une traînée de l'animal dans le fond.
Une chauve-souris en transit ne reste sur un pixel que quelques images sur 30 : elle disparaît du fond.

**Pourquoi une fenêtre centrée.** La fenêtre regarde une demi-seconde avant et une demi-seconde après.
Le traitement n'est pas en direct, donc on peut utiliser le futur. Le fond suit ainsi les changements lents de lumière sans retard.

**Pourquoi pas autre chose.**

- *Différence entre deux images* : c'est simple, mais chaque objet apparaît deux fois (où il était et où il est).
  Les objets lents sont ratés.
- *Mélanges de gaussiennes* (MOG2, utilisé par ThruTracker) : la sortie est binaire. On perd le signe et l'amplitude de l'écart.
  Le reste du traitement en a besoin.
- *Fond fixe sur toute la vidéo* : ne suit ni la lumière ni le gain des jumelles.

**Limite.** Un objet qui reste plus d'une demi-seconde au même endroit entre dans son propre fond et s'efface (#22).

**Calcul.** La médiane est le poste le plus cher de la détection.
Elle est calculée par un **réseau de tri** (`network.rs`) : une suite fixe de comparaisons et d'échanges, appliquée à des blocs de 64 pixels à la fois.
Le réseau est élagué pour ne fixer que la ou les deux valeurs du milieu. Pour un nombre pair de valeurs, le fond est leur demi-somme.
Les images de la fenêtre sont toutes gardées, en entiers de 16 bits. Une fenêtre qui dépasserait 4 Gio est refusée avant la lecture.

## 3. Compenser le gain automatique

**Outil.**
On retire au résidu la différence entre la luminosité moyenne de l'image et celle des images du fond (`engine/src/detector.rs`).
Elle se calcule en entiers jusqu'à une seule division, hors des zones masquées par `--osd-region`.

**Pourquoi.** Les jumelles ajustent leur gain : toute l'image s'éclaircit d'un coup. Sans correction, chaque pixel dépasserait le seuil.
Le modèle est le plus simple possible : un décalage identique pour tous les pixels.

**Limites.** Un changement de contraste étire les valeurs au lieu de les décaler : il n'est pas corrigé.
Le fond médian n'est pas corrigé non plus : pendant la demi-seconde où la fenêtre mélange des images d'avant et d'après un saut, le résidu en garde une partie.
Autour d'un saut de 30 niveaux ou plus, toute l'image s'allume.
Le filtre des images saturées (§ 6) ignore alors quelques secondes sans créer de piste ([02 § 2](02-methode.md#2-compensation-du-gain-automatique)).

## 4. Décider pixel par pixel : le seuil

**Outil.** Le **résidu** est `image − fond − décalage de gain` sur l'image filtrée.
Un pixel est retenu si la valeur absolue du résidu dépasse strictement **son propre seuil** : max(12, 8 × bruit du pixel) (`engine/src/threshold.rs`).
Les valeurs sont en [02 § 3](02-methode.md#3-seuil-dans-les-deux-sens).

**Pourquoi la valeur absolue.** L'animal peut être plus sombre que le fond (devant la roche chaude) ou plus clair (devant un ciel froid).
Le signe n'est pas supposé.

**Pourquoi pas un seuil fixe.** Un seuil unique traite pareil la roche calme et la végétation qui bouge.
Il ne s'adapte pas non plus d'une vidéo à l'autre. Assez bas pour la 092, il laisse passer le bruit des vidéos agitées.
Sur la 089, la 090 et la 091, le bruit de fond dépasse 12 en quelques points de chaque image ([02 § 3](02-methode.md#3-seuil-dans-les-deux-sens)).

**Le seuil par pixel** (`noise_factor`, 8, et plancher `threshold`, 12).
*Outil* : on mesure le bruit de chaque pixel sur les images de la fenêtre du fond par la **MAD**.
C'est la médiane des écarts absolus à la médiane, multipliée par 1,4826 pour valoir un écart-type sur un bruit gaussien.
Les écarts sont corrigés du gain de chaque image. Le seuil devient max(plancher, k × bruit).
Il n'est calculé que si la fenêtre a au moins 7 échantillons.
Seuls comptent les pixels non masqués dont le résidu dépasse déjà les trois quarts du plancher, car les autres ne peuvent pas être retenus.
La carte des seuils ne dépend que de la fenêtre : chaque image ne dépend que de ses voisines.
*Choix* : on prend la MAD plutôt que l'écart-type parce qu'un passage d'animal sur une ou deux images de la fenêtre ne la fait presque pas monter.
On prend la fenêtre plutôt qu'une moyenne accumulée depuis le début pour garder la même indépendance entre images.
Il faut un plancher parce que la MAD sur 11 images est souvent très faible.
Sur l'extrait de test à 480 px, descendre le plancher de 4 à 1 niveau multiplie par 60 les pixels au-dessus de 6 fois le bruit (v0.1.1).
*Limites* : 11 images estiment mal le bruit d'un pixel. Une feuille agitée passe encore parfois.
Une cible faible qui passe sur des pixels agités peut être perdue.
Sur la 092 (v0.1.1), le passage réel de 3:42.10 a un pic de 13 à 22 après filtre.
Sur deux de ses six images, il tombe sous un seuil relevé à 22 à 36 et n'a plus assez de détections pour faire une piste.
Le ciel saturé à 0 a une MAD nulle et aucun résidu sombre possible : le seuil par pixel n'y gagne rien.

**Le seuil bas** (`WEAK_RATIO`, 0,75, constante de `engine/src/threshold.rs`).
Un pixel dont le résidu dépasse les trois quarts de son seuil est retenu comme **faible** (par défaut, plancher 9 et 6 fois le bruit).
Au-dessus du seuil entier, il est **fort**.
Les pixels faibles ne servent qu'aux taches faibles encadrées ([§ 5 ter](#5-ter-garder-les-taches-faibles-qui-comblent-un-vol)) : aucune détection forte ne change à la sortie de la détection.
*Pourquoi 0,75* : c'est la seule fraction mesurée qui gagne sans abîmer la 092.
À 0,6 et à 0,5, les détections faibles détournent le suivi, qui coupe alors des pistes en deux.
La 092 passe de 26 à 36 et 24 positions estimées, avec 4 et 3 pistes nouvelles (17 octobre 2026, #21).
Le seuil fort ne bouge pas : le relever ferait perdre des détections fortes, donc des passages.

## 5. Regrouper les pixels en taches

**Outils.** Deux opérations classiques de morphologie mathématique.

- **Fermeture** (`engine/src/closing.rs`) : une dilatation puis une érosion, avec un disque de `merge_radius`.
  Elle bouche les trous et recolle les fragments proches d'un même animal sans grossir l'objet.
  Le rayon vaut 6 px d'origine, soit 4 px de travail à 960 sur 1440, arrondi au plus proche.
- **Composantes connexes** (`components.rs`) : chaque groupe de pixels qui se touchent (même par un coin) reçoit un numéro.
  On en tire la boîte englobante, le centre de gravité, l'aire et l'amplitude maximale.

**Choix.**
L'aire ne compte que les pixels réellement au-dessus du seuil et pas ceux ajoutés par la fermeture.
La fermeture sert à regrouper, pas à mesurer.
Une tache est gardée entre `min_area` (4) et `max_area` (2 700) px² d'origine, bornes comprises.
À 960 px de large, il faut donc deux pixels de travail ([02 § 4](02-methode.md#4-taches)).

**Deux passages** (`engine/src/detector.rs`).
Le premier ne prend que les pixels forts.
Fermeture et composantes y donnent les détections fortes, identiques à l'octet à celles d'un seuil sans partie faible.
Le second n'a lieu que s'il reste des pixels faibles.
Il reprend tous les pixels et ne garde que les taches sans aucun pixel fort : ce sont les détections faibles.
*Pourquoi deux passages* : relier en un seul passage les pixels faibles aux taches fortes grossit et déplace les détections fortes.
Sur la 092 à 0,75, un seul passage perdait 114 détections et ajoutait 8 pistes (#21).

## 5 bis. Écarter les images que le décodeur a réparées

**Le problème.**
Sur 17 des 22 vidéos des sessions du 2 septembre et du 2 octobre 2026, il manque quelques octets à la fin de certaines images.
Le décodeur bouche le trou par *dissimulation d'erreur* : il recopie un morceau d'image voisine.
L'image réparée ressemble à une vraie scène.
Une image P réparée sert de référence aux suivantes, donc le dégât dure jusqu'à l'image-clé suivante (8 à 16 images).
Avec la v0.1.1 et sans ce filtre, la 122 donnait des centaines de fausses pistes.
Avec le seul filtre de stabilité, les deux tiers de la vidéo étaient ignorés comme « jumelles qui bougent ».

**Outil.**
Pour chaque image, la lecture qui sert à la détection donne aussi son horodatage `pts`, si c'est une image-clé et les signalements d'erreur du décodeur (`media/src/decode_errors.rs`).
Une image est abîmée si le décodeur la signale ou si elle suit un trou dans les `pts` (`engine/src/lost.rs`).
Il y a trou quand l'écart à l'image précédente, divisé par le pas habituel et arrondi, dépasse une image.
Sur la 122, 13 images n'ont jamais été décodées sans que le décodeur le signale.
Il manque des images avant la première ou après la dernière ? La plage part du début ou de la fin de la vidéo.
La plage écartée va de l'image abîmée à l'image qui précède la prochaine image-clé (`engine/src/damage.rs`).

**Pourquoi une marge.** Le fond d'une image est la médiane de ±½ s d'images (§ 2).
Une image saine à moins d'une demi-fenêtre d'une image réparée a donc un fond faux.
Sur la 122, la v0.1.1 sans marge laissait 77,8 s illisibles, mais aussi 50,3 s instables et 4 pistes.
Les 81 images qui déclenchaient encore le filtre de stabilité étaient toutes à 9 images au plus d'une plage abîmée, d'un côté ou de l'autre.
Les 4 pistes finissaient toutes 1 à 6 images avant une plage.
Chaque plage est donc élargie de la demi-fenêtre du fond de chaque côté (15 images à 30 i/s).
L'élargissement se fait avant la fusion des plages, la stabilité et le suivi.

**Pourquoi les signalements du décodeur.**
Ils viennent de la lecture même qui sert à la détection.
Chaque signalement tombe donc sur la bonne image, sans deuxième passe sur le fichier et sans rien coûter de plus.
Le journal de FFmpeg écrirait un message par image réparée sur la sortie d'erreur.
Il est mis en silence une fois pour toutes (`media/src/init.rs`) : les images abîmées sont comptées et situées dans l'image résumé et `params.json`.
Le nombre d'images est contrôlé : les images décodées doivent être les images annoncées par le fichier, moins les pertes expliquées.
Une perte s'explique par un saut de `pts`, par des images manquantes avant la première ou par des paquets illisibles à la fin.
Sinon la vidéo est comptée en échec (« la vidéo est abîmée ou incomplète »). Les autres vidéos du lot continuent (`engine/src/input.rs`).

**Ce que la sortie en dit.**
Le temps écarté est signalé à part des mouvements, à trois endroits.
Le bandeau de l'image résumé porte des lignes « illisible 1:12 - 1:15 », trois au plus, puis « + N autres, X illisibles en tout ».
La frise du temps le montre aussi, et `params.json` le donne dans `damaged_s`. Ce temps **inclut la marge**.
`damaged_frames` compte les images signalées par le décodeur ou qui suivent un trou : 196 sur la 122.
Il ne compte pas les images écartées (6 043).

**Ce que ça coûte.**
Les chiffres ci-dessous sont ceux de la v0.1.1. Le Rust les reproduit plage pour plage sur 25 des 26 vidéos de la référence.
Sur la 124, il écarte 43 images de plus, que le décodeur signale par sa dissimulation ([11](11-migration-rust.md)).
Ce sont les chiffres du Mac.
Sous Linux (lumiel), le même décodeur signale 14 images de plus sur 8 vidéos, toujours les mêmes d'une lecture à l'autre.

| Vidéo | Images signalées | Illisible | Instable restant | Union | Pistes avant → après |
| --- | --- | --- | --- | --- | --- |
| 122 (195 erreurs) | 196 | 201,2 s en 79 plages | 0 s | 201,2 s sur 300 | 25 → 0 |
| 093 (11 erreurs) | 11 | 14,0 s en 8 plages | 2,1 s | 16,0 s | 4 → 3 |
| 092 (aucune) | 0 | 0 s | 0 s | 0 s | 14 → 14 |

Sur une vidéo très abîmée, la marge coûte cher : avec 15 images de marge, il reste environ 98 s analysées sur les 300 de la 122.
Sur la 093, une seule période instable reste au lieu de quatre.

**Limites.**

- Une tranche très abîmée peut ne produire aucun signalement : l'image passe alors pour saine.
  On l'a vu avec la v0.1.1 sur une vidéo fabriquée en coupant 60 % d'une tranche.
- Le dégât s'arrête à l'image-clé parce que les images-clés des jumelles sont des IDR, sans image B.
  Un autre appareil pourrait ne pas respecter cette hypothèse.
- Dans une plage abîmée, les pixels que le décodeur invente varient avec le nombre de fils du décodeur : une raison de plus pour écarter la plage entière.
- Le fond de l'image résumé écarte aussi les images abîmées.

## 5 ter. Garder les taches faibles qui comblent un vol

**Outil.**
Une **interpolation linéaire** entre deux détections fortes (`engine/src/bracketing.rs`).
Elle porte sur toutes les détections de la vidéo, après la détection et avant tout le reste.
Une détection faible est gardée quand quatre conditions tiennent.

- Une détection forte la précède et une autre la suit, chacune à 3 images au plus (`SUPPORT_FRAMES`) et à 60 px d'origine par image d'écart au plus (`SUPPORT_STEP`).
- Ces deux appuis sont à 20 px l'un de l'autre au moins (`SUPPORT_MOVE`) : une source immobile n'est pas un vol.
- Elle tombe à 10 px au plus (`BRACKET_TOLERANCE`) de la position interpolée entre les deux, au prorata des images de chaque côté.
- Aucune détection forte de sa propre image n'est à 12 px ou moins entre centres (`BESIDE_STRONG`) : ce serait un morceau de cette tache, pas un trou.

Les autres détections faibles sont jetées. Les détections fortes passent toutes sans changement.

**Pourquoi encadrée.**
Le but est de remplacer les positions que le suivi invente pendant un trou par la vraie tache, quand elle n'a manqué le seuil que de peu.
Sur les 26 vidéos, les pistes rapides de la v0.2.0 (150 px/s et plus) ont 4 236 positions estimées pour 25 079 points (16,9 %).
Sur la 092, elles en ont 26 pour 320. Chaque piste rapide de la v0.2.0 se compare à la piste qui partage le plus de détections avec elle.
Mesures du 17 octobre 2026 (#21) :

- *Encadrée, 10 px* : 4 236 → 3 358 positions estimées sur les 26 vidéos (−20,7 %). La 092 passe de 26 à 20 et garde ses 14 pistes.
- *Appuyée d'un seul côté* : la 092 passe de 26 à 39 positions estimées, avec 4 pistes nouvelles.
  Les détections faibles au bout d'une piste l'entraînent ailleurs.
- *Encadrée, 20 px* : la 092 passe à 21, mais le passage de 3:50.87 se coupe en deux pistes.
- *Prolongée au-delà des appuis* (deux détections fortes du même côté) : c'est la seule variante qui retrouve la cible connue de 3:42.10.
  Il lui faut 0,6 et 40 px, et elle ajoute 9 pistes nouvelles sur la 092.
- *20 px de mouvement exigé* : ne change presque rien. Le bruit ajouté ne vient donc pas de sources immobiles.

**Les détections faibles comptent** dans le filtre des images saturées (§ 6) comme toutes les détections.
Sur les 26 vidéos, les images écartées changent sur 7 d'entre elles : 32 de plus et 26 de moins (098) sur 45 716.
Aucune piste perdue n'y tombe.

**Limites.**
Les détections faibles gardées sont nombreuses là où le bruit est fort : environ 237 000 sur les 26 vidéos, dont 156 000 sur la 103 (+30 % de détections).
Elles n'y ajoutent aucune piste. 57 % d'entre elles sont à 12-36 px d'une détection forte de leur image.
Ce sont des morceaux de la même tache plutôt que des trous comblés, et le critère entre centres les laisse passer.
Les rejeter jusqu'à 36 px garderait le gain (3 381 positions estimées au lieu de 3 358).
Mais deux réunions jugées justes sur la 090 seraient perdues : la règle n'a pas été retenue (17 octobre 2026).
La cible connue de 3:42.10 sur la 092 n'a que 4 images détectées. Ses voisines tombent à 40 px de la prolongation, et elle reste sans piste.
La mémoire et le temps ne bougent presque pas (deux passages de `compare 092` au calme sur lumiel le 17 octobre 2026).
Avec les détections faibles, la 092 prend 450 Mo et 29,4 à 29,8 s. Sans elles, elle prend 441 à 442 Mo et 29,2 à 29,4 s.

## 6. Écarter les images où les jumelles bougent

**Outil.** Un simple **comptage** (`tracking/src/stability.rs`).
Une image est suspecte si elle porte au moins `max_blobs` (20) taches de plus que 6 fois le nombre médian de taches par image de la vidéo ([02 § 4 bis](02-methode.md#4-bis-images-saturées-de-taches)).
Les images abîmées (§ 5 bis) sont retirées du comptage et de la médiane.

**Pourquoi ça marche.** Quand le cadre glisse, les images de la fenêtre du fond ne sont plus alignées.
Chaque contour laisse un résidu, et une image passe de quelques taches à plusieurs centaines.
Un décalage figé est rattrapé par la fenêtre de 1 s : c'est le glissement qui crée les taches.
Le nombre de taches est donc un indicateur de mouvement, et il est gratuit.
Mesurer le décalage du cadre par corrélation de phase coûterait 3,2 ms par image, soit environ 29 s par vidéo de 5 min (v0.1.1).

**Pourquoi une limite relative.** La médiane de la vidéo adapte la limite au niveau de bruit de chaque enregistrement.

**Choix.**
Les détections sont retirées `pad_s` (une seconde) avant et après chaque image suspecte, sans déborder des images lisibles.
Deux périodes que sépare au plus une seconde sont réunies.
Les détections de ces périodes sont vidées avant le suivi (`tracking/src/exclusion.rs`). La détection elle-même ne change pas.
Une piste qui traverse une période est coupée en deux.
La période est toujours signalée : l'outil préfère dire « je n'ai pas regardé ici » que produire des centaines de fausses pistes.

**Limites.** La règle repère des images saturées, pas seulement des mouvements.
Sur la 089, des images à plus de 20 taches apparaissent entre 7,5 et 10,7 s sans mouvement mesurable.
Un essaim de 20 animaux dans une image serait pris pour un mouvement.

## 7. Relier les taches dans le temps : le suivi

**Outils.**
Une **prédiction à vitesse constante** et une **association gloutonne au plus proche voisin** (`tracking/src/track.rs`, `track/growing.rs` et `track/association.rs`).

- Pour chaque piste ouverte, on prédit où elle devrait être à l'image suivante en prolongeant sa vitesse.
- On calcule la distance entre chaque prédiction et chaque tache et on garde les paires à moins de `max_jump` (120 px).
  On les attribue de la plus proche à la plus lointaine.
- Une tache sans piste ouvre une nouvelle piste. Une piste sans tache pendant plus de `max_gap` (6) images est close.

Valeurs et mesures de ces réglages : [02 § 5](02-methode.md#5-suivi).

**Les choix qui viennent des mesures.**

- *La vitesse se mesure sur 3 images au moins.*
  Le capteur tourne à 50 Hz et la vidéo à 30 images par seconde : le pas de l'animal suit un motif « double, simple, simple ».
  Une vitesse mesurée sur deux images se trompe d'un facteur 2 une fois sur trois : sur la 092, la piste la plus rapide (3:48) décrochait 2 images trop tôt.
  La vitesse se prend donc sur le point le plus récent qui a au moins 3 images de recul, sinon sur l'avant-dernier point.
- *Les pistes qui ont déjà une vitesse choisissent en premier.*
  Sinon un fragment né à l'instant vole la tache de la vraie piste, qui se retrouve coupée : le passage de 3:58 de la 092 sortait en deux.

**Fortes d'abord** (`track/association.rs`, `track.rs`).
À chaque image, l'attribution se fait en deux tours.
Les détections fortes passent d'abord, exactement comme si les faibles n'existaient pas.
Puis les pistes restées sans détection prennent les faibles dans le même ordre. Une détection faible restée libre n'ouvre jamais de piste.
*Pourquoi* : sans ces deux règles, les détections faibles mélangeaient des pistes.
Sur les 26 vidéos, l'utilisateur a jugé à l'œil les 15 pistes perdues de la variante sans ces règles (17 octobre 2026, #21).
Il y a trouvé 6 réunions de deux chauves-souris différentes, par trois mécanismes mesurés.

- Une piste prend une détection faible plus proche de sa prédiction que la forte (099 à 3:33.15 : 12,6 px contre 21).
  La forte reste seule et ouvre une piste parasite qui saute sur l'animal voisin.
- La forte laissée seule ou une faible isolée tombe à moins de 36 px de deux animaux qui volent près l'un de l'autre.
  Elle fait alors pont dans la fusion des jumelles (§ 8) : 099 à 3:22.40 et 3:44.21.
- Une piste enchaîne par une détection faible sur la tache de l'animal suivant (095 à 4:46.68, 100 à 2:01.18).

Avec les deux règles, les 6 réunions disparaissent et les 3 réunions jugées justes restent (090 à 2:28.15 et 2:46.50, 099 à 3:21.66).
Sur les 26 vidéos avant recollage, 965 pistes sur 972 s'apparient à la v0.2.0, dont 876 concordantes.
On compte 7 pistes perdues et 37 ajoutées, et la 092 garde ses 14 pistes.
Des 7 perdues, 2 sont des réunions justes. La troisième, à la 099, n'en perd aucune.
Pour 4 d'entre elles, la piste reste une seule piste dont le début bouge de plus de 5 images (096 à 1:42, 098 à 2:27, 101 à 3:42, 125 à 1:11).
La dernière disparaît (097 à 0:16) pour une cause non isolée.

**La portée d'une piste** (`track/reach.rs`, `track/association.rs`, `track/growing.rs`).
Une piste qui a déjà une vitesse ne prend une tache que si celle-ci est à moins de **50 px plus le chemin qu'elle devait parcourir** de l'endroit prévu.
Ce chemin est sa vitesse multipliée par le nombre d'images depuis son dernier point.
Une piste qui avance de 10 px par image et a manqué 2 images peut prendre une tache à 50 + 30 = 80 px de l'endroit prévu.
La limite de 120 px reste au-dessus. Une piste d'un seul point n'a pas encore de vitesse : elle n'est tenue que par cette limite.
Le choix de 50 px et d'une fois le chemin est mesuré en [02 § 5](02-methode.md#5-suivi).

- *Le problème.* Sans portée, une piste lente peut sauter jusqu'à 120 px sur une autre chauve-souris.
  À la 090 vers 2:09.87, une piste qui avance de 12,6 px par image a perdu son animal 4 images plus tôt.
  Elle prend une tache à 109 px de l'endroit prévu : 3 pistes pour 2 chauves-souris.
  À la 095 vers 4:46.68, une piste qui avance de 7,5 px par image reste 3 images sans tache.
  Elle prend ensuite une tache à 116 px, sur une autre chauve-souris.
- *Pourquoi pas le ralentissement ni la direction.*
  Le recollage ([§ 9 bis](#9-bis-recoller-un-vol-caché-derrière-la-végétation)) refuse un pas plus lent que le quart de la vitesse de la piste.
  Appliquée à l'échelle d'une image, la même règle change 312 pistes sur 1 002.
  Elle ne voit aucun des deux cas, qui sont des sauts trop rapides.
  La direction ne sépare pas non plus : le saut fautif de la 095 ne tourne que de 39°, alors que 1 230 des 36 292 taches prises par une piste lancée tournent de plus de 90°.
  Refuser au-delà de 90° change 59 pistes et rate la 095.
- *Le bord.*
  La portée ne joue pas quand le dernier point de la piste, la tache ou le point d'où la vitesse est mesurée est à moins de 30 px d'un bord.
  C'est la même marge que pour le recollage. Un animal qui entre a d'abord une tache coupée par le bord.
  Le centre de cette tache avance moins vite que lui.
  Sa vitesse est sous-estimée, et la portée refusait de vrais débuts de vol (096 à 1:42, 095 à 4:27 et 4:36, 099 à 0:19, 100 à 3:09).
  Avec le dernier point et la tache épargnés au bord, 39 pistes changent au lieu de 56. La 090 et la 095 restent corrigées.
  Le point d'où la vitesse est mesurée compte aussi.
  À la 125 vers 3:09.44, une chauve-souris entre à 6 px du bord droit en tache énorme et floue.
  La portée la coupait en deux pistes à 31 px du bord, sur une vitesse prise depuis la tache coupée.
- *La vitesse effondrée.*
  Quand la vitesse de la piste (mesurée sur au moins 3 images) tombe sous 0,4 fois sa vitesse sur ses 5 derniers points, la portée prend cette seconde vitesse.
  Une tache qui se dédouble ou un point à contretemps peut faire stagner la piste une image.
  À la 095 vers 4:04.85, une piste de 3 points tombe ainsi à 0,6 px par image contre 16 sur ses points et refusait la suite de sa chauve-souris.
  À la 102 vers 1:27.78, une piste tombe à 3,1 px par image contre 14 et perdait son dernier point.
  Le seuil donne le même résultat de 0,25 à 0,6. À 0,2, la 102 reste coupée. À 0,75, la 090 vers 2:28.88 change à tort.
  Mesurée sur toute la piste plutôt que sur ses 5 derniers points, la même règle change aussi ce cas de la 090.
- *Les cas regardés à l'œil.*
  L'utilisateur a jugé sur une planche les 32 endroits où la portée changeait les pistes des 26 vidéos (18 octobre 2026).
  Il en a trouvé 21 mieux et 5 pires. Les autres sont sans effet visible ou ont un défaut présent dans les deux versions.
  Le bord du point de départ de la vitesse et la vitesse effondrée corrigent 3 des 5 pires et rien d'autre.
  Sur ces cas, ils redonnent exactement le suivi sans portée, jugé juste.
- *Ce que ça donne.* Sur les 26 vidéos, on compte 1 002 pistes avant recollage, autant que sans portée.
  Après recollage, il y en a 921 au lieu de 922. La 092 garde ses 14 pistes avec les mêmes débuts et les mêmes fins.
  Deux points y changent de tache, de 1 à 4 px (0:35.20 et 1:49.82).
- *Ce qu'elle ne corrige pas.*
  À la 100 vers 2:00 à 2:02, une piste naît sur un éclat de la tache d'une autre et n'a pas encore de vitesse.
  Elle saute à 110 px en 7 images sur une troisième chauve-souris, au rythme d'un vol normal.
  Elle lui prend ensuite sa tache pour 0,4 px d'écart (12,2 contre 12,6 px de l'endroit prévu).
  Cela fait 3 pistes pour 2 chauves-souris, et aucune règle de vitesse ne le voit.
  À la 090 vers 2:12.07, deux pistes suivent une même chauve-souris grosse et rapide (30 à 50 px par image).
  Elles ne se réunissent pas en jumelles.
  À la 098 vers 1:06.70, la portée coupe 2 points qui reviennent en arrière au bout d'un morceau de vol d'une autre chauve-souris, jusque-là jeté.
  Le morceau devient une piste, et le recollage la soude à tort à la piste d'avant.

**Pourquoi pas plus savant.** Un filtre de Kalman ou une affectation optimale (algorithme hongrois) sont les outils habituels.
Avec quelques taches par image, le glouton donne le même résultat et reste court.

**Limites.**
Une interruption de plus de 6 images coupe le vol en deux pistes.
Le recollage les réunit quand le trou ne dure pas plus de 19 images ([§ 9 bis](#9-bis-recoller-un-vol-caché-derrière-la-végétation)).
La piste commence quand la tache dépasse le seuil, donc parfois après l'arrivée réelle de l'animal.
Au banc, une cible de chasse sur dix commence avec 3 images de retard ou plus (#42).
Sur la 092, la piste de 3:57 commence sur l'image où l'animal arrive (6 octobre 2026).
Trois valeurs de la portée sont des constantes du code : le bord à 30 px, le seuil de la vitesse effondrée à 0,4 et ses 5 points.

## 8. Fusionner les pistes jumelles

**Outil.** Une structure **union-find** regroupe des éléments par paires jusqu'à former des familles (`tracking/src/track/twins.rs`).

**Règle.**
Deux pistes qui coexistent, partagent au moins une image et restent à moins de `twin_distance` (36 px) l'une de l'autre sur *toutes* leurs images communes sont un seul animal vu en deux morceaux.
Elles sont fusionnées : boîte englobante commune, aires additionnées, centre pondéré par l'aire, amplitude la plus forte.
Le point réuni est fort si l'un des deux l'est.

**Pourquoi « toutes ».** Deux vrais animaux qui volent ensemble s'écartent au moins une fois.
Deux fragments d'un même animal ne s'écartent jamais. Cette règle n'a été vérifiée que sur des trajectoires fabriquées (#26).

**Limite.**
La règle se juge paire par paire, et les familles se forment de proche en proche.
Une piste d'un seul point suffit donc à réunir deux animaux proches si elle est à moins de 36 px des deux sur sa seule image (mesuré sur la 099, #21).
« Fortes d'abord » (§ 7) retire les ponts faits de détections faibles.
Exiger que toutes les pistes d'une famille soient jumelles deux à deux supprimait aussi les autres ponts, mais ajoutait 3 pistes sur la 092 : cette règle n'a pas été retenue.

## 9. Garder ce qui ressemble à un vol

Trois filtres s'appliquent, du plus simple au plus fin (`tracking/src/track.rs` et `types/src/track.rs`).
Leurs valeurs sont mesurées en [02 § 7](02-methode.md#7-filtres-finaux) :

- **Au moins `min_hits` détections** (6) : élimine les étincelles.
- **Au moins `min_travel` (45 px) entre le début et la fin** : élimine un point qui scintille sur place.
- **Virage médian sous `max_median_turn` (0,8 radian)** : le virage est l'angle entre deux pas successifs non nuls.
  Un vol va quelque part, donc ses virages sont petits.
  Du bruit enchaîné par le suivi part dans tous les sens, avec un virage médian de 1,5 radian.
  La médiane tolère un demi-tour isolé, ce qu'une moyenne ne ferait pas. Une piste trop courte pour avoir un virage est gardée.

Les pistes gardées passent ensuite au recollage ([§ 9 bis](#9-bis-recoller-un-vol-caché-derrière-la-végétation)).
Les morceaux de 2 à 5 détections que `min_hits` a écartés y vont aussi, mis de côté pour servir de pont.
Les pistes sont enfin numérotées dans l'ordre de leur première image.

**Limite.** Un vol très lent avance de quelques pixels par image, et l'arrondi des positions fait tourner sa direction mesurée.
Un fragment court d'un vol lent peut donc être rejeté.

## 9 bis. Recoller un vol caché derrière la végétation

**Le problème.**
Les vidéos 095 à 100 sont prises du même endroit.
Une chauve-souris y passe souvent derrière une touffe de végétation accrochée à la falaise puis réapparaît de l'autre côté.
Pendant ce temps, elle n'est plus dans l'image.
Le suivi tient 6 images sans la voir (`max_gap`) puis ferme la piste, et la suite du vol ouvre une seconde piste.
Un animal compte alors pour deux. Or c'est le nombre de pistes que la naturaliste lit en premier.

**Ce qui a été mesuré** (référence v0.2.0, 26 vidéos, 17 octobre 2026, sans décoder sauf pour regarder les trous).

- *Où la cible disparaît.*
  Sur le cadrage de la 095 à la 100, les images manquantes dans les pistes se concentrent dans deux zones sombres sur l'image : de la végétation.
  Les coordonnées qui suivent sont en pixels de la vidéo d'origine.
  En bas, vers x 920 à 1080 et y 720 à 800, il manque jusqu'à 81 % des images des pistes qui y passent, contre 2 à 10 % autour.
  En haut, vers x 920 à 1040 et y 400 à 520, il en manque jusqu'à 70 %.
  Même les pistes qui traversent ces zones sans se couper y perdent leur cible plus de 4 images sur 10.
- *Ce que voit la détection pendant le trou.*
  À l'endroit où la chauve-souris devrait être, l'écart au fond tombe au niveau du bruit : 0 à 13 niveaux, contre 16 à 41 juste avant.
  Le seuil reste à son plancher de 12. La végétation agitée ne relève donc pas le seuil, et la cible n'est pas juste trop faible.
  La végétation est froide, donc sombre. Une chauve-souris chaude devant elle ressortirait : elle passe derrière.
  Baisser le seuil n'y changerait rien : il n'y a rien à détecter.
- *Combien de temps elle reste cachée.*
  Une chauve-souris vole à 12 à 13 px par image près de ces zones.
  Elle traverse chacune en 8 images en médiane, et en 12 à 18 images pour les plus lentes (une sur dix).

**La règle des deux bouts** (`tracking/src/track/rejoin.rs`). Deux pistes sont un même vol quand quatre conditions tiennent.

- La seconde commence après la fin de la première, après un trou de **plus de 6 images** et de **19 images au plus**.
  Cela fait 8 à 20 images entre le dernier point de l'une et le premier de l'autre, à peine plus d'une demi-seconde.
  Sous 7 images, le suivi l'aurait reliée lui-même. S'il ne l'a pas fait, il avait une raison.
- **Les deux bouts s'accordent.** On mesure la vitesse de la fin de la première piste et celle du début de la seconde, sur leurs 5 points les plus proches du trou.
  On prolonge la fin de la première sur toute la durée du trou à la moyenne de ces deux vitesses.
  La seconde piste doit démarrer à **60 px au plus** de ce point. C'est comme si l'animal virait au milieu du trou.
  Prendre les deux bouts plutôt que la première piste seule suit une chauve-souris qui tourne pendant qu'elle est cachée.
- Aucune image du trou n'appartient à une **période ignorée** (jumelles qui bougent, image abîmée) : on ne sait pas ce qui s'y est passé.
- Ni la fin de la première ni le début de la seconde ne sont **à moins de 30 px d'un bord** de l'image.
  Une chauve-souris qui sort du cadre et y revient n'est jamais recollée, car rien ne dit que c'est la même.
  Les fins de piste s'entassent près des bords (188 à moins de 5 px, puis 81, 49, 26 par tranche de 5 px) et retombent au niveau du reste vers 30 px.
  Sur la règle des deux bouts seule, le bord écarte 16 paires.

Une fin peut parfois se prolonger par plusieurs débuts, ou un début continuer plusieurs fins.
Les paires sont alors prises de la plus proche à la plus lointaine, chaque fin et chaque début une seule fois.
Un vol caché deux fois devient une seule piste.
Aucun point n'est inventé dans le trou : l'image résumé et les extraits le comblent en ligne droite, comme les trous plus courts.
Valeurs des réglages et leur justification : [02 § 7 bis](02-methode.md#7-bis-recollage).

**Ce qui n'est pas recollé.**
Une disparition de plusieurs secondes n'est pas recollée.
Sur la 096, une chauve-souris entre dans la végétation haute et une autre piste en sort 5,9 s plus tard.
L'utilisateur a jugé que ce n'était pas le même passage. À cette échelle, ni la vitesse ni la direction ne disent plus rien.
Sur la 095 à la 100, 124 fins de piste sur 709 ont une autre piste qui démarre 1 à 6 s plus tard à moins de 130 px.
Si l'on décale les pistes de 10 ou 20 s, on en trouve encore 92 et 83 : deux recollages sur trois souderaient deux animaux.
Une sortie du cadre suivie d'un retour ne l'est pas davantage.

**Pourquoi pas autre chose.**

- *Prolonger la première piste seule, dans un cône qui s'élargit avec le temps.*
  On l'a calibré sur des trous creusés dans des pistes continues, donc à coup sûr sur un même animal.
  Le cône part de 37° à 8 images et s'ouvre de 2,7° par image.
  À trou égal (8 à 20 images sans la règle du bord), il soude 91 paires pour 7,7 au hasard, contre 83 pour 3,3 avec les deux bouts.
  Un virage pendant le trou l'oblige à s'ouvrir.
- *Baisser le seuil dans les zones de végétation* : il n'y a rien sous le seuil.
- *Une affectation optimale* : elle chercherait la meilleure combinaison de toutes les paires à la fois au lieu de les prendre de la plus proche à la plus lointaine.
  Elle ne changerait le résultat que là où une fin ou un début a plusieurs candidats : 5 fois sur les 70 paires que la règle des deux bouts seule retient.
  L'ordre du plus proche y suffit tant qu'aucun cas jugé ne le contredit.

**Les morceaux trop courts pour faire une piste.**
Une chauve-souris peut se cacher deux fois de suite : derrière une touffe, puis derrière la suivante.
Entre les deux, on ne la voit que 2 à 5 images. Ce morceau a trop peu de points pour être gardé : il en faut 6 (§ 9).
Sans lui, les deux pistes de part et d'autre sont trop loin l'une de l'autre dans le temps pour être recollées.
Juste avant le début d'une piste ou juste après sa fin, le même morceau est souvent le vrai début ou la vraie fin du vol.
La cible y était encore faible. Sur les 26 vidéos, on compte 9 363 morceaux de moins de 6 détections, dont 1 562 d'au moins 2.

- *La pile à part.* Les morceaux d'au moins 2 détections que le nombre de points a écartés sont mis de côté.
  Ils ne servent qu'à **relier deux pistes gardées** ou à **prolonger une piste gardée**, par la règle des deux bouts ci-dessus.
  Le trou fait 7 à 19 images et le départ est à 60 px au plus du point prévu, jamais au bord ni à travers une période ignorée.
  Deux morceaux ne se relient jamais entre eux. Un morceau qui ne touche aucune piste disparaît : il ne devient jamais une piste.
  Les liens entre deux pistes gardées sont pris d'abord, de la plus proche à la plus lointaine.
  Viennent ensuite ceux des morceaux, sur les bouts restés libres.
  Un morceau ne peut donc qu'ajouter un lien, jamais défaire une soudure entre deux pistes.
  Les points d'un morceau font partie de la piste mais ne décident pas de son zoom ([02](02-methode.md#extrait-zoomé-et-ralenti)).
- *L'allure.* La chauve-souris ne ralentit pas d'un coup derrière un buisson.
  On compare deux vitesses à celle de la piste gardée à son bout (ses 5 points les plus proches du trou).
  La première est celle du saut par-dessus le trou : la distance entre le bout de la piste et le morceau, divisée par les images qui les séparent.
  Elle doit en valoir au moins le quart (0,25). La seconde est celle du morceau lui-même, de son premier à son dernier point.
  Elle doit en valoir au moins 0,45. Un morceau d'un seul point n'aurait que le saut.
  Entre deux pistes gardées, la règle vaut aussi : le saut doit valoir au moins le quart de la vitesse de chacune des deux.
  La direction ne compte pas : la porte de 60 px autour du point prévu la borne déjà.
  Sur la planche des morceaux, certains bons cas s'écartent même de 55° à 151° de la direction de leur piste, plus que les douteux.
- *Pourquoi aussi entre deux pistes gardées.*
  L'allure refuse 2 des 67 soudures de la règle des deux bouts, sur la 095 (4:59.33) et la 097 (2:17.33).
  La seconde piste y repart presque là où la première s'est arrêtée, une demi-seconde plus tard.
  L'animal volait à 6 à 9 px par image, mais le saut ne va qu'à 1,6 et 0,5 px par image.
  À l'œil, rien ne prouve que ce soit la même chauve-souris. Dans le doute, on sépare.
  Les 8 paires jugées « même passage » sur la première planche passent, avec un saut à 0,62 fois la vitesse au moins.
  Les 2 jugées « non » auraient aussi été refusées par l'allure.

**Ce que ça donne.**

- *Sur les 26 vidéos*, l'allure, les morceaux, les détections faibles et la portée du [§ 7](#7-relier-les-taches-dans-le-temps--le-suivi) donnent 81 soudures.
  Elles se répartissent par vidéo (093 : 3, 094 : 1, 095 : 13, 096 : 17, 097 : 17, 098 : 10, 099 : 7, 100 : 5, 101 : 1, 102 : 3, 125 : 3, 129 : 1) et ramènent 1 002 pistes à 921.
  La 092 garde ses 14 pistes. Sans la portée, il y en aurait 80. La soudure de plus est celle de la 098 vers 1:06.70, qui est fausse (§ 7).
- *Fausses soudures au hasard* : on applique les mêmes règles à des pistes décalées de 10 ou 20 s, qui ne peuvent pas être le même animal.
  On mesure ainsi le taux de soudures fausses ([02 § 7 bis](02-methode.md#7-bis-recollage)).
- *Le banc de chasse* est très dense (720 cibles synthétiques en 5 minutes).
  La règle des deux bouts seule y collait une fois sur 15 deux cibles différentes qui se suivaient au même endroit.
  Avec les morceaux et l'allure, des cibles commencent plus tôt et se coupent moins.
  Cette cible est retrouvée, et le banc trouve 455 cibles, comme sans recollage.
- *Les cas regardés à l'œil* : sur une planche de 10 paires, l'utilisateur en a jugé 8 « même passage » et 2 « non » (le trou de 5,9 s et une sortie du cadre).
  La règle recolle les 8 et aucun des 2. Le plus grand écart des 8 au point de rencontre est de 45 px, et le plus long trou fait 13 images.

**Limites.** Un recollage ne se voit pas comme tel dans les résultats : la piste recollée a simplement un trou.
Une soudure fausse reste possible quand deux chauves-souris se suivent de près au même endroit.
Une seule détection entre deux buissons reste perdue.

## 10. Aller vite sans changer le résultat

- **Le décodage a ses propres fils** : FFmpeg décode plusieurs images à la fois (`media/src/decode.rs`).
  Un fil de lecture garde 36 images d'avance, soit un lot de 32 plus 4 (`engine/src/read_ahead.rs`).
- **Le reste se partage entre tous les cœurs** : les fils de calcul (`rayon`) préparent (gris, réduction, flou) puis détectent un lot de 32 images à la fois (`engine/src/detector.rs`).
  Chaque image ne dépend que de sa fenêtre, et tout le calcul est en entiers jusqu'au seuil.
  Le résultat est donc identique au bit près avec 1, 3 ou 4 fils et des lots de 1 à 64 images. Un test le vérifie.
- **Taches mesurées en une passe** : l'aire et le pic de toutes les taches d'une image sont comptés en un seul parcours des pixels au-dessus du seuil, au lieu d'un masque de l'image par tache.
  À 1440 px, le bruit fait des centaines de taches par image. C'est alors cette passe unique qui garde la détection rapide.
- **Le rendu ne relit pas tout** : pour chaque groupe d'extraits, il se place par horodatage sur une image-clé saine avant le groupe (`render/src/source.rs`).

Les temps de chaque étape et l'usage des cœurs sont dans [09](09-profilage.md).

## 11. Savoir si c'est juste

Sans vérité terrain, tout réglage est un pari. Deux outils y répondent.

- **Le banc de cibles synthétiques** ([08](08-banc-de-mesure.md), crate `bench`) incruste de fausses chauves-souris dans une vraie vidéo.
  Ce sont des taches gaussiennes de taille et de contraste connus.
  Elles suivent des courbes de Bézier ou des vols de chasse simulés, avec la saccade des jumelles.
  On compte ce que l'outil retrouve, avec un **intervalle de Wilson** qui dit si un écart entre deux réglages est réel ou dû au hasard.
- **La vidéo de référence** : 14 pistes sur la 092, vérifiées par un test automatique (`cli/tests/reference_092.rs`).
  Selon le verdict de la naturaliste du 9 octobre 2026, 13 sont des chauves-souris.
  La piste lente de 3:29 n'en est pas une ([03](03-resultats-092.md#verdict-de-manon-9-octobre-2026)).

Ce que le banc ne dit pas : si une piste réelle est une chauve-souris. Cela reste le jugement de la naturaliste.

## Ce que cette logique suppose

| Hypothèse | Si elle est fausse |
| --- | --- |
| Les jumelles sont fixes | Les périodes de mouvement sont ignorées et signalées |
| L'animal bouge de plus que sa taille en une seconde | Il s'efface dans le fond |
| L'animal s'écarte du fond filtré de plus de 12 niveaux et de 8 fois le bruit du pixel (1,4826 × MAD), sauf entre deux images où il le fait, à 3 images au plus de chaque côté (alors 9 niveaux et 6 fois le bruit suffisent) | Il n'est pas vu ; le ciel saturé est aveugle ; devant de la végétation agitée, il faut un contraste plus fort |
| La cible a à peu près la taille du filtre gaussien (1,5 px d'origine) | Plus petite, elle perd du contraste ; plus grande, elle est moins rehaussée |
| Une cible allume au moins deux pixels de travail (960 px sur une source de 1440) | L'image est perdue (`min_area`) |
| Le gain des jumelles change peu en une demi-seconde | Autour d'un saut de 30 niveaux ou plus, quelques secondes sont ignorées |
| Le vol est régulier d'une image à la suivante | La piste se coupe ou est rejetée |
| Deux animaux proches finissent par s'écarter | Ils sont fusionnés en un seul |
| Le décodeur signale chaque image qu'il répare, dissimulation comprise | Une image réparée en silence passe pour une vraie scène |
| Les images-clés arrêtent le dégât (IDR, sans image B) | Le dégât déborde au-delà de la plage écartée, et le rendu peut repartir d'une image fausse |
| Toute image perdue s'explique par un saut de `pts`, un manque avant la première image ou des paquets illisibles à la fin | La vidéo est comptée en échec |
| Les `pts` connus croissent strictement | Les extraits ne se font pas : la vidéo est comptée en échec, son CSV et son image résumé déjà écrits |
