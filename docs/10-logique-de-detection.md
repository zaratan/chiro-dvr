# La logique de détection, expliquée

Ce document explique **comment l'outil raisonne** : quel outil mathématique ou technique sert à chaque étape, pourquoi celui-là plutôt qu'un autre, et ce qu'il ne sait pas faire.
Chaque étape nomme le module du code qui la fait (chemins relatifs à `crates/`) et le défaut de ses réglages.
Les valeurs des réglages et leurs mesures sont dans [02-methode.md](02-methode.md) ; les temps de calcul dans [09](09-profilage.md) ; les pistes d'amélioration dans [07](07-ameliorer-la-detection.md).

## L'idée en une phrase

Une chauve-souris est **une petite tache qui s'écarte du fond pendant quelques images et qui se déplace de façon régulière**.
Tout le traitement découle de ces trois mots : fond, tache, régularité.

## Les techniques en simple

Neuf techniques font le travail, dans l'ordre où elles s'appliquent.
Chacune se lit en trois temps : ce qu'elle fait, une image de la vie courante, ce qu'elle rate.

**Le filtre à la taille de la cible.**
Chaque image est d'abord adoucie à la mesure d'une petite chauve-souris lointaine : le grain du capteur s'atténue plus qu'une tache de cette taille.
C'est le peintre qui plisse les yeux : les petits détails se fondent, les formes de la bonne taille restent.
Une cible bien plus petite que prévu y perd du contraste et peut passer inaperçue.

