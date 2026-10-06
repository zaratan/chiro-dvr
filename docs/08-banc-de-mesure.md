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
- **Vols de chasse** (`--motions hunt circle`, `synthetic/hunting.py`) : la cible entre par
  un bord vers l'intérieur, vole à 8 à 30 px d'origine par image, et enchaîne segments
  presque droits (0,2 à 1 s pour `hunt`, 0,1 à 0,3 s pour `circle`, qui vire plus de la
  moitié du temps) et virages de 45° à 180°. Dans un virage, la vitesse descend à 25 à
  50 % de la croisière au sommet et la vitesse angulaire y culmine, au plus 0,15 à
  0,4 rad par image (un virage court est adouci : sommet réalisé jusqu'à 0,09) ; le rayon
  suit le carré de la vitesse, l'accélération latérale reste constante : d'après le demi-tour mesuré d'une chauve-souris (0,69 s, 2 → 0,5
  m/s, rayon 5,5 cm au sommet, [PMC7608926](https://pmc.ncbi.nlm.nih.gov/articles/PMC7608926/))
  et les pistes de chasse des pipistrelles, plus lentes et sinueuses que le transit
  ([PMC5234808](https://pmc.ncbi.nlm.nih.gov/articles/PMC5234808/)). Saccade et battements
  identiques au transit. Fin à la sortie du champ ou à 4 s (en plein champ : l'animal
  peut aussi sortir en profondeur). Le virage médian géométrique reste sous 0,2 rad par
  image pour 99 % des cibles : c'est le bruit de position de la détection, au sommet des
  virages lents, qui fait monter le virage mesuré. Le banc par défaut reste en transit
  seul (`--motions pass`) ; les classes de chasse sont tirées après celles de transit,
  qui ne changent donc pas. Elles ne changent que comme trajectoires : dans un banc qui
  mélange transit et chasse, les cibles de chasse modifient la détection et le suivi des
  autres ; garder des passes séparées.
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
Les images des périodes ignorées parce que saturées de taches
([02](02-methode.md#4-bis-images-saturées-de-taches)) ne comptent pas non plus : la
cible n'y est ni trouvée ni manquée. Ces périodes sont calculées sur le run injecté, comme
le suivi rejoué ; elles dépendent donc des réglages de détection (seuil, `work_width`),
et deux réglages ne se comparent plus sur exactement le même dénominateur s'ils en
produisent. Sur la 092, aucune période. Les images illisibles (images abîmées au décodage
et leur marge, [10](10-logique-de-detection.md)) ne comptent pas davantage : le banc passe
par la même fonction `exclude` que la commande, pour le run de référence comme pour le
run injecté. Sur une vidéo très abîmée, beaucoup de cibles tombent ainsi en « not
visible » sans être manquées. Les cibles visibles moins de `min_hits` images
sont écartées (« not visible »).

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
- **Chasse en 2D, sans distance** : vitesses et rayons en pixels, indépendants de la
  taille de la tache ; le rayon au sommet (5 à 100 px) peut être un peu plus serré que
  0,1 envergure pour σ 5. Ce qui décide du virage mesuré est le pas au sommet (2 px par
  image au moins) face au bruit du centroïde, que le banc mesure réellement.
- **Saccade modélisée** : le banc reproduit la saccade, donc un correctif de la saccade
  y paraîtra efficace par construction. Le run réel (trous, fragments) reste juge.
- **Coût** : deux détections complètes (référence et injectée), puis le suivi est rejoué
  à la demande depuis le cache. Le cache est invalidé par un changement de réglage de
  détection, de tirage ou de stabilité (`--max-blobs`, `--unstable-pad` : ils changent
  les pistes de référence qui guident le placement), et par toute modification du code de
  `video.py`, `median.py`, `detect.py`, `track.py`, `stability.py`, `spans.py`,
  `damage.py`, `probe.py`, `exclusion.py`, `parallel.py` ou du paquet `synthetic/`
  (empreinte de leur contenu dans la clé), et par la version d'`ffprobe`, qui repère les
  images abîmées. Le rapport note
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

## Vols de chasse (4 octobre 2026)

Banc dédié sur la 092 : `--motions hunt circle --amplitudes -60 -30 --sigmas 1.5 3 5
--per-class 60`, 34,7 s. 720 cibles tirées (12 classes × 60), 592 visibles (les 128
autres ne quittent jamais les coins noirs de l'oculaire : une chasse courte qui entre
près d'un coin n'atteint pas le disque visible ; environ 49 par classe en pratique),
371 trouvées avec `min_hits` 5 (comme le transit, la classe −30/σ1,5 est invisible et
−30/σ3 difficile), 0 fausse piste. Le banc par défaut est inchangé.

Comparaison appariée sur les mêmes cibles, depuis le cache (cibles trouvées avec le
réglage actuel et perdues avec le nouveau) :

| Réglage de suivi | Transit (115) | Chasse (371) |
| --- | --- | --- |
| virage médian ≤ 0,6 rad | −1 | −1 |
| virage médian ≤ 0,8 ou 1,0 rad | 0 | 0 |
| `min_hits` 6 | 0 | −14 |
| `min_hits` 7 | −1 | −24 |

Les cibles perdues avec `min_hits` plus haut sont surtout peu contrastées (−30) :
visibles une quinzaine d'images, détectées sur 5 ou 6.

Retenu le 4 octobre : `max_median_turn` 0,8 et `min_hits` 6. Bancs rejoués avec ces
réglages : transit 115 trouvées sur 253 visibles, 0 fausse piste ; chasse 357 trouvées sur
584 visibles (136 non visibles), 0 fausse piste. `min_hits` sert aussi au banc pour dire
qu'une cible est visible et trouvée : à `min_hits` 6 pour le suivi seul (critère resté à
5), la chasse perd 11 cibles au lieu de 14. Un critère « trouvée » indépendant du suivi
rendrait la comparaison plus honnête.

## Seuil par pixel (D1)

5 octobre 2026, 480 px, graine 1, transit, `--workers 1` (le décodeur d'OpenCV occupe
plusieurs cœurs), temps sur un second lancement.
`threshold` est le plancher, `noise_factor` (k) le relèvement local
([02](02-methode.md#3-seuil-dans-les-deux-sens)) ; k = 0 est le seuil fixe actuel. Entre
parenthèses : complétude médiane. Les classes à −15 ne sont trouvées par aucun réglage
sauf à seuil 15 et 12, où le bruit domine (colonne −15). « Mêmes cibles » : les cibles
tirées sont identiques à celles du réglage par défaut (le placement suit les pistes de
référence de chaque réglage) ; sinon la ligne ne se lit qu'en ordre de grandeur.

| Seuil, k | −60, σ 1,5 | −60, σ 3 | −60, σ 5 | −30, σ 1,5 | −30, σ 3 | −30, σ 5 | −15 | Pistes de réf. | Fausses | Mêmes cibles | Temps | CPU | Mémoire |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **25, 0 (défaut)** | 27/28 (0,66) | 24/26 (0,95) | 28/30 (0,97) | 0/28 | 10/27 (0) | 26/27 (0,43) | 0 | **14** | 0 | oui | 32 s | 224 s | 205 Mo |
| 25, 4 | 27/28 (0,66) | 24/26 (0,95) | 28/30 (0,97) | 0/28 | 10/27 (0) | 26/27 (0,43) | 0 | 14 | 0 | oui | 155 s | 343 s | 265 Mo |
| 25, 6 | 27/28 (0,62) | 24/26 (0,94) | 28/30 (0,97) | 0/28 | 8/27 (0) | 24/27 (0,39) | 0 | 14 | 0 | oui | 154 s | 343 s | 263 Mo |
| 25, 8 | 25/28 (0,51) | 24/26 (0,92) | 28/30 (0,97) | 0/28 | 3/27 (0) | 22/27 (0,32) | 0 | 13 | 0 | non | 154 s | 343 s | 255 Mo |
| 18, 0 | 27/28 (0,93) | 26/26 (0,98) | 28/30 (0,97) | 7/28 | 26/27 (0,77) | 27/27 (0,93) | 1 | 91 | 6 | non | 33 s | 224 s | 222 Mo |
| 18, 4 | 27/28 (0,93) | 26/26 (0,97) | 28/30 (0,98) | 6/28 | 26/27 (0,73) | 27/27 (0,92) | 1 | 25 | 2 | oui | 156 s | 344 s | 271 Mo |
| 18, 6 | 27/28 (0,87) | 26/26 (0,97) | 28/30 (0,98) | 1/28 | 24/27 (0,66) | 27/27 (0,89) | 0 | 15 | 0 | oui | 155 s | 343 s | 262 Mo |
| **18, 8** | 26/28 (0,70) | 26/26 (0,95) | 28/30 (0,98) | 0/28 | **22/27 (0,47)** | **27/27 (0,76)** | 0 | **14** | **0** | oui | 155 s | 343 s | 256 Mo |
| 15, 0 | 22/27 (0,83) | 25/27 (0,97) | 26/29 (0,93) | 13/27 (0,16) | 22/26 (0,88) | 22/25 (0,94) | 4 | 2 801 | 1 841 | non | 43 s | 233 s | 412 Mo |
| 15, 4 | 26/28 (0,90) | 26/26 (0,97) | 27/29 (0,97) | 16/27 (0,25) | 25/27 (0,93) | 24/27 (0,94) | 4 | 1 219 | 213 | non | 159 s | 348 s | 347 Mo |
| 15, 6 | 27/28 (0,87) | 26/26 (0,98) | 28/30 (0,98) | 10/27 | 25/27 (0,80) | 27/27 (0,93) | 3 | 136 | 7 | non | 156 s | 345 s | 281 Mo |
| 15, 8 | 26/28 (0,73) | 26/26 (0,96) | 28/30 (0,98) | 5/28 | 26/27 (0,63) | 27/27 (0,87) | 1 | 29 | 2 | non | 155 s | 344 s | 272 Mo |
| 12, 0 | 4/20 | 7/21 | 9/26 | 1/21 | 6/22 | 4/17 | 8 | 13 857 | 11 387 | non | 320 s | 512 s | 1 954 Mo |
| 12, 4 | 10/23 | 12/23 (0,60) | 10/27 | 5/21 | 12/22 (0,42) | 11/21 (0,46) | 11 | 9 409 | 6 907 | non | 206 s | 395 s | 917 Mo |
| 12, 6 | 16/26 (0,74) | 23/25 (0,94) | 23/29 (0,90) | 5/26 | 25/28 (0,79) | 24/25 (0,93) | 9 | 3 351 | 2 172 | non | 163 s | 352 s | 448 Mo |
| 12, 8 | 24/26 (0,71) | 27/27 (0,96) | 28/30 (0,94) | 4/27 | 25/27 (0,71) | 26/28 (0,89) | 5 | 1 258 | 329 | non | 158 s | 347 s | 339 Mo |

Les temps de ce tableau sont ceux de la première version de la carte de bruit, calculée
sur toute l'image ; elle ne l'est plus que sur les pixels au-dessus du plancher (voir
« Coût » plus bas).

Banc de chasse (`--motions hunt circle --amplitudes -60 -30 --per-class 60`), cibles
trouvées sur les visibles, toutes classes :

| Seuil, k | Trouvées | −30, σ 3 (hunt / circle) | −30, σ 5 (hunt / circle) | Pistes de réf. | Fausses | Temps |
| --- | --- | --- | --- | --- | --- | --- |
| 25, 0 (défaut) | 357 / 584 | 14/48 · 12/48 | 38/51 · 38/48 | 14 | 0 | 34 s |
| 18, 0 | 459 / 579 | 40/48 · 41/48 | 49/51 · 43/47 | 91 | 6 | 34 s |
| 18, 6 | 448 / 584 | 42/48 · 41/48 | 49/51 · 44/48 | 15 | 0 | 156 s |
| 18, 8 | 430 / 584 | 39/48 · 36/48 | 48/51 · 44/48 | 14 | 0 | 156 s |

Sur la 092 (commande complète) :

- **18, 8** : les 14 pistes, aucune en plus, 3:36 et 3:58 toujours suivis. Cinq pistes
  commencent plus tôt : 1:49 de 0,30 s, 3:57 de 0,14 s (4 images, 3:57.18 au lieu de
  3:57.32), 3:51 de 0,10 s, 1:12 de 0,07 s, 2:24 de 0,04 s. 0:51 (ignorable) commence
  0,07 s plus tard et tombe de 9 à 6 détections, **exactement `min_hits`** : à la limite,
  une détection de moins et elle disparaît. 2:44 perd une détection ; les autres en
  gagnent de 0 à 5. 90 s au lieu de 29 en mesure initiale, 31 à 35 s depuis que le bruit
  n'est calculé que sur les pixels au-dessus du plancher (voir plus bas).
- **18, 6, écarté** : 15 pistes. La nouvelle (2:45.02 → 2:45.38, 9 détections) chevauche
  la fin de 2:44, qui passe de 44 à 39 détections. Verdict de l'utilisateur (5 octobre) :
  ce n'est pas un second animal, c'est la même chauve-souris que celle de 2:43.95, sortie
  en double. La fusion des jumelles n'est pas modifiée pour autant (#26).
- **18, 0** : 91 pistes.

Lecture :

- **Baisser le seuil fixe est ce qui apporte le gain**, D1 ce qui le rend utilisable. À 18
  sans relèvement, les cibles à −30 de σ 3 passent de 10 à 26 sur 27, mais la vraie vidéo
  donne 91 pistes et le banc 6 fausses pistes. Avec k = 8, 22 sur 27 restent trouvées, et
  la vidéo revient à ses 14 pistes, sans fausse piste.
- **À seuil 25, D1 ne fait que perdre** : il ne peut que relever le seuil. À k = 8, une
  piste de référence disparaît.
- **En dessous de 18, le plancher ne tient plus** : à 15 ou 12, même k = 8 laisse des
  dizaines à des milliers de pistes de bruit. La MAD sur 11 images est trop instable pour
  servir seule.
- **Coût** : 2 médianes de plus par image, en flottants, calculées sur un seul fil. En
  première version, sur toute l'image : banc 32 → 155 s, CPU 224 → 343 s, mémoire +50 Mo.
  Depuis, le bruit n'est calculé que sur les pixels dont le résidu dépasse déjà le
  plancher (les autres ne peuvent pas être retenus) : mêmes détections au bit près, banc
  à 18 et 8 en 45 s et 236 s de CPU (machine chargée), 092 en 31 à 35 s au lieu de 93 à
  96 s ([09](09-profilage.md)).
- Les petites cibles faibles (−30, σ 1,5) restent introuvables à 480 px sans bruit en
  plus : c'est la cible du filtrage à la taille de la cible et de la pleine résolution
  (B2 et B1 dans [07](07-ameliorer-la-detection.md)).


## Filtre à la taille de la cible (B2)

5 octobre 2026, graine 1, transit, `--workers 1`, `min_area` 4 sauf mention. σ est
`--target-sigma` en pixels d'origine, k est `--noise-factor`. Entre parenthèses :
complétude médiane. « À la limite » : pistes de référence qui ont exactement `min_hits`
détections. « Mêmes cibles » : cibles tirées identiques à celles du réglage par défaut
(le placement suit les pistes de référence de chaque réglage) ; sinon la ligne ne se lit
qu'en ordre de grandeur. Temps marqués * : pris sous charge, deux bancs à la fois ; les
autres sous le verrou de mesure, un banc à la fois, avec un autre agent qui alterne ses
propres mesures. Les comptes ne dépendent pas de la charge.

Témoins et changements pris séparément :

| Réglage | −60, σ 1,5 | −60, σ 3 | −60, σ 5 | −30, σ 1,5 | −30, σ 3 | −30, σ 5 | −15 | Pistes de réf. | À la limite | Fausses | Non visibles | Mêmes cibles | Temps | CPU | Mémoire |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **480 px, défaut** | 27/28 (0,66) | 24/26 (0,95) | 28/30 (0,97) | 0/28 | 10/27 (0,00) | 26/27 (0,43) | 0 | 14 | 0 | 0 | 17 | oui | 32 s | 222 s | 205 Mo |
| 480 px, σ 0, seuil 18, k 8 (D1 seul) | 26/28 (0,70) | 26/26 (0,95) | 28/30 (0,98) | 0/28 | 22/27 (0,47) | 27/27 (0,76) | 0 | 14 | 1 | 0 | 17 | oui | 45 s * | 236 s * | 205 Mo |
| 960 px, σ 0, seuil 18, k 8 (D1 seul) | 24/27 (0,62) | 26/26 (0,94) | 28/30 (0,97) | 0/28 (0,00) | 22/27 (0,41) | 27/27 (0,61) | 0 | 16 | 1 | 0 | 18 | non | 140 s | 529 s | 320 Mo |
| 960 px, σ 0, seuil 25, k 0 (résolution seule) | 27/28 (0,94) | 25/26 (0,96) | 28/30 (0,98) | 1/28 (0,00) | 17/27 (0,35) | 26/27 (0,62) | 0 | 118 | 39 | 5 | 17 | non | 119 s | 509 s | 311 Mo |

À 960 px, filtre seul (k 0) et filtre avec seuil par pixel (k 8) :

| Réglage | −60, σ 1,5 | −60, σ 3 | −60, σ 5 | −30, σ 1,5 | −30, σ 3 | −30, σ 5 | −15 | Pistes de réf. | À la limite | Fausses | Non visibles | Mêmes cibles | Temps | CPU | Mémoire |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 960 px, σ 1, seuil 8, k 8 | 26/27 (0,87) | 26/26 (1,00) | 29/30 (1,00) | 16/28 (0,18) | 27/27 (0,90) | 27/27 (0,96) | 42 | 42 | 12 | 2 | 18 | non | 416 s * | 710 s * | 383 Mo |
| 960 px, σ 1, seuil 12, k 0 | 26/28 (0,93) | 25/25 (0,97) | 26/29 (0,96) | 21/25 (0,50) | 24/26 (0,93) | 24/26 (0,96) | 34 | 1408 | 372 | 493 | 32 | non | 275 s * | 664 s * | 467 Mo |
| 960 px, σ 1, seuil 12, k 8 | 27/28 (0,86) | 26/26 (1,00) | 30/30 (0,99) | 8/28 (0,00) | 26/27 (0,86) | 27/27 (0,96) | 15 | 14 | 0 | 0 | 17 | oui | 324 s * | 702 s * | 363 Mo |
| 960 px, σ 1, seuil 16, k 0 | 27/28 (0,98) | 26/26 (1,00) | 28/30 (0,98) | 11/28 (0,00) | 26/27 (0,91) | 27/27 (1,00) | 4 | 16 | 1 | 0 | 17 | oui | 318 s * | 691 s * | 351 Mo |
| 960 px, σ 1, seuil 16, k 8 | 27/28 (0,84) | 26/26 (0,98) | 28/30 (0,98) | 0/28 (0,00) | 26/27 (0,75) | 27/27 (0,93) | 0 | 14 | 0 | 0 | 17 | non | 301 s | 660 s | 348 Mo |
| 960 px, σ 1,5, seuil 8, k 0 | 20/26 (0,86) | 21/24 (0,95) | 22/28 (0,93) | 19/23 (0,70) | 21/25 (0,94) | 20/26 (0,87) | 35 | 4078 | 798 | 2575 | 45 | non | 318 s | 675 s | 659 Mo |
| 960 px, σ 1,5, seuil 8, k 8 | 26/27 (0,93) | 26/26 (1,00) | 29/30 (1,00) | 22/28 (0,29) | 27/27 (0,96) | 27/27 (1,00) | 48 | 21 | 1 | 1 | 18 | oui | 298 s | 667 s | 359 Mo |
| 960 px, σ 1,5, seuil 12, k 0 | 27/28 (0,98) | 26/26 (1,00) | 30/30 (0,99) | 14/28 (0,08) | 26/27 (0,98) | 27/27 (1,00) | 26 | 14 | 0 | 1 | 17 | oui | 274 s | 650 s | 360 Mo |
| 960 px, σ 1,5, seuil 12, k 8 | 27/28 (0,89) | 26/26 (1,00) | 29/30 (0,98) | 1/28 (0,00) | 26/27 (0,91) | 27/27 (1,00) | 12 | 14 | 0 | 0 | 17 | oui | 292 s | 661 s | 342 Mo |
| 960 px, σ 1,5, seuil 16, k 0 | 27/28 (0,88) | 25/26 (0,97) | 28/30 (0,98) | 0/28 (0,00) | 25/27 (0,72) | 27/27 (0,95) | 0 | 14 | 0 | 0 | 17 | oui | 243 s | 635 s | 374 Mo |
| 960 px, σ 1,5, seuil 16, k 8 | 27/28 (0,81) | 25/26 (0,97) | 28/30 (0,98) | 0/28 (0,00) | 25/27 (0,68) | 27/27 (0,92) | 0 | 14 | 0 | 0 | 17 | oui | 245 s | 639 s | 351 Mo |
| 960 px, σ 2, seuil 8, k 0 | 27/28 (0,99) | 26/26 (1,00) | 29/30 (0,99) | 23/28 (0,45) | 27/27 (1,00) | 27/27 (1,00) | 56 | 18 | 2 | 0 | 17 | non | 248 s | 644 s | 346 Mo |
| 960 px, σ 2, seuil 8, k 8 | 27/28 (0,90) | 26/26 (1,00) | 28/30 (0,98) | 5/28 (0,00) | 26/27 (0,94) | 27/27 (1,00) | 48 | 16 | 2 | 0 | 17 | oui | 276 s | 660 s | 353 Mo |
| 960 px, σ 2, seuil 12, k 0 | 27/28 (0,90) | 26/26 (0,98) | 28/30 (0,98) | 0/28 (0,00) | 26/27 (0,91) | 27/27 (1,00) | 12 | 15 | 0 | 0 | 17 | oui | 264 s | 645 s | 363 Mo |
| 960 px, σ 2, seuil 12, k 8 | 27/28 (0,79) | 26/26 (0,98) | 28/30 (0,98) | 0/28 (0,00) | 26/27 (0,88) | 27/27 (1,00) | 8 | 15 | 0 | 0 | 17 | oui | 260 s | 647 s | 347 Mo |
| 960 px, σ 2, seuil 16, k 0 | 23/28 (0,41) | 25/26 (0,95) | 28/30 (0,98) | 0/28 (0,00) | 22/27 (0,47) | 27/27 (0,88) | 0 | 14 | 0 | 0 | 17 | oui | 242 s | 632 s | 345 Mo |
| 960 px, σ 2, seuil 16, k 8 | 21/28 (0,37) | 25/26 (0,95) | 28/30 (0,98) | 0/28 (0,00) | 21/27 (0,46) | 27/27 (0,88) | 0 | 14 | 1 | 0 | 17 | oui | 250 s | 641 s | 346 Mo |

Le réglage 960 px, σ 1, seuil 8, k 0 a été arrêté par le garde-fou après 12 minutes (son
voisin au seuil 12 donnait déjà 1 408 pistes de référence).

Autres largeurs, et `min_area` :

| Réglage | −60, σ 1,5 | −60, σ 3 | −60, σ 5 | −30, σ 1,5 | −30, σ 3 | −30, σ 5 | −15 | Pistes de réf. | À la limite | Fausses | Non visibles | Mêmes cibles | Temps | CPU | Mémoire |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 480 px, σ 1,5, seuil 12, k 0 | 27/28 (0,97) | 26/26 (1,00) | 28/30 (0,98) | 18/28 (0,30) | 26/27 (0,97) | 27/27 (1,00) | 30 | 33 | 9 | 0 | 17 | non | 66 s | 258 s | 276 Mo |
| 480 px, σ 1,5, seuil 16, k 0 | 27/28 (0,86) | 25/26 (0,98) | 28/30 (0,98) | 0/28 (0,00) | 24/27 (0,69) | 27/27 (0,95) | 0 | 14 | 0 | 0 | 17 | oui | 65 s | 257 s | 246 Mo |
| 1440 px, σ 1,5, seuil 12, k 0 | 27/28 (1,00) | 26/26 (1,00) | 29/30 (0,99) | 18/28 (0,29) | 26/27 (0,98) | 27/27 (1,00) | 30 | 16 | 1 | 0 | 17 | oui | 561 s | 921 s | 511 Mo |
| 1440 px, σ 1,5, seuil 12, k 8 | 27/28 (0,91) | 26/26 (1,00) | 29/30 (0,98) | 2/28 (0,00) | 26/27 (0,92) | 27/27 (1,00) | 15 | 14 | 0 | 0 | 17 | oui | 585 s | 948 s | 521 Mo |
| 1440 px, σ 1,5, seuil 16, k 0 | 27/28 (0,92) | 25/26 (0,97) | 28/30 (0,98) | 0/28 (0,00) | 25/27 (0,76) | 27/27 (0,95) | 0 | 14 | 0 | 0 | 17 | oui | 543 s | 908 s | 514 Mo |
| 1440 px, σ 1,5, seuil 16, k 8 | 27/28 (0,84) | 25/26 (0,97) | 28/30 (0,98) | 0/28 (0,00) | 25/27 (0,73) | 27/27 (0,94) | 0 | 14 | 0 | 0 | 17 | oui | 568 s | 936 s | 523 Mo |
| 1440 px, σ 2, seuil 12, k 0 | 27/28 (0,93) | 26/26 (0,98) | 28/30 (0,98) | 0/28 (0,00) | 26/27 (0,93) | 27/27 (1,00) | 13 | 15 | 0 | 0 | 17 | oui | 542 s | 921 s | 515 Mo |
| 960 px, σ 1,5, seuil 12, k 0, min_area 9 | 27/28 (0,96) | 26/26 (0,99) | 29/30 (0,98) | 1/28 (0,00) | 26/27 (0,96) | 27/27 (1,00) | 19 | 14 | 0 | 0 | 17 | oui | 238 s | 630 s | 351 Mo |
| 960 px, σ 1,5, seuil 12, k 0, min_area 18 | 27/28 (0,67) | 26/26 (0,98) | 28/30 (0,98) | 0/28 (0,00) | 26/27 (0,85) | 27/27 (1,00) | 9 | 14 | 0 | 0 | 17 | oui | 236 s | 632 s | 349 Mo |
| 1440 px, σ 1,5, seuil 12, k 0, min_area 9 | 27/28 (0,95) | 26/26 (1,00) | 28/30 (0,98) | 1/28 (0,00) | 26/27 (0,97) | 27/27 (1,00) | 19 | 14 | 0 | 0 | 17 | oui | 533 s | 913 s | 523 Mo |
| 1440 px, σ 1,5, seuil 12, k 0, min_area 18 | 26/28 (0,68) | 26/26 (0,98) | 28/30 (0,98) | 0/28 (0,00) | 26/27 (0,88) | 27/27 (1,00) | 9 | 14 | 0 | 0 | 17 | oui | 540 s | 917 s | 524 Mo |

Banc de chasse (`--motions hunt circle --amplitudes -60 -30 --per-class 60`), cibles
trouvées sur 584 visibles, 0 fausse piste partout :

| Réglage | Trouvées | −30, σ 1,5 (hunt · circle) | −30, σ 3 | Pistes de réf. | Temps |
| --- | --- | --- | --- | --- | --- |
| 480 px, défaut | 357 | 0/48 · 0/53 | 14/48 · 12/48 | 14 | 34 s |
| 480 px, seuil 18, k 8 (D1) | 430 | 0/48 · 3/53 | 39/48 · 36/48 | 14 | 156 s * |
| **C1** : 960 px, σ 1,5, seuil 12, k 0 | 510 | 30/48 · 28/53 | 46/48 · 43/48 | 14 | 234 s |
| **C2** : 1440 px, σ 1,5, seuil 12, k 0 | 525 | 38/48 · 34/53 | 46/48 · 43/48 | 16 | 533 s |
| **C3** : 960 px, σ 1,5, seuil 12, k 8 | 456 | 3/48 · 6/53 | 45/48 · 42/48 | 14 | 243 s |
| **C4** : 960 px, σ 2, seuil 8, k 0 | 540 | 43/48 · 43/53 | 46/48 · 45/48 | 18 | 238 s |
| **C5** : 1440 px, σ 1,5, seuil 12, k 8 | 470 | 11/48 · 11/53 | 45/48 · 41/48 | 14 | 556 s |

(Le temps D1 est celui de la première version de la carte de bruit.)

Sur la 092 (commande complète, sous le verrou, `pgrep` vide) : défaut 30 s, 175 s de
CPU, 627 Mo ; C1 135 s, 378 s, 702 Mo ; C2 283 s, 518 s, 838 Mo ; C3 135 s, 379 s,
707 Mo. Les passages confirmés de 3:36 et 3:58 restent suivis par les trois.
Écart de début en images (négatif : plus tôt), et de détections :

| Passage (défaut) | C1 | C2 | C3 |
| --- | --- | --- | --- |
| 0:07.39, 17 | 0, +3 | 0, +4 | 0, +3 |
| 0:33.94, 39 | 0, +2 | 0, +2 | 0, +1 |
| 0:51.62, 9 | 0, 0 | 0, 0 | 0, −2 |
| 0:53.75, 34 | 0, +2 | 0, +2 | 0, 0 |
| 1:12.67, 31 | −10, +7 | −10, +7 | −2, +4 |
| 1:49.23, 23 | −9, +10 | −9, +10 | −9, +9 |
| 2:02.82, 12 | 0, +7 | 0, +7 | 0, +5 |
| 2:24.67, 11 | −4, +6 | −4, +6 | −1, +2 |
| **2:36.99, 7** (valide) | **+5**, +1 | −4, +6 | 0, +3 |
| 2:43.95, 44 | 0, +1 | 0, +1 | 0, 0 |
| 3:29.61, 287 | 0, +3 | 0, +3 | 0, +3 |
| 3:48.29, 16 | 0, +2 | 0, +2 | 0, +1 |
| 3:51.02, 21 | −4, +3 | −4, +3 | −4, +3 |
| 3:57.32, 18 | −4, +9 | −4, +9 | −4, +9 |
| Pistes nouvelles | aucune | 3:42.10 → 3:42.30, 6 détections (**à la limite**), 1 431 px/s, amplitude 22 ; 3:51.22 → 3:51.69, 8 détections, qui chevauche 3:50.89 → 3:51.69 | aucune |

Lecture :

- **Le filtre apporte le plus gros gain mesuré, mais il ne paie qu'avec la résolution** : à
  960 px, σ 1,5 et seuil 12 sans carte de bruit (C1), les petites cibles faibles
  (−30, σ 1,5) passent de 0 à 14 sur 28, les −30 σ 3 de 10 à 26 avec une complétude de
  0 à 0,98, et 26 cibles à −15 apparaissent, pour 14 pistes de référence et 1 fausse
  piste. Le même filtre à 480 px donne 33 pistes de référence, dont 9 à la limite.
- **La résolution seule** (960 px sans filtre) laisse passer le bruit : 118 pistes de
  référence, 5 fausses. **D1 seul** à 960 px : 16 pistes de référence, dont 1 à la
  limite, 0 fausse, peu de gain.
- **Filtre et seuil par pixel ensemble** (C3) : 0 fausse piste, mais la carte de bruit
  reprend l'essentiel du gain sur les petites cibles faibles (−30 σ 1,5 : 1 sur 28 ; chasse
  456 contre 510). Le filtre fait déjà le travail de débruitage.
- **1440 px** (C2) gagne encore un peu (−30 σ 1,5 : 18 sur 28 ; chasse 525) pour deux fois
  le temps de 960 px, et sort deux pistes nouvelles sur la 092.
- **σ et seuil vont ensemble**, car le filtre abaisse le pic d'une petite tache d'autant
  plus que σ est grand. σ 1 laisse passer le bruit sans carte (1 408 pistes de référence
  au seuil 12). σ 2 perd les petites cibles faibles au seuil 12 (0 sur 28) mais, au
  seuil 8, donne l'un des meilleurs résultats (−30 σ 1,5 : 23 sur 28, −15 : 56) pour
  18 pistes de référence, dont 2 à la limite, et 0 fausse piste. σ 1,5 au seuil 12 est
  le meilleur réglage qui garde 14 pistes de référence. Au seuil 8, σ 1,5 explose sans
  carte (4 078) ; au seuil 16, les cibles faibles sont perdues.
  **`min_area`** 9 ou 18 perd les petites cibles faibles (14 → 1 ou 0 sur 28) et ne
  retire qu'une fausse piste (1 → 0) : 4 reste.
- **Coût** : la médiane sur des images en flottants et le flou. Banc 32 s → 4 min à
  960 px, 9 min à 1440 px ; la 092 passe de 30 s à 135 s (960 px) ou 283 s (1440 px).

### Sur six autres vidéos

Les quatre candidats et le défaut ont été passés sur la 092 et sur six autres vidéos
sans erreur de décodage (089, 090, 091, 125, 126, 127 ; comparaison détaillée hors
dépôt). Avec un seuil fixe abaissé et sans seuil par pixel (C1, C2, C4), la 089, la 090
et la 091 donnent des centaines à des milliers de pistes (C1 : 560, 374, 394 ; C4 :
5 479, 4 925, 3 974), alors que la 092 et la 125 restent propres. Ces vidéos ont en
permanence 3 à 5 taches de bruit par image avec C1 (0 sur la 092 et la 125), d'un pixel
de travail et d'amplitude juste au-dessus du seuil, que le suivi enchaîne en fausses
pistes ; ce n'est pas un effet des mouvements de jumelles (la 090 n'en a aucun). C3, avec
le seuil par pixel, garde une médiane de 0 tache par image et 1, 20 et 21 pistes sur ces
trois vidéos (4, 12 et 23 au défaut). **Un seuil fixe abaissé ne se règle pas sur une
seule vidéo calme** : le banc, fait sur la 092, ne le montrait pas.

C5 (C3 en pleine résolution, 1440 px) tient aussi sur les sept vidéos (14, 1, 17, 22, 48,
10 et 7 pistes) pour plus du double du temps de C3. Ni C3 ni C5 ne trouvent le passage
réel de 3:42.10 sur la 092 : la cible n'y dépasse le plancher que de 1 à 10 niveaux, et
le seuil par pixel la perd sur 2 de ses 6 images, où 8σ vaut 22 à 36. Elle tombe alors
sous `min_hits`. C1 la perd autrement : à 960 px, une de ses images n'a qu'un pixel de
travail au-dessus du seuil, sous `min_area`.

## Décision du 6 octobre 2026

C3 devient le réglage par défaut : 960 px, filtre de 1,5 px, seuil 12, facteur de bruit 8.
L'utilisateur a regardé une à une les six pistes de l'ancien réglage que C3 ne sort
plus (089 : 1:34.91, 1:46.27, 3:13.33 ; 091 : 0:00.23 ; 125 : 2:03.35, 4:04.18) : toutes
fausses. Il a validé comme réelles les pistes 8 et 9 de C3 sur la 125.

Sur les sept vidéos :

| | 092 | 089 | 090 | 091 | 125 | 126 | 127 | Total | Calcul (réel / CPU) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Ancien réglage | 14 | 4 | 12 | 23 | 18 | 4 | 0 | 75 | 4 / 20 min |
| **C3 (défaut)** | 14 | 1 | 20 | 21 | 47 | 12 | 7 | 122 | 16 / 45 min |
| C5 (C3 à 1440 px) | 14 | 1 | 17 | 22 | 48 | 10 | 7 | 119 | 38 / 67 min |
| C1, C2, C4 (sans seuil par pixel) | 14 à 18 | 560 à 5 479 | 374 à 4 925 | 394 à 3 974 | 48 à 57 | 11 à 13 | 7 à 13 | | |

Bancs refaits avec les nouveaux défauts, identiques clé par clé à ceux de C3 : transit
148 cibles trouvées, 14 pistes de référence, 0 fausse piste (260 s, 640 s de CPU) ;
chasse 456 sur 584, 0 fausse piste (254 s). Limite connue : le passage réel de 3:42.10
sur la 092 est manqué ([03](03-resultats-092.md#cibles-connues-pour-les-issues-suivantes)).
