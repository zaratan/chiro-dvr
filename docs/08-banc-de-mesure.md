# Banc de mesure par cibles synthétiques

Sans vérité terrain, on règle la détection à l'aveugle.
Le banc incruste de fausses chauves-souris dans une vraie vidéo, à des positions et des instants connus, puis compare ce que l'outil retrouve à ce qui a été injecté.
Aucun pipeline publié en écologie ne le fait ([07](07-ameliorer-la-detection.md)).

Un banc fait deux détections complètes de la vidéo : le run de référence, sur la vidéo telle quelle, et le run injecté, avec les cibles.
Les mesures de la fin de ce document datent de la v0.1.1 (3 au 6 octobre 2026) et ont fixé les réglages par défaut.
Celles du banc actuel, et sa garde contre les bancs enregistrés par la v0.1.1, sont dans [11](11-migration-rust.md#banc-de-mesure).

## La commande et ses sorties

Le banc est la crate `bench`, binaire `batdetect-bench`.

```bash
cargo run --release -p bench -- in/video_092_original.mp4                    # 9 classes × 30 cibles
cargo run --release -p bench -- in/video_092_original.mp4 --motions hunt circle
cargo run --release -p bench -- --replay out/bench/video_092_original --max-gap 8 -o out/gap8
```

- **Tirer des cibles** (par défaut) : `--per-class`, `--amplitudes`, `--sigmas`, `--motions`, `--seed`.
  La vidéo se donne avant `--amplitudes`, qui prend plusieurs valeurs.
- **Rejouer des cibles** : `--targets FICHIER` reprend les cibles d'un `bats.jsonl.gz` au lieu de les tirer.
  `--against DOSSIER` rejoue un banc enregistré par la v0.1.1 et dresse la garde : cibles perdues et gagnées, échec sous la borne basse de Wilson d'une classe ([11](11-migration-rust.md#garde)).
  Le banc enregistré fixe les réglages.
  La v0.1.1 ne recollait pas les vols cachés et n'avait pas de portée : contre elle, `--against` demande `--rejoin-hidden 0 --reach-margin 0`.
- **Réévaluer** : `--replay DOSSIER` relit un banc déjà écrit et le réévalue avec d'autres réglages de suivi, de stabilité ou d'appariement.
  Il ne refait aucune détection, et répond donc tout de suite.
- **Réglages** : les dix de la détection (`--threshold`, `--min-area`, `--max-area`, `--bg-window`, `--bg-step`, `--osd-region`, `--merge-radius`, `--work-width`, `--noise-factor`, `--target-sigma`), la stabilité (`--max-blobs`, `--unstable-pad`), le suivi (`--max-jump`, `--max-gap`, `--min-hits`, `--min-travel`, `--twin-distance`, `--max-median-turn`, `--rejoin-hidden`, `--rejoin-distance`, `--rejoin-piece-points`, `--rejoin-junction-pace`, `--rejoin-piece-pace`, `--reach-margin`, `--reach-travel`) et l'appariement (`--match-radius`).
  `batdetect-bench --help` donne leurs défauts, qui sont ceux de la commande.
- **Sorties** dans `-o` (`out/bench` par défaut), un dossier par vidéo.
  Il contient le banc lui-même : `run.json`, les détections de référence et injectées (`reference.jsonl.gz` et `injected.jsonl.gz`), les cibles et leur contraste effectif (`bats.jsonl.gz`).
  `bench.json` donne les réglages, la version du code, le résumé par classe et le résultat par cible, avec les clés des bancs de la v0.1.1 plus `targets` et `seconds`.
  `--against` écrit dans `<vidéo>_<dossier>/` et ajoute `guard.txt` ; `--replay` écrit `bench.json` dans `-o`.
  Une sortie qui écraserait l'entrée est refusée.
- La sortie standard donne le tableau par classe, le chemin du rapport et, avec `--against`, la garde.
  Code de sortie : 0, 1 (échec, ou garde en échec), 2 (usage).

## Les cibles

- **Forme** : une tache gaussienne signée, ajoutée en pleine résolution **avant** la réduction de l'image, là où se perd le contraste.
  Amplitudes par défaut −15, −30 et −60 niveaux ; tailles σ = 1,5, 3 et 5 px d'origine.
  Pour comparaison, les vraies chauves-souris de la 092 font des taches de 150 à 400 px d'origine au-dessus du seuil.
- **Trajectoire** : une courbe de Bézier qui entre par un bord et sort par un autre, sans apparition brutale en plein champ.
  Elle va de 15 à 60 px d'origine par image et accélère (progression en u^a, a de 1 à 1,8).
  Elle reproduit la **saccade de période 3** mesurée sur les Symbion : un pas double, deux simples, phase au hasard ([07](07-ameliorer-la-detection.md#cadence-saccadée)).
- **Battements** : l'amplitude est multipliée par un facteur au hasard de 0,6 à 1 à chaque image.
- **Vols de chasse** (`--motions hunt circle`, `crates/bench/src/synthetic/hunting.rs`) : la cible entre par un bord vers l'intérieur et vole à 8 à 30 px d'origine par image.
  Elle enchaîne des segments presque droits et des virages de 45° à 180°.
  Les segments durent 0,2 à 1 s pour `hunt`, 0,1 à 0,3 s pour `circle`, qui vire plus de la moitié du temps.
  Au sommet d'un virage, la vitesse descend à 25 à 50 % de la croisière, et la vitesse angulaire culmine, à 0,15 à 0,4 rad par image au plus (un virage court est adouci, avec un sommet réalisé jusqu'à 0,09).
  Le rayon suit le carré de la vitesse, et l'accélération latérale reste constante.
  Ce modèle vient du demi-tour mesuré d'une chauve-souris (0,69 s, de 2 à 0,5 m/s, rayon de 5,5 cm au sommet, [PMC7608926](https://pmc.ncbi.nlm.nih.gov/articles/PMC7608926/)) et des pistes de chasse des pipistrelles, plus lentes et sinueuses que le transit ([PMC5234808](https://pmc.ncbi.nlm.nih.gov/articles/PMC5234808/)).
  La saccade et les battements sont ceux du transit.
  Le vol finit à la sortie du champ ou au bout de 4 s, en plein champ : l'animal peut aussi sortir en profondeur.
  Pour 99 % des cibles, le virage médian géométrique reste sous 0,2 rad par image ; c'est le bruit de position de la détection, au sommet des virages lents, qui fait monter le virage mesuré.
  Le banc par défaut reste en transit seul (`--motions pass`).
  Les classes de chasse sont tirées après celles de transit, dont les trajectoires ne changent donc pas.
  Mais dans un banc qui mélange les deux, les cibles de chasse changent la détection et le suivi des autres : on garde des bancs séparés.
- **Placement** : tirage par rejet.
  Aucune cible n'est à moins de 3 rayons d'appariement d'une piste du run de référence, ou d'une autre cible au même instant : on évite ainsi les crédits indus et les fusions de jumelles.
- **Reproductible** : chaque cible a sa graine, tirée de `--seed` dans l'ordre des numéros (`crates/bench/src/random.rs`), et ajouter une classe ne change pas les cibles déjà tirées.
  La même graine redonne les mêmes cibles sur un même système ; d'un système à l'autre, on rejoue les cibles par `--targets`.
  Les cibles tirées par la v0.1.1 ne se retirent pas : ses bancs se rejouent par `--against`.

## Ce qui compte comme visible

Une cible est visible à une image si son centre est à plus de 2σ des bords et si son **contraste effectif**, après écrêtage, atteint 8 niveaux.
Les cibles visibles moins de `min_hits` images sont écartées (« not visible »).

Plusieurs cas ne comptent ni comme trouvés ni comme manqués :

- une tache sombre sur le ciel saturé à 0, qui n'a aucun contraste effectif ;
- les images des périodes ignorées parce que saturées de taches ([02](02-methode.md#4-bis-images-saturées-de-taches)) ;
- les images illisibles, abîmées au décodage, et leur marge ([10](10-logique-de-detection.md)) : le banc passe par la même fonction `exclude` que la commande, pour les deux runs.
  Sur une vidéo très abîmée, beaucoup de cibles tombent ainsi en « not visible ».

Les périodes saturées sont calculées sur le run injecté, comme le suivi rejoué.
Elles dépendent donc des réglages de détection (seuil, `work_width`) : deux réglages qui en produisent ne se comparent plus sur exactement le même dénominateur.
Sur la 092, il n'y en a aucune.

Le banc ne connaît aucun affichage de caméra : une cible injectée sous un affichage fixe reste comptée visible, alors qu'en vrai l'affichage la cacherait.
Le biais est faible, car l'affichage occupe peu de place.

## Appariement et métriques

- Une détection couvre une cible si elle est à moins de `--match-radius` (24 px d'origine) de sa position vraie à la même image.
- Chaque piste est attribuée à la cible qu'elle couvre le plus, si au moins 50 % de ses points la couvrent.
- **Trouvée** : au moins `min_hits` images visibles couvertes par ses pistes.
- **Complétude** : images visibles couvertes ÷ images visibles.
- **Retard de début** et **fin anticipée** : en images, entre la première (dernière) image visible et la première (dernière) couverte.
- **Fragments** : nombre de pistes attribuées à la cible.
- **Fausses pistes** : pistes du run injecté attribuées à aucune cible et sans équivalent dans le run de référence.
- Chaque taux est donné avec un intervalle de Wilson à 95 %.
  Avec 30 cibles par classe, il fait encore environ ±17 points à 50 % : un écart de quelques points entre deux réglages n'est pas significatif.

Le banc ne compte que les détections réelles des pistes : l'interpolation à l'affichage ne le change pas.

## Limites

- **Pas d'artefacts de compression** : on injecte après le décodage.
  Sur l'original (12 Mb/s, sans B-frames), le biais est faible, mais il va dans le sens optimiste.
- **Forme simplifiée** : une gaussienne ronde, pas une silhouette d'ailes.
- **Chasse en 2D, sans distance** : vitesses et rayons sont en pixels, indépendants de la taille de la tache.
  Le rayon au sommet (5 à 100 px) peut être un peu plus serré que 0,1 envergure pour σ 5.
  Ce qui décide du virage mesuré est le pas au sommet (2 px par image au moins) face au bruit du centroïde, et ce bruit-là, le banc le mesure pour de vrai.
- **Saccade modélisée** : comme le banc reproduit la saccade, un correctif de la saccade y paraîtra efficace par construction.
  Le run réel (trous, fragments) reste juge.
- **`min_hits` sert deux fois** : au suivi, et au banc pour dire qu'une cible est visible et trouvée.
  Changer `min_hits` change donc aussi le critère de mesure ([vols de chasse](#vols-de-chasse-4-octobre-2026)).
- **Coût** : environ 31 s sur la 092 en `release` pour les deux détections ; `--replay` réévalue un banc écrit en moins d'une seconde.
  Le rapport note la version du code avec `git describe --always --dirty` ([11](11-migration-rust.md#banc-de-mesure)).
- **Placement figé** : les cibles sont placées loin des pistes du run de référence, calculées avec les réglages de suivi du premier run.
  Rejouer avec un autre réglage de suivi réutilise ces mêmes cibles, ce qui garde les comparaisons appariées.
- **Masques** : une cible sous une `--osd-region` n'est pas comptée visible.
  Pour mesurer le coût d'un masque, on compare le nombre de cibles trouvées plutôt que la complétude.

## Mesures de la v0.1.1 (3 au 6 octobre 2026)

Ces mesures ont été faites avec le banc de la v0.1.1, sur `video_092_original.mp4`, graine 1.
Elles justifient les réglages par défaut d'aujourd'hui.
Leurs chiffres ne se comparent pas à l'unité près à ceux du banc actuel, et leurs temps ne valent que pour la v0.1.1.

Dans les tableaux, une classe se note par son amplitude et sa taille : « −30, σ 3 » est la classe d'amplitude −30 et de σ = 3 px d'origine.
Les colonnes des premières sections donnent le contraste effectif mesuré (−46 pour −60, −24 pour −30).
Entre parenthèses : la complétude médiane.
« Pistes de référence » : les pistes du run de référence, celles que verrait la naturaliste.
Le banc ne compte comme fausses que les pistes absentes de la référence ; le bruit présent dans les deux runs n'y apparaît pas, d'où cette colonne.
« Mêmes cibles » : les cibles tirées sont identiques à celles du réglage par défaut (le placement suit les pistes de référence de chaque réglage) ; sinon, la ligne ne se lit qu'en ordre de grandeur.

### Résolution de travail et `min_area`

Avec le réglage d'alors (480 px, `min_area` 18, seuil fixe 25), 251 cibles sont évaluées sur 270 tirées (19 jamais visibles assez longtemps).
Il faut un contraste effectif d'environ −45 niveaux **et** une tache d'au moins σ = 3 px d'origine : à −46 et σ 1,5, une seule cible sur 28 est trouvée.
La chauve-souris de 3:57 à son arrivée (−30 à −70 en pleine résolution, petite) tombe exactement dans cette zone ([07](07-ameliorer-la-detection.md#la-piste-14-357)).

| Réglage | −46, σ 1,5 | −46, σ 3 | −24, σ 3 | −24, σ 5 | Pistes de référence | Fausses pistes | Temps |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 480 px, `min_area` 18 | 1/28 | 24/26 (0,86) | 0/26 | 15/27 (0,10) | 14 | 0 | 1 min 59 |
| 720 px, `min_area` 18 | 0/28 | 24/26 (0,87) | 0/26 | 17/27 (0,20) | 13 | 0 | 2 min 42 |
| 960 px, `min_area` 18 | 0/28 | 24/26 (0,87) | 0/26 | 19/27 (0,26) | 12 | 0 | 4 min 36 |
| **480 px, `min_area` 4** | **26/28 (0,62)** | 24/26 (0,91) | **10/26** | **26/27 (0,39)** | **14** | **0** | 1 min 58 |
| 720 px, `min_area` 4 | 21/25 (0,80) | 24/26 (0,87) | 15/24 (0,30) | 18/23 (0,54) | **7 652** | 318 | 2 min 49 |
| 960 px, `min_area` 4 | 25/27 (0,87) | 25/26 (0,91) | 18/26 (0,30) | 26/27 (0,56) | **498** | 7 | 4 min 34 |

À 720 px avec `min_area` 4, le bruit a gêné le tirage (43 cibles jamais visibles au lieu de 19) : ce jeu de cibles n'est pas le même.
Le run en pleine résolution (1440 px) a été arrêté après 25 minutes sans résultat.

- **C'est `min_area` qui bloquait, pas la résolution.**
  À 480 px, l'abaisser de 18 à 4 px² fait passer les petites cibles contrastées de 1 à 26 sur 28, et les grosses peu contrastées de 15 à 26 sur 27, sans piste de bruit en plus (14 pistes de référence).
- **La réduction à 480 px est un débruitage**, puisque chaque pixel de travail moyenne 3×3 pixels d'origine.
  À plus haute résolution, le même seuil de 25 laisse passer le bruit, en centaines ou en milliers de pistes.
  À 720 px, `min_area` 4 px² vaut 1 pixel de travail, donc un pixel isolé suffit ; à 960 px, il en faut 2, d'où moins de bruit qu'à 720.
- **La haute résolution apporte de la complétude** : à 960 px, 0,87 contre 0,62 sur les petites cibles fortes, 0,56 contre 0,39 sur les grosses faibles.
  Elle n'est exploitable qu'avec un seuil adapté au bruit local (D1) et un filtrage à la taille de la cible (B2), qui remplace le débruitage de la réduction.

### Sans masque d'affichage (4 octobre 2026)

480 px, `min_area` 4, visibilité sans profil de caméra, les deux lignes mesurées avec le même code :

| Réglage | −46, σ 1,5 | −46, σ 3 | −24, σ 5 | Fausses pistes | Pistes de référence |
| --- | --- | --- | --- | --- | --- |
| Anciennes bandes (`--osd-region 0,0,1,0.07 --osd-region 0,0.9,1,1`) | 26/28 (0,59) | 24/26 (0,86) | 26/27 (0,37) | 0 | 14 |
| **Sans masque (défaut)** | **27/28 (0,66)** | 24/26 **(0,95)** | 26/27 **(0,44)** | 0 | 14 |

La vitesse estimée sur 3 images ne change rien de mesurable au banc (une seule complétude bouge de 0,01) ; sur la vraie vidéo, elle prolonge de 2 images la piste la plus rapide.

### Vols de chasse (4 octobre 2026)

Banc dédié sur la 092 : `--motions hunt circle --amplitudes -60 -30 --sigmas 1.5 3 5 --per-class 60`, 34,7 s.
720 cibles sont tirées (12 classes × 60), et 592 sont visibles.
Les 128 autres ne quittent jamais les coins noirs de l'oculaire : une chasse courte qui entre près d'un coin n'atteint pas le disque visible, et il reste en pratique environ 49 cibles par classe.
Avec `min_hits` 5, 371 sont trouvées, sans fausse piste.
Comme en transit, la classe −30, σ 1,5 est invisible et −30, σ 3 difficile.

Comparaison appariée sur les mêmes cibles : cibles trouvées avec le réglage d'alors et perdues avec le nouveau.

| Réglage de suivi | Transit (115) | Chasse (371) |
| --- | --- | --- |
| virage médian ≤ 0,6 rad | −1 | −1 |
| virage médian ≤ 0,8 ou 1,0 rad | 0 | 0 |
| `min_hits` 6 | 0 | −14 |
| `min_hits` 7 | −1 | −24 |

Les cibles perdues quand `min_hits` monte sont surtout peu contrastées (−30) : visibles une quinzaine d'images, elles sont détectées sur 5 ou 6.

Retenu le 4 octobre : `max_median_turn` 0,8 et `min_hits` 6.
Rejoués avec ces réglages, le transit trouve 115 cibles sur 253 visibles et la chasse 357 sur 584 (136 non visibles), sans fausse piste.
Si `min_hits` passe à 6 pour le suivi seul, le critère « trouvée » restant à 5, la chasse perd 11 cibles au lieu de 14.

### Seuil par pixel (D1)

5 octobre 2026, 480 px, transit.
`threshold` est le plancher, `noise_factor` (k) le relèvement local ([02](02-methode.md#3-seuil-dans-les-deux-sens)) ; k = 0 est le seuil fixe d'alors.
Les classes à −15 ne sont trouvées par aucun réglage, sauf aux seuils 15 et 12, où le bruit domine (colonne −15).
Les temps sont ceux d'une première version qui calculait la carte de bruit sur toute l'image.

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

Banc de chasse (`--motions hunt circle --amplitudes -60 -30 --per-class 60`), cibles trouvées sur les visibles, toutes classes :

| Seuil, k | Trouvées | −30, σ 3 (hunt / circle) | −30, σ 5 (hunt / circle) | Pistes de réf. | Fausses | Temps |
| --- | --- | --- | --- | --- | --- | --- |
| 25, 0 (défaut) | 357 / 584 | 14/48 · 12/48 | 38/51 · 38/48 | 14 | 0 | 34 s |
| 18, 0 | 459 / 579 | 40/48 · 41/48 | 49/51 · 43/47 | 91 | 6 | 34 s |
| 18, 6 | 448 / 584 | 42/48 · 41/48 | 49/51 · 44/48 | 15 | 0 | 156 s |
| 18, 8 | 430 / 584 | 39/48 · 36/48 | 48/51 · 44/48 | 14 | 0 | 156 s |

Sur la 092, avec la commande complète :

- **18 et 8** garde les 14 pistes, sans piste en plus.
  0:51 tombe de 9 à 6 détections, **exactement `min_hits`** : une détection de moins et elle disparaît.
- **18 et 6** sort 15 pistes. La nouvelle (2:45.02 → 2:45.38, 9 détections) chevauche la fin de 2:44, qui passe de 44 à 39 détections.
  Pour l'utilisateur (5 octobre), ce n'est pas un second animal mais la chauve-souris de 2:43.95, sortie en double.
  La fusion des jumelles n'a pas été modifiée pour autant (#26).
- **18 et 0** sort 91 pistes.

Ce qu'on en tire :

- **Le gain vient de la baisse du seuil fixe, et D1 la rend utilisable.**
  À 18 sans relèvement, les cibles −30, σ 3 passent de 10 à 26 sur 27, mais la vraie vidéo donne 91 pistes et le banc 6 fausses.
  Avec k = 8, 22 sur 27 restent trouvées, et la vidéo revient à ses 14 pistes, sans fausse piste.
- **À seuil 25, D1 ne fait que perdre**, puisqu'il ne peut que relever le seuil : à k = 8, une piste de référence disparaît.
- **Sous 18, le plancher ne tient plus** : à 15 ou 12, même k = 8 laisse des dizaines à des milliers de pistes de bruit.
  La MAD sur 11 images est trop instable pour servir seule.
- **Coût** : 2 médianes de plus par image, en flottants.
  Calculé sur toute l'image, le banc passe de 32 à 155 s, et de 224 à 343 s de CPU, avec 50 Mo de plus.
  Le bruit n'est utile que sur les pixels dont le résidu dépasse déjà le plancher, puisque les autres ne peuvent pas être retenus.
  Calculé sur eux seuls, avec les mêmes détections au bit près, le banc à 18 et 8 prend 45 s et 236 s de CPU (machine chargée).
- Les petites cibles faibles (−30, σ 1,5) restent introuvables à 480 px sans bruit en plus : c'est le rôle du filtre à la taille de la cible et de la pleine résolution (B2 et B1 dans [07](07-ameliorer-la-detection.md)).

### Filtre à la taille de la cible (B2)

5 octobre 2026, transit, `min_area` 4 sauf mention.
σ est `--target-sigma` en pixels d'origine, k est `--noise-factor`.
« À la limite » : pistes de référence qui ont exactement `min_hits` détections.
Les temps marqués * ont été pris sous charge, deux bancs à la fois, et les autres un banc à la fois ; les comptes ne dépendent pas de la charge.

Témoins, et chaque changement pris seul :

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

Le réglage 960 px, σ 1, seuil 8, k 0 a été arrêté par le garde-fou après 12 minutes : son voisin au seuil 12 donnait déjà 1 408 pistes de référence.

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

Banc de chasse (`--motions hunt circle --amplitudes -60 -30 --per-class 60`), cibles trouvées sur 584 visibles, sans fausse piste nulle part.
Le temps de D1 est celui de la première version de la carte de bruit.

| Réglage | Trouvées | −30, σ 1,5 (hunt · circle) | −30, σ 3 | Pistes de réf. | Temps |
| --- | --- | --- | --- | --- | --- |
| 480 px, défaut | 357 | 0/48 · 0/53 | 14/48 · 12/48 | 14 | 34 s |
| 480 px, seuil 18, k 8 (D1) | 430 | 0/48 · 3/53 | 39/48 · 36/48 | 14 | 156 s * |
| **C1** : 960 px, σ 1,5, seuil 12, k 0 | 510 | 30/48 · 28/53 | 46/48 · 43/48 | 14 | 234 s |
| **C2** : 1440 px, σ 1,5, seuil 12, k 0 | 525 | 38/48 · 34/53 | 46/48 · 43/48 | 16 | 533 s |
| **C3** : 960 px, σ 1,5, seuil 12, k 8 | 456 | 3/48 · 6/53 | 45/48 · 42/48 | 14 | 243 s |
| **C4** : 960 px, σ 2, seuil 8, k 0 | 540 | 43/48 · 43/53 | 46/48 · 45/48 | 18 | 238 s |
| **C5** : 1440 px, σ 1,5, seuil 12, k 8 | 470 | 11/48 · 11/53 | 45/48 · 41/48 | 14 | 556 s |

Sur la 092, avec la commande complète, les trois candidats suivent toujours 3:36 et 3:58.
Écart de début en images (négatif : plus tôt) et écart de détections, par rapport au réglage d'alors :

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

Ce qu'on en tire :

- **Le filtre apporte le plus gros gain mesuré, mais il ne paie qu'avec la résolution.**
  À 960 px, σ 1,5 et seuil 12 sans carte de bruit (C1), les petites cibles faibles (−30, σ 1,5) passent de 0 à 14 sur 28.
  Les −30, σ 3 passent de 10 à 26, avec une complétude de 0 à 0,98, et 26 cibles à −15 apparaissent, pour 14 pistes de référence et 1 fausse piste.
  Le même filtre à 480 px donne 33 pistes de référence, dont 9 à la limite.
- **La résolution seule** (960 px sans filtre) laisse passer le bruit : 118 pistes de référence, 5 fausses.
  **D1 seul** à 960 px gagne peu : 16 pistes de référence, dont 1 à la limite, et aucune fausse.
- **Filtre et seuil par pixel ensemble** (C3) : aucune fausse piste, mais la carte de bruit reprend l'essentiel du gain sur les petites cibles faibles (−30, σ 1,5 : 1 sur 28 ; chasse 456 contre 510).
  Le filtre fait déjà le travail de débruitage.
- **1440 px** (C2) gagne encore un peu (−30, σ 1,5 : 18 sur 28 ; chasse 525), pour deux fois le temps de 960 px, et sort deux pistes nouvelles sur la 092.
- **σ et seuil vont ensemble**, car le filtre abaisse le pic d'une petite tache d'autant plus que σ est grand.
  σ 1 laisse passer le bruit sans carte (1 408 pistes de référence au seuil 12).
  σ 2 perd les petites cibles faibles au seuil 12 (0 sur 28).
  Au seuil 8, il donne l'un des meilleurs résultats (−30, σ 1,5 : 23 sur 28, −15 : 56), pour 18 pistes de référence, dont 2 à la limite, et aucune fausse.
  σ 1,5 au seuil 12 est le meilleur réglage qui garde 14 pistes de référence.
  Au seuil 8, σ 1,5 explose sans carte (4 078) ; au seuil 16, les cibles faibles sont perdues.
- **`min_area`** 9 ou 18 perd les petites cibles faibles (14 → 1 ou 0 sur 28) et ne retire qu'une fausse piste (1 → 0) : il reste à 4.
- **Coût** : la médiane sur des images en flottants, et le flou.
  Le banc passe de 32 s à 4 min à 960 px, et à 9 min à 1440 px ; la 092, de 30 s à 135 s (960 px) ou 283 s (1440 px).

#### Sur six autres vidéos

Les quatre candidats et le défaut d'alors ont été passés sur la 092 et sur six autres vidéos sans erreur de décodage : 089, 090, 091, 125, 126 et 127 (comparaison détaillée hors dépôt).

Avec un seuil fixe abaissé et sans seuil par pixel (C1, C2, C4), la 089, la 090 et la 091 donnent des centaines à des milliers de pistes (C1 : 560, 374, 394 ; C4 : 5 479, 4 925, 3 974).
La 092 et la 125 restent propres.
Avec C1, ces trois vidéos ont en permanence 3 à 5 taches de bruit par image, contre 0 sur la 092 et la 125.
Ces taches font un pixel de travail, leur amplitude est juste au-dessus du seuil, et le suivi les enchaîne en fausses pistes.
Les mouvements de jumelles n'y sont pour rien : la 090 n'en a aucun.
C3, avec le seuil par pixel, garde une médiane de 0 tache par image, et sort 1, 20 et 21 pistes sur ces trois vidéos (4, 12 et 23 avec le défaut d'alors).
**Un seuil fixe abaissé ne se règle pas sur une seule vidéo calme** : le banc, fait sur la 092, ne le montrait pas.

C5, c'est-à-dire C3 en pleine résolution (1440 px), tient aussi sur les sept vidéos (14, 1, 17, 22, 48, 10 et 7 pistes), pour plus du double du temps de C3.
Ni C3 ni C5 ne trouvent le passage réel de 3:42.10 sur la 092.
La cible n'y dépasse le plancher que de 1 à 10 niveaux, et le seuil par pixel la perd sur 2 de ses 6 images, où 8σ vaut 22 à 36 : elle tombe alors sous `min_hits`.
C1 la perd autrement : à 960 px, une de ses images n'a qu'un pixel de travail au-dessus du seuil, sous `min_area`.

### Décision du 6 octobre 2026

C3 devient le réglage par défaut : 960 px, filtre de 1,5 px, seuil 12, facteur de bruit 8.
L'utilisateur a regardé une à une les six pistes de l'ancien réglage que C3 ne sort plus, et les a toutes jugées fausses ; il a validé comme réelles les pistes 8 et 9 de C3 sur la 125 ([03](03-resultats-092.md#cibles-connues-pour-les-issues-suivantes)).

Sur les sept vidéos :

| | 092 | 089 | 090 | 091 | 125 | 126 | 127 | Total | Calcul (réel / CPU) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Ancien réglage | 14 | 4 | 12 | 23 | 18 | 4 | 0 | 75 | 4 / 20 min |
| **C3 (défaut)** | 14 | 1 | 20 | 21 | 47 | 12 | 7 | 122 | 16 / 45 min |
| C5 (C3 à 1440 px) | 14 | 1 | 17 | 22 | 48 | 10 | 7 | 119 | 38 / 67 min |
| C1, C2, C4 (sans seuil par pixel) | 14 à 18 | 560 à 5 479 | 374 à 4 925 | 394 à 3 974 | 48 à 57 | 11 à 13 | 7 à 13 | | |

Bancs refaits avec les nouveaux défauts, identiques clé par clé à ceux de C3 : en transit, 148 cibles trouvées, 14 pistes de référence et aucune fausse piste (260 s, 640 s de CPU) ; en chasse, 456 sur 584 et aucune fausse piste (254 s).
Ce sont les nombres de cibles trouvées par la v0.1.1 que reprend la garde de `--against` ([11](11-migration-rust.md#garde)).
Le passage réel de 3:42.10 sur la 092 reste manqué ([03](03-resultats-092.md#cibles-connues-pour-les-issues-suivantes)).
