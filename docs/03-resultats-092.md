# Résultats sur video_092

La référence est désormais l'original copié en USB (`video_092_original.mp4`, 14
pistes, voir « Original contre export »). Le tableau ci-dessous et l'historique des
réglages ont été mesurés sur l'export de Stream Vision 2, avant qu'on dispose de
l'original.

## Export de Stream Vision 2 (historique, 12 pistes)

Vidéo du 2 septembre 2026, 20:37, falaise, jumelles posées, 5 min. Réglages par
défaut, 12 pistes.

| # | Début → fin | Durée | Vitesse (px/s) | Statut |
| --- | --- | --- | --- | --- |
| 1 | 0:07.53 → 0:07.97 | 0,43 s | 854 | à revoir (courte, faible amplitude : 49) |
| 2 | 0:34.27 → 0:35.07 | 0,80 s | 834 | à revoir |
| 3 | 0:53.80 → 0:54.70 | 0,90 s | 938 | à revoir |
| 4 | 1:12.70 → 1:13.47 | 0,77 s | 1149 | à revoir |
| 5 | 1:49.23 → 1:49.97 | 0,73 s | 1210 | à revoir |
| 6 | 2:02.93 → 2:03.30 | 0,37 s | 1093 | à revoir |
| 7 | 2:24.70 → 2:25.03 | 0,33 s | 1420 | à revoir |
| 8 | 2:44.00 → 2:45.27 | 1,27 s | 585 | à revoir (fragmentée, fusionnée) |
| 9 | 3:29.63 → 3:39.03 | 9,40 s | 95 | **contient le 3:36 confirmé par Manon**, nature à éclaircir |
| 10 | 3:48.40 → 3:48.70 | 0,30 s | 1882 | à revoir |
| 11 | 3:51.10 → 3:51.53 | 0,43 s | 1609 | à revoir (fragmentée) |
| 12 | 3:57.40 → 3:58.00 | 0,60 s | 1350 | **confirmée par Manon (3:58)** |

Les passages rapides traversent le champ en diagonale, du haut à droite vers le bas à
gauche pour la plupart, en 0,3 à 1,3 s.

## Original contre export

Même vidéo, mêmes réglages par défaut, original copié en USB (12,1 Mb/s) contre export
de Stream Vision 2 (1,04 Mb/s). Les temps concordent à 0,1 s près.

| Passage | Export : points / images (complétude) | Original | Remarque |
| --- | --- | --- | --- |
| 0:07 | 6 / 14 (43 %) | 10 / 17 (59 %) | |
| 0:34 | 19 / 25 (76 %) | 30 / 35 (86 %) | commence 0,33 s plus tôt |
| **0:51.65** | — | 6 / 8 | **nouveau** : lent (257 px/s), petit (18 px), faible (amplitude 40), au bord droit dans la végétation. Douteux |
| 0:53 | 19 / 28 (68 %) | 24 / 29 (83 %) | |
| 1:12 | 24 / 24 | 29 / 30 (97 %) | finit 0,16 s plus tard |
| 1:49 | 20 / 23 (87 %) | 23 / 24 (96 %) | |
| 2:02 | 9 / 12 | 10 / 15 | |
| 2:24 | 8 / 11 | 10 / 12 | |
| **2:37.16** | — | 5 / 9 | **nouveau** : rapide (1 265 px/s), devant la végétation en haut à gauche. Candidat sérieux |
| 2:44 | 30 / 39 (77 %) | 38 / 40 (95 %) | |
| 3:29 (lente) | 283 / 283 | 285 / 285 | |
| 3:48 | 8 / 10 | 14 / 15 (93 %) | |
| 3:51 | 13 / 14 | 16 / 16 (100 %) | |
| 3:57 | 14 / 19 (74 %) | 18 / 22 (82 %) | commence 2 images plus tôt, mais toujours 4 images après l'arrivée vue à l'œil |

Sur les 11 passages rapides communs, on passe de 170 à 222 points détectés (+31 %),
et 2 passages nouveaux apparaissent. Le calcul prend 4 min 29 au lieu de 2 min 40, à
cause du décodage d'un fichier 12 fois plus lourd.

## Original avec `min_area` = 4 (réglage par défaut depuis le 4 octobre 2026)

Mêmes 14 passages, presque tous plus complets ; plusieurs commencent plus tôt.

