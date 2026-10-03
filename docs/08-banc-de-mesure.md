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

Une cible est visible à une image si son centre est à plus de 2σ des bords, hors de
l'affichage réel des jumelles (profil Symbion, indépendant des réglages de détection), et
si son **contraste effectif** après écrêtage atteint 8 niveaux. Une tache sombre sur le
ciel saturé à 0 n'a aucun contraste effectif : elle n'est pas comptée comme manquée.
Ce dénominateur ne dépend pas des masques de détection, ce qui permet de comparer deux
réglages de masque. Les cibles visibles moins de `min_hits` images sont écartées
(« not visible »).

## Appariement et métriques

- Une détection couvre une cible si elle est à moins de `--match-radius` (8 px de travail)
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
  à la demande depuis le cache. Seul un changement de réglage de détection ou de tirage
  relance les détections.

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