**La scène sans animal** ([§ 2](#2-estimer-le-fond--la-médiane-temporelle)).
Pour chaque endroit de l'image, l'outil range les valeurs des images voisines dans le temps et garde celle du milieu : la scène telle qu'elle est quand rien ne la traverse.
C'est le photographe qui prend une dizaine de photos d'un monument envahi de touristes et garde, pour chaque endroit, ce que la plupart des photos montrent : les passants disparaissent, le monument reste.
Elle rate un animal qui s'attarde au même endroit : il entre dans le décor.

**Un seuil propre à chaque endroit** ([§ 4](#4-décider-pixel-par-pixel--le-seuil)).
Après avoir corrigé les changements de luminosité de toute l'image ([§ 3](#3-compenser-le-gain-automatique)), l'outil retient un pixel s'il s'écarte assez de la scène sans animal, plus clair ou plus sombre ; « assez » vaut un minimum partout, relevé là où l'endroit s'agite dans les instants qui entourent l'image.
Il faut toujours parler assez fort pour être entendu ; près d'une route, il faut crier.
Il rate une cible faible qui passe devant de la végétation agitée.

**Les taches** ([§ 5](#5-regrouper-les-pixels-en-taches)).
Les pixels retenus qui se touchent, ou presque, forment une tache ; une tache de taille plausible sur une image est un point.
C'est entourer d'un seul trait les gouttes d'une même éclaboussure.
Elles ratent une cible qui n'allume presque aucun pixel sur une image : elle n'y laisse pas de point.

**Les taches faibles encadrées** ([§ 5 ter](#5-ter-garder-les-taches-faibles-qui-comblent-un-vol)).
Une tache un peu trop faible pour le seuil, au-dessus des trois quarts, est gardée si une vraie tache la précède et une autre la suit de près, et si elle tombe sur le chemin entre les deux.
C'est le pointillé qu'on devine entre deux traits nets d'une même route.
Elle ne sert qu'à boucher les trous d'un vol déjà vu : elle n'ouvre jamais de piste, et une piste ne la prend que si aucune vraie tache n'est à sa portée.
Elle rate une cible qui reste faible plusieurs images de suite, sans vraie tache de part et d'autre.

**Les périodes ignorées** ([§ 6](#6-écarter-les-images-où-les-jumelles-bougent)).
Quand l'image se remplit soudain de taches, le plus souvent parce que les jumelles ont bougé, ou quand le fichier vidéo est abîmé, l'outil ne cherche rien sur ce moment et ses abords, et le signale sur l'image résumé.
C'est un observateur qui note « je n'ai pas regardé » plutôt que d'inventer.
Une chauve-souris qui passe pendant ces périodes est manquée, et un essaim qui remplit l'image passe pour un mouvement des jumelles : ces périodes sont à revoir à l'œil.

**Le suivi image par image** ([§ 7](#7-relier-les-taches-dans-le-temps--le-suivi)).
L'outil prolonge le trajet de chaque piste pour prédire où elle sera à l'image suivante, et lui donne le point le plus proche de cette prédiction, s'il n'est pas trop loin, une vraie tache passant toujours avant une tache faible ; une vraie tache restée seule ouvre une nouvelle piste.
C'est suivre des yeux un ballon lancé en devinant sa course.
Un vol qui disparaît plus de 6 images en sort en deux pistes, que le recollage réunit ensuite quand il le peut.
Une piste ne commence qu'une fois que la cible se détache assez du fond, après l'arrivée réelle de l'animal.

**La fusion des morceaux** ([§ 8](#8-fusionner-les-pistes-jumelles)).
Deux pistes qui restent tout près l'une de l'autre sur toutes leurs images communes sont réunies : ce sont deux morceaux du même animal, comme les deux phares d'une même voiture la nuit.
Deux animaux qui volent côte à côte, tout près, du début à la fin sont comptés pour un seul.

**Ressembler à un vol** ([§ 9](#9-garder-ce-qui-ressemble-à-un-vol)).
Une piste n'est gardée que si elle a assez de points, si elle finit assez loin de son point de départ et si elle garde à peu près sa direction d'une image à l'autre.
Un oiseau traverse le jardin ; une poussière danse sur place dans un rayon de soleil.
Ce filtre peut rejeter un vol très lent, qui paraît hésitant, ou une boucle qui revient à son point de départ.

**Le recollage** ([§ 9 bis](#9-bis-recoller-un-vol-caché-derrière-la-végétation)).
Une chauve-souris qui passe derrière un buisson disparaît de l'image quelques images, puis réapparaît de l'autre côté : le suivi en fait deux pistes.
L'outil prolonge la fin de la première et le début de la seconde ; si les deux prolongements se rejoignent au bon moment, c'est le même vol.
C'est voir une voiture passer sous un pont et savoir que c'est la même qui en ressort.
Il ne recolle pas une piste qui finit ou commence tout près du bord de l'image, où la chauve-souris a pu sortir du cadre, ni une disparition de plusieurs secondes.
Un bout de vol trop court pour faire une piste, aperçu entre deux buissons, peut servir de pont entre deux pistes ou en prolonger une ; seul, il ne devient jamais une piste.
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
| Réduction | `engine/src/gray.rs`, `reduction.rs`, `blur.rs`, `prepare.rs` | table de gris, moyenne de surface, flou gaussien en entiers | Que voit le capteur, en plus petit, à la taille d'une chauve-souris ? |
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

Seuls quatre réglages sont des options de la commande : `--threshold`, `--min-hits`, `--max-blobs` et `--work-width`, plus le masque `--osd-region`.
Les autres ont une valeur fixe, écrite dans `params.json` ; ceux de la détection, de la stabilité et du suivi restent réglables dans le banc ([08](08-banc-de-mesure.md)).

## 1. Lire et réduire l'image

**Outil.** FFmpeg, lié dans le binaire (`media/src/decode.rs`), décode la vidéo.
Seule la luminance sert : chaque valeur passe par une table de gris (`engine/src/gray.rs`), mesurée sur la conversion d'OpenCV qu'employait la v0.1.1, pour garder les mêmes niveaux.
L'image est ensuite réduite à `work_width` (960 px par défaut) par **moyenne de surface**, avec un seul arrondi (`reduction.rs`) : sur une vidéo de 1440 px, chaque pixel réduit moyenne 1,5 × 1,5 pixels d'origine.
Puis un **flou gaussien** d'écart-type `target_sigma` (1,5 px d'origine, la taille d'une petite chauve-souris lointaine, soit 1 px de travail à 960 sur 1440) est appliqué à chaque image réduite, avant tout le reste (`blur.rs`, `prepare.rs`).

**Pourquoi le flou.**
C'est le **filtre adapté** (*matched filter*) d'une tache gaussienne dans du bruit, le meilleur rapport signal sur bruit possible pour une cible de cette taille : il moyenne le bruit d'un pixel à ses voisins sans écraser une tache de quelques pixels.
Il remplace le débruitage qu'apportait jusqu'au 6 octobre 2026 la réduction à 480 px, qui faisait perdre 30 à 50 % de leur contraste aux cibles de moins de 3 px ([07](07-ameliorer-la-detection.md)).

**Choix.**
Filtrer l'image avant le fond plutôt que le résidu après, pour que le fond, le résidu et le bruit de chaque pixel soient mesurés dans le même espace.
960 px plutôt que 1440 : sur sept vidéos, la pleine résolution donne presque les mêmes pistes pour 2,4 fois le temps de calcul ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)).
Des entiers plutôt que des flottants : le noyau du flou a des poids entiers de somme 4 096, porte à 4 écarts-types, replie les bords en miroir, et l'image filtrée garde ses valeurs au 1/64 de niveau.
Le calcul est exact, donc le même quel que soit le découpage du travail entre les fils (§ 10).

**Limites.** Une tache plus grande que le filtre est moins rehaussée, une plus petite perd du contraste.
Sur une vidéo de 1440 px réduite à 960, un pixel de travail vaut 2,25 px² : `min_area` 4 exige deux pixels, et une cible qui n'en allume qu'un sur une image perd cette image.
`--work-width` change la largeur de travail (au moins 16 px, jamais plus que la source) ; `target_sigma` ne se règle que dans le banc.

**Détail technique.**
Un fil décode en avance (36 images : un lot de 32, plus 4) ; la conversion en gris, la réduction et le flou sont faits par les fils de calcul, un lot de 32 images à la fois (§ 10).
Toutes les coordonnées ressortent en pixels d'origine : le centre d'un pixel réduit `c` vaut `(c + 0,5) × échelle − 0,5` (`types/src/scale.rs`).

## 2. Estimer le fond : la médiane temporelle

**Outil.**
Pour chaque pixel, la **médiane** de ses valeurs sur une seconde centrée sur l'image étudiée (`bg_window_s` 1,0), en prenant une image sur 3 (`bg_step` 3) : à 30 i/s, une demi-fenêtre de 15 images et 11 valeurs (`engine/src/window.rs`).
Près du début et de la fin de la vidéo, la fenêtre est tronquée.

**Pourquoi la médiane.**
C'est une statistique *robuste* : tant que l'animal occupe le pixel sur moins de la moitié des images, la médiane l'ignore complètement.
Une moyenne, elle, garderait une traînée de l'animal dans le fond.
Une chauve-souris en transit ne reste sur un pixel que quelques images sur 30 : elle disparaît du fond.

**Pourquoi une fenêtre centrée.** La fenêtre regarde une demi-seconde avant et une demi-seconde après.
Le traitement n'est pas en direct, donc on peut utiliser le futur : le fond suit ainsi les changements lents de lumière sans retard.

**Pourquoi pas autre chose.**

- *Différence entre deux images* : simple, mais voit chaque objet deux fois (où il était, où il est) et rate les objets lents.
- *Mélanges de gaussiennes* (MOG2, utilisé par ThruTracker) : la sortie est binaire.
  On perd le signe et l'amplitude de l'écart, dont le reste du traitement a besoin.
- *Fond fixe sur toute la vidéo* : ne suit ni la lumière ni le gain des jumelles.

**Limite.** Un objet qui reste plus d'une demi-seconde au même endroit entre dans son propre fond et s'efface (issue #22).

**Technique de calcul.** La médiane est le poste le plus cher de la détection.
Elle est calculée par un **réseau de tri** (`network.rs`) : une suite fixe de comparaisons et d'échanges, appliquée à des blocs de 64 pixels à la fois, élaguée pour ne fixer que la ou les deux valeurs du milieu (pour un nombre pair de valeurs, le fond est leur demi-somme).
Une fenêtre dont les images (toutes gardées, en entiers de 16 bits) dépasseraient 4 Gio est refusée avant la lecture.

## 3. Compenser le gain automatique

**Outil.**
On retire au résidu la différence entre la luminosité moyenne de l'image et celle des images du fond, calculée en entiers jusqu'à une seule division, hors des zones masquées par `--osd-region` (`engine/src/detector.rs`).

**Pourquoi.** Les jumelles ajustent leur gain : toute l'image s'éclaircit d'un coup. Sans correction, chaque pixel dépasserait le seuil.
Le modèle est le plus simple possible : un décalage identique pour tous les pixels.

**Limite.** Un changement de contraste, qui étire les valeurs au lieu de les décaler, n'est pas corrigé.

## 4. Décider pixel par pixel : le seuil

**Outil.** Le **résidu** est `image − fond − décalage de gain`, sur l'image filtrée.
Un pixel est retenu si la valeur absolue du résidu dépasse strictement **son propre seuil** : max(12, 8 × bruit du pixel) (`engine/src/threshold.rs`).

**Pourquoi la valeur absolue.** L'animal peut être plus sombre que le fond (devant la roche chaude) ou plus clair (devant un ciel froid).
Le signe n'est pas supposé.

**Pourquoi pas un seuil fixe.** Jusqu'au 6 octobre 2026, le seuil était fixe à 25 : sur la vidéo de référence, le bruit restait dessous.
Mais un seuil unique traite pareil la roche calme et la végétation qui bouge, et surtout il ne s'adapte pas d'une vidéo à l'autre.
Mesuré sur sept vidéos : un seuil fixe abaissé à 12, bon sur la 092, laisse des centaines de pistes de bruit sur la 089, la 090 et la 091, dont le bruit de fond dépasse 12 en quelques points de chaque image ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)).

**Le seuil par pixel** (`noise_factor`, 8 par défaut, 0 pour revenir au seuil fixe ; le plancher est `--threshold`, 12 par défaut).
*Outil* : pour chaque pixel, on mesure son bruit sur les images de la fenêtre du fond par la **MAD** (médiane des écarts absolus à la médiane, multipliée par 1,4826 pour valoir un écart-type sur un bruit gaussien) ; les écarts sont corrigés du gain de chaque image.
Le seuil devient max(plancher, k × bruit).
Il n'est calculé que si la fenêtre a au moins 7 échantillons, et seulement pour les pixels dont le résidu dépasse déjà les trois quarts du plancher et qui ne sont pas masqués : les autres ne peuvent pas être retenus.
*Choix* : la MAD plutôt que l'écart-type, parce qu'un passage d'animal sur une ou deux images de la fenêtre ne la fait presque pas monter ; la fenêtre plutôt qu'une moyenne qui s'accumule depuis le début, pour que chaque image ne dépende que de ses voisines ; le plancher, parce que la MAD sur 11 images est souvent très faible (mesuré avec la v0.1.1 sur l'extrait de test à 480 px : descendre le plancher de 4 à 1 niveau multiplie par 60 les pixels au-dessus de 6 fois le bruit).
*Limites* : 11 images estiment mal le bruit d'un pixel, si bien qu'une feuille agitée passe encore parfois.
Une cible faible qui passe sur des pixels agités peut être perdue : sur la 092 (mesuré avec la v0.1.1), le passage réel de 3:42.10 (pic de 13 à 22 après filtre) tombe sur deux de ses six images sous un seuil relevé à 22 à 36, et n'a plus assez de détections pour faire une piste.
Un saut de gain de 30 niveaux, qu'absorbait l'ancien seuil de 25, allume désormais toute l'image : le filtre des images saturées (§ 6) ignore quelques secondes autour du saut, sans créer de piste.

**Le seuil bas** (`WEAK_RATIO`, 0,75, constante de `engine/src/threshold.rs`).
Un pixel dont le résidu dépasse les trois quarts de son seuil (plancher 9 et 6 fois le bruit, par défaut) est retenu comme **faible** ; au-dessus du seuil entier, il est **fort**, exactement comme avant.
Les pixels faibles ne servent qu'aux taches faibles encadrées ([§ 5 ter](#5-ter-garder-les-taches-faibles-qui-comblent-un-vol)) : aucune détection forte ne change à la sortie de la détection.
*Pourquoi 0,75* : c'est la seule fraction mesurée qui gagne sans abîmer la 092.
À 0,6 et à 0,5, les détections faibles détournent le suivi, qui coupe des pistes en deux : la 092 passe de 26 à 36 et 24 positions estimées, avec 4 et 3 pistes nouvelles (mesuré le 17 octobre 2026, #21).
Le seuil haut reste celui d'aujourd'hui : le relever ferait perdre des détections fortes, donc des passages.

## 5. Regrouper les pixels en taches

**Outils.** Deux opérations classiques de morphologie mathématique.

- **Fermeture** (une dilatation puis une érosion, avec un disque de `merge_radius` 6 px d'origine, soit 4 px de travail à 960 sur 1440, arrondi au plus proche ; `engine/src/closing.rs`) : elle bouche les trous et recolle les fragments proches d'un même animal, sans grossir l'objet.
- **Composantes connexes** (`components.rs`) : chaque groupe de pixels qui se touchent, y compris par un coin, reçoit un numéro.
  On en tire la boîte englobante, le centre de gravité, l'aire et l'amplitude maximale.

**Choix.**
L'aire compte seulement les pixels réellement au-dessus du seuil, pas ceux ajoutés par la fermeture : la fermeture sert à regrouper, pas à mesurer.
Une tache est gardée entre `min_area` 4 et `max_area` 2 700 px² d'origine, bornes comprises.
À 960 px sur une source de 1440, un pixel réduit vaut 2,25 px² : il faut donc deux pixels au-dessus du seuil, et une tache d'un seul pixel est rejetée (à 480 px, un seul suffisait).
Plus haut, 9 ou 18 px² perdent les petites cibles faibles sans retirer de bruit (mesuré avec la v0.1.1).

**Deux passages** (`engine/src/detector.rs`).
Le premier ne prend que les pixels forts : fermeture et composantes donnent les détections fortes, identiques à l'octet à celles d'un seuil sans partie faible.
Le second, seulement s'il reste des pixels faibles, reprend tous les pixels et ne garde que les taches sans aucun pixel fort : ce sont les détections faibles.
*Pourquoi deux passages* : relier les pixels faibles aux taches fortes, en un seul passage, grossit et déplace les détections fortes.
Sur la 092, à 0,75, ce premier essai perdait 114 détections et ajoutait 8 pistes (#21).

## 5 bis. Écarter les images que le décodeur a réparées

**Le problème.**
Sur 17 des 22 vidéos des sessions du 2 septembre et du 2 octobre 2026, il manque quelques octets à la fin de certaines images.
Le décodeur bouche le trou par *dissimulation d'erreur* : il recopie un morceau d'image voisine, et l'image réparée ressemble à une vraie scène.
Une image P réparée sert de référence aux suivantes, donc le dégât dure jusqu'à l'image-clé suivante (8 à 16 images).
Sur la 122, la v0.1.1 en tirait des centaines de fausses pistes avant ce filtre ; une fois le filtre de stabilité en place, les deux tiers de la vidéo ignorés comme « jumelles qui bougent ».

**Outil.**
La lecture qui sert à la détection donne aussi, pour chaque image, les signalements d'erreur du décodeur (`media/src/decode_errors.rs`), son horodatage (`pts`) et si c'est une image-clé.
Une image est abîmée si le décodeur la signale, ou si elle suit un trou dans les `pts` : l'écart à l'image précédente, divisé par le pas habituel et arrondi, dépasse une image (`engine/src/lost.rs` ; 13 images jamais décodées sur la 122, sans aucun signalement).
Il manque des images avant la première ou après la dernière ? La plage part du début ou de la fin de la vidéo.
La plage écartée va de l'image abîmée à l'image qui précède la prochaine image-clé (`engine/src/damage.rs`).

**Pourquoi une marge.** Le fond d'une image est la médiane de ±½ s d'images (§ 2).
Une image saine à moins d'une demi-fenêtre d'une image réparée a donc un fond faux.
Mesuré sur la 122 avec la v0.1.1, sans marge : les 81 images qui déclenchaient encore le filtre de stabilité étaient toutes à 9 images au plus d'une plage abîmée, avant comme après, et les 4 pistes restantes finissaient toutes 1 à 6 images avant une plage.
Chaque plage est donc élargie de la demi-fenêtre du fond de chaque côté (15 images à 30 i/s), avant la fusion des plages, avant la stabilité et avant le suivi.

**Pourquoi les signalements du décodeur.**
Ils viennent de la lecture même qui sert à la détection : chaque signalement tombe sur la bonne image, sans deuxième passe sur le fichier et sans rien coûter de plus.
Le journal de FFmpeg, qui écrirait un message par image réparée sur la sortie d'erreur, est mis en silence une fois pour toutes (`media/src/init.rs`) : les images abîmées sont comptées et situées dans l'image résumé et `params.json`.
Le nombre d'images est contrôlé : les images décodées doivent être les images annoncées par le fichier, moins les pertes expliquées : un saut de `pts`, des images manquantes avant la première, ou des paquets illisibles à la fin.
Sinon la vidéo est comptée en échec (« la vidéo est abîmée ou incomplète »), et les autres vidéos du lot continuent (`engine/src/input.rs`).

**Ce que la sortie en dit.**
Le temps écarté est signalé à part des mouvements : des lignes « illisible 1:12 - 1:15 » dans le bandeau de l'image résumé (trois au plus, puis « + N autres, X illisibles en tout »), la frise du temps, et `damaged_s` dans `params.json`.
Ce temps **inclut la marge**.
`damaged_frames` compte les images signalées par le décodeur ou qui suivent un trou (196 sur la 122), pas les images écartées (6 043).

**Ce que ça coûte.**
Mesuré avec la v0.1.1 ; le Rust donne les mêmes plages sur 25 des 26 vidéos de la référence, et 43 images de plus sur la 124, que le décodeur signale par sa dissimulation ([11](11-migration-rust.md)).
Ce sont les chiffres du Mac : sous Linux (lumiel), le même décodeur signale 14 images de plus sur 8 vidéos, toujours les mêmes d'une lecture à l'autre.

| Vidéo | Images signalées | Illisible | Instable restant | Union | Pistes avant → après |
| --- | --- | --- | --- | --- | --- |
| 122 (195 erreurs) | 196 | 201,2 s en 79 plages | 0 s | 201,2 s sur 300 | 25 → 0 |
| 093 (11 erreurs) | 11 | 14,0 s en 8 plages | 2,1 s | 16,0 s | 4 → 3 |
| 092 (aucune) | 0 | 0 s | 0 s | 0 s | 14 → 14 |

Sur une vidéo très abîmée, la marge coûte cher : sans elle, la 122 gardait 77,8 s illisibles, mais aussi 50,3 s instables et 4 pistes collées aux plages.
Avec 15 images de marge, il reste environ 98 s analysées sur les 300 de la vidéo.

**Limites.**

- Une tranche très abîmée peut ne produire aucun signalement (vu sur une vidéo fabriquée en coupant 60 % d'une tranche, avec la v0.1.1) : l'image passe alors pour saine.
- Le dégât s'arrête à l'image-clé parce que les images-clés des jumelles sont des IDR, sans image B. Un autre appareil pourrait ne pas respecter cette hypothèse.
- Dans une plage abîmée, les pixels que le décodeur invente varient avec le nombre de fils du décodeur : une raison de plus pour écarter la plage entière.
- Le fond de l'image résumé écarte aussi les images abîmées.

## 5 ter. Garder les taches faibles qui comblent un vol

**Outil.**
Une **interpolation linéaire** entre deux détections fortes (`engine/src/bracketing.rs`), sur toutes les détections de la vidéo, après la détection et avant tout le reste.
Une détection faible est gardée si :

- une détection forte la précède et une autre la suit, chacune à 3 images au plus (`SUPPORT_FRAMES`) et à 60 px d'origine par image d'écart au plus (`SUPPORT_STEP`) ;
- ces deux appuis sont à 20 px l'un de l'autre au moins (`SUPPORT_MOVE`) : une source immobile n'est pas un vol ;
- elle tombe à 10 px au plus (`BRACKET_TOLERANCE`) de la position interpolée entre les deux, au prorata des images de chaque côté ;
- aucune détection forte de sa propre image n'est à 12 px ou moins (`BESIDE_STRONG`, entre centres) : ce serait un morceau de cette tache, pas un trou.

Les autres détections faibles sont jetées. Les détections fortes passent toutes, inchangées.

**Pourquoi encadrée.**
Le but est de remplacer les positions que le suivi invente pendant un trou par la vraie tache, quand elle n'a manqué le seuil que de peu.
Sur les 26 vidéos, les pistes rapides (150 px/s et plus) de la v0.2.0 ont 4 236 positions estimées pour 25 079 points (16,9 %) ; sur la 092, 26 pour 320.
Mesures du 17 octobre 2026 (#21) :

- *Encadrée, 10 px* : 4 236 → 3 358 positions estimées sur les 26 vidéos (−20,7 %), et sur la 092, 26 → 20 avec ses 14 pistes inchangées.
  Chaque piste rapide de la v0.2.0 se compare à la piste d'aujourd'hui qui partage le plus de détections avec elle.
  Un premier calcul la comparait à celle qui partageait le plus d'images, parfois la piste d'un autre animal au même moment, et annonçait 2 932 (corrigé à la fusion de #24).
- *Appuyée d'un seul côté* : sur la 092, 26 → 39 positions estimées et 4 pistes nouvelles ; les détections faibles au bout d'une piste l'entraînent ailleurs.
- *Encadrée, 20 px* : la 092 passe à 21, mais le passage de 3:50.87 se coupe en deux pistes.
- *Prolongée au-delà des appuis* (deux détections fortes du même côté) : seule variante qui retrouve la cible connue de 3:42.10, mais à 0,6 et 40 px, avec 9 pistes nouvelles sur la 092.
- *20 px de mouvement exigé* : ne change presque rien ; le bruit ajouté ne vient donc pas de sources immobiles.

**Les détections faibles comptent** dans le filtre des images saturées (§ 6), comme toutes les détections.
Sur les 26 vidéos, les images écartées changent sur 7 d'entre elles : 32 de plus et 26 de moins (098), sur 45 716 ; aucune piste perdue n'y tombe.

**Limites.**
Les détections faibles gardées sont nombreuses là où le bruit est fort : environ 237 000 sur les 26 vidéos, dont 156 000 sur la 103 (+30 % de détections), sans piste en plus.
57 % d'entre elles sont à 12-36 px d'une détection forte de leur image : des morceaux de la même tache plutôt que des trous comblés, que le critère entre centres laisse passer.
Les rejeter jusqu'à 36 px garderait le gain (3 381 positions estimées au lieu de 3 358), mais perdrait deux réunions jugées justes sur la 090 (mesuré le 17 octobre 2026, non retenu).
La cible connue de 3:42.10 sur la 092 n'a que 4 images détectées, et ses voisines tombent à 40 px de la prolongation : elle reste sans piste.
La mémoire et le temps ne bougent presque pas (092 au calme, `compare 092` sur lumiel le 17 octobre 2026, deux passages après la fusion de #24 : 450 Mo contre 441 à 442 sans #21, 29,4 à 29,8 s contre 29,2 à 29,4).

## 6. Écarter les images où les jumelles bougent

**Outil.** Un simple **comptage** (`tracking/src/stability.rs`).
Une image est suspecte si elle porte au moins `max_blobs` (20, `--max-blobs`) taches de plus que 6 fois le nombre médian de taches par image de la vidéo.
Les images abîmées (§ 5 bis) sont retirées du comptage et de la médiane.

**Pourquoi ça marche.** Quand le cadre glisse, les images de la fenêtre du fond ne sont plus alignées.
Chaque contour laisse un résidu, et une image passe de quelques taches à plusieurs centaines.
Le nombre de taches est donc un indicateur de mouvement, et il est gratuit : la mesure directe du décalage, par corrélation de phase, coûterait une trentaine de secondes par vidéo (mesuré avec la v0.1.1).

**Pourquoi une limite relative.** La médiane de la vidéo adapte la limite au niveau de bruit de chaque enregistrement.

**Choix.**
Les détections sont retirées `pad_s` (une seconde) avant et après chaque image suspecte, sans déborder des images lisibles ; deux périodes que sépare au plus une seconde sont réunies.
Les détections de ces périodes sont vidées (`tracking/src/exclusion.rs`). La période est toujours signalée.
L'outil préfère dire « je n'ai pas regardé ici » plutôt que produire des centaines de fausses pistes.
`--max-blobs 0` coupe ce filtre ; `pad_s` ne se règle que dans le banc.

**Limites.** Un essaim de 20 animaux dans une image serait pris pour un mouvement.
Les images abîmées au décodage produisaient le même symptôme ; elles sont retirées avant ce comptage (§ 5 bis) et ne passent plus pour un mouvement.

## 7. Relier les taches dans le temps : le suivi

**Outils.**
Une **prédiction à vitesse constante** et une **association gloutonne au plus proche voisin** (`tracking/src/track.rs`, `track/growing.rs`, `track/association.rs`).

- Pour chaque piste ouverte, on prédit où elle devrait être à l'image suivante en prolongeant sa vitesse.
- On calcule la distance entre chaque prédiction et chaque tache, on garde les paires à moins de `max_jump` (120 px), et on les attribue de la plus proche à la plus lointaine.
- Une tache sans piste ouvre une nouvelle piste. Une piste sans tache pendant plus de `max_gap` (6) images est close.

**Trois choix qui viennent des mesures.**

- *La vitesse se mesure sur 3 images au moins.*
  Le capteur tourne à 50 Hz et la vidéo à 30 images par seconde : le pas de l'animal suit un motif « double, simple, simple ».
  Une vitesse mesurée sur deux images se trompe d'un facteur 2 une fois sur trois.
  La vitesse se prend donc sur le point le plus récent qui a au moins 3 images de recul, sinon sur l'avant-dernier point.
- *Les pistes qui ont déjà une vitesse choisissent en premier.*
  Sinon un fragment né à l'instant vole la tache de la vraie piste, qui se retrouve coupée.
- *La porte est large (120 px)* parce qu'un animal rapide parcourt 60 px par image et peut manquer plusieurs images.

**Fortes d'abord** (`track/association.rs`, `track.rs`).
À chaque image, l'attribution se fait en deux tours : les détections fortes d'abord, exactement comme si les faibles n'existaient pas ; puis les pistes restées sans détection prennent les faibles, dans le même ordre.
Une détection faible restée libre n'ouvre jamais de piste.
*Pourquoi* : sans ces deux règles, les détections faibles mélangeaient des pistes.
L'utilisateur a jugé à l'œil les 15 pistes perdues de la première variante sur les 26 vidéos (17 octobre 2026, #21) : 6 réunions de deux chauves-souris différentes, par trois mécanismes mesurés.

- Une piste prend une détection faible plus proche de sa prédiction que la forte (099 à 3:33.15 : 12,6 px contre 21) ; la forte, restée seule, ouvre une piste parasite qui saute sur l'animal voisin.
- La forte laissée seule, ou une faible isolée, tombe à moins de 36 px de deux animaux qui volent près l'un de l'autre et fait pont dans la fusion des jumelles (§ 8) : 099 à 3:22.40 et 3:44.21.
- Une piste enchaîne, par une détection faible, sur la tache de l'animal suivant (095 à 4:46.68, 100 à 2:01.18).

Avec les deux règles, les 6 réunions disparaissent et les 3 réunions jugées justes restent (090 à 2:28.15 et 2:46.50, 099 à 3:21.66).
Sur les 26 vidéos : 965 pistes appariées à la v0.2.0 sur 972, dont 876 concordantes, 7 perdues et 37 ajoutées ; la 092 garde ses 14 pistes.
Des 7 perdues, 2 sont des réunions justes (la troisième, à la 099, n'en perd aucune), 4 restent une seule piste dont le début bouge de plus de 5 images (096 à 1:42, 098 à 2:27, 101 à 3:42, 125 à 1:11) et une disparaît (097 à 0:16, cause non isolée).
Le recollage du [§ 9 bis](#9-bis-recoller-un-vol-caché-derrière-la-végétation) passe après, et les deux effets ne s'additionnent pas : mesurés ensemble, 889 pistes appariées, 746 concordantes, 83 perdues et 33 ajoutées, contre 905, 844, 67 et 0 pour le recollage seul (une piste réunie à une autre compte pour perdue) ; la 092 garde ses 14 pistes (fusion de #24, 17 octobre 2026).

**La portée d'une piste** (`track/reach.rs`, `track/association.rs`, `track/growing.rs`).
Une piste qui a déjà une vitesse ne prend une tache que si celle-ci est à moins de **50 px plus le chemin qu'elle devait parcourir** de l'endroit prévu : sa vitesse multipliée par le nombre d'images depuis son dernier point.
Une piste qui avance de 10 px par image et a manqué 2 images peut prendre une tache à 50 + 30 = 80 px de l'endroit prévu.
La limite de 120 px reste au-dessus, et une piste d'un seul point, qui n'a pas encore de vitesse, n'est tenue que par elle.

- *Le problème.* Sans portée, une piste lente peut sauter jusqu'à 120 px sur une autre chauve-souris.
  À la 090 vers 2:09.87, une piste qui a perdu son animal 4 images plus tôt prend une tache à 109 px de l'endroit prévu, alors qu'elle avance de 12,6 px par image : 3 pistes pour 2 chauves-souris.
  À la 095 vers 4:46.68, une piste qui avance de 7,5 px par image prend, après 3 images sans tache, une tache à 116 px, sur une autre chauve-souris.
- *Pourquoi pas le ralentissement ni la direction.*
  Refuser un pas plus lent que le quart de la vitesse de la piste, comme le fait le recollage ([§ 9 bis](#9-bis-recoller-un-vol-caché-derrière-la-végétation)), change 312 pistes sur 1 002 à l'échelle d'une image, et ne voit aucun des deux cas, qui sont des sauts trop rapides.
  La direction ne sépare pas non plus : le saut fautif de la 095 ne tourne que de 39°, alors que 1 230 des 36 292 taches prises par une piste lancée tournent de plus de 90° ; refuser au-delà de 90° change 59 pistes et rate la 095.
- *Pourquoi 50 px et une fois le chemin.*
  Sur les 26 vidéos, avant recollage : une marge de 40 px change 74 pistes, 50 px en change 56, 60 px en change 39 mais laisse passer la 090 (109 px pour 110 permis) ; doubler le chemin laisse aussi passer la 090.
- *Le bord.*
  La portée ne joue pas quand le dernier point de la piste, la tache, ou le point d'où la vitesse est mesurée est à moins de 30 px d'un bord, la même marge que le recollage.
  Un animal qui entre a d'abord une tache coupée par le bord, dont le centre avance moins vite que lui : sa vitesse est sous-estimée, et la portée refusait de vrais débuts de vol (096 à 1:42, 095 à 4:27 et 4:36, 099 à 0:19, 100 à 3:09).
  Avec le dernier point et la tache épargnés au bord, 39 pistes changent au lieu de 56, et la 090 et la 095 restent corrigées.
  Le point d'où la vitesse est mesurée compte aussi : à la 125 vers 3:09.44, une chauve-souris entre à 6 px du bord droit, tache énorme et floue, et la portée la coupait en deux pistes à 31 px du bord, sur une vitesse prise depuis la tache coupée.
- *La vitesse effondrée.*
  Quand la vitesse de la piste (mesurée sur au moins 3 images) tombe sous 0,4 fois sa vitesse sur ses 5 derniers points, la portée prend cette seconde vitesse.
  Une tache qui se dédouble ou un point à contretemps peut faire stagner la piste une image : à la 095 vers 4:04.85, une piste de 3 points tombe ainsi à 0,6 px par image, contre 16 sur ses points, et refusait la suite de sa chauve-souris ; à la 102 vers 1:27.78, une piste tombe à 3,1 px par image contre 14, et perdait son dernier point.
  Le seuil donne le même résultat de 0,25 à 0,6 ; à 0,2, la 102 reste coupée, à 0,75, la 090 vers 2:28.88 change, à tort.
  Mesurée sur toute la piste plutôt que sur ses 5 derniers points, la même règle change aussi ce cas de la 090.
- *Les cas regardés à l'œil.*
  L'utilisateur a jugé sur une planche les 32 endroits où la portée changeait les pistes des 26 vidéos (18 octobre 2026) : 21 mieux, 5 pires, les autres sans effet visible ou avec un défaut présent dans les deux versions.
  Le bord du point de départ de la vitesse et la vitesse effondrée corrigent 3 des 5 pires et rien d'autre : ils reviennent exactement au suivi sans portée, jugé juste.
- *Ce que ça donne.* Sur les 26 vidéos, avant recollage, 1 002 pistes, autant que sans portée ; après recollage, 921 au lieu de 922.
  La 092 garde ses 14 pistes, avec les mêmes débuts et les mêmes fins ; deux points y changent de tache, de 1 à 4 px (0:35.20 et 1:49.82).
- *Ce qu'elle ne corrige pas.*
  À la 100 vers 2:00 à 2:02, une piste naît sur un éclat de la tache d'une autre, sans vitesse encore, saute à 110 px en 7 images sur une troisième chauve-souris, au rythme d'un vol normal, puis lui prend sa tache pour 0,4 px d'écart (12,2 contre 12,6 px de l'endroit prévu) : 3 pistes pour 2 chauves-souris, qu'aucune règle de vitesse ne voit.
  À la 090 vers 2:12.07, deux pistes suivent une même chauve-souris, grosse et rapide (30 à 50 px par image), et ne se réunissent pas en jumelles.
  À la 098 vers 1:06.70, la portée coupe 2 points qui reviennent en arrière au bout d'un morceau de vol d'une autre chauve-souris, jusque-là jeté ; le morceau devient une piste, que le recollage soude à tort à la piste d'avant.

**Pourquoi pas plus savant.** Un filtre de Kalman ou une affectation optimale (algorithme hongrois) sont les outils habituels.
Avec quelques taches par image, le glouton donne le même résultat, et il reste court.

**Limites.**
Une interruption de plus de 6 images coupe le vol en deux pistes ; le recollage les réunit quand le trou ne dure pas plus de 19 images ([§ 9 bis](#9-bis-recoller-un-vol-caché-derrière-la-végétation)).
La piste commence quand la tache dépasse le seuil, donc après l'arrivée réelle de l'animal (issue #8).
`max_jump` et `max_gap` ne se règlent que dans le banc.
Les deux réglages de la portée, `reach_margin` (50 px) et `reach_travel` (1), sont écrits dans `params.json` et réglables dans le banc ([08](08-banc-de-mesure.md)) ; le bord (30 px), le seuil de la vitesse effondrée (0,4) et ses 5 points sont des constantes.

## 8. Fusionner les pistes jumelles

**Outil.** Une structure **union-find**, qui regroupe des éléments par paires jusqu'à former des familles (`tracking/src/track/twins.rs`).

**Règle.**
Deux pistes qui coexistent, partagent au moins une image et restent à moins de `twin_distance` (36 px) l'une de l'autre sur *toutes* leurs images communes sont un seul animal vu en deux morceaux.
Elles sont fusionnées : boîte englobante commune, aires additionnées, centre pondéré par l'aire, amplitude la plus forte ; le point réuni est fort si l'un des deux l'est.

**Pourquoi « toutes ».** Deux vrais animaux qui volent ensemble s'écartent au moins une fois ; deux fragments d'un même animal, jamais.
Cette règle n'a été vérifiée que sur des trajectoires fabriquées (issue #26).

**Limite.**
La règle se juge paire par paire, et les familles se forment de proche en proche : une piste d'un seul point, à moins de 36 px de deux animaux proches sur sa seule image, suffit à les réunir (mesuré sur la 099, #21).
« Fortes d'abord » (§ 7) retire les ponts faits de détections faibles.
Exiger que toutes les pistes d'une famille soient jumelles deux à deux supprimait aussi les autres ponts, mais ajoutait 3 pistes sur la 092 : cette règle n'a pas été retenue.

## 9. Garder ce qui ressemble à un vol

Trois filtres, du plus simple au plus fin (`tracking/src/track.rs`, `types/src/track.rs`) :

- **Au moins `min_hits` détections** (6, `--min-hits`) : élimine les étincelles.
- **Au moins `min_travel` (45 px) entre le début et la fin** : élimine un point qui scintille sur place.
- **Virage médian sous `max_median_turn` (0,8 radian)** : le virage est l'angle entre deux pas successifs, les pas nuls mis à part.
  Un vol va quelque part, donc ses virages sont petits ; du bruit enchaîné par le suivi part dans tous les sens, avec un virage médian de 1,5 radian.
  La médiane tolère un demi-tour isolé, ce qu'une moyenne ne ferait pas.

Les pistes gardées passent ensuite au recollage ([§ 9 bis](#9-bis-recoller-un-vol-caché-derrière-la-végétation)), avec les morceaux de 2 à 5 détections que `min_hits` a écartés, mis de côté pour servir de pont ; puis elles sont numérotées dans l'ordre de leur première image.

**Limite.** Un vol très lent avance de quelques pixels par image, et l'arrondi des positions fait tourner sa direction mesurée.

## 9 bis. Recoller un vol caché derrière la végétation

**Le problème.**
Sur les vidéos 095 à 100, prises du même endroit, une chauve-souris passe souvent derrière une touffe de végétation accrochée à la falaise, puis réapparaît de l'autre côté.
Pendant ce temps, elle n'est plus dans l'image : le suivi tient 6 images sans la voir (`max_gap`), puis ferme la piste, et la suite du vol ouvre une seconde piste.
Un animal compte alors pour deux, et c'est le nombre de pistes que la naturaliste lit en premier.

**Ce qui a été mesuré** (référence v0.2.0, 26 vidéos, 17 octobre 2026, sans décoder sauf pour regarder les trous).

- *Où la cible disparaît.*
  Sur le cadrage de la 095 à la 100, les images manquantes dans les pistes se concentrent dans deux zones : en bas, vers x 920 à 1080 et y 720 à 800 (pixels de la vidéo d'origine), il manque jusqu'à 81 % des images des pistes qui y passent, contre 2 à 10 % autour ; en haut, vers x 920 à 1040 et y 400 à 520, jusqu'à 70 %.
  Même les pistes qui traversent ces zones sans se couper y perdent leur cible plus de 4 images sur 10.
  Ce sont deux zones de végétation, sombres sur l'image.
- *Ce que voit la détection pendant le trou.*
  À l'endroit où la chauve-souris devrait être, l'écart au fond tombe au niveau du bruit (0 à 13 niveaux, contre 16 à 41 juste avant), et le seuil reste à son plancher de 12 : ce n'est pas la végétation agitée qui relève le seuil, et ce n'est pas une cible juste trop faible.
  La végétation est froide, donc sombre ; une chauve-souris chaude devant elle ressortirait. Elle passe derrière.
  Baisser le seuil n'y changerait rien : il n'y a rien à détecter.
- *Combien de temps elle reste cachée.*
  Une chauve-souris vole à 12 à 13 px par image près de ces zones, et traverse chacune en 8 images en médiane, 12 à 18 images pour les plus lentes (une sur dix).

**La règle des deux bouts** (`tracking/src/track/rejoin.rs`). Deux pistes sont un même vol si :

- la seconde commence après la fin de la première, après un trou de **plus de 6 images** (sinon le suivi l'aurait reliée lui-même, et s'il ne l'a pas fait, c'est qu'il avait une raison) et de **19 images au plus**, soit 8 à 20 images entre le dernier point de l'une et le premier de l'autre, à peine plus d'une demi-seconde ;
- **les deux bouts s'accordent** : on mesure la vitesse de la fin de la première piste et celle du début de la seconde (sur leurs 5 points les plus proches du trou), on prolonge la fin de la première sur toute la durée du trou à la moyenne de ces deux vitesses, et la seconde piste doit démarrer à **60 px au plus** de ce point.
  C'est comme si l'animal virait au milieu du trou : prendre les deux bouts, plutôt que la première piste seule, suit une chauve-souris qui tourne pendant qu'elle est cachée ;
- aucune image du trou n'appartient à une **période ignorée** (jumelles qui bougent, image abîmée) : on ne sait pas ce qui s'y est passé ;
- ni la fin de la première ni le début de la seconde ne sont **à moins de 30 px d'un bord** de l'image : une chauve-souris qui sort du cadre et y revient n'est jamais recollée, rien ne dit que c'est la même.
  Les fins de piste s'entassent près des bords (188 à moins de 5 px, puis 81, 49, 26 par tranche de 5 px) et retombent au niveau du reste vers 30 px.

Quand une fin peut se prolonger par plusieurs débuts, ou un début continuer plusieurs fins, les paires sont prises de la plus proche à la plus lointaine, chaque fin et chaque début une seule fois.
Un vol caché deux fois devient une seule piste.
Aucun point n'est inventé dans le trou : l'image résumé et les extraits le comblent en ligne droite, comme les trous plus courts.

**Ce qui n'est pas recollé.**
Une disparition de plusieurs secondes : sur la 096, une chauve-souris entre dans la végétation haute et une autre piste en sort 5,9 s plus tard ; l'utilisateur a jugé que ce n'était pas le même passage.
À cette échelle, ni la vitesse ni la direction ne disent plus rien, et sur la 095 à la 100, 124 fins de piste sur 709 ont une autre piste qui démarre 1 à 6 s plus tard à moins de 130 px, contre 92 et 83 si l'on décale les pistes de 10 ou 20 s : deux recollages sur trois souderaient deux animaux.
Pas davantage une sortie du cadre suivie d'un retour.

**Ce que ça donne.**

- *Sur les 26 vidéos* : 67 soudures, 972 pistes ramenées à 905, toutes sur 11 vidéos (093 : 2, 094 : 1, 095 : 11, 096 : 13, 097 : 15, 098 : 8, 099 : 7, 100 : 5, 101 : 1, 102 : 1, 125 : 3).
  70 paires remplissaient les conditions : 2 fins et 3 débuts en avaient deux, et l'ordre du plus proche a tranché.
  16 autres paires ont été écartées parce qu'un bout touchait le bord.
  L'allure (plus bas) en refuse 2 de plus, sur la 095 et la 097 : 65 soudures, 907 pistes. La 092 garde ses 14 pistes.
- *Avec les détections faibles* ([§ 5 ter](#5-ter-garder-les-taches-faibles-qui-comblent-un-vol)) : 80 soudures au lieu de 67, sur les mêmes 11 vidéos (093 : 3, 094 : 1, 095 : 14, 096 : 16, 097 : 18, 098 : 9, 099 : 7, 100 : 5, 101 : 1, 102 : 3, 125 : 3), qui ramènent 1 002 pistes à 922.
  Des 67 soudures d'avant, 65 se refont. À la 097 vers 2:42, deux bouts se recollent à une autre suite que sans détections faibles.
  À la 098 vers 2:27, le bout du milieu manque (§ 7) et les deux autres ne se recollent plus.
- *Avec l'allure, les détections faibles et la portée du [§ 7](#7-relier-les-taches-dans-le-temps--le-suivi)* : 81 soudures (093 : 3, 094 : 1, 095 : 13, 096 : 17, 097 : 17, 098 : 10, 099 : 7, 100 : 5, 101 : 1, 102 : 3, 125 : 3, 129 : 1), qui ramènent 1 002 pistes à 921.
  Sans la portée, 80 : la soudure de plus est celle de la 098 vers 1:06.70, fausse (§ 7).
- *Fausses soudures au hasard* : les mêmes règles appliquées à des pistes décalées de 10 ou 20 s, qui ne peuvent pas être le même animal, en soudent 2,7 en moyenne.
  Sur le banc de chasse, très dense (720 cibles synthétiques en 5 minutes), une soudure sur 15 a collé deux cibles différentes qui se suivaient au même endroit.
- *Les cas regardés à l'œil* : sur une planche de 10 paires, l'utilisateur en a jugé 8 « même passage » et 2 « non » (le trou de 5,9 s, une sortie du cadre) ; la règle recolle les 8 et aucun des 2.
  Le plus grand écart des 8 au point de rencontre est de 45 px, le plus long trou de 13 images.
- *Le zoom* : une piste recollée garde son extrait zoomé si l'un de ses bouts, seul, y avait droit (plus grande tache sous 100 px² ou écart au fond sous 40, [02](02-methode.md#extrait-zoomé-et-ralenti)).
  Sans cela, la piste recollée prendrait la plus grande tache et le plus fort écart de ses bouts, et le bout net ferait perdre son zoom au bout faible : 10 pistes sur les 26 vidéos l'auraient perdu.
  Une piste qui n'est pas recollée suit la règle de toujours.
- *Pourquoi 60 px et 19 images* : voir [02 § 7 bis](02-methode.md#7-bis-recollage).
  Plus long, le prolongement ne vaut plus grand-chose : sur des vols continus, prolonger une piste de 20 images manque déjà sa vraie position de 114 px une fois sur deux, de 216 px à 30 images.

**Pourquoi pas autre chose.**

- *Prolonger la première piste seule, dans un cône qui s'élargit avec le temps.*
  Calibré sur des trous creusés dans des pistes continues (un même animal, à coup sûr), le cône part de 37° à 8 images et s'ouvre de 2,7° par image ; à trou égal (8 à 20 images, sans la règle du bord), il soude 91 paires pour 7,7 au hasard, contre 83 pour 3,3 avec les deux bouts : un virage pendant le trou l'oblige à s'ouvrir.
- *Baisser le seuil dans les zones de végétation* : il n'y a rien sous le seuil.
- *Une affectation optimale* (chercher la meilleure combinaison de toutes les paires à la fois, plutôt que de les prendre de la plus proche à la plus lointaine) : elle ne changerait le résultat que là où une fin ou un début a plusieurs candidats, 5 fois sur 70 ; l'ordre du plus proche y suffit tant qu'aucun cas jugé ne le contredit.

**Les morceaux trop courts pour faire une piste.**
Une chauve-souris peut se cacher deux fois de suite : derrière une touffe, puis derrière la suivante.
Entre les deux, on ne la voit que 2 à 5 images.
Ce morceau a trop peu de points pour être gardé (il en faut 6, § 9), et sans lui les deux pistes de part et d'autre sont trop loin l'une de l'autre dans le temps pour être recollées.
Le même morceau, juste avant le début d'une piste ou juste après sa fin, est souvent le vrai début ou la vraie fin du vol, quand la cible était encore faible.

- *La pile à part.* Les morceaux d'au moins 2 détections que le nombre de points a écartés sont mis de côté.
  Ils ne servent qu'à **relier deux pistes gardées** ou à **prolonger une piste gardée**, par la règle des deux bouts ci-dessus : trou de 7 à 19 images, départ à 60 px au plus du point prévu, jamais au bord ni à travers une période ignorée.
  Deux morceaux ne se relient jamais entre eux, et un morceau qui ne touche aucune piste disparaît : il ne devient jamais une piste.
  Les liens entre deux pistes gardées sont pris d'abord, de la plus proche à la plus lointaine, puis ceux des morceaux sur les bouts restés libres : un morceau ne peut qu'ajouter un lien, jamais défaire une soudure entre deux pistes.
  Les points d'un morceau font partie de la piste, mais ne comptent pas pour le zoom, que seules les pistes gardées décident : sinon, un morceau net versé au bout d'une piste faible lui ferait perdre son zoom, ce qui arriverait à 6 pistes sur les 26 vidéos.
- *L'allure.* La chauve-souris ne ralentit pas d'un coup derrière un buisson.
  On compare deux vitesses à celle de la piste gardée à son bout (ses 5 points les plus proches du trou) : celle du saut par-dessus le trou (distance entre le bout de la piste et le morceau, divisée par les images qui les séparent), qui doit en valoir au moins le quart (0,25), et celle du morceau lui-même, de son premier à son dernier point, qui doit en valoir au moins 0,45 ; un morceau d'un seul point n'aurait que le saut.
  Entre deux pistes gardées, la règle vaut aussi : le saut doit valoir au moins le quart de la vitesse de chacune des deux.
  La direction ne compte pas : la porte de 60 px autour du point prévu la borne déjà.
- *Pourquoi ces seuils.*
  Sur une planche de 10 morceaux qui reliaient ou prolongeaient une piste, l'utilisateur en a jugé 8 « même animal » et 2 douteux.
  Les douteux vont trop lentement : un morceau presque immobile là où la piste avait disparu (ses deux sauts à 0,02 et 0,10 fois la vitesse des pistes, le morceau à 0,40), un autre qui se traîne (morceau à 0,29).
  Chez les 8 bons, le saut va au moins à 0,33 fois la vitesse de la piste et le morceau au moins à 0,49 fois.
  La direction, elle, ne sépare rien : certains bons cas s'écartent de 55° à 151° de la direction de leur piste, plus que les douteux.
  Les seuils se placent entre les deux groupes ; l'écart est mince, sur 10 cas.
- *Pourquoi pas un seul point.*
  Admettre les morceaux d'une seule détection ajouterait 35 prolongements, sur une seule image chacun, et avancerait de 8 images le début de la piste 5 de la 092 (1:12.60), une chauve-souris confirmée, sur la foi d'un seul point.
- *Pourquoi aussi entre deux pistes gardées.*
  L'allure refuse 2 des 67 soudures de la règle des deux bouts : sur la 095 (4:59.33) et la 097 (2:17.33), la seconde piste repart presque là où la première s'est arrêtée, une demi-seconde plus tard, alors que l'animal volait à 6 à 9 px par image ; le saut ne va qu'à 1,6 et 0,5 px par image.
  Regardées à l'œil, rien ne prouve que ce soit la même chauve-souris : dans le doute, on sépare.
  Les 8 paires jugées « même passage » sur la première planche passent (saut à 0,62 fois la vitesse au moins), et les 2 jugées « non » auraient été refusées aussi par l'allure.

**Ce que donnent les morceaux et l'allure.** Sur les 26 vidéos, 9 363 morceaux de moins de 6 détections, dont 1 562 d'au moins 2.
Avec l'allure partout : 65 soudures directes, 3 ponts (deux sur la 096, un sur la 129) et 79 morceaux qui prolongent une piste ; 972 pistes ramenées à 904.
Sans l'allure, il y aurait 4 ponts et 88 prolongements : les deux douteux de la planche en faisaient partie. La 092 ne bouge pas.
Sur le banc de chasse, des cibles commencent plus tôt et se coupent moins, et la cible que la règle des deux bouts soudait à une autre qui la précédait au même endroit est retrouvée : 455 cibles trouvées, comme sans recollage.

**Limites.** Un recollage ne se voit pas comme tel dans les résultats : la piste recollée a simplement un trou.
Une soudure fausse reste possible quand deux chauves-souris se suivent de près au même endroit.
Une seule détection entre deux buissons reste perdue.
Les réglages `rejoin_hidden` (19), `rejoin_distance` (60 px), `rejoin_piece_points` (2), `rejoin_junction_pace` (0,25) et `rejoin_piece_pace` (0,45) sont écrits dans `params.json` et réglables dans le banc ([08](08-banc-de-mesure.md)) ; leur justification est dans [02](02-methode.md).

## 10. Aller vite sans changer le résultat

- **Le décodage a ses propres fils** : FFmpeg décode plusieurs images à la fois (`media/src/decode.rs`), et un fil de lecture garde 36 images d'avance (`engine/src/read_ahead.rs`).
- **Le reste se partage entre tous les cœurs** : les fils de calcul (`rayon`) préparent (gris, réduction, flou) puis détectent un lot de 32 images à la fois (`engine/src/detector.rs`).
  Chaque image ne dépend que de sa fenêtre, et tout le calcul est en entiers jusqu'au seuil : le résultat est identique au bit près avec 1, 3 ou 4 fils et des lots de 1 à 64 images, ce qu'un test vérifie.
- **Taches mesurées en une passe** : l'aire et le pic de toutes les taches d'une image sont comptés en un seul parcours des pixels au-dessus du seuil, au lieu d'un masque de l'image par tache.
  À 1440 px, où le bruit fait des centaines de taches par image, c'est ce qui garde la détection rapide.
- **Le rendu ne relit pas tout** : pour chaque groupe d'extraits, il se place par horodatage sur une image-clé saine avant le groupe (`render/src/source.rs`).

Les temps de chaque étape, et l'usage des cœurs, sont dans [09](09-profilage.md).

## 11. Savoir si c'est juste

Sans vérité terrain, tout réglage est un pari. Deux outils y répondent.

- **Le banc de cibles synthétiques** ([08](08-banc-de-mesure.md), crate `bench`) incruste de fausses chauves-souris dans une vraie vidéo : taches gaussiennes de taille et de contraste connus, sur des courbes de Bézier ou des vols de chasse simulés, avec la saccade des jumelles.
  On compte ce que l'outil retrouve, avec un **intervalle de Wilson** qui dit si un écart entre deux réglages est réel ou dû au hasard.
- **La vidéo de référence** : 14 pistes sur la 092, vérifiées par un test automatique (`cli/tests/reference_092.rs`).
  Selon le verdict de la naturaliste du 9 octobre 2026, 13 sont des chauves-souris ; la piste lente de 3:29 n'en est pas une ([03](03-resultats-092.md#verdict-de-manon-9-octobre-2026)).

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
