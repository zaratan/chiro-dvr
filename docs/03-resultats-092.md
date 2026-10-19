# Résultats sur video_092

La 092 est la vidéo de référence du projet : 5 minutes de falaise filmées jumelles posées.
On en garde l'original copié en USB (`video_092_original.mp4`).
Ce document tient ses pistes et les cibles que l'outil rate encore.
Il tient aussi le verdict de Manon, qui en fait la vérité terrain du test `slow`.
Les temps viennent des `pts` de la vidéo.
Dans les sections datées, ceux de la v0.1.1 tombent de 0 à 2 centièmes plus tard : elle divisait le numéro d'image par la cadence moyenne.

## Les 14 pistes (17 octobre 2026)

Les pistes viennent des réglages par défaut ([02](02-methode.md)) avec les détections faibles encadrées (#21, [10](10-logique-de-detection.md#5-ter-garder-les-taches-faibles-qui-comblent-un-vol)).
La colonne v0.2.0 donne les détections de la référence des tests (12 octobre 2026).

| # | Début → fin | Détections | v0.2.0 |
| --- | --- | --- | --- |
| 1 | 0:07.39 → 0:08.13 | 23 | 20 |
| 2 | 0:33.93 → 0:35.26 | 40 | 40 |
| 3 | 0:51.62 → 0:51.88 | 7 | 7 |
| 4 | 0:53.75 → 0:54.88 | 35 | 34 |
| 5 | 1:12.59 → 1:13.86 | 36 | 35 |
| 6 | 1:48.92 → 1:49.99 | 32 | 32 |
| 7 | 2:02.81 → 2:03.48 | 18 | 17 |
| 8 | 2:24.62 → 2:25.12 | 13 | 13 |
| 9 | 2:36.98 → 2:37.44 | 10 | 10 |
| 10 | 2:43.94 → 2:45.37 | 44 | 44 |
| 11 | 3:29.59 → 3:39.21 | 290 | 290 |
| 12 | 3:48.27 → 3:48.84 | 17 | 17 |
| 13 | 3:50.87 → 3:51.67 | 24 | 24 |
| 14 | 3:57.16 → 3:58.03 | 27 | 27 |

Les 14 pistes ont le même début et la même fin que celles de la v0.2.0.
Toutes s'y apparient (début à ±5 images, points à ±3).
Quatre pistes gagnent des détections. 0:07.39 en gagne 3. 0:53.75, 1:12.59 et 2:02.81 en gagnent une chacune.
Ce sont 6 positions que le suivi estimait pendant un trou et que la vraie tache remplace.
Les pistes rapides passent ainsi de 26 à 20 positions estimées.
Par rapport à la v0.2.0, la détection ajoute 20 détections faibles sur 19 images et n'en perd aucune.
Les pistes en prennent ces 6. Les 14 autres restent hors piste.
La cible de 3:42.10 reste manquée ([cibles connues](#cibles-connues-pour-les-issues-suivantes)).

Le réglage par défaut du 6 octobre 2026 avait gardé les 14 pistes du réglage précédent (480 px, seuil fixe 25).
Cinq commençaient plus tôt, de 1 à 9 images : 9 images à 1:49 et 4 à 3:57.
À 3:57, la chauve-souris sort de la végétation.
L'utilisateur l'a jugé propre à l'œil ([08](08-banc-de-mesure.md#décision-du-6-octobre-2026)).

## Verdict de Manon (9 octobre 2026)

Sur les 14 pistes du réglage par défaut, les mêmes qu'aujourd'hui, **13 sont des chauves-souris**, y compris 0:51 et 2:37.
La piste de 0:51 passe surtout en bordure droite de l'écran mais elle n'est pas lente.
La seule qui n'en est pas une est la **piste 3:29 → 3:39, la chose lente**.
Le passage « 3:36 » confirmé en septembre était cette chose : il n'est plus compté comme un passage.
La vérité terrain de la 092 est donc : 13 chauves-souris, 1 objet lent à 3:29, et le 3:58 reste le passage historique.
Règle donnée le même jour : mieux vaut un faux positif facile à écarter à l'extrait, comme cette piste lente, qu'un passage manqué ([02](02-methode.md)).
Le passage de 3:42.10 ([plus bas](#cibles-connues-pour-les-issues-suivantes)) reste à lui montrer.

## L'objet lent de 3:29

Une tache sombre avance lentement et sans à-coups pendant 9,4 s de la gauche vers le centre.
Elle remonte ensuite la paroi.
Elle va à 95 px/s, contre 600 à 1 900 pour les autres pistes : ce n'est pas un vol.
Rien d'autre ne bouge à 3:36.
La piste reste produite, puisqu'un faux positif facile à écarter vaut mieux qu'un passage manqué.

## Cibles connues, pour les issues suivantes

- **092, 3:42.10 → 3:42.30** : objet réel confirmé par l'utilisateur en zoomant et en ralentissant ×0,5.
  Il est rapide (1 430 px/s) et entre par le haut de l'image.
  Il est aussi faible : pic de 13 à 22 après filtre selon les images (v0.1.1, 6 octobre 2026).
  **Le réglage par défaut le manque** : sur 2 de ses 6 images, il passe sur des pixels agités où le seuil par pixel monte à 22 à 36.
  Il ne reste alors que 4 détections pour un minimum de 6.
  On le trouve à 1440 px sans seuil par pixel. Ce réglage explose sur d'autres vidéos ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)).
  Manon doit encore confirmer que c'est une chauve-souris.
- **125, pistes 8 et 9 du réglage par défaut** : réelles selon l'utilisateur.
  La piste 8 va de 0:28.41 à 0:29.64 avec 37 détections. La piste 9 va de 0:28.94 à 0:29.74 avec 25 détections.
  Le réglage d'avant le 6 octobre n'en sortait qu'une (0:29.04 → 0:29.34, 8 détections).
- **Six pistes du réglage d'avant le 6 octobre que l'actuel ne sort plus**, toutes jugées fausses par l'utilisateur.
  Sur la 089, elles sont à 1:34.91, 1:46.27 et 3:13.33. Sur la 091, à 0:00.23. Sur la 125, à 2:03.35 et 4:04.18.

## Original contre export (historique, septembre et octobre 2026)

Cette mesure justifie la règle de copier les vidéos en USB ([01](01-contexte.md)).
Avec les réglages par défaut d'alors, on a comparé sur la même vidéo l'original (12,1 Mb/s) à l'export de Stream Vision 2 (1,04 Mb/s).
Les temps des pistes concordent à 0,1 s près.
Sur les 11 passages rapides communs, l'original donne 222 points détectés au lieu de 170 (+31 %).
Il fait aussi apparaître deux passages, 0:51 et 2:37. Manon les a reconnus comme des chauves-souris le 9 octobre.
La plupart des passages rapides traversent le champ en diagonale, du haut à droite vers le bas à gauche.
Ils mettent 0,3 à 1,3 s.
