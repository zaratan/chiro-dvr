# Exploration : migrer batdetect vers Rust ?

Étude du 10 octobre 2026, branche `exploration/rust` (depuis v0.1.1, c9948d2), worktree
`~/Projects/dvr-wt/rust`. Rien n'est à merger : le prototype est dans `rust-proto/` (non suivi),
`src/`, `tests/` et `docs/` n'ont pas changé, et `batdetect` garde son comportement par défaut.

## Recommandation (mise à jour après le second prototype et ses revues, 10 octobre 2026, 15 h 30)

Le rapport a deux parties.

- **La première partie (sections 1 à 5)** répond à la question posée le matin : un portage **identique
  au bit près**. Sa conclusion était : ne pas migrer.
- **La seconde partie (section 6)** répond à la demande de l'après-midi : l'identité au bit près n'est
  plus exigée, seulement « la même détection », et il fallait aller au plus vite. L'utilisateur a une
  issue ouverte pour améliorer la détection : la fidélité parfaite à Python n'est donc pas un critère.

**Nouvelle recommandation : passer tout batdetect en Rust, par étapes, en commençant par le moteur de
lecture et de détection (option E, section 7).**

L'utilisateur préfère un seul langage, et le projet a une semaine : premier commit le 3 octobre
2026, 71 commits, environ 3 800 lignes de code, 5 000 lignes de tests et 2 700 lignes de docs. À ce
stade, réécrire coûte à peu près autant que ce qui a déjà été construit. Le gain dure toute la vie de
l'outil :

