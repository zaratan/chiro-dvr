# Lot 5 : migration vers Rust — décisions et ordonnancement (10 octobre 2026)

Lot sans code. Point de départ : l'exploration du 10 octobre, branche `exploration/rust`, rapport
`rust-proto/archive/exploration-rust.md`. Le lot rassemble les décisions de l'utilisateur, le plan
d'ordonnancement relu par quatre agents (architecte, expert Rust, ingénieur principal, UX) et les issues
du jalon « Migration Rust ». Rien n'est publié avant la 0.2.0, qui réunit les jalons « Migration Rust » et
« Prise en main naturaliste ».

## Faits mesurés

- **092, commande complète, sous verrou, M1 Max.** Ce prototype garde Python pour le suivi et les
  sorties.

  | | Temps réel | CPU | Mémoire |
  | --- | --- | --- | --- |
  | Python | 134 s | 437 s | 705 Mo |
  | Prototype Rust (mesure de la tour) | 19,0 s | 131 s | 616 Mo |
  | Prototype Rust (mesures de l'agent) | 17,9 / 18,1 s | | |

  Le prototype était compilé en `target-cpu=native`. Les 14 pistes ont les mêmes débuts et les mêmes
  points que le test `reference`, sans écart.
- **Les 26 vidéos de `~/dossier sans titre/img_0000/`.**
  - 958 paires de pistes dans les tolérances.
  - 9 divergences, chacune liée à une image au ras du seuil. Ce lien est prouvé par substitution de
    l'image sur la 099 et la 100, et repéré dans les données pour les autres.
  - **Manon les a regardées et les juge acceptables (10 octobre 2026).**
- **Images abîmées lues au décodage** (`decode_error_flags`).
  - 25 vidéos sur 26 sont identiques à ffprobe. La 124 a une image en erreur de plus, soit 43 images
    exclues en plus, loin de toute piste.
  - Sur une vidéo abîmée exprès, le moteur voit davantage qu'ffprobe.

## Décisions de l'utilisateur (10 octobre 2026)

1. **Tout batdetect passe en Rust.** L'identité au bit près avec la 0.1.1 n'est pas un critère.
2. **Rust à côté.**
   - Le Python est figé, et la référence v0.1.1 est enregistrée une fois (R0).
   - Rust grandit dans le même dépôt avec sa propre commande.
   - Aucun raccord Python ↔ Rust à l'exécution.
3. **Rien n'est publié avant la 0.2.0**, qui réunit « Migration Rust » et « Prise en main
   naturaliste ».
   - Pas de chemin de secours : si un bug grave touche la 0.1.1, un binaire de test est construit à la
     main depuis le tag.
4. **Ergonomie codée une seule fois, sur la commande Rust.**
   - #56, #63, #64, #65, #86 et #91 se codent juste après la commande Rust, avant la bascule, dans
     leur forme finale. L'affichage de la 0.1.1 ne se recopie pas.
   - Le reste vient après la bascule.
   - #56, #59 et #60 passent dans le jalon Rust.
5. **Docs.**
   - Tout de suite : #72, #75, #76, #78.
   - Sur la version Rust : #73 (après R11), #77 (après R10), #74, #79, #80, #81.
6. **Détection.** La 0.2.0 détecte comme la 0.1.1. Les améliorations (#9, #21 à #24, #39, #42…) viennent
   après, en Rust, au banc.
7. **Banc.** Il passe en Rust dans la 0.2.0, rejoue les cibles tirées par le Python, et sert de garde
   de bascule.
8. **#93** : réécrite en « binaires Rust pour macOS, Linux et Windows », dans le jalon Rust.
9. **#10** : un binaire de test, chronométré par Manon sur une soirée, après les extraits, les zooms
   et l'image résumé Rust. Elle ne décide plus de #67 à #70.
10. **Issues liées.**
    - #12, #35 et #19 rejoignent le jalon Rust.
    - #16 et #52 seront fermées à la bascule.
    - #45 perd ses points Python mais garde ses points de CI.
    - #13 reste hors jalon.
11. **Images abîmées** : le `decode_error_flags` du moteur est adopté, avec un test canari.
12. **Encodage des extraits** : VideoToolbox sur Mac, openh264 ailleurs et en repli, construit dès R1.
    Compilé dans notre binaire, openh264 n'a pas la couverture de brevets de Cisco : c'est la même
    situation que x264 aujourd'hui.
13. **Prototype** copié dans `rust-proto/archive/` (FFmpeg, ligne `configure`, comparaison, journaux,
    rapport). L'utilisateur commite `rust-proto/` sur `exploration/rust` et pousse.
14. **Mode `quick` et `--mode` supprimés.**
    - **Moteur générique** (R4).
    - **Options de la commande** (R11), en trois niveaux.
      - Essentiel : vidéos, `-o`, `--zoom`, `-h`, `--version`.
      - Utile : `--annotated`, `--max-tracks`, `--clip-margin`, `--osd-region`, `--force`.
      - Avancé : `--threshold`, `--min-hits`, `--max-blobs`, `--work-width`.
    - Les autres réglages restent au banc, et pourront revenir dans la commande au besoin.
15. **Traduction** : Fluent, avec ICU4X branché par `set_formatter` pour les nombres.
16. **Bascule dès la parité**, avant le reste de l'ergonomie.
17. **Validation de l'aspect des sorties par l'utilisateur seul**, à l'aveugle, sur huit cas difficiles.
    C'est une garde de la bascule. Manon ne fait que chronométrer (#10).
18. **#67 à #70 sont gardées.** #67 fusionne dans #68 : une seule barre pour la soirée.
19. **LGPL** : FFmpeg lié en statique. Le `NOTICE` renvoie au code du tag, à l'archive et au script.

Adopté sur avis des relecteurs, sans question à l'utilisateur :
- R0 (référence) ;
- R1 coupée (Windows à part, en temps limité) ;
- socle de dessin commun ;
- extraits en deux issues ;
- cible processeur fixée ;
- règles du dépôt transposées (zéro `#[allow]`, `unsafe` confiné, couverture à 95 %) ;
- une panique qui n'arrête pas la soirée.

## Plan d'ordonnancement

### Principes

- **Python figé** jusqu'à la bascule. La référence v0.1.1 (R0) remplace toute exécution du Python.
- **main toujours vert**, CI Python et Rust côte à côte.
- **Chaque étape a ses tests par module** et se compare à la référence : détections, pistes, banc.
- **Chaque décision se prend avant de lancer son étape.**
- **Les risques incertains d'abord** : Windows (R2) dès que R1 est fait ; l'essai VideoToolbox sur le
  runner dans R1.
- **Bascule dès la parité.** L'ergonomie restante se code dans un dépôt sans Python.

### Étapes

| # | Contenu | Points | Dépend de | En parallèle |
| --- | --- | --- | --- | --- |
| 0 | Docs de méthode : #72, #75, #76, #78 | 8 | — | 1 à 3 |
| 1 | R0 : référence v0.1.1. R1 : construction macOS et Linux | 3 + 5 | — | R0, R1 et docs |
| 1b | R2 : construction Windows, en temps limité | 8 | R1 | jusqu'à #93 |
| 2 | R3 : moteur aux réglages par défaut | 13 | R0, R1 | R2 |
| 3 | R4 : moteur générique. R5 : suivi et CSV (+ #12, #35 CSV). R7 : socle de dessin | 8 + 13 (+ 1) + 2 | R3 / R0 et R3 / R1 | les trois |
| 4 | R6 : banc. R8 : extraits et encodage (+ #19). R10 : image résumé | 13 + 13 (+ 2) + 8 | R4 et R5 / R5 et R7 / R5 et R7 | les trois |
| 5 | R9 : zoom, ralenti, vue brute, vidéo annotée. Puis #10 : mesure chez Manon | 8 ; 2 | R8 ; R9 et R10 | R11 après R9 |
| 6 | R11 : commande (options, Fluent, lots) | 13 | R4, R9, R10 | — |
| 7 | Forme finale de l'aide et du terminal : #56, #63, #64, #65, #86, #91 | 15 | R11 | en grappes |
| 8 | R12 : bascule (garde : banc, 26 vidéos, référence, validation à l'aveugle) | 8 | R6, étape 7 | — |
| 9 | Ergonomie restante (voir les grappes) | 44 | R12 (#68 attend aussi #10) | 10 |
| 10 | #93 : binaires macOS, Linux et Windows (+ #60) | 8 + 1 | R2, R12 | 9 |
| 11 | Docs sur la version Rust : #74, #79, #80, #81 (#73 et #77 plus tôt). Puis CHANGELOG, 0.2.0 | 18 | — | — |

**Totaux.**
- Jalon « Migration Rust » : 140 points. Ce sont les 115 des treize nouvelles issues, plus #10, #93,
  #12, #35, #19, #56, #59 et #60.
- « Prise en main naturaliste » : 77 points ouverts, après la fusion de #67 dans #68.
- 0.2.0 en tout : environ 217 points.

**Pas plus de trois worktrees à la fois.** Le chemin critique : R1 → R3 → R5 → R8 → R9 → R11 → étape 7 →
R12.

### Grappes d'ergonomie après la bascule (regroupées par fichiers)

1. **Lancement** : #55 (erreurs d'options), #62 (annonce au départ), #82 (écriture vérifiée), #61
   (espace disque).
2. **Avancement** : #66 (dossier interrompu), #68 (barre de soirée), #69 (notification et Finder), #70
   (relance).
3. **Outils** : #58 (doctor), #59 (complétion). #60 vient avec #93.
4. **Fichiers produits** : #83, #84 (CSV), #71 (page de la soirée).

Les corps de ces issues citent du code Python. La tour relit chacune contre le code Rust avant de la
lancer.

### Risques

- **FFmpeg et openh264 sous Windows** (R2). Le temps est limité à 8 points. Repli dans l'ordre : vcpkg
  statique, puis BtbN en dynamique, sur décision de l'utilisateur.
- **VideoToolbox absent des runners de GitHub** : l'essai se fait dans R1. S'il échoue, #19 ne se ferme
  pas par la CI ; c'est à l'utilisateur de trancher.
- **Aspect des sorties** : polices, lissage et encodeur changent.
  - Un SSIM masqué compare chaque sortie à la v0.1.1.
  - L'utilisateur valide à l'aveugle avant la bascule.
  - Les images validées sont ensuite figées en test.
- **Divergences au-delà des 9 connues** : montrées à l'utilisateur dès R5.
- **Perte du Python** : la référence R0 et l'outil de comparaison Rust restent après la bascule.
- **Volume de tests** : 5 051 lignes Python (55 fichiers) à transposer, étape par étape.
- **Mémoire chez Manon (16 Go)** : prototype à 616 Mo. R3 borne l'éclair plein cadre, R8 les
  encodeurs.

## Issues

Créées le 10 octobre 2026, jalon « Migration Rust », toutes en Backlog avec points et priorités.

| Repère | Issue | Points | Priorité |
| --- | --- | --- | --- |
| R0 référence v0.1.1 | #94 | 3 | P1 |
| R1 construction macOS et Linux | #95 | 5 | P1 |
| R2 construction Windows | #96 | 8 | P3 |
| R3 moteur | #97 | 13 | P1 |
| R4 moteur générique | #98 | 8 | P2 |
| R5 suivi et CSV | #99 | 13 | P1 |
| R6 banc | #100 | 13 | P2 |
| R7 socle de dessin | #101 | 2 | P2 |
| R8 extraits et encodage | #102 | 13 | P2 |
| R9 zoom, ralenti, vue brute, vidéo annotée | #103 | 8 | P2 |
| R10 image résumé | #104 | 8 | P2 |
| R11 commande | #105 | 13 | P2 |
| R12 bascule | #106 | 8 | P2 |

**Issues existantes modifiées.**
- Réécrites : #93 et #10.
- Paragraphe de décision en tête : #12, #19, #35, #56, #59, #60, #63, #64, #65, #68 (passée à 5 points),
  #86, #91.
- Passées dans « Migration Rust » : #10, #12, #19, #35, #56, #59, #60, #93.
- Commentées : #16, #45, #52, #73, #76, #83.
- Fermée : #67, fusionnée dans #68.

Textes sources dans `~/Projects/dvr-wt/rapports/lot5-issues/`, créés par `lot5-apply.py`. Prototype et
rapport sur la branche `exploration/rust` (f3faf66), dans `rust-proto/archive/`.
