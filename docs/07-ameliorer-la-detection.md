# Améliorer la détection : diagnostic et pistes

Document d'exploration du 3 octobre 2026. Les chiffres viennent de mesures sur
`video_092` (réglages par défaut, 12 pistes), dans sa **version exportée** par Stream
Vision 2 (1 Mb/s). L'original (12 Mb/s, voir A0) améliore une partie de ces constats :
ils sont à refaire dessus. Ce qui n'a pas été mesuré est marqué
comme hypothèse. Les numéros de piste sont ceux de [03-resultats-092.md](03-resultats-092.md).

## Le symptôme

Une naturaliste voit la chauve-souris avant que la piste ne commence et après
qu'elle s'arrête. Pendant la piste, la boîte clignote. Le tracé devrait être continu
d'un bord à l'autre de son passage visible.

## Ce que disent les mesures

### Le cas de la piste 12, vu par l'œil

La capture de l'utilisateur montre un point sombre dans la bande de végétation au-dessus
de la falaise, à environ (1084, 315) en pixels d'origine. En cherchant le minimum du
résidu (image moins fond médian, lissé par une gaussienne de σ = 2 px) :

| Image | Temps | Position (travail) | Résidu pleine résolution | Résidu à 480 px |
| --- | --- | --- | --- | --- |
| 7116 | 3:57.20 | (356, 97) | −24 | −18 |
| 7117 | 3:57.23 | (353, 102) | −35 | −19 |
| 7118 | 3:57.27 | (345, 109) | −34 | −21 |
| 7119 | 3:57.30 | (326, 123) | −67 | −22 |
| 7120 | 3:57.33 | (326, 124) | −33 | −11 |
| 7121 | 3:57.37 | (308, 138) | −18 | −10 |
| 7122 | 3:57.40 | (292, 154) | — | −70 : **début de la piste** |

La chauve-souris est là **6 images avant** le début de la piste. Trois causes
s'additionnent :

1. **La réduction à 480 px dilue le point.** Elle perd environ un tiers de son contraste
   (−24 à −35 en pleine résolution, −18 à −22 à 480 px) et passe sous le seuil de 25.
2. **Le fond est peu contrasté à cet endroit.** La végétation est plus sombre que la
   roche, donc l'écart avec la chauve-souris y est plus faible.
3. **La trajectoire est courbe et accélère.** Elle passe d'environ 4 à 20 px par image en
   piquant vers la gauche. Une extrapolation à vitesse constante depuis le début de la
   piste se trompe de 30 px dès 4 images en arrière. C'est pourquoi la recherche en ligne
   droite faite plus tôt n'avait rien trouvé avant l'image 7121.

Avant 7116, rien ne dépasse le bruit : elle sort de la végétation ou du feuillage de
l'arbre.

En sortie, à gauche, le résidu tombe à **exactement 0** deux images après la fin. Elle
passe devant le ciel saturé, où l'image ne contient plus aucune information.

### Les extrémités de toutes les pistes

Pour chaque piste, la trajectoire a été prolongée à vitesse constante (estimée sur 3
points) de 1 à 5 images avant et après. On a cherché le pic de résidu filtré (σ = 1)
dans un rayon de 8 px, et on l'a comparé au 99e centile des pics dans des fenêtres
tirées au hasard (z = 14,9). Images consécutives au-dessus de ce seuil :

| Piste | Avant | Après | Remarque |
| --- | --- | --- | --- |
| 1 | 1 | 1 | |
| 2 | 3 | ≥ 4 | finit contre la bande masquée du bas |
| 3 | entre par le bord droit | 2 | finit contre la bande du bas |
| 4 | 1 | 1 | |
| 5 | 0 | 1, puis sort du champ | |
| 6 | 1 | 0 | |
| 7 | 0 | 0 | |
| 8 | 1 | ≥ 4 | finit contre la bande du bas |
| 9 | 1 | ≥ 4 | objet lent, il reste en place |
| 10 | 3, jusqu'au bord | 0 | entre par le haut |
| 11 | 0 | 3, jusqu'au bord | finit contre la bande du bas |
| 12 | 1 (6 en suivant la courbe) | 1, puis ciel saturé | |

