# Profilage : où part le temps de calcul

Le profil complet date du 16 octobre 2026 (#201), sur un Apple M1 Max : la 092 entière se traite en 19,81 s, dont 74 % de détection.
Deux optimisations de la détection l'ont suivi le 17 octobre (#214 et #213) ; elles sont mesurées à part, plus bas.
Le profil du Python jusqu'à la v0.1.1 se lit dans ce fichier au tag `python-final`.

## Profil du 16 octobre 2026

### Protocole

- **Machine** : Apple M1 Max (10 cœurs, 64 Go), macOS 27.0 (26A428), Rust 1.99.0, FFmpeg 7.1.1 et openh264 de `mise run rust:ffmpeg`.
- **Code** : commit b38748a, branche `rust/201-profil`.
  Ses `crates/*/src` et ses `Cargo.*` sont ceux de f364698, le commit mesuré par #198.
- **Binaire** : `cargo build --release --locked -p batdetect` (LTO `fat`), lancé avec `-o out/mN` et `LANG=fr_FR.UTF-8`.
  Vidéos : une copie de la 092 entière (9 011 images, 1440×1080, 5 min) et l'extrait du dépôt (`tests/fixtures/video_092_original_3m24-4m05.mp4`, 1 230 images).
- **Temps et pic** (passages 1 à 5, de 3 h 21 à 3 h 28) : `verrou chrono /usr/bin/time -l …` (`/Users/zaratan/Projects/dvr-wt/MESURES.md`).
  Avant chaque passage, on vérifie le calme selon `MESURES.md` : aucun outil lourd, et moins de 100 % pour la somme des `%CPU` des autres processus (relevés de 78,6 à 99,9 %).
  Aucune mise en veille pendant la série (la dernière date de 2 h 28).
  Le CPU est la somme du temps utilisateur et du temps système.
- **Étapes** : hors terminal, la commande écrit une ligne sur la sortie d'erreur au début de chaque étape, puis une ligne de fin.
  Un filtre en Python horodate chaque ligne à son arrivée (`time.time()`, à la milliseconde), et une étape dure l'écart entre deux lignes.
  Au commit mesuré, les étapes de `crates/cli/src/process.rs` contiennent :
  - **détection** : lecture, réduction et flou, fond, seuil, taches, et le fond médian de l'image résumé ;
  - **suivi** : exclusion des plages abîmées et instables, pistes, ménage des sorties précédentes, CSV, essai de l'encodeur (8 images grises encodées par VideoToolbox) et `params.json`, qui s'écrit aujourd'hui en dernier, après les extraits ;
  - **image résumé** : dessin et PNG ;
  - **extraits** : extraits et zoom, rendus ensemble ; le profil ne sépare que l'encodage openh264 du zoom.
- **Profil** (passage 8, de 3 h 28 à 3 h 29) : `/usr/bin/sample <pid> 150 1 -mayDie -file …` sur la 092 entière, sous `verrou lourd` et sans contrôle de calme.
  C'est l'échantillonneur livré avec macOS : il voit tous les fils et écrit l'arbre d'appels en texte.
  Réglé sur une milliseconde, il a pris 16 551 passages en 54,7 s, un toutes les 3,3 ms en moyenne (3,2 ms pendant la détection, 4,6 ms pendant les extraits).
  Le binaire `release` garde ses symboles ; avec la LTO `fat`, une fonction inlinée compte pour celle qui l'appelle.
  Un premier profil (passage 6, `sample … 60`) s'est arrêté avant les extraits.
  Il ne sert qu'à recouper les parts de la détection : 23,6 % et 20,3 %, contre 22,7 % et 20,3 % au passage 8.
- **Rangement des échantillons** : on écarte ceux dont le sommet de pile est une attente (`__psynch_cvwait`, `__psynch_mutexwait`, `__psynch_rw*`, `semaphore_wait_trap`, `semaphore_timedwait_trap`, `__semwait_signal`, `__ulock_wait`, `__workq_kernreturn`, `kevent`, `mach_msg2_trap`, `_pthread_cond_wait*`).
  Chacun des autres va au premier poste qui convient, dans cet ordre :
  1. encodage openh264 (`Wels*` au sommet) ;
  2. copie vers VideoToolbox (bibliothèque VideoToolbox au sommet) ;
  3. fond et seuil (`Threshold::light` dans la pile) ;
  4. réduction et flou (`prepare_pending` dans la pile) ;
  5. piles réduites à `thread_start` ;
  6. rendu (`media::convert` ou `media::encode` dans la pile) ;
  7. rayon qui cède la main (`swtch_pri`) ;
  8. décodage (fonction de FFmpeg au sommet, rangée par son nom) ;
  9. rendu (`render::*` dans la pile) ;
  10. image résumé, suivi, puis le reste.
- **Fils** (passage 7, à 3 h 01) : `ps -M -p <pid>` toutes les 250 ms, sous `verrou lourd`, pendant qu'un autre calcul chargeait la machine (détection en 26,0 s au lieu de 14,7).

### Temps de bout en bout et par étape

| Vidéo | Passage | Réel | CPU | Pic (résident) | Détection | Suivi | Image résumé | Extraits |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 092 entière | 1 | 19,81 s | 143,2 s | 1,04 Go | 14,69 s | 0,26 s | 0,24 s | 4,60 s |
| 092 entière | 2 | 20,66 s | 143,7 s | 1,05 Go | 15,50 s | 0,25 s | 0,24 s | 4,67 s |
| 092 entière | 3 | 19,73 s | 143,6 s | 1,06 Go | 14,54 s | 0,24 s | 0,24 s | 4,69 s |
| extrait | 4 | 4,51 s | 23,6 s | 0,70 Go | 2,29 s | 0,24 s | 0,23 s | 1,75 s |
| extrait | 5 | 4,45 s | 23,7 s | 0,70 Go | 2,21 s | 0,24 s | 0,23 s | 1,77 s |

- **092, médianes** : 19,81 s réels, 143,6 s de CPU (7,2 cœurs en moyenne sur tout le passage), 1,05 Go.
  La détection prend 74 % du temps réel (613 images par seconde), les extraits 24 %, le suivi et l'image résumé 2,4 % à eux deux.
  Chaque passage produit 14 pistes, un zoom et 15 fichiers vidéo.
- **Extrait** : 4,48 s réels, 23,6 s de CPU, 0,70 Go.
  La détection n'y prend que la moitié du temps (2,25 s, 547 images par seconde).
  Sur une vidéo courte, les extraits (1,76 s pour 4 pistes), le suivi et l'image résumé (0,47 s) pèsent plus.
- **Suivi et image résumé** : de 0,23 à 0,25 s chacun, sur la 092 comme sur l'extrait, qui a pourtant 7 fois moins d'images et 3,5 fois moins de pistes.
  Dans le profil, l'essai de l'encodeur (`batdetect::process::encoders`) a 520 échantillons sur le fil principal ; l'exclusion, les pistes et les fichiers n'en ont aucun.
  Sous l'échantillonneur, une partie de cet essai attend dans `dlopen` : le profil désigne l'essai, mais seul le chronomètre donne sa durée.
- **Bruit** : 5 % sur le temps réel de la 092 (19,73 à 20,66 s, tout dans la détection), 0,3 % sur le CPU.
  Le même code livré, mesuré la veille au calme par #198 (f364698, 15 octobre 2026, même machine), donnait 19,0 s réels, 128,5 s de CPU et 1,09 Go.
  L'écart (+4 % de temps réel, +12 % de CPU, −3 % de pic) dépasse le bruit d'une série pour le CPU, et sa cause n'est pas isolée.

### Où passe le CPU

La 092 entière sous `sample` (passage 8) donne 163 650 échantillons hors attente.
L'échantillonneur allonge la détection de 2,9 fois et les extraits de 2,3 fois, et son pas varie d'une étape à l'autre.
Les parts à l'intérieur de la détection tiennent (le passage 6 les redonne), mais les parts entre étapes sont faussées d'environ 1,4 fois.

| Poste | Fonctions | Part des échantillons hors attente |
| --- | --- | ---: |
| Décodage H.264 : CABAC | `decode_cabac_residual_*`, `ff_h264_decode_mb_cabac` (FFmpeg) | 34,3 % |
| Fond et seuil | `engine::threshold::Threshold::light` : réseau de tri sur la fenêtre, seuil par pixel | 22,7 % |
| Réduction et flou | `engine::detector::Detector::prepare_pending` → `engine::prepare` : largeur de travail, filtre à la taille de la cible | 20,3 % |
| Décodage H.264 : filtre de boucle | `ff_h264_filter_mb`, `loop_filter`, `*_loop_filter_*` | 6,2 % |
| Décodage H.264 : reste | compensation de mouvement, caches, synchronisation des fils | 4,8 % |
| Rayon cède la main | `swtch_pri` : fils de rayon en attente de travail | 3,7 % |
| Décodage H.264 : IDCT | `ff_h264_idct*_neon` | 2,7 % |
| Système, mémoire | `memmove` (dont la copie après décodage), lecture du fichier | 2,1 % |
| Rendu | `render::*`, `media::convert` (swscale), `media::encode` | 1,3 % |
| Piles réduites à `thread_start` | rien ne dit si c'est du calcul | 1,0 % |
| Encodage du zoom | openh264 (`WelsEnc::*`) | 0,5 % |
| Copie vers VideoToolbox | `vt_Copy_y420_420v` | 0,3 % |
| Image résumé | `summary::*`, `draw::*`, `batdetect::overview` | 0,1 % |

- Le décodage fait presque la moitié (48,0 %), CABAC en tête : la 092 est un H.264 à 12 Mb/s en 1440×1080.
- Un tiers de ce décodage est un second décodage, autour des extraits.
  Les fils créés pendant les extraits (décodeurs `av:h264:df*`, rendu, encodeurs) portent 18,8 % des échantillons, dont 15,6 % de décodage : 25 474 échantillons sur les 78 871 du décodage, 32 %.
  La détection avait déjà décodé ces images une fois.
- La détection fait l'autre grosse part (43,0 %) : fond et seuil, puis réduction et flou, à parts presque égales.
  Les taches, la fermeture et l'exclusion n'ont que 3 échantillons.
- Le rendu hors décodage pèse 2,1 % avec openh264 et la copie vers VideoToolbox.
  Les extraits s'encodent dans le service de VideoToolbox, hors du processus, et ce profil ne le voit pas.
- Le suivi n'a aucun échantillon dans `tracking::*`.

### Fils

D'après le profil (passage 8), la détection tourne sur 23 fils : le fil principal, 10 fils de rayon (préparation, fond et seuil), 11 fils du décodeur (`av:h264:df0` à `df10`) et le fil de lecture anticipée.
Pendant la détection, 9,9 fils en moyenne ne sont pas en attente à chaque passage de l'échantillonneur (132 909 échantillons pour 13 443 passages), pour 10 cœurs.
Pendant les extraits, les fils créés pour eux en ajoutent 13 en moyenne (30 741 pour 2 372 passages).

`ps -M` (passage 7, machine chargée) compte les fils au fil des étapes :

| Étape | Relevés | Fils (médiane) | Fils (max) |
| --- | ---: | ---: | ---: |
| Détection | 94 | 23 | 23 |
| Suivi | 2 | 13 | 15 |
| Extraits | 18 | 76 | 89 |

La charge par fil de `ps` (`%CPU`) n'est pas reprise : c'est une moyenne lissée sur une minute (`man ps`), qui ne dit rien d'une étape de quelques secondes.

### Ce que disent les chiffres

- La détection occupe déjà presque les 10 cœurs : elle ne gagnera que par un calcul plus court par image, pas par plus de parallélisme.
- Dans le code du dépôt, deux postes comptent : le fond et le seuil (22,7 %), la réduction avec le flou (20,3 %).
  Tout le reste du Rust tient sous 2 %.
- Le second décodage autour des extraits peut s'éviter sans changer un pixel.
  Le reste du décodage ne baisserait que par un réglage du décodeur qui change les pixels (sauter le filtre de boucle, 6,2 %), et donc la détection.
- Le suivi et l'image résumé coûtent 0,5 s fixes, presque tout dans l'essai de l'encodeur.
- Le CPU par étape n'est pas mesuré : l'échantillonneur fausse les parts entre étapes, et `ps` ne donne qu'une charge lissée.

## Sous Linux (lumiel, 16 octobre 2026)

Mesures de l'utilisateur sur lumiel (Rocky Linux 10, x86_64) avec le binaire `release` de `main` du 16 octobre 2026, comparées au même binaire sur le M1 Max le même jour.
Le pic est la mémoire résidente au plus haut.

| Vidéo | Machine | Réel | CPU | Pic |
| --- | --- | ---: | ---: | ---: |
| 092 entière | M1 Max | 19,0 s | 128,5 s | 1,09 Go |
| 092 entière | lumiel | 28,7 s | 238,9 s | ≈ 1,27 Go |
| extrait | lumiel | 9,00 s | 41,1 s | 503 Mo |
| extrait, `--zoom all` | lumiel | 10,49 s | 65,5 s | 951 Mo |

Sur la 092, lumiel met 1,5 fois plus de temps réel et 1,9 fois plus de CPU que le M1 Max.
Sur l'extrait, les 4 zooms de `--zoom all`, encodés par openh264, ajoutent 1,5 s réelles, 24,4 s de CPU et 448 Mo au pic.

## Préparation vectorisée (#214, 17 octobre 2026)

#214 calcule le gris par sa forme close et réduit les rapports courants (3:2 au défaut de 960) par blocs fixes, sans changer un pixel ([11](11-migration-rust.md)).
On mesure avant (a868062) et après (23fa9b4) des binaires `release`, avec `LANG=fr_FR.UTF-8` et un dossier de sortie neuf à chaque passage, sous `verrou chrono` (`/usr/bin/time -v` sur lumiel, `-l` sur le M1 Max).
La détection est l'écart entre les lignes « détection » et « suivi » de la sortie d'erreur, horodatées.

- **lumiel** : Ryzen 5 3600X (6 cœurs, 12 fils), Rocky Linux 10, Rust 1.99.0, cible `x86-64-v2`.
  D'autres sessions y tournent : le CPU des autres processus est compté par `/proc/stat` pendant chaque passage, et un passage pendant une compilation hors verrou est écarté (6 passages).
- **M1 Max** : la machine du profil, calme vérifié avant chaque série (somme des `%CPU` des autres processus de 19 à 85 %).

Préparation d'une image (gris, réduction, flou) sur un fil, 200 images de l'extrait, par une sonde temporaire hors du dépôt (meilleur de 5 passes) :

| Machine | Largeur | Avant | Après | Gain |
| --- | ---: | ---: | ---: | ---: |
| lumiel | 960 | 2,85 ms | 1,88 ms | 34 % |
| lumiel | 1440 | 3,73 ms | 3,34 ms | 10 % |
| M1 Max | 960 | 1,74 ms | 1,00 ms | 43 % |
| M1 Max | 1440 | 2,18 ms | 1,92 ms | 12 % |

- Sur lumiel, à 960, le gris passe de 0,54 à 0,18 ms et la réduction hors gris de 1,23 à 0,63 ms.
  Les deux flous (0,44 et 0,63 ms), déjà vectorisés, ne changent pas.
- Sur le M1 Max, à 960, la réduction gris compris passe de 1,18 à 0,44 ms, et le flou horizontal reste à 0,19 ms.
- À 1440, la réduction se limite au gris : seul lui gagne.

Détection de la 092 entière, paires avant / après alternées :

| Machine | Paires | Avant | Après | Gain médian | CPU | Pic |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| lumiel | 6 | 19,31 à 20,85 s | 18,47 à 19,97 s | 4,4 % (3,5 à 10,2) | 240 à 241 → 233 à 235 s | 1,30 à 1,34 Go des deux côtés |
| M1 Max | 3 | 12,09 à 12,91 s | 11,26 à 11,57 s | 10,4 % (6,7 à 10,4) | 125 à 127 → 119 à 120 s | 1,05 → 1,05 à 1,10 Go |

- Sur lumiel, la préparation ne fait qu'environ 11 % du CPU (2,85 ms × 9 011 images = 25,7 s sur 241 s), contre 20,3 % des échantillons sur le M1 Max.
  Le décodage y pèse plus, d'où un gain moitié moindre.
- Sur l'extrait, lumiel gagne 3,9 % en médiane (2,94 à 2,99 s avant, 2,84 à 2,96 s après, 3 paires), puis 2,8 % sur une paire.
- Les pistes ne changent pas : CSV identique à l'octet sur la 092 du M1 Max, empreinte de l'extrait identique (`slow_extract`), aucun écart de `compare local` sur les 26 vidéos de lumiel.

## Médiane du fond à 28 comparateurs (#213, 17 octobre 2026)

#213 remplace, pour les 11 échantillons de la fenêtre par défaut, les 32 comparateurs du réseau élagué par une sélection de la médiane à 28, sans changer une détection ([11](11-migration-rust.md)).
Les mesures sont faites sur lumiel, la machine décrite pour #214, avec des binaires `release` sous `verrou chrono`.
Avant : 396344e ; après : le même commit plus le réseau à 28.

On isole d'abord la part du fond et du seuil par une sonde temporaire hors du dépôt, qui compte les cycles (`rdtsc`) autour du réseau et du seuil relevé.
C'est la détection seule de la 092 entière, sur un fil (`RAYON_NUM_THREADS=1`) : 3 passages au code d'avant, puis un au réseau à 28.
Les détections des 4 passages ont la même empreinte.

| Passage | Réseau | Détection | `Threshold::light` | dont réseau | dont seuil relevé (MAD) |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 32 | 66,9 s | 32,8 s | 18,3 s (69,50 milliards de cycles) | 0,06 s |
| 2 | 32 | 62,2 s | 30,5 s | 17,3 s (65,90 milliards de cycles) | 0,05 s |
| 3 | 32 | 64,1 s | 31,6 s | 17,7 s (67,13 milliards de cycles) | 0,05 s |
| 4 | 28 | 59,8 s | 28,2 s | 15,2 s (57,68 milliards de cycles) | 0,05 s |

- Sur un fil, `Threshold::light` fait la moitié de la détection, et le réseau 56 % de `light` : environ 700 cycles pour chacun des 97 318 800 blocs de 64 pixels.
  Le reste de `light` est la copie des échantillons et la boucle par pixel.
- Le seuil relevé ne compte pas : 300 444 appels sur 9 011 images, 0,2 % de `light`.
  Le plancher l'évite presque partout.
- À 28 comparateurs, le réseau passe de 65,90 à 57,68 milliards de cycles (−12,5 %, comme 28 sur 32).
  La détection sur un fil passe de 62,2 à 59,8 s (−3,9 %), contre le passage 2, le plus rapide des trois d'avant et lancé juste avant le passage 4.
- Garder les 11 lignes dans les registres, par paquets de 8 voies, est plus lent : 109,13 et 102,96 milliards de cycles pour le réseau à 32 et à 28 comparateurs, un passage chacun.

Puis la détection de la 092 entière par la commande, avec `LANG=fr_FR.UTF-8` et un dossier de sortie neuf à chaque passage, sous `/usr/bin/time -v`, en 5 paires avant / après alternées.
La détection est l'écart entre les lignes « détection » et « suivi » de la sortie d'erreur, horodatées.

| Paire | Avant | Après | Gain | CPU avant → après |
| ---: | ---: | ---: | ---: | --- |
| 1 | 20,81 s | 19,36 s | 7,0 % | 243,8 → 242,6 s |
| 2 | 19,98 s | 19,78 s | 1,0 % | 244,7 → 243,9 s |
| 3 | 28,60 s | 19,73 s | écartée | 252,1 → 242,8 s |
| 4 | 19,50 s | 19,77 s | −1,4 % | 245,2 → 243,9 s |
| 5 | 20,03 s | 19,58 s | 2,2 % | 244,7 → 242,7 s |

- La paire 3 est écartée : 8 s au-dessus des autres passages d'avant, une charge de la machine dont le CPU n'a pas été relevé.
- Sur les 4 autres, la détection passe de 20,08 à 19,62 s en moyenne : 2,3 % en moyenne, 1,6 % en médiane.
  Le CPU baisse de 244,6 à 243,3 s en moyenne (0,5 %), et le pic reste de 1,30 à 1,35 Go des deux côtés.
- Le gain reste petit parce que le réseau ne pèse qu'environ 7 % du CPU de la commande (17,7 s sur un fil, contre 244 s en tout).
- Les pistes ne changent pas : le CSV est identique à l'octet sur les 5 paires, `compare local` donne les mêmes détections sur les 26 vidéos hors des plages abîmées (`--same-as`), et `rust:reference` trouve les 14 pistes de la 092.

Sur le M1 Max (macOS 27), même commande sous `/usr/bin/time -l`, en 3 paires alternées : avant 46980bf, après 5da2a01.
La détection passe de 11,37 à 11,13 s en moyenne, soit 1,7 % en médiane (3,2, 1,6 et 1,7 %, les trois du même signe).
Le CPU baisse de 1,5 % en médiane, de 121,8 à 119,7 s en moyenne, et le pic reste autour de 1 Go des deux côtés.
Le CSV des pistes est identique à l'octet dans les 3 paires.
Le gain est du même ordre que sur lumiel.
