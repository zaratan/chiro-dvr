# Banc de mesure par cibles synthétiques

Sans vérité terrain, on règle la détection à l'aveugle. Le banc incruste de fausses
chauves-souris dans une vraie vidéo, à des positions et des instants connus, puis compare
ce que l'outil retrouve à ce qui a été injecté. Aucun pipeline publié en écologie ne le
fait (voir [07](07-ameliorer-la-detection.md)).

```bash
uv run batdetect-bench in/video_092_original.mp4            # 9 classes × 30 cibles
uv run batdetect-bench in/video_092_original.mp4 --max-gap 8 # rejoue le suivi en quelques secondes
```

Sorties dans `out/bench/<nom>/` : `bench.json` (réglages, version du code, résumé par
classe, résultat par cible) et `cache.pkl` (détections de référence et injectées).

## Les cibles

- **Forme** : tache gaussienne signée, ajoutée en pleine résolution **avant** la réduction
  de l'image, là où se perd le contraste. Amplitude par défaut −15, −30, −60 niveaux ;
  taille σ = 1,5, 3 et 5 px d'origine. Les vraies chauves-souris de la 092 ont des taches
  de 150 à 400 px d'origine au-dessus du seuil.
- **Trajectoire** : courbe de Bézier qui entre par un bord et sort par un autre (pas
  d'apparition brutale en plein champ), vitesse de 15 à 60 px d'origine par image,
  accélération (progression en u^a, a de 1 à 1,8), et la **saccade de période 3** mesurée
  sur les Symbion (un pas double, deux simples, phase aléatoire).
- **Battements** : amplitude multipliée par un facteur aléatoire de 0,6 à 1 à chaque image.
- **Placement** : tirage par rejet. Aucune cible à moins de 3 rayons d'appariement d'une
  piste réelle (run de référence) ou d'une autre cible au même instant, pour éviter les
  crédits indus et les fusions de jumelles.
- **Reproductible** : une graine par cible (`SeedSequence.spawn`) ; ajouter une classe ne
  change pas les cibles déjà tirées.

## Ce qui compte comme visible

Une cible est visible à une image si son centre est à plus de 2σ des bords et si son
**contraste effectif** après écrêtage atteint 8 niveaux. Le banc ne connaît aucun
affichage de caméra : une cible injectée sous un affichage fixe reste comptée visible,
alors qu'en vrai l'affichage la cacherait (biais faible, l'affichage occupe peu de place). Une tache sombre sur le
ciel saturé à 0 n'a aucun contraste effectif : elle n'est pas comptée comme manquée.
Ce dénominateur ne dépend pas des masques de détection, ce qui permet de comparer deux
réglages de masque. Les cibles visibles moins de `min_hits` images sont écartées
(« not visible »).

## Appariement et métriques

- Une détection couvre une cible si elle est à moins de `--match-radius` (24 px d'origine)
  de sa position vraie à la même image.
- Chaque piste est attribuée à la cible qu'elle couvre le plus, si au moins 50 % de ses
  points la couvrent.
- **Trouvée** : au moins `min_hits` images visibles couvertes par ses pistes.
- **Complétude** : images visibles couvertes ÷ images visibles.
- **Retard de début** et **fin anticipée** : en images, entre la première (dernière) image
  visible et la première (dernière) couverte.
- **Fragments** : nombre de pistes attribuées à la cible.
- **Fausses pistes** : pistes du run injecté attribuées à aucune cible et sans
  équivalent dans le run de référence.
- Chaque taux est donné avec un intervalle de Wilson à 95 %. Avec 30 cibles par classe,
  il fait encore environ ±17 points à 50 % : un écart de quelques points entre deux
  réglages n'est pas significatif.

Le banc ne compte que les détections réelles des pistes : l'interpolation d'affichage ne
le change pas.

## Limites

- **Pas d'artefacts de compression** : l'injection se fait après décodage. Sur l'original
  (12 Mb/s, sans B-frames) le biais est faible, mais il est optimiste.
- **Forme simplifiée** : une gaussienne ronde, pas une silhouette d'ailes.
- **Saccade modélisée** : le banc reproduit la saccade, donc un correctif de la saccade
  y paraîtra efficace par construction. Le run réel (trous, fragments) reste juge.
- **Coût** : deux détections complètes (référence et injectée), puis le suivi est rejoué
  à la demande depuis le cache. Le cache est invalidé par un changement de réglage de
  détection ou de tirage, et par toute modification du code de `pipeline.py`,
  `parallel.py` ou `synthetic.py` (empreinte de leur contenu dans la clé). Le rapport note
  la version du code avec `git describe --dirty`.
- **Placement figé** : les cibles sont placées loin des pistes du run de référence, calculées
  avec les réglages de suivi du premier run. Rejouer avec un autre réglage de suivi
  réutilise ces mêmes cibles, ce qui garde les comparaisons appariées.
- **Masques** : une cible sous une `--osd-region` n'est pas comptée visible. Pour mesurer le
  coût d'un masque, comparer le nombre de cibles trouvées plutôt que la complétude.

## Référence avant les gains rapides

`video_092_original.mp4`, réglages par défaut, graine 1, code au commit de la sous-phase
1A. 270 cibles tirées, 251 évaluées (19 jamais visibles assez longtemps). Calcul : 4 min 31.

| Amplitude | σ (px) | n | Trouvées | IC 95 % | Complétude médiane | Retard (images) | Fin anticipée | Contraste effectif |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| −60 | 1,5 | 28 | 1 | 1 – 18 % | 0 | 14 | 19 | −46 |
| −60 | 3 | 26 | 24 | 76 – 98 % | 0,86 | 1 | 1 | −46 |
| −60 | 5 | 29 | 27 | 78 – 98 % | 0,92 | 0 | 1 | −47 |
| −30 | 1,5 | 28 | 0 | 0 – 12 % | 0 | — | — | −23 |
| −30 | 3 | 26 | 0 | 0 – 13 % | 0 | — | — | −24 |
| −30 | 5 | 27 | 15 | 37 – 72 % | 0,10 | 6 | 5 | −24 |
| −15 | 1,5 / 3 / 5 | 87 | 0 | 0 – 12 % | 0 | — | — | −12 |

Aucune fausse piste. Lecture :

- **La limite actuelle est nette** : il faut un contraste effectif d'environ −45 niveaux
  **et** une tache d'au moins σ = 3 px d'origine. À −24, seules les grosses taches sont
  parfois trouvées, et à 10 % de complétude.
- **Les petites taches sont invisibles même fortes** : −46 de contraste à σ = 1,5 donne
  1 cible trouvée sur 28. C'est la réduction à 480 px qui les dilue sous le seuil de 25,
  comme mesuré sur la vraie piste de 3:57.
- La chauve-souris de 3:57 à son arrivée (−30 à −70 en pleine résolution, petite) tombe
  exactement dans la zone que l'outil rate aujourd'hui.

## Résolution de travail et `min_area` (sous-phase 1A bis)

Mêmes cibles (graine 1), détection parallèle sur 10 processus, vérifiée identique au bit
près au traitement séquentiel sur la vidéo complète (la ligne 480 px / 18 reproduit
exactement la référence séquentielle). « Pistes de référence » = pistes trouvées sur la vraie vidéo sans
injection : c'est ce que verrait la naturaliste. Le banc ne compte comme fausses pistes que
celles absentes de la référence : **le bruit présent dans les deux runs n'y apparaît pas**,
d'où cette colonne.

| Réglage | −46, σ 1,5 | −46, σ 3 | −24, σ 3 | −24, σ 5 | Pistes de référence | Fausses pistes | Temps |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 480 px, `min_area` 18 | 1/28 | 24/26 (0,86) | 0/26 | 15/27 (0,10) | 14 | 0 | 1 min 59 |
| 720 px, `min_area` 18 | 0/28 | 24/26 (0,87) | 0/26 | 17/27 (0,20) | 13 | 0 | 2 min 42 |
| 960 px, `min_area` 18 | 0/28 | 24/26 (0,87) | 0/26 | 19/27 (0,26) | 12 | 0 | 4 min 36 |
| **480 px, `min_area` 4** | **26/28 (0,62)** | 24/26 (0,91) | **10/26** | **26/27 (0,39)** | **14** | **0** | 1 min 58 |
| 720 px, `min_area` 4 | 21/25 (0,80) | 24/26 (0,87) | 15/24 (0,30) | 18/23 (0,54) | **7 652** | 318 | 2 min 49 |
| 960 px, `min_area` 4 | 25/27 (0,87) | 25/26 (0,91) | 18/26 (0,30) | 26/27 (0,56) | **498** | 7 | 4 min 34 |

Entre parenthèses : complétude médiane. À 720 px avec `min_area` 4, le bruit a gêné le
tirage (43 cibles jamais visibles au lieu de 19) : ce jeu de cibles n'est pas le même.
Le run en pleine résolution (1440 px) a été arrêté après 25 minutes sans résultat.

Lecture :

- **`min_area` bloquait, pas la résolution.** À 480 px, l'abaisser de 18 à 4 px² fait
  passer les petites cibles contrastées de 1 à 26 sur 28, et les grosses peu contrastées
  de 15 à 26 sur 27, sans aucune piste de bruit en plus (14 pistes de référence).
- **La réduction à 480 px est un débruitage.** Chaque pixel de travail moyenne 3×3 pixels
  d'origine. À plus haute résolution, le même seuil de 25 laisse passer le bruit : des
  centaines ou des milliers de pistes. À 720 px, `min_area` 4 px² vaut 1 pixel de travail,
  donc un pixel isolé suffit ; à 960 px, il en faut 2, d'où moins de bruit qu'à 720.
- **La haute résolution apporte de la complétude** (0,87 contre 0,62 sur les petites cibles
  fortes à 960 px, 0,56 contre 0,39 sur les grosses faibles), mais elle n'est exploitable qu'avec un
  seuil adapté au bruit local (D1) et un filtrage à la taille de la cible (B2), qui
  remplace le débruitage qu'apportait la réduction.
- **La détection parallèle** divise le temps par 2,3 à 480 px (1 min 58 contre 4 min 36).
  Une première version se positionnait dans le fichier avec `CAP_PROP_POS_FRAMES` : sur
  la vidéo complète, une tranche était décalée d'une image. Chaque processus lit désormais
  depuis le début et saute les images jusqu'à sa tranche (`grab`, 0,84 ms par image).

## Gains rapides (sous-phase 1B)

480 px, `min_area` 4, nouvelle définition de « visible » (sans profil de caméra), les deux
lignes mesurées avec le même code :

| Réglage | −46, σ 1,5 | −46, σ 3 | −24, σ 5 | Fausses pistes | Pistes de référence |
| --- | --- | --- | --- | --- | --- |
| Anciennes bandes (`--osd-region 0,0,1,0.07 --osd-region 0,0.9,1,1`) | 26/28 (0,59) | 24/26 (0,86) | 26/27 (0,37) | 0 | 14 |
| **Sans masque (défaut)** | **27/28 (0,66)** | 24/26 **(0,95)** | 26/27 **(0,44)** | 0 | 14 |

La vitesse estimée sur 3 images ne change rien de mesurable au banc (une seule complétude
bouge de 0,01) ; sur la vraie vidéo, elle prolonge de 2 images la piste la plus rapide.
L'interpolation ne touche que l'affichage et n'a pas d'effet sur le banc par construction.
