# Améliorer la détection : diagnostic et pistes

Pourquoi les pistes commencent après l'arrivée de la chauve-souris, finissent avant sa sortie et ont des trous, et ce qu'on peut y faire.
Le diagnostic date du 3 octobre 2026 : il est mesuré sur l'original de la 092 avec le réglage par défaut d'alors (480 px, seuil fixe 25), et sur l'export de Stream Vision 2 quand c'est dit.
Le catalogue d'idées qui suit garde celles qui sont faites, marquées comme telles, et celles qui restent à essayer.
Ce qui n'a pas été mesuré est marqué comme hypothèse.
La logique en place aujourd'hui est décrite dans [10](10-logique-de-detection.md).

## Le symptôme

La naturaliste voit la chauve-souris avant le début de la piste et après sa fin.
Le 3 octobre, la boîte clignotait aussi pendant la piste, à chaque image sans détection ; l'interpolation (E1) l'a corrigé.

## Diagnostic sur l'original (3 octobre 2026)

`video_092_original.mp4` (12,1 Mb/s, sans B-frames), 14 pistes ; les numéros sont ceux du tableau de [03](03-resultats-092.md).

### Trous et extrémités

Les pistes rapides manquent 39 images sur 272 (14 %, contre 22 % sur l'export).
12 pistes sur 14 ont des trous, de 1 à 4 images.

Pour mesurer ce qu'on peut gagner aux extrémités, on a prolongé chaque piste à vitesse constante (estimée sur 3 points), de 1 à 5 images avant et après.
À chaque position prolongée, on cherche le pic de résidu filtré (σ = 1) dans un rayon de 8 px, et on le compare au 99e centile des pics dans des fenêtres tirées au hasard (z = 14,9 sur l'export).
On compte les images consécutives au-dessus de ce seuil.
Les pistes peuvent encore gagner de 1 à 5 images à leurs extrémités : #2 et #4 en gagnent 5 après, #6 4 avant, #14 3 avant.
Ce sont des ordres de grandeur, car l'extrapolation est rectiligne et le seuil n'est pas validé.

Six pistes (#1, #2, #3, #4, #10 et #13) finissaient à 10 px au plus d'un bord ou des bandes masquées d'alors, en haut et en bas de l'image.

L'original règle une partie des trous, mais ni les extrémités ni la saccade.

### La piste 14 (3:57)

La chauve-souris apparaît à l'image 7122 (3:57.16), au même endroit que sur la capture de l'utilisateur, mais la piste ne commence qu'à 7127.
Sur ces cinq images, le résidu vaut −42, −31, −30, −71 et −59 en pleine résolution, mais −25, −26, −22, −30 et −30 à 480 px.
**La réduction de l'image coûte 30 à 50 % du contraste, et c'est elle qui retarde le début de la piste.**
Avant 7122, rien ne dépasse le bruit : elle sort de la végétation à ce moment-là.

Deux autres causes s'ajoutent, mesurées sur le même passage de l'export.
Le fond est peu contrasté à cet endroit : la végétation est plus sombre que la roche, donc l'écart avec la chauve-souris y est plus faible.
Et la trajectoire est courbe et accélère, d'environ 4 à 20 px par image en piquant vers la gauche.
Une extrapolation à vitesse constante depuis le début de la piste se trompe de 30 px dès 4 images en arrière : une recherche en ligne droite ne trouve rien.
En sortie, à gauche, le résidu tombe à exactement 0 deux images après la fin : elle passe devant le ciel saturé, où l'image ne contient plus aucune information.

### Bruit, affichage et ciel

- **Le fond n'est pas figé partout.**
  Sur l'export, le codec recopie les blocs immobiles au bit près : pour presque tous les pixels, la médiane de la valeur absolue du résidu (MAD) vaut 0, et le bruit n'est fait que de mises à jour de blocs.
  Sur l'original, ce n'est le cas que de 36 % des pixels : le bruit du capteur réapparaît.
  Un seuil par pixel fondé sur la MAD (D1) devient donc pertinent, avec un plancher pour les pixels où elle est nulle.
- **L'affichage incrusté ne déclenche presque rien.**
  Sur 9 000 images, on a compté combien de fois chaque pixel dépasse le seuil.
  Sur l'original, aucun pixel ne dépasse 100 fois ; la bande du haut compte 1 dépassement et celle du bas 301.
  Sur l'export, les 18 pixels qui dépassaient 100 fois étaient tous dans la ligne du compteur et de l'horloge.
  La raison de cette différence n'est pas comprise (hypothèse : un rendu de l'affichage différent).
  D'où l'absence de masque par défaut ([01](01-contexte.md)).
- **Le ciel est aveugle.**
  21,2 % des pixels valent exactement 0, presque tout le ciel (19,7 % sur l'export).
  Aucune détection n'y est possible : la chauve-souris y est saturée elle aussi.
- **Type d'image.**
  L'original n'a pas de B-frames : 87 % des images P des pistes ont une détection, et 78 % des images I.
  Sur l'export (72 % de B-frames), c'était 75 % des B et 85 % des P : l'écart va dans le sens de l'hypothèse « les B-frames effacent les petits objets », mais l'échantillon est trop petit pour conclure.

### Résolution

Le capteur des Symbion DXT50 fait 1280×1024 : la vidéo à 1440×1080 est à peu près à la résolution native, et son spectre ne montre pas de coupure nette.
La réduction à 480 px de large jetait donc environ 2,7 fois la résolution du capteur, ce qui est cohérent avec la perte de contraste mesurée sur la piste 14.

### Cadence saccadée

Le capteur tourne à 50 Hz, la vidéo est à 30 i/s.
Sur les pistes rapides, le pas entre deux images consécutives suit un motif de période 3 : un pas double puis deux pas simples.
Piste 5 de l'original : 8,7 · 18,7 · 9,9 · 8,8 · 17,7 · 9,6 · 9,2 · 19,6…
Aucune image n'est dupliquée à l'identique, et la saccade est la même sur l'export.
La phase du motif varie d'une piste à l'autre : sur l'export, les pas doubles tombent à l'image mod 3 = 0, 1 et 2 dans 13, 19 et 9 cas sur environ 45.
On ne sait pas comment les jumelles convertissent les 50 Hz en 30 i/s.

Conséquences :

- une prédiction sur les deux derniers points se trompe d'un facteur 2 une image sur trois, d'où la vitesse estimée sur au moins 3 images (E1 bis) ;
- l'interpolation à l'affichage est linéaire : pendant un trou, elle peut s'écarter d'un pas au plus de la position réelle, sans conséquence pour une boîte.

## Ce que fait la littérature

**Le contraste négatif s'explique par la physiologie.**
En vol, les ailes ne sont qu'à environ 1 °C au-dessus de l'air.
À distance, ce sont elles qui occupent les pixels, alors que la roche garde la chaleur de la journée.
Une chauve-souris lointaine devant une roche chaude est donc sombre, et le contraste peut s'annuler au crossover thermique.

| Pipeline | Détection | Ce qu'on en retient |
| --- | --- | --- |
| ThermalTracker (PNNL, Matzner 2015) | maximum par pixel sur une traversée, seuil μ + 3σ par pixel | seuil par pixel ; ne voit que les cibles chaudes |
| Boston University (Betke 2008) | écart à la moyenne par pixel, en fraction de σ | seuil fixe écarté : près de la végétation, la cible a l'intensité du fond |
| Brevet US Army (Sabol & Melton 2009) | fond = mode de l'histogramme par pixel, puis valeur absolue de la différence | le plus proche de notre cas : la valeur absolue rend la détection indifférente au signe ; la prédiction tient compte du nombre d'images manquées |
| BatCount (Bentley 2023) | médiane locale, filtre sur les objets plus clairs ou plus sombres | 51 à 95 % selon la caméra, toujours en sous-comptage |
| ThruTracker (Corcoran) | soustraction de fond, lissage gaussien et dilatation réglés sur le plus petit objet | mêmes réglages que les nôtres : écart maximal, longueur minimale de piste |
| Yarbrough et al. (2023-2026) | MOG2 volontairement permissif, puis tri des imagettes par un CNN | détecter large, puis trier : 99 % entre biologique et non biologique |

**Validation.**
BatCount tire des segments de 30 s au hasard et les fait compter par deux personnes (au moins 96,5 % d'accord).
Matzner corrige le comptage par (N − faux positifs) ÷ taux de détection, et recommande de mesurer la détection en fonction du contraste et de la taille.
Aucun de ces pipelines n'injecte de cibles synthétiques : c'est ce que fait notre banc (G2, [08](08-banc-de-mesure.md)).

**Battements d'ailes.**
À 30 i/s, la limite de Nyquist (15 Hz) est au niveau de la fréquence de battement (10 à 15 Hz pour Tadarida, environ 11 Hz pour une pipistrelle).
Il faudrait du 50 i/s et des cibles de plus de quelques pixels.
Pour distinguer chauve-souris et oiseau, la forme de la trajectoire marche mieux : kurtosis et changement de direction donnent 82 % de bonne classification (Cullinan 2015).

## Ce qui est fait

- **A0** : vidéos copiées en USB ([03](03-resultats-092.md#original-contre-export-historique-septembre-et-octobre-2026)).
- **G2** : banc de cibles synthétiques ([08](08-banc-de-mesure.md)).
- **`min_area` de 18 à 4 px²** (4 octobre 2026) : c'était le vrai verrou des petites cibles ([08](08-banc-de-mesure.md#résolution-de-travail-et-min_area)).
- **E1 bis** (4 octobre) : vitesse sur 3 images. **E1** : interpolation à l'affichage.
- **F1, revu** : pas de masque par défaut plutôt qu'un profil de caméra, pour rester indépendant de l'appareil.
- **B1, B2 et D1** (6 octobre, #7) : détection à 960 px, filtre à la taille de la cible (σ 1,5 px), seuil par pixel (plancher 12, facteur 8).
  Ces réglages ont été choisis sur sept vidéos ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)).
  Sur la 092, ils gardent les 14 pistes en les rendant plus complètes ; sur les sept vidéos, on passe de 75 à 122 pistes, et les 6 qui disparaissent ont été jugées fausses par l'utilisateur.
  Le passage réel de 3:42.10 sur la 092 reste manqué ([03](03-resultats-092.md#cibles-connues-pour-les-issues-suivantes)).

Depuis, la logique a gagné les détections faibles encadrées entre deux détections fortes ([10, § 5 ter](10-logique-de-detection.md#5-ter-garder-les-taches-faibles-qui-comblent-un-vol)), la portée d'une piste (§ 7) et le recollage d'un vol caché derrière la végétation, proche de E4 ([§ 9 bis](10-logique-de-detection.md#9-bis-recoller-un-vol-caché-derrière-la-végétation)).

## Catalogue des idées

Chaque idée donne le symptôme visé, ce qu'on en attend et son coût. Les sources sont en fin de document.

### A. La prise de vue

- **A0. Copier les originaux en USB au lieu d'exporter par Stream Vision 2.** Fait.
  Sur la 092 : 12 fois plus de débit, plus de B-frames, +31 % de points détectés sur les passages communs et 2 passages nouveaux.
  C'est de loin le plus gros gain mesuré.

Réglages des Symbion DXT50 à proposer à Manon, d'après le manuel.
Leur coût est nul et leur gain peut être le plus fort, mais chacun est à valider par un essai comparatif (A8) avec les réglages réels des jumelles.

- **A1. Compression vidéo sur OFF** (menu Media) : « compression minimale, qualité supérieure, fichiers beaucoup plus volumineux ».
  L'hypothèse d'une compression sur ON venait du débit de l'export, environ 1 Mb/s.
  C'est le moyen le plus direct contre les blocs et les B-frames écrasées.
- **A2. Gain et contraste manuels** plutôt que l'AGC, réglés pour que le ciel ne sature pas : le ciel noir saturé est une zone aveugle.
- **A3. Cadrage** : garder la zone de passage devant la roche, où le contraste est fort, et limiter le ciel et la végétation sombre.
- **A4. Rester au grossissement de base (2×)**, sans zoom numérique, comme sur nos vidéos.
- **A5. Calibrage semi-automatique**, déclenché juste avant d'enregistrer.
  En automatique, l'image se fige environ une seconde à chaque calibrage, et un passage peut tomber dedans.
- **A6. Stabilisation d'image sur OFF** sur trépied.
  Elle est faite pour la main levée ; si elle agit aussi sur l'enregistrement, elle peut déplacer le fond (à vérifier).
- **A6 bis. Filtre de lissage sur OFF**, à tester : il « rend l'image plus lisse et plus uniforme » et risque d'effacer les petits points.
  **Amplification** Normal, Haut ou Ultra : effet inconnu sur des cibles de quelques pixels, à comparer.
- **Luminosité et contraste fixes** entre les sessions (option « Enregistrer les paramètres de l'image thermique »), pour que les vidéos restent comparables.
  Le manuel ne dit pas leur effet sur l'enregistrement.
- **A7. Commencer l'enregistrement au début de l'émergence**, car le fond change de température autour du coucher du soleil (Ahlberg 2025).
- **A8. Essai comparatif** : même scène, même soir, trois enregistrements de 5 min, avec les réglages actuels, compression OFF, puis compression OFF et lissage OFF.
  On compare le débit, le nombre de pistes, la complétude et le bruit de fond mesuré par l'outil.

### B. Prétraitement de l'image

- **B1. Travailler en pleine résolution, ou plus près.** Fait à 960 px.
  1440 px donne presque les mêmes pistes pour 2,4 fois le temps ([08](08-banc-de-mesure.md#sur-six-autres-vidéos)).
  Reste possible : ne passer en pleine résolution qu'autour des pistes (D3).
- **B2. Filtrage adapté** (gaussienne, LoG ou DoG à l'échelle de la cible) sur le résidu signé : c'est l'optimum théorique pour une tache dans du bruit.
  **Fait** (`--target-sigma 1.5`), sous forme d'un flou gaussien appliqué aux images avant le calcul du fond ([08](08-banc-de-mesure.md#filtre-à-la-taille-de-la-cible-b2)).
  À 960 px, σ 1,5 et seuil 12, les cibles à −30 de σ 1,5 passent de 0 à 14 sur 28 et celles de σ 3 de 10 à 26, avec 14 pistes de référence.
- **B3. Débloquage spatial** (`deblock`, `spp` ou `pp7` de FFmpeg) avant la détection, pour réduire les fragments dus aux blocs.
  Pas de débruitage temporel (`hqdn3d`, `atadenoise`) : il efface précisément les petits objets rapides.
- **B4. Restauration par réseau** (MFQE 2.0, BasicVSR++) : à écarter sauf preuve contraire.
  Ces réseaux sont entraînés sur de la vidéo naturelle en couleur, et risquent d'effacer ou d'inventer des petits points.
- **B5. Stabilisation** (corrélation de phase ou ECC) avant la détection, pour les vidéos où les jumelles bougent, comme la 027.

### C. Modèle de fond

- **C1. Médiane avec exclusion** : fond calculé sur [t−15, t−3] ∪ [t+3, t+15], pour qu'un objet lent (3:29) ne contamine plus son propre fond.
- **C2. Différence à 3 images** : min(|It − It−k|, |It − It+k|).
  Quasi gratuite et adaptée aux cibles qui se déplacent de plus que leur taille ; elle peut se combiner avec la médiane.
- **C3. MOG2, KNN ou ViBe**, ce qu'utilise ThruTracker.
  Leur sortie est binaire : on perdrait le signe et l'amplitude du résidu, dont le reste de la chaîne a besoin.
- **C4. RPCA ou GoDec** (fond de rang faible et cible creuse) : coûteux, pour un gain qui paraît faible sur une scène fixe face à une médiane bien faite (hypothèse non mesurée).

### D. Détection

- **D1. Seuil par pixel** : résidu ÷ bruit local (MAD temporelle), avec un plancher.
  Les zones calmes (roche) descendent plus bas que 25, et les zones agitées (végétation au vent) restent protégées.
  **Fait** (`--noise-factor 8`, plancher `--threshold 12`), mesuré au banc ([08](08-banc-de-mesure.md#seuil-par-pixel-d1)) et sur sept vidéos.
  C'est lui qui empêche le bruit de fond de la 089, de la 090 et de la 091 de devenir des centaines de pistes quand le seuil descend.
  Le ciel saturé n'est pas masqué : la MAD y est nulle et le plancher s'applique.
- **D2. Hystérésis 3D (x, y, t)** : garder un pixel faible (seuil bas, par exemple 12) s'il est relié dans l'espace ou dans le temps à un pixel fort (seuil haut, 25).
  C'est le mécanisme qui récupère les débuts et fins de piste et comble les trous.
  Il faut garder en mémoire les résidus de quelques images ([06](06-questions-ouvertes.md)).
- **D3. Seconde passe guidée** : autour de chaque extrémité de piste, relire la vidéo en pleine résolution et chercher avec un seuil abaissé.
  La zone de recherche s'élargit avec la distance à la dernière détection, pour tolérer les virages et les accélérations (piste 14).
  Coût : environ 60 images relues par piste.
- **D4. Track-before-detect local** : additionner le résidu normalisé le long de trajectoires candidates sur plusieurs images, et accepter si la somme dépasse un seuil qui croît en √N.
  Un point trop faible sur une image devient visible par cohérence de mouvement, comme pour l'œil.
  On pourrait tester un éventail de vitesses et de courbures autour de la vitesse connue.
- **D5. Contraste local multi-échelle signé** (MPCM, top-hat blanc et noir), issu de la littérature infrarouge sur les petites cibles.
  MPCM rehausse les cibles sombres comme les claires : une alternative à D1 et D2 si le résidu au fond ne suffit pas.
- **D6. Réseaux de neurones** pour petites cibles infrarouges (DNANet, SCTransNet, DTUM en multi-images), via BasicIRSTD : option longue.
  Ils sont entraînés sur des scènes militaires et aériennes non compressées, avec des cibles chaudes ; sans réentraînement sur nos vidéos annotées, le transfert n'est pas garanti.
- **D7. Détecter large, puis trier** : seuil bas, puis classement des imagettes ou des pistes (forme de trajectoire, taille, vitesse) par un classifieur.
  C'est l'approche de Yarbrough et al. ; les validations de Manon serviraient d'exemples d'entraînement.

### E. Suivi

- **E1. Interpolation dans les trous** : une position à chaque image entre deux détections, marquée comme interpolée, pour que la boîte ne clignote plus.
  Fait.
- **E1 bis. Vitesse estimée sur une période de saccade** (3 images au moins) au lieu des deux derniers points. Fait.
- **E2. Filtre de Kalman et lissage RTS** à la place de la prédiction : la zone de recherche grandit avec l'incertitude pendant un trou, et les trous sont interpolés selon la dynamique.
- **E3. Suivi bidirectionnel** : relancer le suivi sur la vidéo à l'envers.
  Un début de piste devient une fin, prédite par une vitesse déjà bien estimée.
  Avec D2 ou D3, c'est la manière la plus naturelle d'étendre les débuts.
- **E4. Recollage de pistes** : relier deux pistes dont les extrapolations se rejoignent au-delà de `max_gap` (LapTrack, ou flot de coût minimal).
  C'est l'approche du groupe de Betke pour les chauves-souris.
- **E5. Extrapolation au bord**, affichée en pointillés et hors statistiques : quand une piste s'arrête à moins de 0,3 s environ du bord ou entre dans le ciel saturé, la prolonger jusqu'à la sortie.
  C'est le seul recours quand l'image ne contient plus d'information, comme à la sortie de la piste 14.

### F. Masques et zones

- **F1. Masque d'affichage automatique** : masquer les pixels qui dépassent le seuil plus de N fois sur la vidéo, après une dilatation de quelques pixels.
  Sur l'export de la 092, il ne retiendrait que la ligne du compteur et de l'horloge ; sur l'original, rien.
- **F2. Masque de saturation** : les zones à 0 ou 255 sont aveugles.
  Une piste qui y entre est « perdue dans le ciel », ce qui justifie l'extrapolation E5 et une explication à la naturaliste.
- **F3. Carte de détectabilité** par vidéo : contraste attendu d'une cible selon la zone.
  Elle montre où l'outil est aveugle et aide au cadrage (A3).

### G. Validation

- **G1. Vérité terrain** : faire annoter par Manon, sur quelques passages, l'image où la chauve-souris entre et celle où elle sort.
  On mesure alors la complétude (la part du passage visible que la piste couvre) et le rappel par passage.
- **G2. Injection de cibles synthétiques** dans la vraie vidéo : taches sombres de taille, de contraste et de trajectoire connus, courbes et passages devant la végétation compris.
  On mesure le taux de détection selon le contraste : une vérité terrain gratuite, et la limite de détection réelle de l'outil.
  Fait ([08](08-banc-de-mesure.md)).
- **G3. Protocole publié** : segments de 30 s tirés au hasard et comptés par deux personnes, pour en tirer le taux de détection qui corrige les comptages.
- **G4. Non-régression élargie** : ajouter au test `slow` des assertions de complétude dès qu'une amélioration est en place (par exemple, sur le début de la piste de 3:57).

## Sources

- Pipelines en écologie : ThermalTracker ([Matzner et al.](https://www.pnnl.gov/sites/default/files/media/file/Two-dimensional%20thermal%20video%20analysis%20of%20offshore%20bird%20and%20bat%20flight.pdf)), [Betke et al. 2008](https://www.mammalsociety.org/uploads/Betke%20et%20al%202008.pdf), [brevet US8116527](https://patents.google.com/patent/US8116527B2/en), [BatCount](https://pmc.ncbi.nlm.nih.gov/articles/PMC10019661/), [guide ThruTracker](https://sonarjamming.com/wp-content/uploads/2021/05/thrutracker-user-guide_1.9b.pdf), [Yarbrough et al.](https://www.biorxiv.org/content/10.1101/2023.02.26.530152v1.full.pdf), classification par trajectoire ([Cullinan 2015](https://www.pnnl.gov/sites/default/files/media/file/Classification%20of%20birds%20and%20bats%20using%20flight%20tracks.pdf)), ThruTracker ([bioRxiv](https://www.biorxiv.org/content/10.1101/2021.05.12.443854v1)).
- Physiologie : température des ailes en vol ([Springer](https://link.springer.com/article/10.1007/s10344-012-0688-1)).
- Acquisition : [guide BCT 2021](https://cdn.bats.org.uk/uploads/images/Thermal-Imaging-Bat-Survey-Guidelines_KFW_BCT-DATED-2021.pdf), [Ahlberg et al. 2025](https://www.eaglehill.us/nabr-pdfs-special/nabr-004-Kloepper.pdf), [manuel Pulsar Merger](https://pulsarvision.com/manuals/pulsar-merger-lrf/xt50).
- Petites cibles infrarouges, méthodes classiques et réseaux : [BasicIRSTD](https://github.com/XinyiYing/BasicIRSTD), [ISTD-python](https://github.com/Tianfang-Zhang/ISTD-python), [awesome-infrared-small-targets](https://github.com/Tianfang-Zhang/awesome-infrared-small-targets), MPCM ([Wei et al. 2016](https://www.sciencedirect.com/science/article/abs/pii/S0031320316300358)), multi-images ([DTUM](https://github.com/TinaLRJ/Multi-frame-infrared-small-target-detection-DTUM)).
- Track-before-detect : programmation dynamique ([PMC3355457](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3355457/)), filtrage de vitesse ([DSTO](https://apps.dtic.mil/sti/tr/pdf/ADA415922.pdf)), synthetic tracking en astronomie ([Shao et al. 2014](https://arxiv.org/abs/1309.3248)).
- Compression et petits objets : [arXiv 2211.05805](https://arxiv.org/abs/2211.05805).
  Aucune étude trouvée sur les B-frames et les petites cibles thermiques.
- Suivi : [LapTrack](https://github.com/yfukai/laptrack), groupe de Betke ([Wu, Kunz, Betke 2011](https://mlanthology.org/cvpr/2011/wu2011cvpr-efficient/)).