Ces chiffres sont des ordres de grandeur. L'extrapolation est rectiligne et le seuil
n'est pas validé. Mais la tendance est nette : **la plupart des pistes peuvent gagner 1 à
4 images à chaque bout**, et les plus gros gains sont aux bords de la bande masquée du bas.

### Les trous à l'intérieur des pistes

Sur 12 pistes, 10 ont des trous, de 1 à 5 images, pour un total de 49 images manquantes
sur 502. La boîte disparaît pendant ces images.

### L'affichage incrusté

Sur 9 000 images, on a compté combien de fois chaque pixel dépasse le seuil :

- seuls 18 pixels dépassent 100 fois, tous dans la ligne du compteur et de l'horloge
  (y = 11 à 15 en pixels de travail) ;
- la bande du bas ne s'active presque jamais (972 dépassements en tout, contre 6 554
  pour la bande du haut) ;
- la piste lente (n° 9) s'active fortement, mais sous 100.

**La bande du bas, 10 % de la hauteur, masque donc du vide sur la 092**, et elle coupe
cinq pistes.

### La compression

- La structure est I BBBP, avec une image-clé toutes les 29 images : 72 % de B-frames,
  24 % de P et 3 % d'I.
- Détection dans les pistes, sans la n° 9 : I 6/8, P 45/53 (85 %), B 119/158 (75 %).
  L'écart va dans le sens de l'hypothèse « les B-frames effacent les petits objets »,
  mais l'échantillon est trop petit pour conclure.
- **Le fond est figé au bit près.** Pour la plupart des pixels, la médiane de la valeur
  absolue du résidu vaut 0. Le codec recopie les blocs immobiles à l'identique. Le bruit
  n'est donc pas un bruit de capteur gaussien : ce sont des mises à jour de blocs.
  Conséquence : un écart-type par pixel tombe à 0, et un seuil en z-score doit avoir un
  plancher.

### Saturation

19,7 % des pixels valent exactement 0, presque tout le ciel. Aucune détection n'y est
possible : la chauve-souris y est saturée elle aussi.

### Résolution

Le capteur des Symbion DXT50 fait 1280×1024 : la vidéo à 1440×1080 est à peu près à la
résolution native. Le spectre d'une image ne montre d'ailleurs pas de coupure nette. La
réduction à 480 px de large jette donc environ 2,7 fois la résolution du capteur, ce qui
est cohérent avec la perte de contraste mesurée sur la piste 12.

### Cadence saccadée

Le capteur tourne à 50 Hz, la vidéo est à 30 i/s. Sur les pistes rapides, le pas entre
deux images consécutives suit un motif de période 3 : un pas double puis deux pas simples.
Piste 4 : 18,9 · 9,5 · 8,7 · 17,9 · 9,7 · 9,8 · 18,9 · 10,1… Aucune image n'est dupliquée
à l'identique. La phase du motif varie d'une piste à l'autre (pas doubles à l'image
mod 3 = 0, 1, 2 : 13, 19 et 9 cas sur environ 45). La cause exacte de la conversion
interne n'est pas connue.

Conséquences :

- une prédiction sur les deux derniers points se trompe d'un facteur 2 une image sur
  trois. Corrigé le 4 octobre : la vitesse est estimée sur au moins 3 images (E1 bis) ;
- l'interpolation à l'affichage reste linéaire : elle peut s'écarter d'un pas au plus de
  la position réelle pendant un trou, ce qui est sans conséquence pour une boîte.

## Diagnostic refait sur l'original (étape 0)

Mêmes mesures, sur `video_092_original.mp4` (12,1 Mb/s, sans B-frames), avec les
réglages par défaut : 14 pistes.