| Passage | `min_area` 18 | `min_area` 4 |
| --- | --- | --- |
| 0:07 | 0:07.43, 10 points | 0:07.39, 13 points |
| 0:34 | 30 | 33 |
| 0:51 (ignorable, 4 octobre) | 0:51.65, 6 | 0:51.62, 9 |
| 0:53 | 24 | 28 |
| 1:12 | 29, fin 1:13.63 | 31, fin 1:13.87 |
| 1:49 | 23 | 23 |
| 2:02 | 10 | 12 |
| 2:24 | 10 | 11 |
| 2:37 (valide, 4 octobre) | 2:37.16, 5 | 2:36.99, 7 |
| 2:44 | 38 | 40 |
| 3:29 (lente) | 285 | 287 |
| 3:48 | 14 | 14 |
| 3:51 | 16 | 17 |
| 3:57 | 18 | 18 |

Statuts donnés par l'utilisateur le 4 octobre : 2:37 est un vrai passage, 0:51 peut
être ignoré. 2:37 n'a que 7 détections : `min_hits` ne doit pas dépasser 7.

Calcul : 3 min 46 avec la détection parallèle sur 10 processus (rendu des vidéos compris).

## Après les gains rapides (4 octobre 2026)

`min_area` 4, sans masque, vitesse sur 3 images : toujours 14 passages. Les 5 pistes qui
s'arrêtaient contre l'ancienne bande du bas vont plus loin (0:07 jusqu'à 8,09 s, 0:34
jusqu'à 35,27 s, 0:53 jusqu'à 54,88 s, 2:44 jusqu'à 45,38 s, 3:51 jusqu'à 51,69 s), et 3:48
gagne 2 images (fin à 48,86 s). Entre 0 et 7 images interpolées par piste à l'affichage.
Calcul : 3 min 35.

## La piste 9

Une tache sombre qui avance lentement et sans à-coups pendant 9,4 s, de la gauche vers
le centre, puis remonte la paroi : 95 px/s contre 600 à 1900 pour les autres. Ce n'est
pas un vol. Rien d'autre ne bouge à 3:36. Hypothèses non tranchées : chauve-souris qui
rampe sur la roche, autre animal, ou passage vu par Manon que le détecteur ne capte pas.
À montrer à Manon (`split/09_3m29s63.mp4`).

## Historique des réglages sur cette vidéo

| Étape | Pistes | Changement |
| --- | --- | --- |
| Prototype | 13 | 2:44 en deux pistes |
| Appariement au plus proche | 14 | 3:58 coupé en deux par un fragment |
| Priorité aux pistes avec vitesse | 14 | 3:58 réparé, 3:51 coupé en deux |
| `merge_radius` = 2 | 13 | 3:51 réparé |
| `twin_distance` = 12 | 12 | 2:44 réparé |
| Borne de `max_gap` corrigée (6 images manquantes tolérées) | 12 | identique |

## Médiane par tri par comparaisons (4 octobre 2026)

Les 14 pistes sont identiques au bit près à celles de `np.median` (égalité des pistes
complètes). Détection seule, selon le nombre de processus (`--workers`) :

| Processus | 1 | 2 | 3 | 4 | 6 | 10 (ancien défaut) |
| --- | --- | --- | --- | --- | --- | --- |
| Temps | 31,0 s | 23,7 s | 24,7 s | 27,3 s | 31,6 s | 47,3 s |
| CPU | 114 s | 150 s | 184 s | 216 s | 282 s | 415 s |

Avant ce changement, avec 10 processus : 66,6 s et 534 s de CPU. Le décodage domine
désormais : chaque tranche relit la vidéo depuis le début ([09](09-profilage.md)).

Avec la fenêtre en uint8 et le fil lecteur, pistes toujours identiques : 15,1 s avec
1 processus, 42,3 s avec 10 ([09](09-profilage.md)).

## Images saturées ignorées (4 octobre 2026)

Aucune image de la 092 n'atteint la limite (au plus 4 taches par image) : rien n'est
ignoré, les 14 pistes et le banc sont identiques ([02](02-methode.md#4-bis-images-saturées-de-taches)).

## `min_hits` 6 et filtre de virage (4 octobre 2026)

Mêmes 14 pistes, mêmes débuts et mêmes nombres de détections. La plus courte, 2:37
(7 détections), est valide ; la plus sinueuse a un virage médian de 0,35 rad pour un
seuil de 0,8 ([02](02-methode.md#7-filtres-finaux)).