- une seule chaîne d'outils (cargo) au lieu de mise, uv, ruff, basedpyright, pytest et PyInstaller ;
- un binaire de quelques Mo pour macOS, Linux et Windows (#93) ;
- une seule chaîne de détection ;
- la sortie de la GPL possible.

**La première étape est mesurée (option D, section 6).** Un moteur Rust décode la vidéo **une seule
fois**, repère les images abîmées pendant ce décodage (la passe ffprobe disparaît), détecte, et sert
les images des extraits. Python garde provisoirement le reste.

- **Mesures sur la 092** :
  - commande complète : **134 s → 18 s** ;
  - CPU : 437 s → 132 s ;
  - mémoire : 705 Mo → 570 Mo ;
  - pistes : les 14, mêmes débuts, mêmes nombres de points.
- **Sur 26 vidéos** :
  - **958 paires de pistes** dans les tolérances du test de référence. 8 pistes Python et 14 pistes Rust
    n'ont pas de correspondant. 20 vidéos sur 26 ont exactement les mêmes pistes.
  - Chaque écart vient d'une seule image au ras du seuil. Pas de biais visible d'un côté ou de l'autre.
  - Le test `reference` de la 092 passe tel quel.
- **Pas de PyO3**, donc ni maturin, ni module chargé par dyld (l'incident de macOS 27 ne touche que les
  bibliothèques chargées). Le moteur est un binaire de 3,6 Mo lié à un FFmpeg LGPL.

**Le reste de la migration ne rapporte presque plus de temps** : Python ne pèse que 2 à 4 s sur les 18
(estimation non mesurée). Il rapporte un seul langage et la distribution. **Ordre conseillé** :

1. le moteur ;
2. le banc et `synthetic/`, pour supprimer la détection Python et n'avoir qu'une chaîne de détection ;
3. le suivi et les tableaux ;
4. les sorties graphiques ;
5. la CLI et les langues ;
6. la distribution.

L'ensemble est estimé à **≈ 100 points**, dont environ 35 pour la première étape. **#93 se ferait à l'étape 6, en Rust**, plutôt que maintenant
avec PyInstaller sur trois plateformes, travail qui serait jeté.

Les options A' (`--workers 3`, 84 s) et B (cœur PyO3, 71 s) de la première partie ne servent plus que
si l'on renonce à Rust.

## Conditions de mesure

- Machine : la machine de développement, Apple M1 Max (10 cœurs, 64 Go), macOS 27.0. **Ce n'est pas la
  machine de Manon** (M3, 4 cœurs de performance et 4 d'efficacité, 16 Go) : rien n'y a été mesuré.
- Toutes les mesures lourdes ont tourné sous `lockf mesure.lock`, une commande par verrou, avec
  `/usr/bin/time -l`. D'autres agents tournaient hors verrou, avec une charge de 5 à 15. Le temps réel
  est donc bruité, d'environ ±5 % : deux lancements Python ont donné 134,0 et 136,0 s, et trois
  lancements de B 78,0, 71,5 et 71,0 s. Le temps CPU, lui, est stable.
- Réglages : mode `normal` par défaut (960 px, flou de 1,5 px, seuil 12, seuil par pixel k = 8),
  encodeur VideoToolbox, `--workers 1` sauf mention contraire.
- Logiciels : Python 3.14.8, numpy 2.5.3, OpenCV 5.0.0 (roue `opencv-python-headless`, avec FFmpeg
  7.1.1 embarqué), Rust 1.93 nightly et 1.91 stable, PyO3 0.29, maturin 1.15, ffmpeg-next 9.
- Vidéos : la 092 (`in/`), l'extrait de test, et les 089 et 091 (`dossier sans titre/img_0000/`), en
  lecture seule.

## 1. Profil actuel de la 092 (mode normal, `--workers 1`)

Commande complète, instrumentée par un lanceur hors dépôt qui chronomètre chaque fonction sans toucher
au code (`profile_run.py`, dans le scratchpad) :

| | Valeur |
| --- | --- |
| Temps réel | **134,0 s** (second lancement, non instrumenté : 136,0 s) |
| Temps CPU (user + sys) | 437 s (439 s) |
| Mémoire maximale | 705 Mo (723 Mo) |
| Pistes | 14, CSV identique entre les deux lancements |

| Étape | Temps | Part du total | Par image |
| --- | --- | --- | --- |
| **Détection** (fil principal) | **119,9 s** | **89 %** | 13,3 ms |
| dont médiane du fond (`temporal_median`, float32, 11 images 960×720) | 81,8 s | 61 % | 9,1 ms |
| dont taches (`find_blobs`) | 16,3 s | 12 % | 1,8 ms |
| dont reste (résidu en float64, moyennes, attente du lecteur) | 9,5 s | 7 % | 1,1 ms |
| dont flou (`target_blur`) | 7,8 s | 6 % | 0,9 ms |
| dont seuil par pixel (`raised_threshold`) | 4,5 s | 3 % | 0,5 ms |
| Fil lecteur, en parallèle : décodage | 5,1 s | (masqué) | 0,6 ms |
| Fil lecteur, en parallèle : réduction et gris | 20,7 s | (masqué) | 2,3 ms |
| Rendu des 14 extraits et du zoom | 12,1 s | 9 % | |
| Fond médian de l'image résumé | 1,4 s | 1 % | |
| Attente de la passe ffprobe après la détection | 0,0 s | 0 % | |
| Suivi, exclusion, CSV, image résumé | < 0,1 s | 0 % | |

- **Le temps réel est dans un seul fil Python, dont 61 % pour la médiane du fond.** Elle est déjà
  vectorisée (réseau de tri par `np.minimum` et `np.maximum`), mais elle fait 90 passes sur des
  images float32 de 2,7 Mo, sur un seul cœur.
- **Le temps CPU est ailleurs.** Sur 437 s, le fil principal en prend 119 et le fil lecteur environ 25.
  Il reste environ 290 s, estimées par différence et non ventilées :
  - les fils de décodage de FFmpeg, sur deux lectures (la détection, puis le rendu jusqu'à 3:58) ;
  - la passe ffprobe, environ 55 s (docs/09) ;
  - les encodeurs.
- **La passe ffprobe est masquée tant que la détection dure plus d'environ 57 s.** Dès qu'on accélère
  la détection, elle redevient le plancher (§3).
- docs/05, section « Pourquoi Python », est périmée : elle cite une médiane de 0,7 ms en uint8, d'avant
  le passage au float32 par défaut (9,1 ms aujourd'hui). Je le signale, sans corriger : ce n'est pas
  dans le périmètre.

## 2. Dépendances et équivalents Rust

batdetect appelle 18 fonctions d'OpenCV et une trentaine de numpy. Pour la détection, seules six
comptent : la lecture vidéo, `resize`, `cvtColor`, `GaussianBlur`, `morphologyEx` et
`connectedComponentsWithStats`.

| Besoin | Aujourd'hui | Crate Rust candidate | Licence | Maturité | Identique au bit près ? |
| --- | --- | --- | --- | --- | --- |
| Lecture vidéo et conversion en BGR | `cv2.VideoCapture` (FFmpeg 7.1.1 de la roue, swscale bicubique) | `ffmpeg-next` 9 (ou `rsmpeg`), liaison à FFmpeg | WTFPL pour la crate, LGPL ou GPL pour FFmpeg | bonne, très utilisée | **Oui avec FFmpeg 7.1.1** : 1 230/1 230 images identiques. **Non avec FFmpeg 9.0.2** : 0/300 images identiques, jusqu'à 3 niveaux d'écart |
| Réduction `INTER_AREA` ×2/3 | `cv2.resize` | aucune ne reproduit OpenCV (`fast_image_resize` et `image` filtrent autrement) | — | — | Oui par réimplémentation (≈ 80 lignes) : 1 230/1 230. **Non au facteur 2** : OpenCV y calcule en entiers (relevé par la revue) |
| Gris | `cv2.cvtColor` | réimplémentation (5 lignes) | — | — | Oui avec les coefficients 15 bits d'OpenCV (`BY15/GY15/RY15`). Les coefficients 14 bits classiques ratent 0,3 % des pixels |
| Flou gaussien float32 | `cv2.GaussianBlur` | `kornia-imgproc`, `imageproc` : autres arrondis | Apache-2.0, MIT | moyenne | Oui pour le noyau de 9, par réimplémentation imitant le chemin NEON d'OpenCV (FMA) : 1 230/1 230. **Non pour un noyau ≤ 5**, qui prend un autre chemin d'OpenCV (revue) |
| Noyau gaussien | `getGaussianKernel` (arithmétique logicielle `softdouble`) | calcul en f64 avec l'`exp` de la libm | — | — | Identique en float32 pour σ = 1. À vérifier pour chaque σ exposé |
| Fermeture morphologique, ellipse | `cv2.morphologyEx` | `imageproc::morphology` (autres éléments structurants) | MIT | moyenne | Oui par réimplémentation (calcul entier) : 1 228/1 228 |
| Composantes 8-connexes et statistiques | `cv2.connectedComponentsWithStats` | `imageproc::region_labelling` (autre numérotation) | MIT | moyenne | Oui par réimplémentation numérotée comme Spaghetti (premier bloc 2×2) : 1 228/1 228. Identique aussi au chemin parallèle d'OpenCV en 3000×2000 (revue) |
| numpy : médiane, moyennes, seuil | numpy | `ndarray` ou boucles simples | MIT/Apache | bonne | Oui. La moyenne float32 dépend de la somme par paires de numpy : elle est copiée, et à figer par un test à chaque montée de numpy |
| Parallélisme | un fil lecteur, numpy sur un fil, `--workers` en processus | `rayon` | MIT/Apache | excellente | sans objet |
| Dessin des extraits (`rectangle`, `polylines`, `circle`, `putText`) | OpenCV | `tiny-skia`, `imageproc::drawing`, `ab_glyph` | BSD-3, MIT | bonne | Non requis (rendu visuel). Les polices Hershey d'OpenCV n'existent pas en Rust |
| Image résumé (PNG, pastilles, flèches, légende) | OpenCV et numpy | `image`, `tiny-skia` | MIT/Apache, BSD-3 | bonne | Non requis, mais tout le dessin serait à refaire |
| Encodage des extraits | sous-processus `ffmpeg` (Homebrew) | le même, ou `ffmpeg-next` en interne | GPL avec libx264 ; openh264 sous BSD ; VideoToolbox sur Mac seulement | — | Sans objet : VideoToolbox n'est pas déterministe (docs/09) |
| ffprobe (images abîmées) | sous-processus `ffprobe` | le même, ou lecture des journaux de libav en interne | — | — | Non mesuré. L'ordre des messages de libav a déjà piégé le projet (docs/09) |
| Arguments, traduction | argparse et gettext (catalogue partagé avec argparse) | `clap` avec `gettext-rs` ou `fluent` | MIT/Apache | bonne | Sans objet. Les textes propres de `clap` ne passent pas par gettext |
| Liaison Python | — | `pyo3` 0.29, `numpy` 0.29, `maturin` 1.15 | MIT/Apache, BSD-2 | bonne | Sans objet |

**L'arithmétique se reproduit ; la difficulté est le nombre de chemins.** Chaque fonction d'OpenCV a
plusieurs chemins, selon le type, la taille du noyau, le facteur d'échelle et le processeur (NEON, IPP,
AVX). Le prototype ne reproduit que ceux des réglages par défaut. La revue en a trouvé quatre autres,
qu'une option peut atteindre et que le prototype ne reproduit pas :

- la réduction au facteur 2 ;
- le flou à noyau ≤ 5 ;
- `--mode quick`, où la moyenne se fait sur des u8 ;
- les zones OSD.

## 3. Mesures

### Le levier Python déjà disponible : `--workers`

`--workers 1` est le défaut depuis docs/09, qui l'a fixé quand la détection était limitée par le
décodage (480 px, uint8). Aujourd'hui, elle est limitée par le calcul : 13,3 ms par image dans le fil
principal, contre environ 3 ms pour le lecteur.

| 092, commande complète | Temps réel | CPU | Mémoire max | CSV |
| --- | --- | --- | --- | --- |
| `--workers 1` (défaut) | 134,0 / 136,0 s | 437 / 439 s | 705 / 723 Mo | référence |
| `--workers 2` | 91,1 s | 493 s | 611 Mo | identique (`cmp`) |
| `--workers 3` | 84,3 s (charge 15 pendant la mesure) | 567 s | 602 Mo | identique (`cmp`) |

Sans changer de code, la 092 passe à 84 s. En contrepartie, chaque tranche relit la vidéo depuis le
début : environ 130 s de CPU en plus. Sur la machine de Manon, le gain dépend de ses 4 cœurs de
performance. Ce n'est pas mesuré.

### Prototype 1 : cœur en Rust appelé par PyO3 (option B)

Le noyau fusionné remplace dans `detect_frames` les lignes « médiane, résidu, seuil par pixel ». Il
fait environ 375 lignes : 205 de noyau, 170 de liaison. Le reste ne change pas : décodage et flou par
OpenCV, `find_blobs`, suivi, rendu. Il s'active par `BATDETECT_RUST=fused` dans `rust-proto/run.py`,
qui remplace la fonction au chargement, sans modifier `src/`.

Médiane seule, 11 images 960×720 en float32 :

| Méthode | Temps |
| --- | --- |
| `np.median` | 92 ms |
| Réseau de tri numpy (actuel) | 8,9 ms |
| Même réseau numpy, 4 fils Python par bandes (meilleur cas) | 4,8 ms |
| Même réseau numpy, 8 fils | 8,3 ms (le GIL et les petits appels annulent le gain) |
| Rust, un cœur | 5,1 ms |
| Rust, `rayon` | **0,85 ms** |

Sur la 092 :

| Variante | Temps réel | CPU | Mémoire max | Identité |
| --- | --- | --- | --- | --- |
| Cœur Rust `rayon`, commande complète | **78,0 / 71,5 / 71,0 s** | 443 / 438 / 436 s | 670 à 676 Mo | CSV et `params.json` identiques (`cmp`), trois fois |
| Cœur Rust sur un cœur, commande complète | 103,9 s | 407 s | 670 Mo | CSV identique |
| Python, détection seule | 120,7 s | 319 s | 349 Mo | 780 détections, référence |
| Cœur Rust `rayon`, détection seule | 47,1 s | 317 s | 331 Mo | dictionnaire des détections identique octet pour octet (pickle) |

Le lancement instrumenté de B (71,0 s) se décompose ainsi :

| Étape | Temps |
| --- | --- |
| Détection | 49,8 s |
| Attente de la passe ffprobe | **7,2 s** |
| Rendu | 12,1 s |
| Fond de l'image résumé | 1,4 s |

**La passe ffprobe, sur un seul fil, redevient le plancher.** Elle dure environ 57 s, contre 49,8 s pour
la détection. Le contrôle multifil décrit par docs/09 (8 à 16 s, avec une passe complète seulement en
cas d'erreur) gagnerait encore environ 7 s, en A avec workers comme en B.

Identité sur d'autres vidéos, détection seule : les mêmes détections octet pour octet que Python sur
l'extrait (402), la 089 (4 791) et la 091 (34 524 détections, jumelles qui bougent). La détection seule
prend 47 à 51 s, contre 121 à 125 s en Python.

Le CPU ne baisse pas : le gain est du temps réel, et il vient surtout de `rayon`. Sur un seul cœur, Rust
ne bat numpy que d'un facteur 1,7, et B tombe à 104 s. Avec `rayon`, environ 30 s de CPU de plus
qu'avec un seul cœur ressemblent à de l'attente active (hypothèse, non instrumentée).

### Prototype 2 : détection complète en Rust, binaire autonome (pour estimer C)

`rust-proto/target/ff711-lgpl/release/detect` lit la vidéo par FFmpeg 7.1.1, puis enchaîne réduction,
gris, flou, moyenne, fenêtre, médiane, seuil par pixel et taches. Il ne fait ni stabilité, ni plages
abîmées, ni suivi, ni rendu. Il est lié statiquement à un FFmpeg 7.1.1 LGPL réduit au décodage (h264 et
hevc, mov, swscale), construit en 9 s, et ne dépend d'aucune bibliothèque hors système : 4,4 Mo.

| Vidéo | Python, détection seule | Rust autonome `rayon` | Rust, noyaux sur un cœur | Identité |
| --- | --- | --- | --- | --- |
| Extrait (1 230 images) | 16,4 s, 344 Mo | 6,8 à 8,4 s, 233 Mo | 19,5 s | 402/402 détections identiques |
| 092 (9 011 images) | 120,7 s, 319 s CPU, 349 Mo | **41,4 s**, 337 s CPU, 226 Mo | 141,5 s, 263 s CPU | **780/780 identiques** |
| 089 | 124,7 s, 374 Mo | 44,2 s, 237 Mo | | **4 791/4 791 identiques** |
| 091 | 122,5 s, 520 Mo | 42,2 s, 240 Mo | | **34 524/34 524 identiques** |

Étape par étape, sur les 1 230 images de l'extrait, contre OpenCV et numpy : réduction, gris, flou,
moyenne, fermeture, composantes et `find_blobs` sont identiques sur toutes les images.

- **Un portage au bit près de la détection par défaut est faisable**, décodage compris, sur trois
  vidéos entières.
- **Un portage naïf est plus lent que Python.** OpenCV est optimisé (NEON, plusieurs fils) : avec des
  noyaux sur un seul cœur, le binaire prend 141 s contre 121 s. Il ne gagne qu'en parallélisant à la
  main la réduction et le flou.
- **C n'irait pas plus vite que B.** La détection descend à 41 s, mais la passe ffprobe en dure environ
  57. Estimation de la commande complète : 57 + 12 + 1,4 ≈ 71 s, autant que B. Seul un contrôle
  ffprobe multifil changerait ce plancher, et il est possible aussi en Python.
- Mémoire : 226 à 240 Mo contre 349 à 520 Mo. La limite de 16 Go n'est en jeu dans aucune option.

### Ce qui a surpris

- **Le décodage dépend de la version de FFmpeg, pas du langage.** Le prototype reproduit l'appel
  d'OpenCV (`sws_getCachedContext(..., SWS_BICUBIC)`, sans espace colorimétrique) et donne des images
  BGR identiques à OpenCV, à deux conditions : être lié au FFmpeg 7.1.1 de la roue, ou à un FFmpeg
  7.1.1 LGPL compilé à part. Lié au FFmpeg 9.0.2 de Homebrew, il en diffère de 1 à 3 niveaux sur toutes
  les images.
  - docs/09 attribuait l'écart de 18 niveaux, mesuré avec ffmpeg en sous-processus, à la conversion BGR.
  - Hypothèse non vérifiée : l'essentiel viendrait de ce qu'ffmpeg en ligne de commande respecte
    l'étiquette BT.709 de la vidéo, alors qu'OpenCV convertit en BT.601.
- **Le gris d'OpenCV 5 utilise des coefficients sur 15 bits** (3735/19235/9798). La première version
  du prototype, en 14 bits, ratait un niveau sur 61 images de l'extrait. Cela suffisait à changer
  l'amplitude d'une détection au sixième chiffre. Hypothèse non vérifiée : ce chemin pourrait expliquer
  une part de l'écart Mac/Linux de docs/05, si le chemin x86 (IPP) arrondit autrement.
- **Incident de chaîne de construction, non résolu.** Sur macOS 27, avec l'éditeur de liens d'Apple
  ld-27037, dyld refuse de charger certains modules PyO3 (« mis-aligned LINKEDIT string pool »).
  - Cause mesurée : quand la table des symboles indirects a une longueur impaire, la table de chaînes
    qui la suit n'est plus alignée sur 8 octets.
  - Le contournement du prototype ajoute un import, ce qui rend la longueur paire. C'est un pile ou face
    qui cassera au prochain import qui la rend paire d'avance.
  - `rust-lld` ne lit pas le SDK de macOS 27.
  - C'est un bloquant pour B tant qu'il n'est pas résolu, par exemple par un correctif d'Apple ou un
    contrôle `stroff % 8` après l'édition de liens.

## 4. Les options

A' (Python avec `--workers`) et B' (« B large », Rust pour le décodage et les pixels, Python pour le
reste) sont ajoutées après la revue d'architecture.

| | A. Python, défaut actuel | A'. Python, `--workers 3` | B. Cœur Rust par PyO3 | B'. Rust pour le décodage et les pixels, Python pour le reste, sans OpenCV | C. Réécriture complète |
| --- | --- | --- | --- | --- | --- |
| **Identité sur la 092** | référence | CSV identique (`cmp`) | CSV identique (`cmp`, 3 fois). Les chemins hors défaut retombent sur Python | détection identique (prototype 2) ; les quatre chemins hors défaut restent à écrire | comme B' ; après la bascule, Rust devient la référence |
| **Identité Mac / Linux / Windows** | non : 4 images, 2 points d'écart Mac/Linux (docs/05) | idem | idem | possible : décodage et arithmétique maîtrisés partout. Non mesuré : l'assembleur de swscale diffère selon le processeur | idem B' |
| **Temps sur la 092** (M1 Max) | 134-136 s | **84 s** | **71-78 s** | ≈ 71 s, estimé (plancher ffprobe) | ≈ 71 s, estimé (plancher ffprobe) |
| **CPU** | 437 s | 567 s | 436-443 s | non mesuré | non mesuré (détection : 337 s contre 319 s) |
| **Mémoire max** | 705-723 Mo | 602 Mo | 670-676 Mo | détection : 226 Mo | détection : 226 Mo |
| **Binaire macOS** | PyInstaller : 63 Mo téléchargés, 147 Mo installés | idem | idem, plus 0,7 Mo | PyInstaller sans la roue OpenCV (≈ 100 bibliothèques de moins) : plus petit, non mesuré | détection seule : 4,4 Mo. ffmpeg et ffprobe restent nécessaires pour l'encodage et la passe des images abîmées |
| **Linux, Windows (#93)** | une roue OpenCV par plateforme, licences à vérifier une par une | idem | idem, plus une extension compilée par plateforme (maturin, matrice de 3 systèmes) | une extension et un FFmpeg LGPL à construire par plateforme | cargo construit sur chaque runner ; FFmpeg à construire par cible ; `ffmpeg.exe` et `ffprobe.exe` à livrer sous Windows |
| **Licence du binaire** | GPL v3 (FFmpeg GPL de la roue OpenCV) | idem | idem | **MIT avec FFmpeg LGPL** | **MIT avec FFmpeg LGPL**, mais un encodeur H.264 logiciel est GPL (libx264) ou passe par openh264 ; VideoToolbox n'existe que sur Mac |
| **Effort (planning poker)** | 0 | 2 (changer le défaut, docs/09, test de la machine de Manon) | 13 : 8 (module, branchement, tests d'identité, docs 05/09/10) + 5 (construction de l'extension en CI et dans PyInstaller) | ≈ 30, non chiffré finement : décodage, 5 fonctions OpenCV, quatre chemins hors défaut, dessin des extraits et de l'image résumé sans OpenCV | 80 à 100 : une dizaine d'issues de 8, plus les chemins hors défaut, les polices, et peut-être une nouvelle validation par Manon |
| **Maintenance pour une personne** | inchangée | inchangée | chaîne Rust dans `mise.toml`, `maturin` à la place de `uv_build`. **Le noyau existe en deux versions pour toujours** (repli Python), dans la zone même où le banc expérimente (docs/07) | deux langages, frontière au niveau de l'image | un langage à terme ; plus de montées d'OpenCV ni de numpy qui changent les résultats |
| **Ce qui se perd** | rien | rien | rien (banc, `synthetic/`, tests et docs/10 restent en Python) | OpenCV pour le dessin (à refaire avec numpy ou Pillow) | numpy pour le banc, les vidéos synthétiques, les 5 000 lignes de tests sur images fabriquées, l'écosystème de docs/07 ; docs/10 à réécrire |
| **Risques** | temps à 1440 px (283 s, docs/09) | CPU et chaleur sur un portable à 4 cœurs de performance | incident dyld (bloquant), montée de numpy, suivi de PyO3 à chaque version de Python, sursouscription `rayon` + `--workers` | réplication d'OpenCV chemin par chemin | idem B', plus la durée et une régression de qualité pendant la migration |

## 5. Recommandation détaillée

Critères dans l'ordre de l'utilisateur :

1. **Identité au bit près.** Sur une même plateforme, A, A' et B la tiennent ; B' et C la tiennent pour
   la détection par défaut. Entre plateformes, aucune option ne la tient aujourd'hui. Seules B' et C
   pourraient la tenir, avec une nouvelle référence validée une fois par Manon. C'est le seul critère
   qui puisse justifier Rust, et seulement si #93 l'exige.
2. **16 Go.** Aucune option n'en approche (600 à 720 Mo).
3. **Distribution.** B l'alourdit : une extension par plateforme, plus l'incident dyld. B' et C
   l'allègent (sortie de la GPL, binaire plus petit), mais ffmpeg et ffprobe restent à livrer, et
   l'encodage H.264 hors Mac pose sa propre question de licence.
4. **Temps.** A' prend la plus grosse part du gain sans code : 134 → 84 s. B gagne 13 s de plus ;
   C rien de plus que B.
5. **Maintenance.** A et A' : rien. B : deux noyaux à tenir en phase. C : un chantier de plusieurs mois
   de soirées.

**Donc :**

- **Maintenant : A', pas de Rust.** Mesurer `--workers 2` et `3` sur la machine de Manon (temps et CPU),
  puis changer le défaut si le gain s'y confirme. C'est un réglage, à décider par l'utilisateur.
- **Ne pas faire B.** 13 s de gain ne paient ni la chaîne Rust, ni le noyau en double, ni un bloquant
  d'éditeur de liens, ni le surcoût de construction pour #93.
- **Ne pas faire C.** Elle coûte six à sept fois B pour le même temps. Elle reproduit OpenCV chemin
  par chemin (quatre trous déjà trouvés) et prive le banc et les tests de numpy.
- **Rouvrir la question sous la forme B'**, si #93 doit donner les mêmes résultats sur toutes les
  plateformes, ou si la GPL devient gênante. Le prototype 2 en est la moitié la plus risquée, déjà
  faite au bit près.

### À mesurer pour en être sûr

1. **Machine de Manon** : A et A' (`--workers 2`, `3`), temps réel et CPU, sur secteur et sur batterie.
2. **Contrôle ffprobe multifil** (docs/09) en complément de A' : environ 7 s, à remesurer.
3. Pour B' seulement : FFmpeg LGPL 7.1.1 sous Linux x86 et Windows, identique entre plateformes ? Non
   mesuré : l'assembleur de swscale et le chemin d'OpenCV y diffèrent. Essayer
   `SWS_BITEXACT | SWS_ACCURATE_RND`.
4. Vidéos abîmées (093, 122) : identité du décodage Rust quand le décodeur répare des images. La revue
   note que le prototype arrête la lecture sur un paquet corrompu, alors qu'OpenCV continue.
5. Piste Python pure, non essayée : une médiane glissante, où chaque pixel fait entrer une valeur et en
   sortir une, au lieu de 45 échanges. Résultat identique au bit près ; à comparer à A'.

### Issues proposées (non créées)

Pour A' :

- « Choisir `--workers` par défaut d'après la machine de Manon (M3) : temps, CPU, CSV identique » — 2
- « Passe ffprobe multifil, passe complète seulement en cas d'erreur (docs/09) » — 3
- « docs/05 : mettre à jour “Pourquoi Python” avec le profil de la détection en float32 » — 1

Pour B', seulement si la décision n° 1 est « oui » :

- « Étude : décodage FFmpeg LGPL identique entre macOS, Linux et Windows » — 5

## 6. Second prototype : un moteur Rust rapide, « même détection » plutôt que mêmes bits

Demandé l'après-midi : optimiser au maximum, sans exiger l'identité au bit près. Code dans
`rust-proto/fast/` (environ 1 500 lignes de Rust) et `rust-proto/fast_run.py` (environ 200 lignes, qui
branche le moteur sous la CLI sans toucher à `src/`). Un agent a validé la détection des images abîmées
pendant le décodage ; un autre a comparé Python et Rust sur les 26 vidéos de `img_0000`. Toutes les
mesures ont tourné sous le verrou, sur le M1 Max.

### Ce qui change dans la chaîne

1. **Une seule lecture de la vidéo**, au lieu de trois (détection OpenCV, passe ffprobe, rendu OpenCV).
   Le décodeur FFmpeg 7.1.1, en logiciel et en fils de frame, lève sur chaque image le champ
   `decode_error_flags`. Sur la 122 (195 erreurs, 13 images perdues), la 093 (11) et la 092 (0), il
   donne **exactement** les images en erreur, les sauts, le nombre d'images et les plages abîmées de
   ffprobe. Le drapeau `AV_FRAME_FLAG_CORRUPT` n'est jamais levé, et les messages du journal se
   rattachent mal en multifil : c'est bien `decode_error_flags` qu'il faut lire. Il faut des fils de
   frame : avec des fils de tranche, h264 coupe la dissimulation d'erreur et donc le drapeau.
2. **Le plan Y du décodeur, sans conversion BGR.** Sur ces vidéos thermiques, 99,7 % des pixels sont
   neutres. La conversion Y → gris est une table de 256 valeurs mesurée sur la sortie d'OpenCV
   (swscale n'arrondit pas comme la formule théorique). La réduction 3:2 se fait en entiers, avec
   l'arrondi 8 bits de Python. Sans cet arrondi (première version), une piste de l'extrait se coupait
   en deux.
3. **Le flou** est la gaussienne d'OpenCV (σ = 1 pixel de travail), quantifiée à 4096 par axe, en entiers
   32 bits. La première version, binomiale [1 4 6 4 1], était plus plate au centre (0,375 contre 0,399) :
   l'amplitude s'écartait de 1 à 2 %.
4. **La médiane du fond** se fait en u16 (8 pixels par instruction NEON) avec un réseau de Batcher élagué
   aux sorties du milieu, vérifié par le principe 0-1 pour 1 à 16 échantillons. Le résidu est en float32.
   Le seuil par pixel n'est calculé qu'aux candidats. Le noyau sort directement la liste des pixels
   au-dessus du seuil : plus de carte float64 de l'image entière.
5. **La fermeture et les composantes** sont creuses, proportionnelles aux pixels touchés.
6. **Le parallélisme** se fait par image : préparation des images en parallèle, puis une image cible par
   cœur.
7. **Le rendu** : un petit serveur d'images, `framesrc`, se positionne exactement sur l'image-clé grâce à
   l'index du mp4, décode et convertit en BGR comme OpenCV, et ne sert que les images des extraits.
   Avec les mêmes détections, les extraits (en x264) et l'image résumé sont identiques à l'octet près
   à ceux de Python. Les extraits sont rendus en parallèle, chacun avec son encodeur.

### Mesures sur la 092 (commande complète, sous verrou)

| Variante | Temps réel | CPU | Mémoire max | Pistes |
| --- | --- | --- | --- | --- |
| Python actuel | 134,0 / 136,0 s | 437 s | 705 Mo | 14 (référence) |
| Python `--workers 3` | 84,3 s | 567 s | 602 Mo | identiques |
| Cœur PyO3 (option B) | 71-78 s | 438 s | 670 Mo | identiques |
| **Moteur Rust, décodage logiciel séquentiel (option D)** | **17,9 / 18,1 s** | **132 s** | **570 Mo** | **14, mêmes débuts, mêmes points** |
| Moteur Rust, VideoToolbox en 8 tranches | 13,5 s | 76 s | 1,3-1,5 Go | 14 ; **aveugle aux images abîmées** |

Le temps de la variante D se décompose ainsi :

| Poste | Temps |
| --- | --- |
| Lecture et détection (`fastdet`) | 11,0 s, dont environ 60 s de CPU de décodage |
| Rendu des 15 extraits | 5,4 s |
| Suivi, CSV, image résumé et démarrage de Python | environ 1,4 s |

Détection seule, d'autres variantes pour situer :

| Détection seule, 092 | Temps réel | CPU | Identité |
| --- | --- | --- | --- |
| Python | 120,7 s (+ ffprobe 54 s de CPU en parallèle) | 319 s | référence |
| `fastdet` (logiciel, une passe, dégâts compris) | 10,9 à 11,5 s | 97-98 s | 781 détections ; les 780 de Python retrouvées à 3 px près, amplitude à 0,02 % près en médiane |
| `fastdet2`, VideoToolbox, 8 tranches | 6,4 s | 42 s | octet pour octet comme `fastdet`, mais sans détection des dégâts |
| `fastexact` (plan Y puis BGR d'OpenCV en parallèle, arithmétique d'OpenCV et numpy) | 22,7 / 24,3 s | 195 s | **identique au bit près** à Python sur la 092, la 089, la 091 et l'extrait |

`fastexact` donne la borne : la fidélité au bit près coûte environ deux fois la variante D, et reste
5 fois plus rapide que Python.

### Sur 26 vidéos (agent de comparaison)

Même `Probe` des deux côtés, puis `exclude` et `track_detections` avec les réglages par défaut :

- **Pistes** : 966 en Python, 972 en Rust. **958 paires** dans les tolérances du test de référence
  (début ±5 images, points ±3) ; 8 pistes Python et 14 pistes Rust n'ont pas de correspondant dans les
  tolérances. **20 vidéos sur 26** ont exactement les mêmes pistes, dont la 092 : le test `reference`
  passe tel quel avec le moteur Rust.
- **Détections** : hors des plages ignorées, Rust retrouve 98,2 % à 100 % des détections Python selon la
  vidéo. Les taux bas sur toute la vidéo (72 % sur la 120) viennent des plages abîmées ou instables,
  vidées des deux côtés avant le suivi.
- **Temps de détection cumulé** : 52,7 min en Python contre 5,5 min en Rust (×9,7), 46 min de CPU.
- **Les 9 divergences** (094, 097, 098, 099, 100, 126) viennent chacune d'une seule image. Le suivi est
  sensible à ce qui se passe au ras du seuil : fusion des jumelles à 36 px, `min_travel` de 45 px,
  `min_hits` de 6.
  - **Dans 5 cas, une tache de 4 px n'existe que chez Python** :
    - 094, 097 à 0:20, 097 à 2:42 et 098 à 0:41 : la piste Python est plus longue, d'un seul tenant, ou
      la seule à exister ;
    - 100 à 1:36 : cette tache fait disparaître côté Python deux pistes que Rust garde (mécanisme non
      isolé).
  - **Dans 3 cas, une tache de 4 px n'existe que chez Rust** : 098 à 0:05 et 099 à 0:27 (courtes pistes en
    plus, de 7 et 6 points) ; 126 à 1:31 (piste coupée en 27 + 11 points au lieu de 32).
  - **Dans le dernier cas** (099 entre 0:14 et 0:22), c'est la forme d'une tache de 43 à 45 px qui
    diffère, et des pistes échangent leurs points.
  - **Pas de biais visible** entre les deux chaînes, sur 9 cas et sans vérité terrain. Ce que sont ces
    pistes, c'est à Manon de le dire : temps et détails dans `scratchpad/allvideos/resultats.md`.

### VideoToolbox : rapide, mais pas seul

- Une session décode à environ 210 images par seconde. **8 sessions en parallèle** lisent la 092 en
  **5,2 s pour environ 4 s de CPU**, contre 7,2 s et 60 s en logiciel. Le Y décodé est identique à celui
  du logiciel.
- **Il ne signale aucune image réparée.** Sur la 093, il masque les 11 erreurs sans rien dire ; sur la
  122, il sort 8 623 images, contre 8 998 pour le décodage logiciel et ffprobe.
- La version en tranches contrôle les `pts` contre l'index du mp4 et se replie sur le décodage
  séquentiel si une image manque. Mais elle ne peut pas voir une erreur masquée.
- Un balayage logiciel allégé (sans transformée inverse ni filtre de boucle), lancé à côté pour lever
  les drapeaux d'erreur, coûte encore 52 s de CPU au lieu de 61. VideoToolbox plus balayage ne gagne donc
  que du temps réel sur une machine à beaucoup de cœurs, pas de CPU. **Non retenu par défaut.**
- Le décodage logiciel en tranches ne gagne rien non plus (11,0 s contre 10,9 s) : le CPU est déjà plein.
- Le gain absolu n'est que d'environ 4,5 s par vidéo. VideoToolbox n'existe que sur Mac, alors que #93
  impose le décodage logiciel partout. Le débit des 8 sessions est celui du M1 Max, pas forcément du M3.
  Enfin, si VideoToolbox ne démarre pas, `fastdet2` panique au lieu de se replier (revue).
  **Conclusion : `fastdet2` sort du périmètre** ; il reste dans le prototype comme mesure.

### Limites du prototype, relevées en revue

Revues `rust-expert` (code) et `tech-architect` (rapport). Ce qui a été corrigé pendant l'étude :

- **Bogue de numérotation du rendu, corrigé.** `framesrc` choisissait les images par numéro
  d'échantillon mp4, alors que la détection les numérote dans l'ordre de décodage. Sur une vidéo qui perd
  des images (103, 120, 122), les boîtes des extraits auraient été décalées : de 13 images (0,43 s) sur
  la 103 après 3:37. Désormais :
  - les images sont choisies par `pts` ;
  - chaque image servie porte son `pts`, et Python vérifie qu'il correspond au numéro attendu ;
  - un extrait qui commence dans une zone abîmée est décodé depuis l'image-clé qui précède cette zone.

  Vérifié sur la 103, extraits en x264 à détections égales : **15 extraits sur 16 identiques à l'octet
  près**. Le 16e (3:40) ne diffère que sur des images réparées par le décodeur. Or **OpenCV lui-même
  n'est pas déterministe sur ces images** : trois lectures Python successives de la 103 donnent une
  fois 14 images différentes.
- **`fastdet` sort en erreur** (code 2) sur un format de pixel non géré, au lieu de tronquer sa sortie
  sans rien dire.

Ce qui reste à faire avant tout passage en production. C'est inclus dans l'effort de l'étape 1, à la
section 7 :

- **Le moteur ne vaut que pour du 1440×1080 aux réglages par défaut.**
  - Il réduit aux 2/3 toute taille divisible par 3, et son flou suppose une échelle de 1,5. Une vidéo
    1920×1080 serait traitée en 1280×720 au lieu de 960×540, **sans message**.
  - Pas de zones OSD, pas de mode `quick`, réseau de médiane élagué pour 11 échantillons.
  - Il faut refuser ce qu'il ne sait pas faire, puis généraliser. Les réglages que le banc explore sont
    justement hors défaut.
- **Plage complète (`yuvj420p`)** acceptée à tort : la table Y → gris correspond à la plage limitée.
  Mesuré par la revue : jusqu'à 18 niveaux d'écart, soit un seuil effectif environ 14 % plus bas.
- **Images abîmées : `decode_error_flags` signale davantage qu'ffprobe.**
  - Sur une copie de l'extrait abîmée exprès, le moteur signale les images 248, 555 et 860 ; ffprobe
    seulement la 555. Les deux autres ne sont que des messages d'information (« concealing 4680 DC »),
    alors que 77 % des macroblocs de la 248 sont masqués.
  - Le moteur exclurait donc 77 images de plus. C'est sans doute mieux, mais c'est un changement de
    comportement à documenter (docs/02, docs/10).
  - À l'inverse, une erreur sans masquage (SEI tronqué) lui échapperait.
  - Sur les 26 vraies vidéos : 25 identiques à ffprobe ; sur la 124, une image en erreur de plus.
  - Un test canari (vidéo abîmée fabriquée à des positions fixes) doit garder ce drapeau sous contrôle
    à chaque montée de FFmpeg. Avec 1 fil comme avec 4, les résultats sont identiques sur la 122 : le
    vrai interdit, ce sont les fils de tranche.
  - `rapports/plan-1.md` disait que h264 ne remplit jamais ce champ. La mesure dit le contraire (valeur
    12, masquage actif et tranches décodées), avec FFmpeg 7.1.1.
- **Le contrôle du nombre d'images ne contrôle plus rien.** Avec une seule lecture, les deux nombres
  comparés par `check_frame_count` viennent du même décodage. Il faut une source indépendante (nombre
  d'échantillons du mp4 comparé au nombre d'images décodées, en tenant compte des pertes connues). Il
  faut aussi qu'une erreur d'entrée-sortie ne passe plus pour une fin de fichier : l'itérateur de
  paquets de ffmpeg-next s'arrête sans erreur.
- **Images par seconde** : le moteur prend `r_frame_rate`, Python `CAP_PROP_FPS`. C'est identique sur
  ces vidéos ; à 25 i/s, la demi-fenêtre pourrait différer. Le plus simple : la passer depuis
  l'appelant.
- **Erreurs Python** : un échec du moteur doit devenir une `VideoError`, pour que le lot continue avec
  les vidéos suivantes.
- **Reproductibilité de la construction** : le FFmpeg statique vient du scratchpad et a été compilé pour
  macOS 27. Il faut un script de construction épinglé (version, somme sha256, ligne `configure`,
  `-mmacosx-version-min=14`) et la ligne `configure` dans le NOTICE. `--disable-autodetect` est ce qui
  garde la LGPL.
- **Mémoire** : 0,56 Go en séquentiel. Le seul réglage non borné est le nombre de tranches de
  `fastdet2`, environ 0,2 à 0,3 Go par tranche. Un éclair plein cadre (obturateur thermique) ferait
  réserver environ 160 Mo par cible dans la fermeture : à borner.

### Images abîmées : moteur contre ffprobe sur les 26 vidéos

ffprobe (arguments de `probe.py`) a été comparé aux tables du moteur, vidéo par vidéo :

- **25 vidéos sur 26 identiques** : nombre d'images, images en erreur, sauts de `pts`, images-clés et
  plages abîmées. 19 vidéos ont au moins une erreur (jusqu'à 309 sur la 124), et 3 perdent des images
  (103, 120, 122).
- **La 124** est la seule différence : le moteur signale une image en erreur de plus, la 2197 (310
  contre 309). Cela fait une plage abîmée de plus, soit 43 images exclues en plus (7 458 contre 7 415).
  C'est le même sens que sur la vidéo abîmée fabriquée de la revue : le moteur voit un peu plus de
  masquages qu'ffprobe, jamais moins sur ces 26 vidéos.

### Optimisations encore possibles (estimations, sauf mention « mesuré »)

Profil de départ, la 092 en variante D : 18 s au total.

- Lecture et détection : 11 s, pour 94 s de CPU, dont environ 60 s de décodage H.264 et 34 s de
  calcul.
- Rendu : 5,4 s.
- Python et son démarrage : environ 1,4 s.

| Piste | Gain estimé | Change la détection ? | Où |
| --- | --- | --- | --- |
| Chevaucher les vidéos d'un lot : rendu de la vidéo *n* pendant la détection de *n+1* | dans un lot, environ 12-13 s par vidéo au lieu de 18 | non | CLI, toute étape |
| Rendu sans aller-retour BGR : dessin dans le YUV, encodage dans le processus | rendu 5,4 → environ 2 s, et du CPU | non (aspect identique à valider) | étape 4 |
| Médiane glissante par phase : retirer une valeur et en insérer une au lieu de retrier | **douteux** : le réseau élagué n'a que 32 comparateurs (64 min/max par vecteur de 8 pixels), et retirer puis insérer sans branche en SIMD en coûte à peu près autant. Non codé | non | moteur |
| Préparation : réutiliser les tampons, replier le flou symétrique | environ −3 à −5 s de CPU | non | moteur |
| Décodage de la luminance seule (option `gray` de FFmpeg, à compiler) | **mesuré : −3 % de CPU** (97 → 94 s sur la 092), pas de gain visible en temps réel ; détections et images en erreur identiques (092, 122). **Ne vaut pas l'option** | non | — |
| Fond de l'image résumé calculé pendant la détection | −1,4 s | non | moteur |
| Démarrage de Python supprimé | environ −0,5 s | non | étape 5 |
| PGO et cible processeur explicite | quelques % | non | construction |
| VideoToolbox | décodage de 60 à environ 4 s de CPU | non | **écarté** : ne voit pas les images abîmées (19 vidéos sur 26 en ont) |
| Médiane sur le GPU (Metal) | environ −20 s de CPU | non | écarté pour l'instant : complexe, plus le goulot |
| `skip_loop_filter`, médiane en 8 bits | −13 % de décodage, ou 2 fois plus de voies SIMD | **oui** | à décider au banc, pas comme optimisation |

Plancher : un décodage logiciel complet, environ 7 s en temps réel sur les 10 cœurs du M1 Max. Sur le
M3, c'est à mesurer.

### Option D : ce que coûterait le passage en production

| | Option D |
| --- | --- |
| Effort (planning poker) | **≈ 45**. Moteur propre et testé : 13. Branchement dans la CLI : 8. Généralisation ou refus explicite des cas hors défaut (autres tailles, OSD, `quick`, plage complète) : 5 à 8. Rendu par `pts` avec un extrait de test qui perd des images : 3 à 5. Garde indépendante du nombre d'images et canari des images abîmées : 2. Banc branché sur le moteur : 5. Binaires par plateforme : 8. Plus la vérification par Manon des 9 divergences, et docs 02, 03, 05, 09 et 10 |
| Distribution | deux exécutables, `fastdet` (3,6 Mo) et `framesrc` (4,2 Mo), sans bibliothèque dynamique hors système, ajoutés au paquet PyInstaller. L'encodage passe toujours par le `ffmpeg` de Homebrew |
| Licence | le moteur est MIT + LGPL. Le binaire complet reste GPL tant que Python embarque la roue OpenCV |
| Ce qui change pour Manon | des résultats pas identiques à l'octet près à v0.1.1. Mais **le test `reference` passe tel quel** (14 pistes, mêmes débuts, mêmes points), donc son verdict sur la 092 reste valable. Elle n'aurait à regarder que les 9 divergences des vidéos chargées |
| Ce qui se perd | rien côté Python. **Mais le banc ne mesure pas encore le moteur** : il injecte les fausses chauves-souris dans les images BGR d'OpenCV |
| Risques | deux chaînes de détection à garder en phase tant que le banc et les options hors défaut passent par Python ; FFmpeg à épingler, surtout pour `decode_error_flags` |
| Ce que ça ouvre | les réglages aujourd'hui trop lents (1440 px, plusieurs échelles, B2 de docs/07) |

Rapport de vitesse : **7,4 fois** contre le Python par défaut (134 s), 4,7 fois contre `--workers 3`
(84 s). La mémoire maximale mesurée (570 Mo) est celle du plus gros processus, pas de l'ensemble. Les
binaires ont été compilés pour ce processeur (`-C target-cpu=native`) : à remesurer avec la cible de
distribution. Sur le M3, l'écart devrait se réduire : Python est limité par un seul fil, le moteur par le
nombre de cœurs.

**Pourquoi pas tout simplement accélérer Python ?** `fastexact` fait la même arithmétique que Python, au
bit près, et prend 23 s au lieu de 121. L'écart vient du code compilé et parallèle, et ne se rattrape
pas en numpy. Le mieux mesuré en Python pur est `--workers 3` (84 s, 567 s de CPU) ; avec ffprobe
multifil, on peut viser environ 77 s (estimation).

**Contradiction levée avec la première partie** : les §3 et §5 écartaient la réécriture complète à
cause du « plancher ffprobe ». Ce plancher venait de la double lecture, pas du langage. Le moteur le
supprime en lisant les images abîmées dans la même passe, et cela ne tient pas à l'identité au bit près.

### À mesurer avant de décider

1. **Sur la machine de Manon** (M3) : variante D contre Python, temps et CPU.
2. **Le banc sur le moteur Rust** (rappel et précision sur fausses chauves-souris) : la seule mesure qui
   dise si une variante détecte mieux.
3. **Les 9 divergences de pistes**, à regarder par Manon.
4. FFmpeg LGPL sous Linux et Windows : même décodage, mêmes drapeaux d'erreur ? L'arithmétique du
   moteur est entière, ou en float32 sans FMA implicite. Le moteur devrait donc donner les mêmes
   résultats sur les trois systèmes (non vérifié hors Mac), alors que Python diffère aujourd'hui entre
   Mac et Linux.

## 7. Tout Rust par étapes (option E)

L'utilisateur préfère un seul langage. Chaque étape livre une version utilisable, mesurée sur la 092
et au banc, comme le veut CLAUDE.md. Python rétrécit à mesure que Rust grandit : pas de bascule d'un
coup. Les points comprennent, à chaque étape, les tests du module (règle du dépôt) et les docs
touchées.

### Ce qu'il y a à porter

| Partie | Python aujourd'hui | Équivalent Rust | Difficulté |
| --- | --- | --- | --- |
| Lecture, images abîmées, détection, stabilité, exclusion | 741 lignes + `track` exclu | **fait à 80 % dans le prototype** (`rust-proto/fast/`, environ 1 500 lignes) | options hors défaut à porter : OSD, mode `quick` (σ = 0, u8), autres `--work-width` (réduction générique), fenêtres et pas du fond |
| Suivi, fusion des jumelles, filtres | 168 lignes | portage direct | faible ; c'est le module le plus sensible : à vérifier piste pour piste sur les 26 vidéos |
| Tableaux (CSV, `params.json`), noms de fichiers | ≈ 230 lignes | `csv`, `serde_json` | faible |
| Extraits : boîtes, traces lissées (`LINE_AA`), zoom, ralenti, texte | ≈ 450 lignes | `tiny-skia` (dessin lissé, BSD-3), une police embarquée (`ab_glyph`), FFmpeg en process | moyenne. Les polices Hershey d'OpenCV disparaissent : le rendu change d'aspect, et **les accents deviennent possibles** dans le bandeau |
| Image résumé : fond médian, flèches, pastilles, légende, tranches de 10 min | ≈ 520 lignes | `tiny-skia`, `image` (PNG) | moyenne ; c'est le gros du travail visuel, à faire valider à l'œil |
| Encodage des extraits | sous-processus `ffmpeg` (Homebrew) | FFmpeg en process : `h264_videotoolbox` sur Mac (LGPL) ; ailleurs openh264 (BSD) ou un ffmpeg externe | **décision de licence** (voir plus bas) |
| CLI, options, langues | 796 lignes (argparse traduit par le catalogue gettext) | `clap` + catalogue fr (`gettext-rs` sur le `.po` actuel, ou `fluent`) | moyenne : `clap` n'a pas de traduction de ses propres textes, il faut des gabarits d'aide et d'erreurs |
| Banc et vidéos synthétiques | 861 lignes (numpy) | Rust ; l'injection se fait dans le plan Y avant la préparation | moyenne ; le banc doit mesurer **la** chaîne de production |
| Tests | 5 051 lignes, 55 fichiers, images fabriquées avec numpy | tests Rust ; vidéos de test encodées en interne (ffv1 ou mpeg4, LGPL) | volume plutôt que difficulté ; les tests `slow` et `reference` se transposent tels quels |
| Distribution | PyInstaller (63 Mo), roue OpenCV GPL, Homebrew, `package:check` | `cargo build` par plateforme + FFmpeg LGPL statique, formule Homebrew plus simple, NOTICE de quelques lignes | moyenne : construire FFmpeg pour Linux et Windows dans la CI |

### Étapes

| Étape | Contenu | Points | Ce qu'elle livre |
| --- | --- | --- | --- |
| 1 | Moteur Rust (lecture unique, images abîmées, détection, rendu par `pts`, gardes, généralisation ou refus des cas hors défaut) branché sous la CLI Python ; vérification par Manon des pistes qui changent | ≈ 35 (dont 3 pour Manon) | la vitesse : 092 en 18 s au lieu de 134 |
| 2 | Banc et vidéos synthétiques en Rust ; **suppression de la détection Python** | 13 | une seule chaîne de détection, mesurée ; le terrain de l'issue d'amélioration de la détection |
| 3 | Suivi, stabilité, exclusion, tableaux en Rust ; Python ne garde que les sorties graphiques et la CLI | 8 | le CSV sort du binaire Rust |
| 4 | Extraits, zoom, image résumé, encodage en process | 21 | plus d'OpenCV ni de numpy : la GPL de la roue disparaît |
| 5 | CLI, langues, noms de fichiers, dossiers ; **suppression de Python** | 8 | un seul langage |
| 6 | Distribution : binaires macOS, Linux, Windows, Homebrew, CI (reprend #93) | 13 | #93 |
| **Total** | | **≈ 100** | |

Cohérent avec l'estimation de C dans la première partie (80 à 100). La relecture d'architecture a
relevé l'étape 1, après les limites trouvées en revue (section 6). C'est un peu plus que le volume déjà
construit depuis le 3 octobre. **Je n'en tire pas de durée** : le rythme des semaines passées n'est pas
mesuré en points.

### Ce qui change par rapport au Python

- **Gagné** :
  - un seul outillage (cargo build, test, clippy, fmt remplacent six outils) ;
  - un binaire de quelques Mo, sans interpréteur ;
  - Windows et Linux sans PyInstaller ;
  - une seule chaîne de détection ;
  - les montées de numpy et d'OpenCV ne changent plus les résultats : seul FFmpeg reste à épingler ;
  - les accents dans les images.
- **Perdu** :
  - la souplesse de numpy pour essayer une idée de détection en dix lignes. Le banc en Rust compense
    en partie : on essaie dans le moteur et on mesure ;
  - l'écosystème Python de docs/07 (YOLO, autres détecteurs), qu'il faudrait appeler par ONNX
    (`ort`) si un jour un réseau de neurones entre dans la chaîne ;
  - l'aspect exact des extraits et de l'image résumé (polices, traits). Il faudra une validation à
    l'œil, une fois.
- **Risques** :
  - **l'étape 4** est la plus longue et la moins mesurable ; c'est un travail visuel ;
  - **la référence change à l'étape 1** et c'est irréversible pour le test `reference` : garder la
    v0.1.1 en tag et la comparaison sur 26 vidéos (`allvideos`) jusqu'à la fin de l'étape 2 ;
  - **FFmpeg sous Windows** : le construire en statique dans la CI est le point technique le plus
    incertain de l'étape 6. Repli : les constructions LGPL de BtbN.

### Licence de l'encodage (à trancher avant l'étape 4)

- **Mac** : `h264_videotoolbox`, dans FFmpeg LGPL. Le binaire devient MIT + LGPL.
- **Linux et Windows** : pas d'encodeur matériel garanti.
  - **openh264** (Cisco, BSD-2) : le binaire reste hors GPL. Qualité un peu en dessous de x264 à
    débit égal.
  - **ffmpeg externe** : batdetect appelle un ffmpeg (GPL avec libx264) installé à côté ou livré à
    part. Le binaire batdetect reste MIT + LGPL, mais l'installation demande ffmpeg.
  - **libx264 lié dans le binaire** : celui-ci redevient GPL, comme aujourd'hui.

### Se passer de FFmpeg ?

Pour le décodage : non. Pour le reste, en partie.

- **Décoder du H.264 sans FFmpeg n'est pas réaliste.**
  - Il n'existe pas de décodeur H.264 *High* (CABAC) mûr en Rust pur. Les crates existantes ne font que
    l'analyse du flux (`h264-reader`) ou l'encodage (`less-avc`). En écrire un prendrait des mois.
  - Le décodeur d'openh264 (Cisco, BSD) : prise en charge du profil High non vérifiée.
  - Les décodeurs du système (VideoToolbox, Media Foundation, VAAPI) demandent un code par
    plateforme. Mesuré en section 6 : VideoToolbox ne voit pas les images abîmées, alors que 19
    vidéos sur 26 en ont. C'est le `decode_error_flags` de FFmpeg qui les repère comme ffprobe.

| Rôle | Aujourd'hui | Sans FFmpeg ? |
| --- | --- | --- |
| Lecture du mp4 | `mov` de FFmpeg | possible (`mp4`, `mp4parse`), peu d'intérêt tant que le décodeur reste FFmpeg |
| Décodage H.264 | FFmpeg | **non** |
| Conversion en BGR (`swscale`) | rendu | oui : plus besoin d'imiter OpenCV, dessin direct dans le YUV |
| Encodage des extraits | `ffmpeg` en ligne de commande | Mac : VideoToolbox (dans le FFmpeg LGPL, ou en direct) ; ailleurs : openh264 |
| ffprobe | passe séparée | déjà supprimé par le moteur |

**Recommandation : garder FFmpeg réduit au minimum**, démuxeur et décodeurs h264/hevc, en LGPL. Le
binaire batdetect n'a alors plus besoin d'aucun `ffmpeg` installé.

Le vrai coût de FFmpeg est ailleurs :

- le construire pour trois plateformes dans la CI, Windows surtout ;
- les obligations de la LGPL en liaison statique : fournir de quoi relier, ou passer en liaison
  dynamique.

Lier dynamiquement des constructions LGPL existantes allégerait le premier point. À vérifier à
l'étape 6.

### Issues proposées (non créées), si E est retenue

Une issue par étape, avec ses points :

- « Moteur Rust : lecture unique, images abîmées, détection, branché sous la CLI » — 21
- « Moteur Rust : refus puis généralisation des cas hors défaut (tailles, OSD, `quick`, plage complète) » — 8
- « Rendu des extraits par le moteur, images choisies par `pts`, avec un extrait de test qui perd des images » — 3
- « Vérification par Manon des pistes qui changent (9 divergences sur 26 vidéos) » — 3
- « Banc et vidéos synthétiques en Rust, suppression de la détection Python » — 13
- « Suivi, stabilité, exclusion et tableaux en Rust » — 8
- « Extraits, zoom, image résumé et encodage en Rust » — 21 (découpable en deux de 13 et 8)
- « CLI, langues et noms de fichiers en Rust, fin de Python » — 8
- « Binaires macOS, Linux et Windows » — 13, qui remplace le plan actuel de #93

## Décisions à prendre (par l'utilisateur)

Déjà décidé le 10 octobre : la fidélité au bit près n'est pas un critère, et un seul langage est
préférable. Il reste, dans cet ordre :

1. **Lancer la migration par étapes (option E), en commençant par l'étape 1 ?** Recommandation : oui. Le
   moteur est écrit aux trois quarts. Il donne 134 → 18 s sur la 092 et passe le test `reference` tel
   quel ; il ne reste à Manon que les 9 divergences des vidéos chargées à regarder.
2. **Ordre avec l'amélioration de la détection** (#22, #23, #39, B2 de docs/07), qui change l'algorithme
   et se met au point au banc :
   - **(a)** améliorer d'abord en Python, puis porter le résultat. Chaque amélioration s'écrit deux
     fois, et le banc Python sert de terrain d'essai ;
   - **(b)** faire d'abord les étapes 1 et 2 (moteur, puis banc et vidéos synthétiques en Rust),
     supprimer la détection Python, puis améliorer en Rust, au banc.

   Recommandation : **(b)**. Avec un seul langage comme cible, (a) fait écrire chaque amélioration deux
   fois. La vitesse du moteur rend aussi abordables les essais lourds (1440 px, plusieurs échelles). Le
   coût de (b), c'est d'attendre l'étape 2 (environ 13 points) avant de reprendre ces issues.
3. **#93 maintenant en Python, ou à l'étape 6 en Rust ?** Recommandation : à l'étape 6. Faire des
   binaires PyInstaller pour Linux et Windows maintenant, puis les jeter, coûte 8 points. Seule
   exception : un naturaliste qui attendrait Linux ou Windows avant la fin de la migration.
4. **Encodage des extraits hors Mac** (avant l'étape 4) : openh264, ffmpeg externe ou libx264 (GPL) ?
   Recommandation : openh264 si la qualité des extraits convient à Manon, sinon ffmpeg externe.
5. **Images abîmées** : le moteur, qui lit `decode_error_flags`, exclut un peu plus d'images qu'ffprobe
   (masquages qu'ffprobe ne note qu'en information ; vu sur une vidéo abîmée fabriquée). L'adopter tel
   quel et le documenter ? Recommandation : oui, avec un test canari.

Les décisions de la première partie (`--workers`, licence, #93 en Python) tombent si E est retenue.

Rien n'est migré tant que ces décisions ne sont pas prises. Les deux prototypes restent dans
`rust-proto/`, non suivis.

## Revues

**`rust-expert`, sur le prototype.** Appliqué :

- provenance des coefficients du gris corrigée : ce sont les `BY15/GY15/RY15` d'OpenCV ;
- arrêt propre de `run.py` au-delà de 64 échantillons du fond, au lieu d'une panique.

Non appliqué, consigné comme limite du prototype :

- facteur de réduction 2, flou à noyau ≤ 5 ;
- mode quick et zones OSD dans le binaire autonome ;
- paquet corrompu qui arrête la lecture, alors qu'OpenCV continue ;
- `avg_frame_rate` au lieu de `av_guess_frame_rate` ;
- tampons réalloués à chaque image.

Rien de cela ne touche les chiffres des réglages par défaut. La revue confirme aussi :

- Rust ne contracte jamais `a*b+c` en FMA sans `mul_add` ;
- la somme par paires de numpy est faite en un seul appel jusqu'à au moins 518 400 éléments ;
- la numérotation des composantes est identique au chemin parallèle d'OpenCV ;
- le diagnostic de l'incident dyld est le bon, et le contournement reste un pile ou face.

**`tech-architect`, sur la première version de ce rapport.** Appliqué :

- mesure de A' (`--workers 2` et `3`) ;
- mesure de l'attente ffprobe dans B ;
- estimation de C avec la passe ffprobe ;
- ajout de B' ;
- question d'identité entre plateformes posée explicitement ;
- coût récurrent du noyau en double ;
- licence de l'encodage hors Mac ;
- chiffres de la 089 et de la 091 ;
- docs/05 signalé comme périmé.

La recommandation de tête, qui mettait B en avant, a changé après la mesure de A'.

**Second prototype, `rust-expert` (code) et `tech-architect` (rapport).** Appliqué :

- `framesrc` choisit les images par `pts` et vérifie chaque image servie. Vérifié sur la 103 : 15
  extraits sur 16 identiques, le dernier ne différant que sur des images réparées, où OpenCV lui-même
  varie d'un lancement à l'autre.
- `fastdet` sort en erreur sur un format de pixel non géré.
- Les limites restantes sont listées et chiffrées dans l'effort de l'étape 1 (section 6, « Limites du
  prototype »).
- Chiffres corrigés :
  - pistes sans correspondant des deux côtés ;
  - attribution des taches en cause (100 et 126) ;
  - « pas de biais visible » au lieu de « aucune ne détecte mieux » ;
  - rapport de vitesse contre `--workers 3` ;
  - mémoire du plus gros processus seulement ;
  - effort de l'étape 1 relevé à environ 35, total à environ 100.
- Contradiction avec la première partie levée (plancher ffprobe).
- Le test `reference` passe tel quel : il n'y a pas de nouvelle référence à refaire.
- Une seule liste de décisions.
- `fastdet2` (VideoToolbox) sorti du périmètre.
- Validation ffprobe étendue aux 26 vidéos : 25 identiques ; sur la 124, le moteur voit une image en erreur
  de plus (section 6).

## Fichiers

- Prototype : `~/Projects/dvr-wt/rust/rust-proto/`, non suivi par git :
  - `core/` : noyaux et détection d'une image ;
  - `py/` : module PyO3 ;
  - `decode/` : binaire autonome ;
  - `run.py` : branchement par variable d'environnement ;
  - `compare.py`, `stages.py`, `same_detections.py`, `kernel.py` : comparaisons.
- Sorties : `~/Projects/dvr-wt/rust/out/` (`ref-py`, `ref-py2`, `py-w2`, `py-w3`, `rs-fused0` à `3`,
  et pour le second prototype `fast-092`, `fast3-092` à `fast5-092`, `x264-py`, `x264-pyrender*`).
- Second prototype : `rust-proto/fast/` (binaires `fastdet`, `fastdet2`, `framesrc`, `fastexact`),
  `rust-proto/damage/` (agent des images abîmées), `rust-proto/fast_run.py`. Construction :
  `FFMPEG_DIR=<scratchpad>/ff711-vt CARGO_TARGET_DIR=target/fast-dev RUSTFLAGS="-C target-cpu=native"
  cargo build --release -p fast --features ffmpeg-next/static` ; lancement :
  `FASTDET=rust-proto/target/fast-dev/release/fastdet uv run python rust-proto/fast_run.py VIDEO -o OUT`.
- Comparaison sur 26 vidéos : `scratchpad/allvideos/resultats.md` et un dossier par vidéo.
- Journaux et scripts de mesure, dans le scratchpad de la session : `profile_092.log`, `series.log`,
  `series2.log`, `series3.log`, `stages_full2.log`.
- Rien n'a été écrit dans git. `src/`, `tests/` et `docs/` sont inchangés.