| Constat | Export (1 Mb/s) | Original (12 Mb/s) |
| --- | --- | --- |
| Images manquantes dans les pistes rapides | 49 sur 219 (22 %) | 39 sur 272 (14 %) |
| Pistes avec des trous | 10 sur 12 | 12 sur 14, trous de 1 à 4 images |
| Pistes qui finissent contre un masque ou un bord (≤ 10 px) | 5 | 6 (#1, #2, #3, #4, #10, #13) |
| Extrémités récupérables (prolongation rectiligne) | 1 à 4 images | toujours 1 à 5 images : #2 et #4 +5 après, #6 +4 avant, #14 +3 avant |
| Détection par type d'image | B 75 %, P 85 % | P 87 %, I 78 % (pas de B) |
| Saccade de période 3 | oui | **oui** : piste 5, 8,7 · 18,7 · 9,9 · 8,8 · 17,7 · 9,6 · 9,2 · 19,6… |
| Pixels dont le résidu est exactement figé (MAD = 0) | presque tous | 36 % : le bruit du capteur réapparaît |
| Ciel saturé à 0 | 19,7 % | 21,2 % |
| Activité de l'affichage incrusté | 18 pixels au-dessus de 100 dépassements | **aucun** ; 1 dépassement dans la bande du haut, 301 dans celle du bas |

**Piste 14 (3:57).** La chauve-souris apparaît à l'image 7122 de l'original (3:57,16), au
même endroit que sur la capture de l'utilisateur. La piste ne commence qu'à 7127. Sur
les cinq premières images, le résidu vaut −42, −31, −30, −71 et −59 en pleine résolution,
mais −25, −26, −22, −30 et −30 à 480 px. **La réduction de l'image coûte toujours 30 à
50 % du contraste, et c'est elle qui retarde le début de la piste.** Avant 7122, rien
ne dépasse le bruit : elle sort de la végétation à ce moment-là.

**Ce qui change dans les priorités :**

- **L'original règle une partie des trous**, mais pas les extrémités ni la saccade.
  L'interpolation (E1), la vitesse sur 3 images (E1 bis) et la seconde passe ou
  l'hystérésis (D2, D3) restent utiles.
- **La pleine résolution (B1) remonte** : c'est la cause mesurée du début tardif de la
  piste 14.
- **Le masque d'affichage (F1) devient plus simple.** Sur l'original, l'affichage ne
  déclenche presque rien ; la raison n'est pas comprise, peut-être un rendu de l'affichage
  différent entre l'original et l'export. Les bandes fixes pourraient être réduites au
  strict nécessaire, ce qui libérerait les 6 pistes coupées. À vérifier sur d'autres
  vidéos.
- **Le bruit n'est plus un fond figé** : un seuil par pixel (D1) devient pertinent, et
  il n'a plus besoin d'un plancher artificiel sur un tiers des pixels.

## Ce que fait la littérature

**Le contraste négatif s'explique par la physiologie.** En vol, les ailes ne sont qu'à
environ 1 °C au-dessus de l'air. À distance, ce sont elles qui occupent les pixels, alors
que la roche garde la chaleur de la journée. Une chauve-souris lointaine devant une roche
chaude est donc sombre, et le contraste peut s'annuler au crossover thermique.

**Les pipelines publiés et ce qu'on en retient :**

| Pipeline | Détection | Ce qu'on retient |
| --- | --- | --- |
| ThermalTracker (PNNL, Matzner 2015) | maximum par pixel sur une traversée, seuil μ + 3σ par pixel | seuil par pixel ; ne voit que les cibles chaudes |
| Boston University (Betke 2008) | écart à la moyenne par pixel, en fraction de σ | seuil fixe écarté : près de la végétation, la cible a l'intensité du fond |
| Brevet US Army (Sabol & Melton 2009) | fond = mode de l'histogramme par pixel, puis valeur absolue de la différence | le plus proche de notre cas : c'est la valeur absolue qui rend la détection indifférente au signe ; la prédiction tient compte du nombre d'images manquées |
| BatCount (Bentley 2023) | médiane locale, filtre sur les objets plus clairs ou plus sombres | 51 à 95 % selon la caméra, toujours en sous-comptage |
| ThruTracker (Corcoran) | soustraction de fond, lissage gaussien et dilatation réglés sur le plus petit objet | mêmes réglages que les nôtres : écart maximal, longueur minimale de piste |
| Yarbrough et al. (2023-2026) | MOG2 volontairement permissif, puis tri des imagettes par un CNN | détecter large, puis trier : 99 % entre biologique et non biologique |

**Validation dans ces études :** segments de 30 s tirés au hasard et comptés par deux
personnes (BatCount, avec au moins 96,5 % d'accord) ; correction du comptage par
(N − faux positifs) ÷ taux de détection (Matzner) ; et une recommandation non suivie de
Matzner : mesurer la détection en fonction du contraste et de la taille. **Aucun
pipeline ne fait d'injection de cibles synthétiques** : ce serait notre apport (G2).

**Battements d'ailes :** à 30 i/s, la limite de Nyquist (15 Hz) est au niveau de la
fréquence de battement (10 à 15 Hz pour Tadarida, environ 11 Hz pour une pipistrelle).
Il faudrait du 50 i/s et des cibles de plus de quelques pixels. Pour distinguer
chauve-souris et oiseau, la forme de la trajectoire suffit mieux : kurtosis et
changement de direction, 82 % de bonne classification (Cullinan 2015).

## Les pistes

Chaque piste indique le symptôme visé, ce qu'on en attend, le coût, et les références.
Les sources détaillées sont en fin de document.

### A. En amont : la prise de vue

- **A0. Copier les originaux en USB au lieu d'exporter par Stream Vision 2.** Mesuré sur
  la 092 : 12 fois plus de débit, plus de B-frames, +31 % de points détectés sur les
  passages communs, et 2 passages nouveaux (voir [03-resultats-092.md](03-resultats-092.md)).
  **C'est de loin le plus gros gain mesuré, et il est déjà acquis.**

Réglages des Symbion DXT50 à proposer à Manon, d'après le manuel. Chacun est à
valider par un essai comparatif (A8).

- **A1. Compression vidéo sur OFF** (menu Media) : « compression minimale, qualité
  supérieure, fichiers beaucoup plus volumineux ». Nos vidéos sont à environ 1 Mb/s,
  donc probablement sur ON. C'est le levier le plus direct contre les blocs et les B-frames écrasées.
- **A2. Gain et contraste manuels** plutôt que l'AGC, réglés pour que le ciel ne sature
  pas. Le ciel noir saturé est une zone aveugle.
- **A3. Cadrage** : garder la zone de passage devant la roche, là où le contraste est
  fort, et limiter le ciel et la végétation sombre.
- **A4. Rester au grossissement de base (2×)**, sans zoom numérique : c'est déjà le cas
  sur nos vidéos.
- **A5. Calibrage semi-automatique**, déclenché juste avant d'enregistrer. En automatique,
  l'image se fige environ une seconde à chaque calibrage, et un passage peut tomber
  dedans.
- **A6. Stabilisation d'image sur OFF** sur trépied. Elle est faite pour la main levée ;
  si elle agit aussi sur l'enregistrement, elle peut déplacer le fond. À vérifier.
- **A6 bis. Filtre de lissage sur OFF**, à tester : il « rend l'image plus lisse et plus
  uniforme », et risque d'effacer les petits points. **Amplification** Normal, Haut,
  Ultra : effet inconnu sur des cibles de quelques pixels, à comparer.
- **Luminosité et contraste fixes** entre les sessions (option « Enregistrer les
  paramètres de l'image thermique ») pour que les vidéos restent comparables. Leur
  effet sur l'enregistrement n'est pas décrit dans le manuel.
- **A7. Commencer l'enregistrement au début de l'émergence**, car le fond change de
  température autour du coucher du soleil (Ahlberg 2025).
- **A8. Essai comparatif** : même scène, même soir, trois enregistrements de 5 min :
  réglages actuels, compression OFF, compression OFF + lissage OFF. On compare le débit,
  le nombre de pistes, la complétude, et le bruit de fond mesuré par l'outil.

Coût nul, gain potentiellement le plus fort. À valider avec les réglages réels des
jumelles.

### B. Prétraitement de l'image

- **B1. Travailler en pleine résolution, ou plus près.** La mesure ci-dessus montre que
  la réduction à 480 px coûte un tiers du contraste des petits points. On peut passer
  `work_width` à 720 ou 1440. Le temps de calcul sera multiplié par environ 2,25 ou 9 ;
  ou bien on ne passe en pleine résolution qu'autour des pistes (voir D3).
- **B2. Filtrage adapté** (gaussienne, LoG ou DoG à l'échelle de la cible) sur le
  résidu signé, à la place ou en plus de la fermeture morphologique. C'est l'optimum
  théorique pour une tache dans du bruit. Coût négligeable.
- **B3. Débloquage spatial** (`deblock`, `spp` ou `pp7` d'ffmpeg) avant la détection,
  pour réduire les fragments dus aux blocs. **Pas de débruitage temporel** (`hqdn3d`,
  `atadenoise`) : il efface précisément les petits objets rapides.
- **B4. Restauration par réseau** (MFQE 2.0, BasicVSR++) : entraînés sur de la vidéo
  naturelle en couleur, ils risquent d'effacer ou d'inventer des petits points. À
  écarter sauf preuve contraire.
- **B5. Stabilisation** (corrélation de phase ou ECC) avant la détection, pour les
  vidéos où les jumelles bougent, comme la 027.

### C. Modèle de fond

- **C1. Médiane avec exclusion** : fond calculé sur [t−15, t−3] ∪ [t+3, t+15]. Un objet
  lent (piste 9) ne contamine plus son propre fond.
- **C2. Différence à 3 images** : min(|It − It−k|, |It − It+k|). Elle est quasi gratuite
  et adaptée aux cibles qui se déplacent de plus que leur taille. On peut la combiner
  avec la médiane.
- **C3. MOG2 / KNN / ViBe** (OpenCV, pybgs) : c'est ce qu'utilise ThruTracker. Mais leur
  sortie est binaire : on perd le signe et l'amplitude du résidu, dont on a besoin pour
  le reste.
- **C4. RPCA / GoDec** (fond de rang faible + cible creuse) : coûteux, et le gain sur
  une scène fixe paraît faible face à une médiane bien faite. Hypothèse non mesurée.

### D. Détection

- **D1. Seuil par pixel** : résidu ÷ bruit local (MAD temporelle), avec un plancher
  puisque le fond est figé au bit près. Masquer explicitement le ciel saturé. Les zones
  calmes (roche) peuvent alors descendre plus bas que 25, et les zones agitées
  (végétation au vent) restent protégées.
- **D2. Hystérésis 3D (x, y, t)** : on garde un pixel faible (seuil bas, par exemple 12)
  s'il est relié dans l'espace **ou dans le temps** à un pixel fort (seuil haut, 25).
  C'est exactement le mécanisme qui récupère les débuts et fins de piste et comble les
  trous. `skimage.filters.apply_hysteresis_threshold` accepte des seuils par pixel.
  Coût : il faut garder un petit volume de résidus en mémoire, quelques images.
- **D3. Seconde passe guidée** : autour de chaque extrémité de piste, relire la vidéo en
  pleine résolution et chercher avec un seuil abaissé dans une porte qui **s'élargit**
  avec la distance à la dernière détection, pour tolérer les virages et les
  accélérations (piste 12). Coût : environ 60 images relues par piste.
- **D4. Track-before-detect local** : additionner le résidu normalisé le long de
  trajectoires candidates sur plusieurs images, et accepter si la somme dépasse un seuil
  qui croît en √N. Un point trop faible sur une image devient visible par cohérence de
  mouvement : c'est ce que fait l'œil. On pourrait tester un éventail de vitesses et de
  courbures autour de la vitesse connue. Aucune bibliothèque Python maintenue : environ
  100 lignes de NumPy.
- **D5. Contraste local multi-échelle signé** (MPCM, top-hat blanc et noir), issu de la
  littérature infrarouge sur les petites cibles. MPCM rehausse les cibles sombres comme
  les claires. Alternative à D1-D2 si le résidu au fond ne suffit pas.
- **D6. Réseaux de neurones** pour petites cibles infrarouges (DNANet, SCTransNet, DTUM
  en multi-images), via BasicIRSTD. Ils sont entraînés sur des scènes militaires et
  aériennes non compressées, avec des cibles chaudes : sans ré-entraînement sur nos
  vidéos annotées, le transfert n'est pas garanti. Option longue.

- **D7. Détecter large, puis trier** : seuil bas, puis classement des imagettes ou des
  pistes (forme de trajectoire, taille, vitesse) par un classifieur. C'est l'approche de
  Yarbrough et al. Les validations de Manon serviraient d'exemples d'entraînement.

### E. Suivi

- **E1. Interpolation dans les trous** : une position à chaque image entre deux
  détections, marquée comme interpolée. Supprime le clignotement. Aucun risque.
- **E1 bis. Vitesse estimée sur une période de saccade** (3 images au moins) au lieu des
  deux derniers points. C'est simple, et ça corrige l'erreur de facteur 2 décrite plus haut.
- **E2. Filtre de Kalman + lissage RTS** à la place de la prédiction sur deux points :
  la porte de recherche grandit avec l'incertitude pendant un trou, et les trous sont
  interpolés de façon cohérente avec la dynamique.
- **E3. Suivi bidirectionnel** : relancer le suivi sur la vidéo à l'envers. Un début de
  piste devient une fin, prédite par une vitesse déjà bien estimée. Combiné à D2 ou D3,
  c'est la manière la plus naturelle d'étendre les débuts.
- **E4. Recollage de tracklets** : relier deux pistes dont les extrapolations se
  rejoignent au-delà de `max_gap` (LapTrack, ou min-cost flow). C'est l'approche du
  groupe de Betke pour les chauves-souris.
- **E5. Extrapolation au bord**, affichée en pointillés et hors statistiques : quand une
  piste s'arrête à moins de ~0,3 s du bord ou entre dans le ciel saturé, prolonger
  jusqu'à la sortie. C'est le seul recours quand l'image ne contient plus d'information,
  comme pour la sortie de la piste 12.

### F. Masques et zones

- **F1. Masque d'affichage automatique** : masquer les pixels qui dépassent le seuil plus
  de N fois sur la vidéo, après une dilatation de quelques pixels. Mesuré : sur la 092,
  il ne retiendrait que la ligne du compteur et de l'horloge, et libérerait la bande du
  bas.
- **F2. Masque de saturation** : les zones à 0 ou 255 sont aveugles. Une piste qui y
  entre est « perdue dans le ciel » : c'est ce qui justifie l'extrapolation E5 et
  l'explication donnée à la naturaliste.
- **F3. Carte de détectabilité** par vidéo : contraste attendu d'une cible selon la
  zone. Elle indique où l'outil est aveugle et aide au cadrage (A3).

### G. Validation : sans elle, on règle à l'aveugle

- **G1. Vérité terrain** : faire annoter par Manon, sur quelques passages, l'image
  d'entrée et l'image de sortie visibles. On mesure alors la complétude (quelle part du
  passage visible la piste couvre) et le rappel par passage.
- **G2. Injection de cibles synthétiques** : incruster dans la vraie vidéo des taches
  sombres de taille, de contraste et de trajectoire connus, y compris des courbes et des
  passages devant la végétation. On mesure le taux de détection selon le contraste. On
  obtient une vérité terrain gratuite et la limite de détection réelle de l'outil.
- **G3. Protocole publié** : segments de 30 s tirés au hasard et comptés par deux
  personnes. On en tire le taux de détection, qui sert à corriger les comptages.
- **G4. Non-régression élargie** : ajouter au test `slow` des assertions de complétude
  (par exemple, la piste 12 commence au plus tard à 7118) dès qu'une amélioration est en
  place.

## État au 4 octobre 2026

Fait et mesuré (voir [08](08-banc-de-mesure.md)) :

- **G2** : banc de cibles synthétiques (`batdetect-bench`) ;
- **A0** : vidéos copiées en USB ;
- **B1, partiellement** : réglages en pixels d'origine. Monter la résolution sans seuil
  adapté au bruit fait exploser les pistes de bruit, on reste donc à 480 px ;
- **`min_area` 18 → 4** : le vrai verrou des petites cibles ;
- **E1 bis** : vitesse sur 3 images ;
- **E1** : interpolation à l'affichage ;
- **F1, revu** : pas de masque par défaut, au lieu d'un profil de caméra, pour rester
  indépendant de l'appareil ;
- **détection parallèle** par tranches de temps.

Prochaine étape côté détection : **B2 + D1** (filtrage à la taille de la cible et seuil
par pixel), seule voie mesurée pour profiter de la pleine résolution, puis D2 et D3.

## Proposition d'ordre

0. **A0** : travailler désormais sur les originaux copiés en USB. Fait pour la 092.
1. **G2 (injection) et G1 (quelques annotations)** : sans mesure de complétude, on ne
   saura pas si une amélioration en est une.
2. **A8 côté terrain, E1 + E1 bis + F1 côté code** : simples et sans risque. Plus de clignotement, et 5 pistes vont jusqu'au
   vrai bord.
3. **B2 + D1 + D2** : filtrage adapté, seuil par pixel et hystérésis 3D. C'est le
   changement de fond de la détection, visé par toutes les sources.
4. **E3 ou D3** : extension des extrémités, en pleine résolution localement pour
   récupérer le contraste perdu (B1).
5. **E5** : extrapolation affichée jusqu'au bord ou au ciel.
6. Seulement si nécessaire : D4 (track-before-detect), E4 (recollage global), D6
  (réseaux).

## Sources

- Pipelines en écologie : ThermalTracker
  ([Matzner et al.](https://www.pnnl.gov/sites/default/files/media/file/Two-dimensional%20thermal%20video%20analysis%20of%20offshore%20bird%20and%20bat%20flight.pdf)),
  [Betke et al. 2008](https://www.mammalsociety.org/uploads/Betke%20et%20al%202008.pdf),
  [brevet US8116527](https://patents.google.com/patent/US8116527B2/en),
  [BatCount](https://pmc.ncbi.nlm.nih.gov/articles/PMC10019661/),
  [guide ThruTracker](https://sonarjamming.com/wp-content/uploads/2021/05/thrutracker-user-guide_1.9b.pdf),
  [Yarbrough et al.](https://www.biorxiv.org/content/10.1101/2023.02.26.530152v1.full.pdf),
  classification par trajectoire
  ([Cullinan 2015](https://www.pnnl.gov/sites/default/files/media/file/Classification%20of%20birds%20and%20bats%20using%20flight%20tracks.pdf)).
- Physiologie : température des ailes en vol
  ([Springer](https://link.springer.com/article/10.1007/s10344-012-0688-1)).
- Acquisition : [guide BCT 2021](https://cdn.bats.org.uk/uploads/images/Thermal-Imaging-Bat-Survey-Guidelines_KFW_BCT-DATED-2021.pdf),
  [Ahlberg et al. 2025](https://www.eaglehill.us/nabr-pdfs-special/nabr-004-Kloepper.pdf),
  [manuel Pulsar Merger](https://pulsarvision.com/manuals/pulsar-merger-lrf/xt50).

- Infrared small target detection, méthodes classiques et réseaux :
  [BasicIRSTD](https://github.com/XinyiYing/BasicIRSTD),
  [ISTD-python](https://github.com/Tianfang-Zhang/ISTD-python),
  [awesome-infrared-small-targets](https://github.com/Tianfang-Zhang/awesome-infrared-small-targets),
  MPCM : [Wei et al. 2016](https://www.sciencedirect.com/science/article/abs/pii/S0031320316300358),
  multi-images : [DTUM](https://github.com/TinaLRJ/Multi-frame-infrared-small-target-detection-DTUM).
- Track-before-detect : programmation dynamique
  ([PMC3355457](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3355457/)), filtrage de
  vitesse ([DSTO](https://apps.dtic.mil/sti/tr/pdf/ADA415922.pdf)), synthetic tracking
  en astronomie ([Shao et al. 2014](https://arxiv.org/abs/1309.3248)).
- Hystérésis par pixel :
  [scikit-image](https://scikit-image.org/docs/stable/api/skimage.filters.html).
- Compression et petits objets : [arXiv 2211.05805](https://arxiv.org/abs/2211.05805).
  Aucune étude trouvée sur les B-frames et les petites cibles thermiques.
- Suivi : [LapTrack](https://github.com/yfukai/laptrack),
  [filterpy](https://github.com/rlabbe/filterpy) (RTS, non maintenu depuis 2018),
  groupe de Betke ([Wu, Kunz, Betke 2011](https://mlanthology.org/cvpr/2011/wu2011cvpr-efficient/)).
- Fonds : [pybgs](https://pypi.org/project/pybgs/), ThruTracker
  ([bioRxiv](https://www.biorxiv.org/content/10.1101/2021.05.12.443854v1)).
