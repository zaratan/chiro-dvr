# Lot 8 : commande définitive et qualité — consignes

Préparé le 13 octobre 2026 (zaratan). Relectures : `/Users/zaratan/Projects/dvr-wt/rapports/lot8-relectures/{commande,qualite}.md`.

| File | Enchaînement |
|---|---|
| `commande` | #113 → #114 → #115 → gardes de #106 (#56, #63, #64, #65, #86, #91) ; part après la fusion de #103 |
| `qualite` | #167 (CI, d'abord) → #154 → #160 → #159 |

#106 (bascule) passe au lot 9, redécoupée (14 points selon la relecture).

## Décisions de l'utilisateur (13 octobre 2026)

1. Les six gardes de #106 entrent au lot 8, dans la file `commande` après #115 ; #106 au lot 9.
2. #114 : `fluent-bundle` directement (pas `i18n-embed-fl`, qui retombe en silence sur l'anglais), avec un test qui exige
   chaque clé dans les deux langues. Revient sur la décision du 10 octobre.
3. Charte, règle C9 : `catch_unwind` compté hors des tests seulement ; #113 y met le seul du code livré (tenue aux pannes),
   justifié dans docs/11.
4. #10 (temps et sessions VideoToolbox sur le Mac de Manon) : dans un mois, script à préparer après #103.
5. Sorties : `--out-dir` obligatoire jusqu'à #86, **avec l'alias court `-o`** (demande de l'utilisateur) ; `--force` → #70 ;
   contrôle d'écriture avant calcul pour toutes les racines (#82 réduite au reste) ; homonyme `<nom>_<ext>` dans
   `batdetect/` (#91 avec #86, casse comprise) ; pas de cas particulier pour les vidéos lues sur les jumelles.
6. Affichage : étapes détection (lecture comprise), suivi, image résumé, extraits ; la ligne d'une vidéo finie reste avec
   son nombre de pistes ; « trop de pistes » à part des pannes dans le bilan ; crochet de panique à nous ; disque plein
   reconnu (`io_kind` dans `media`) ; bilan fichier à #71 ; exemples de l'aide avec le nom de la commande lancée.
7. Langues : `fluent-bundle` + test des deux langues ; formateur FR/EN écrit (pas d'ICU4X ; revient sur le 10 octobre) ;
   variables d'environnement d'abord, `sys-locale` à défaut ; causes des bibliothèques en anglais précédées d'une phrase
   traduite ; #114 traduit le cadre, #64 les phrases par cause.
8. #113 : version tirée de git (`build.rs`), borne de la fenêtre du fond dans le moteur (`dhat`), un résultat par vidéo.
   C9 : `catch_unwind` hors tests, dans `crates/cli` seulement. #115 : CSV étranger laissé et signalé, puce « messages »
   retirée. Docs : docs/11 tout de suite, README à #106 (c). Ordre des gardes : #86+#91, #63, #64, #65, #56.
9. #106 (lot 9) : découpée (a) 3, (b) 3, (c) 5, (d) 3 ; tag `python-final` et sauvegarde des 37 Mo de la référence ;
   validation à l'aveugle sur extraits et zooms, images résumé à découvert ; `release.yml` en refus explicite jusqu'à #93.
10. Qualité : #167 comme prévu (`settings_*` dans `mise run check`) ; #154 élargie à 2 points ; #160 unifiée dans `types` ;
    #159 prouvée au bit près, gain gardé s'il dépasse le bruit.

## Méthode (13 octobre)

Les questions des relectures (`lot8-relectures/{commande,qualite,gardes}.md`) se reprennent dans le plan de chaque issue,
avec les recommandations, et se tranchent aux arrêts. Vérifications : consigne projet (le lourd une fois, à la fin) ;
la tour ne revérifie que ce que la CI ne voit pas.

## Consignes

### #167 — CI d'une PR Rust à 15 minutes (worktree `qualite`, branche `rust/167-ci`)

3 points, sans plan relu. Relecture : `lot8-relectures/qualite.md` § #167 (chemin critique Linux 877 s, run 38350918944).
- Les deux leviers chiffrés : `slow_settings` dans une tâche Linux à part (niveau de test choisi par le nom de fichier,
  par exemple `settings_*`, sans `--skip`) ; le test unitaire 61/81 de #151 (`engine/src/detector/tests.rs:237-266`),
  environ 200 s sous la couverture Linux contre 2 s sans : trouver pourquoi, puis le réduire sans perdre ce qu'il vérifie.
- `slow_extract` et `slow_canary` restent sur les trois systèmes ; les 16 cas ne tournent plus sous Windows (accepté).
- Ne touche pas `crates/cli/src/process.rs` (file `commande`). Ne renomme aucune tâche requise de `main` ; une tâche
  nouvelle doit être ajoutée aux vérifications requises par l'utilisateur : dis-le.
- Cache Actions à 10,66 Go pour 10 Go : une tâche de plus ne doit pas ajouter de cache `target` sans le dire.
- Mesure : la CI de la PR elle-même, après la fusion de #103 (PR #169), qui ajoute deux tests lents ; même liste de tests
  passés par système (`rapports/tests-146-extraire.sh`).
Attendu : `mise run rust:static` et les tests touchés ; le temps de la PR et la liste des tests, au compte rendu.

### #113 — Options décidées et tenue aux pannes (worktree `extraits`, branche `rust/113-options`)

5 points. **Plan d'abord** (`rapports/plan-113.md`), relu par `rust-expert` et `tech-architect`, puis arrêt pour
validation. Relecture : `lot8-relectures/commande.md` § #113 et `gardes.md` (ajustements pour #65). Décisions 3, 5 et 8
de ce fichier.
- Les lots et les codes de sortie 0/1/2 sont déjà faits (#123). Restent : les options décidées (environ 10), `--out-dir`
  avec l'alias court `-o`, obligatoire jusqu'à #86 ; la lecture de `--osd-region` partagée avec le banc ; la tenue aux
  pannes par vidéo (un seul `catch_unwind`, dans `crates/cli`, justifié dans docs/11 ; règle C9 de `mise.toml` corrigée :
  hors tests, `crates/cli` seulement) ; un résultat par vidéo rendu par la boucle (base de #65) ; la version tirée de git
  par `build.rs` dans `params.json` ; la borne de la fenêtre du fond dans le moteur, mesurée par `dhat` (`--bg-window`
  du banc l'atteint déjà).
- `--force` part à #70. Ne touche pas aux messages au-delà du nécessaire (#114 les reprend avec `fluent-bundle`).
- Vérifications selon la consigne projet. Docs : docs/11.

### #154 — L'empreinte d'image en une seule copie (worktree `qualite`, branche `rust/154-empreinte`)

2 points : le banc entre dans l'issue (décision 10). Pas de plan relu. Relecture : `lot8-relectures/qualite.md` § #154.
- Mesuré le 13 octobre sur 5bba4cb (`git grep -c DefaultHasher -- crates`) : 8 fichiers. Sept hachent une image
  `(frame.y(), frame.u(), frame.v())` : `media` `decode/tests.rs:270`, `decode/placed/tests.rs:19`,
  `encode/fanout/tests.rs:26` ; `render` `tests/support/sequential.rs:21` ; `engine` `tests/slow_canary.rs:40`,
  `tests/local_threads.rs:21` ; `cli` `tests/local_clips.rs:82`. Le huitième, `render/src/clips/tests.rs`, hache les
  octets d'un fichier.
- Recommandations de la relecture, à appliquer sauf prémisse fausse, et à lister dans « Décisions prises seul » :
  - un module `media::fingerprint`, une fonction libre sous `cfg(any(test, feature = "testing"))` comme `payload`, avec
    ses tests dans `src/fingerprint/tests.rs` ;
  - dans `render/src/clips/tests.rs`, comparer les octets eux-mêmes, sans hachage ;
  - pour le banc, une feature `testing` sur `bench` qui ne garde qu'une vidéo fabriquée de 96×72 et qu'une cible lente
    (`src/testing.rs` et `tests/support/noisy.rs`), avec `bench` dans `[workspace.dependencies]`. Vérifier l'avis de
    `cargo shear` sur la dépendance de `bench` à lui-même.
- Ne change pas ce que les tests affirment : mêmes empreintes, mêmes vidéos, au bit près.
- Docs : docs/11:910 (`payload` derrière `testing`) et la ligne `media` de `CLAUDE.md`.
- Attendu : `git grep DefaultHasher -- crates` ne trouve plus qu'une définition, dans `media::fingerprint`. Aucun test
  perdu : la liste des tests passés est la même avant et après. `rust:static` et les tests des crates touchées sont
  verts. Les tests longs touchés (`slow_canary`, `local_threads`, `local_clips`) tournent une seule fois, à la fin, sous
  le verrou.

### #160 — `Measures` en double et variante inatteignable dans `reference` (worktree `qualite`, branche `rust/160-mesures`)

2 points. Pas de plan relu. Relecture : `lot8-relectures/qualite.md` § #160. Décision 10 : unifier dans `types`.
- Mesuré le 13 octobre sur 5bba4cb : `types::detection::Measures` (`crates/types/src/detection.rs:27`, public, sans
  serde) et `reference::measures::Measures` (`crates/reference/src/measures.rs:6`, `pub(crate)`, sérialisable), lu par
  `frames.rs`, `bench.rs` et `tracks.rs`. `Error::Detections` (`error.rs:44`) est produite à deux endroits,
  `frames.rs:76` et `bench.rs:127` (le second ajouté par #100 après l'issue), et ne peut arriver ni à l'un ni à l'autre :
  chaque détection reçoit l'image de sa propre ligne, après le contrôle d'ordre.
- À faire : un seul `Measures`, celui de `types`, avec `Serialize`/`Deserialize` et `of(&Detection)` ; supprimer
  `reference/src/measures.rs`. Ranger les détections par `FrameDetections::push`, qui ne peut pas échouer, et retirer
  `Error::Detections`. `types` dépend déjà de `serde` : pas de dépendance nouvelle.
- Format des fichiers inchangé, octet pour octet : mêmes noms et même ordre des champs, pas de `deny_unknown_fields`.
- Docs : docs/11:789 (« `Measures` : les mesures d'une détection sans son image ») dit aussi « sérialisable » ;
  relire les autres mentions de `reference::measures`.
- Attendu : `git grep -n 'struct Measures' -- crates` ne trouve qu'une définition, dans `types` ; `Error::Detections`
  n'existe plus ; `slow_extract` au bit près (il relit `frames.jsonl` committé octet par octet), lancé une seule fois à
  la fin, sous le verrou ; aucun test perdu ; `rust:static` vert.

### #114 — La commande en français et en anglais (worktree `extraits`, branche `rust/114-langues`)

5 points. **Plan d'abord** (`rapports/plan-114.md`), relu par `rust-expert` et `tech-architect`, puis arrêt pour
validation. Relecture : `lot8-relectures/commande.md` § #114. Décisions 2 et 7 de ce fichier.
- Décidé : `fluent-bundle` directement, sans `i18n-embed-fl`, avec une énumération des messages et un test qui exige
  chaque clé, avec ses arguments, dans les deux `.ftl`. Un formateur FR/EN écrit par nous (virgule décimale, U+202F
  pour les milliers, jamais dans le CSV ni les noms de fichiers), pas d'ICU4X. La langue se lit d'abord dans les
  variables d'environnement, dans l'ordre actuel de `cli/src/language.rs`, et par `sys-locale` à défaut seulement.
  Les causes d'erreur des bibliothèques restent en anglais, précédées d'une phrase traduite (#64 traduira les causes
  connues).
- Recommandations de la relecture à reprendre dans le plan (choix 5 à 8) : les noms de fichiers restent dans
  `files::names` ; `[default: …]` et `[possible values: …]` écrits par nos textes ; les erreurs de clap rendues depuis
  `ErrorKind` et `ContextKind` (« error: », « tip: », « For more information ») ; un type `Texts` qui porte la langue
  dans `process`.
- **Le look de l'aide décidé à #113 se garde dans les deux langues** : couleurs, une ligne par option, chaque défaut
  sur sa propre ligne, seule la valeur en couleur, jamais coupé. Planche des deux langues (`-h`, `--help`, deux
  refus), en texte et en couleurs, dans `rapports/planche-114/`.
- Textes français : ceux de #54, repris mot pour mot pour les 10 aides gardées (`src/batdetect/locale/fr/LC_MESSAGES/messages.po`) ;
  `tests/test_language.py` sert de liste. Commentaire de #114 : `detection_refused` en deux copies, une seule source.
- Hors périmètre : #55 (forme des messages d'option, voir son commentaire), #56 (exemples), #64 (causes), #65 (bilan).
- Attendu : sous `LANG=fr_FR.UTF-8`, aucun texte anglais de clap dans `-h`, `--help` et les refus ; « 1,5 » dans
  l'aide ; le test des clés vert ; les tests de la commande verts dans les deux langues. Le plan dit quels tests
  lourds le diff peut changer ; ils tournent une seule fois, à la fin.

### #159 — Le surcoût du moteur générique (worktree `qualite`, branche `rust/159-moteur`)

3 points. Pas de plan relu. Relecture : `lot8-relectures/qualite.md` § #159. Décision 10 : prouver au bit près, ne
garder un gain que s'il dépasse le bruit.
- Le chiffre de 7,5 % (110,0 s contre 102,4 s de CPU sur la 092) n'a pas été remesuré depuis #150, et #100 a touché
  `reduction.rs` sans mesure. Commence par le remesurer contre le moteur 3:2 s'il est encore lançable, sinon contre
  `main` seul, et dis-le.
- Les cinq pistes de la relecture : (1) champs de `Rounding` pris dans les fermetures (`reduction.rs:19`, `rounded`
  l. 123) ; (2) chemin identité en 1:1 ; (3) taps contigus et inverse sur 32 bits (`across`, l. 84) ; (4) noyau
  symétrique plié (`prepare.rs`) ; (5) blocs de 64 pixels entièrement masqués (`threshold.rs:94`). Toutes sont des
  réécritures en entiers : exactes par construction, sous les deux conditions de la relecture (domaine de l'inverse
  sur 32 bits, test exhaustif ; pas de débordement du noyau plié).
- **Budget de mesure, à tenir** (le verrou sert aussi à #114) :
  - par piste, d'abord le temps de l'étape (ms par image, sur un fil), peu bruité ; trois paires A/B de
    `compare 092` en `release` seulement si l'étape gagne plus de 1 %. Les pistes 2 et 5 se mesurent sur le cas
    qu'elles visent (`092-1440`, `osd`) ;
  - les empreintes `compare … --frames` (extrait, `092`, `092-1440`, `settings`, `local`), `main` contre branche,
    `cmp -r`, **une seule fois à la fin**, sur l'ensemble des pistes gardées ;
  - dans le compte rendu, le temps de verrou consommé.
- Docs : docs/11, sections `reduction`, `prepare` et `threshold`, et le temps de la 092, avec les gains mesurés.
- Attendu : pour chaque piste, gardée ou non, son gain mesuré ; empreintes identiques à l'octet ; `rust:static` et
  les tests d'`engine` verts.

### #115 — Ménage des sorties et bout en bout dans les deux langues (worktree `extraits`, branche `rust/115-menage`)

3 points. Pas de plan relu. Relecture : `lot8-relectures/commande.md` § #115. Décision 8 de ce fichier.
- Décidé : un CSV qui n'a pas exactement notre forme (en-tête de 12 colonnes, séparées par des virgules) n'est jamais
  effacé par le ménage : il est laissé, et la commande le signale par un message traduit (Fluent, #114). La mise de
  côté datée reste à #83. La puce « messages » de l'issue est retirée : #114, #63 et #65 la couvrent.
- Recommandations de la relecture, à appliquer sauf prémisse fausse, et à lister dans « Décisions prises seul » :
  - le ménage se fait après la détection et le suivi, avant le CSV, comme la v0.1.1 (`src/batdetect/cli.py`, appel de
    `remove_previous_outputs`) : une détection ratée ne détruit rien. En Rust, dans `process.rs` ;
  - trois tables de noms, FR, EN et les anciens noms (`src/batdetect/output/names.py:30` : `split/`, `.tracks.csv`,
    `.tracks*.png`, `_boxes.mp4`), comparés en NFC (`unicode-normalization` est déjà au workspace) ;
  - le motif des tranches et des parties de l'image résumé (`_3_sur_8`, `_3_of_8`, commentaire de #157 sur l'issue) ;
  - les fichiers partiels d'encodage des deux langues ;
  - un bout en bout en CI sur la vidéo fabriquée, en français puis en anglais dans le même `-o`, et un test `local_`
    sur l'extrait, comparé aux noms de `~/Projects/dvr-wt/reference-v0.1.1/outputs/{fr,en}/` (hors dépôt). La
    v0.1.1 ne fait aucun zoom sur l'extrait : dis ce qu'il en est du Rust.
- Docs : docs/11 seulement (sections `files` et `cli`). Pas docs/05, qui décrit le Python (décision 3 du lot 7).
- Attendu : sur l'extrait, à `-o` égal, en français puis en anglais, les mêmes fichiers sous les mêmes noms que la
  v0.1.1, et rien d'une passe précédente qui traîne (autre langue, anciens noms, parties en trop) ; un CSV étranger
  intact et signalé. Les tests longs que le diff peut changer tournent une seule fois, à la fin, sous le verrou ; dis
  lesquels et pourquoi.

## Décisions prises pendant le lot

- **#167 (13 octobre)** : acceptés, le cache `target/` de `rust-reference` lu sans sauvegarde par `rust-settings`, le cache
  LFS lu seulement, le plancher `>= 100` du test 61/81, « cinq » dans docs/05. Retouche : `rust-windows` ne tire plus la
  720×540. Les tests de 2 à 7 s sous couverture restent tels quels. La tour ajoute `rust-settings` au ruleset de `main`
  quand la PR l'aura rendue verte, juste avant le merge.
- **#113 (13 octobre)** : plan validé. Le rappel des réglages avancés va à #65. Budget de la fenêtre du fond : 4 Gio
  (64 Go sur le poste de l'utilisateur, 16 Go chez Manon), à confirmer par `dhat`. Panique : code 1, crochet par défaut et
  ligne « internal error ». Refus qui nomme l'option. Pas d'alias `--output`. Aide à deux niveaux. Version tirée de git.
  Boucle `run_all` dans `cli::outcome`. `FromStr` de `Region` partagé avec le banc. Borne en octets dans `detection.rs`.
- **#113, aide (13 octobre)** : garder le look d'argparse 3.14. Couleurs : titres et usage en bleu, options en cyan,
  noms de valeurs en jaune, valeurs possibles en vert, coupées hors terminal et avec `NO_COLOR`. Une ligne par option,
  texte en colonne, coupé à la largeur du terminal (feature `wrap_help`), y compris pour `--help`. L'usage reste
  `[OPTIONS]`. Les exemples façon `tldr` restent à #56, qui s'appuiera sur ce rendu. À transmettre à l'arrêt 2.
- **#167, CI de la PR #170** : tout vert en 10 min 39 s, contre 14 min 41 s avant. Les tâches les plus longues sont
  `rust-windows` (10 min 36 s, cache LFS froid au premier passage) et macOS (10 min 23 s). Linux passe les mêmes
  1 102 tests que main. Sous macOS et Windows, les 3 tests des 16 cas ne tournent plus, comme prévu (Linux seulement).
  `rust-settings` est ajouté au ruleset.

## Reprise (13 octobre 2026, vers 11 h, avant l'arrêt du poste)

- **`extraits`** (worktree `dvr-wt/extraits`, branche `rust/113-options`) : #113, sous-phase 2 (pannes, C9, `build.rs`,
  borne de 4 Gio). docs/11 est écrite, la sonde `dhat` de la borne tournait. L'arrêt 2 se signale par
  `rapports/arret2-113.md`. À l'arrêt 2, transmettre le choix de l'aide (couleurs, une ligne par option, voir plus haut),
  à faire avant la fin de #113.
- **`qualite`** (worktree `dvr-wt/qualite`, branche `rust/154-empreinte`, partie de 5bba4cb) : #154 en cours. Le compte
  rendu attendu est `rapports/rapport-154.md`. Ensuite #160, puis #159.
- **Fait aujourd'hui** : #167 fusionnée (PR #170) ; `rust-settings` ajouté au ruleset de `main` ; CI de `main` verte en
  11 min 12 s.
- **Pour reprendre** : relancer chaque agent dans son worktree (`claude --resume`, même session), lui dire de relire son
  état (`git status`, `git diff`) et de reprendre où il en était ; relancer un guetteur par agent
  (`assets/guetter.sh <agent> <fichier attendu>`).
- **Non commités** : dans ce dépôt, `.claude/tower-control/` (`lot-7.md`, `lot-8.md`, `journal.md`,
  `consigne-projet.md`) ; dans `~/dotfiles`, le skill tower-control (`SKILL.md`, `consigne-commune.md`, `herdr.md`,
  `retours.md`, `assets/guetter.sh`).
- **Réglé au retour (12 h 30)** : la règle « la tour ne vérifie que ce que la CI ne voit pas », limitée aux tests longs,
  est ajoutée au skill.
- **#113, arrêt 2 (13 octobre, 13 h 35)** : validé. Sonde `dhat` arrêtée à 5 points (verrou tenu plus de deux heures).
  1,40 Mo par image de fenêtre à 960×720, modèle à 1 % près. Budget de 4 Gio, soit 3 105 images à 960×720 et 1 379 à
  1440×1080. Arrêt de la boucle quand l'écriture d'une ligne échoue : gardé. Look de l'aide transmis pour la
  sous-phase 3, avec une planche en couleurs (`planche-113/aide-couleurs.ansi`).
- **#154 (13 octobre, 13 h 50)** : décisions de l'agent acceptées (module `media::fingerprint`, octets comparés, feature
  `testing` du banc, cible écrite par `recorded::line`). Retouche : l'empreinte hache aussi la taille et la plage de
  couleur. Pas d'issue pour les trois `testing.rs` (vidéos différentes, seules les lignes de l'encodeur sont communes).
  Laissés tels quels : les tables de lints de `media` (règle C6), les deux types `Target` dans les tests du banc.
- **#113, aide (13 octobre, 14 h)** : les valeurs par défaut restent colorées, de la même couleur que les valeurs
  possibles. Elles l'étaient dans une version précédente de la branche, et la correction de la coupure les avait remises
  en texte normal. Planche : pas de défaut coupé ; une seule colonne pour les deux sections si clap le permet ;
  « print the version » en minuscules.
- **#160 (13 octobre, 14 h 50)** : décisions de l'agent acceptées (`push` après le contrôle d'ordre, ligne de la v0.1.1
  relue à l'octet près, tests du format dans `types`). `FrameDetections::new` et `Misplaced` ne servent plus qu'aux tests :
  issue #172 créée (Backlog, 2 points, P3, `qualité`, `complexité: 2`). Clés inconnues acceptées dans une détection :
  rien (fichiers écrits par nos outils).
- **#113, aide (13 octobre, 14 h 45)** : chaque défaut sur sa propre ligne, « [default: » et « ] » en texte normal,
  la valeur en couleur. clap peut couper une ligne à chaque changement de couleur (`StyledStr::wrap`, clap_builder
  4.6.7), donc un défaut en trois couleurs, laissé en fin de texte, pouvait être coupé.
- **#113, compte rendu (13 octobre, 15 h)** : décisions de l'agent acceptées (borne dans `config::half_window`, aide
  courte de clap pour les deux niveaux, `-V` au ton des autres, exemples de #56 dans `before_help` et `after_help`).
  Retouche : un test vérifie que la borne reçoit les pixels de travail. Points de messages commentés sur #55
  (`--osd-region -0.1`, refus impossibles) et #114 (`detection_refused` en double). Tests `should_panic` qui
  bloqueraient au lieu d'échouer : laissés tels quels.
- **Ruleset de `main` (13 octobre, 15 h 30)** : l'exigence de branche à jour est retirée (`strict_required_status_checks_policy`
  à false), et les 9 vérifications requises sont gardées. Raison : avec deux files en parallèle et 10 à 11 min de CI, presque
  chaque merge demandait une mise à jour et une CI de plus. Ce qui compense : la tour prédit chaque fusion avant la Review
  et la joue quand des fichiers sont partagés, et elle lit la CI de `main` après chaque merge. La file de fusion de
  GitHub est réservée aux dépôts d'organisation.
- **#114, plan (13 octobre, 17 h 45)** : validé. WHICH devient CHOIX ; les quatre aides sans équivalent dans la v0.1.1 sont
  traduites au plus près de #54 et montrées sur la planche ; « échec : » et « failed: » sont ajoutés devant la cause ;
  « LGPL version 2.1 or later » reste en anglais. Tests lourds : `local_command` une fois (la CI ne le lance jamais),
  et ni `rust:slow` ni `rust:reference` en local. « [par défaut : 1,5] » est gardé alors que `1,5` est refusé en
  entrée, comme dans la v0.1.1 : suggestion du point commentée sur #55. Contexte de l'agent compacté avant le code
  (87 %).
- **#114, arrêt 1 (13 octobre, 18 h)** : validé, avec les décisions de l'agent (un argument sert dans au moins une
  langue, pas de macro `langid!`, 13 crates nouvelles avec `cargo deny` vert). Le doublon « failed: the detection
  failed: » est commenté sur #64.
- **#114, arrêt 2 (13 octobre, 18 h 15)** : validé. Titres français calqués sur l'anglais, avec majuscule
  (« Utilisation : », « Arguments : », « Options : », « Réglages qui changent la détection : ») : les minuscules de
  la v0.1.1 sont refusées. Le défaut de `--osd-region` passe sur sa propre ligne, « [par défaut : aucune] » /
  « [default: none] ». Acceptés : VIDÉO, les guillemets droits, les deux lignes anglaises, les textes nouveaux de
  l'agent.
- **#114, compte rendu (13 octobre, 18 h 45)** : les espaces avant « : » des lignes de console restent ordinaires, comme
  dans la v0.1.1. La faille du test des clés est acceptée (un message ajouté sans échantillon n'est pas vu ; l'erreur de
  compilation mène au fichier) : pas de `strum`. Le nom de champ `clip_margin_s` écrit en dur est commenté sur #55 ;
  l'ordre de `[workspace.dependencies]` reste tel quel.
