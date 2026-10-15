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

### #86 et #91 — Sortie à côté des vidéos, homonymes à la casse près (worktree `extraits`, branche `rust/86-sortie`)

4 points (3 + 1), une seule PR qui ferme les deux. Pas de plan relu. Relecture : `lot8-relectures/gardes.md` § #86 et
§ #91 ; commentaire de #115 sur #91. Décision 5 de ce fichier.
- Décidé : sans `-o`, les résultats vont dans `<dossier de la vidéo>/batdetect/` ; `-o/--out-dir` reste possible et
  devient facultatif. Le contrôle d'écriture se fait avant tout calcul, pour toutes les racines, celle de `-o`
  comprise (#82 sera réduite à ce qui reste). Sans `-o`, un homonyme se nomme `<nom>_<ext>` dans `batdetect/` ; rien
  ne change avec `-o`. Les homonymes se comptent à la casse près (#91). Pas de cas particulier pour des vidéos lues
  sur le volume des jumelles.
- Recommandations de la relecture, à appliquer sauf prémisse fausse, et à lister dans « Décisions prises seul » :
  - le contrôle d'écriture est une sonde par racine (création exclusive d'un fichier, puis retrait), parce que
    `create_dir_all` réussit sur un dossier existant en lecture seule ; en cas d'échec, tout le lot s'arrête avant
    calcul, avec le code 2, et chaque dossier refusé est nommé (texte par Fluent, #114) ;
  - les homonymes se comptent par racine : sans `-o`, `nuit1/v.mp4` et `nuit2/v.mp4` ne se gênent plus ;
  - la clé des homonymes et du refus `SharedFolder` : nom en NFC (la normalisation NFC que #115 a mise dans `files::previous`, l. 95, à rendre partageable plutôt que copiée,
    si elle convient), puis `to_lowercase`, sur tous les systèmes ; un nom non UTF-8 passe par `to_string_lossy`
    pour la clé et reste un `OsStr` pour le chemin ;
  - les anciennes sorties `~/out/` de la v0.1.1 restent, sans migration (une phrase dans le README à #106).
- Le cas du commentaire sur #91 doit être couvert : sur APFS, `a/Vol.mp4` et `b/vol.mp4` avec `-o` partagent un dossier,
  et le ménage de la seconde effacerait les extraits de la première. Un test le prouve.
- `.gitignore` : un `batdetect/` sous `tests/fixtures/` (CSV, PNG, JSON) ne doit pas finir dans un commit.
- Docs : docs/11 (section `cli` et la ligne du défaut de `-o`).
- Planche dans `rapports/planche-86/` : où vont les résultats sans `-o`, avec `-o`, avec deux homonymes et deux noms
  qui ne diffèrent que par la casse, et le refus d'un dossier non inscriptible, dans les deux langues.
- Attendu : les tests inversés (`-o` absent n'est plus une erreur) et les nouveaux verts ; les tests longs que le diff
  peut changer tournent une seule fois, à la fin, sous le verrou ; dis lesquels et pourquoi.

### #179 — Tests rapides et tests lents en tâches de CI parallèles (worktree `ci`, branche `rust/179-ci-parallele`)

2 points, ajoutée au lot le 14 octobre à la demande de l'utilisateur. Pas de plan relu. En parallèle de la file
`commande` : le diff ne touche que `.github/workflows/ci.yml`, `mise.toml` si besoin, docs/11 et `CLAUDE.md`, jamais
`crates/cli`.
- Mesuré le 14 octobre (CI de `main` après #175, run 38438917879) : macOS 916 s, dont `rust:test` 302 s et `rust:slow`
  461 s (114 s de compilation, 345 s de tests, dont `slow_clips` 172 s et `slow_canary` 47 s) ; Ubuntu 816 s (278 +
  373) ; Windows 812 s (347 + 332) ; `rust-reference` 468 s. Les deux étapes ne partagent pas leur compilation
  (`target/llvm-cov-target` contre `target/ci`).
- À faire : séparer, sur chaque système, les tests rapides (couverture) et les tests lents en deux tâches parallèles,
  chacune avec son cache (clés distinctes, même logique de repli en PR que l'existant). Les vérifications requises du
  ruleset de `main` changent de noms : donne la liste exacte dans le compte rendu ; c'est la tour qui modifie le
  ruleset, au moment du merge.
- `slow_clips` : mesure ce que coûte chacun de ses 4 tests sur les trois systèmes (journaux de CI), et propose une
  réduction qui ne perd rien de ce qu'ils vérifient. Ne l'applique pas : c'est une question pour l'utilisateur.
- Attendu : même liste de tests passés par système, avant et après (`rapports/tests-146-extraire.sh`, à compléter
  avec les nouveaux noms de tâches) ; le temps d'une PR mesuré sur la PR elle-même, au second passage (cache chaud),
  par étape. Tu ne peux pas pousser : donne à la tour ce qu'il faut mesurer, elle le fera sur la PR.

### #180 — Réduire `slow_clips` (worktree `ci`, branche `rust/180-slow-clips`)

2 points, P2. Suite de #179, même agent. L'issue reprend le profil et les trois réductions proposées par l'agent ;
l'utilisateur les a acceptées, plus le passage de `the_similarity_is_one_…` en test rapide.
- Ne rien perdre de ce qui est vérifié : même similarité au bit près, mêmes `pts` et même nombre d'images contrôlés.
  Pas de comparaison d'une image sur N, pas de rendu fondu.
- Le code touché est dans `crates/render` (tests et leur support), pas dans `crates/cli`.
- Attendu : même liste de tests passés par système ; durée de `slow_clips` mesurée en local (sous le verrou, M1 Max)
  avant et après ; la tour mesurera la CI sur la PR (cache chaud).

### #63 — La vidéo et l'étape en cours (worktree `extraits`, branche `rust/63-progression`)

2 points. Pas de plan relu. Relecture : `lot8-relectures/gardes.md` § #63. Décision 6 de ce fichier.
- Décidé : étapes nommées détection (lecture comprise), suivi, image résumé, extraits ; la ligne d'une vidéo finie
  reste, avec son nombre de pistes (« [3/12] video_092 — 14 pistes », ou « — échec »).
- Recommandations de la relecture, à appliquer sauf prémisse fausse, et à lister dans « Décisions prises seul » :
  stderr ; effacement par `\r` puis des espaces (aucune séquence d'échappement) ; les lignes d'événement (extrait
  refait, CSV laissé) écrites au-dessus de la ligne d'étape, qui est redessinée ; `std::io::IsTerminal` pour le mode,
  sans crate ; un protocole d'étapes dans la bibliothèque, appelé par `process`, avec un rapporteur muet pour les tests ;
  pas d'avancement image par image (c'est #68).
- Textes par Fluent, dans les deux langues, avec la majuscule de #115 là où elle s'applique.
- Pièges de #178 : pas d'égalité exacte sur la sortie d'erreur, imports des tests réservés à un système.
- Planche dans `rapports/planche-63/` : la sortie redirigée (une ligne par étape) et, si possible, une capture du mode
  terminal, dans les deux langues.
- Attendu : tests de l'ordre des lignes d'étape sur stderr redirigé ; aucun test lourd (rien ne touche la détection),
  sauf si le diff le demande ; dis-le.

### #64 — Raisons en clair et conseils (worktree `extraits`, branche `rust/64-raisons`)

3 points. **Plan court d'abord** (`rapports/plan-64.md`), relu par `rust-expert`, puis arrêt pour validation : il
porte surtout la table des causes et les textes que Manon lira. Relecture : `lot8-relectures/gardes.md` § #64.
Décisions 6 et 7 de ce fichier. Deux commentaires sur l'issue (#114 : « failed: the detection failed: » ; #115 : un
fichier ouvert sous Windows fait échouer le ménage).
- Décidé : un crochet de panique à nous, qui garde le message et le lieu (forme finale de la ligne « erreur interne »
  de #113) ; le disque plein reconnu, par un accesseur `io_kind()` dans `media` (aucun type d'`ffmpeg_next` ne sort) ;
  les causes des bibliothèques restent en anglais, précédées d'une phrase traduite par cause connue ; « trop de pistes »
  garde le code 1, et le bilan de #65 le rangera à part des pannes.
- Recommandations de la relecture, à reprendre dans le plan : classement par `match` sur `process::Error` et ses
  énumérations, puis `downcast_ref::<io::Error>` le long de la chaîne pour le disque (piège : `#[error(transparent)]`
  saute un niveau) ; « introuvable » distingué de « pas une vidéo » ; « erreur inattendue » suivi du texte brut pour
  le reste ; le conseil de recopier depuis les jumelles en USB pour un fichier abîmé, avec un renvoi à docs/01.
- Le plan donne la table cause → phrase et conseil, en français et en anglais, mot pour mot : c'est ce que
  l'utilisateur validera. Il dit aussi comment le doublon « failed: the detection failed: » disparaît.
- Pièges de #178 : pas d'égalité exacte sur la sortie d'erreur, imports des tests réservés à un système.

### #184 — Un seul passage sur la 092 dans les tests de référence (worktree `ci`, branche `rust/184-reference`)

3 points, P2. **Plan d'abord** (`rapports/plan-184.md`), relu par `tech-architect` et `rust-expert`, puis arrêt pour
validation. Décision de l'utilisateur (14 octobre) : « tu le fais une première fois au début puis tu utilises ces
résultats partout » ; la détection à 1440 reste à part, en parallèle.
- Le plan dit : comment partager entre les quatre crates (`cli`, `compare`, `engine`, `media`) — un seul binaire de test
  ou un résultat écrit une fois et relu — et où vivent les tests ; la liste de toutes les assertions des quatre
  binaires actuels, et où chacune se retrouve ; le temps attendu en CI, et le budget de verrou en local (nombre de
  passages de la 092 et durée estimée de chacun).
- Le test de la commande sur la 092 dépend de la signature de `process` : #63 est fusionnée, ta branche part après
  elle. #64 (dans `extraits`) touche aussi `process` : dis dans le plan ce qui en dépend.

### #187 puis #181 — Ménage des tests lents, puis copies de la CI (worktree `ci`)

Ajoutées au lot le 14 octobre, à la demande de l'utilisateur. Pas de plan relu.
- **#187** (branche `rust/187-tmp`, 1 point) : `crates/cli/tests/slow_command.rs` (`EXTRACT_RUN`, l. 25) et
  `crates/compare/tests/slow_extract.rs` (`RUN`, l. 23) gardent un `TempDir` dans un `static`, jamais supprimé. Relever
  dans le `static` ce dont les tests ont besoin, et supprimer le dossier à la fin de l'initialisation, comme
  `reference_092.rs` (#184). Attendu : aucun dossier nouveau dans le dossier temporaire après `mise run rust:slow`
  (sous le verrou), même liste de tests.
- **#181** (branche `rust/181-ci-composite`, 2 points, après la fusion de #187) : une action composite pour la clé de
  cache de FFmpeg (six copies dans `ci.yml`), une autre pour la mise en place de Windows (deux copies). Attendu : mêmes
  clés de cache, vérifiées sur la PR ; `actionlint` vert ; même liste de tests par système.

### #65 — Le bilan de fin de soirée (worktree `extraits`, branche `rust/65-bilan`)

3 points. **Plan court d'abord** (`rapports/plan-65.md`), avec une maquette du bilan en français et en anglais, puis
arrêt pour validation. Relecture : `lot8-relectures/gardes.md` § #65. Décision 6 de ce fichier.
- Décidé : « trop de pistes » rangé à part des pannes dans le bilan ; pas de bilan écrit dans les dossiers (c'est #71) ;
  la ligne « N pistes -> dossier » de la sortie standard, gardée par #63 en attendant, est remplacée par le bilan.
- Recommandations de la relecture, à reprendre dans le plan : colonnes vidéo, durée, pistes, non analysé (plages
  instables et abîmées réunies), calcul ; le dossier en titre par racine (#86) ; le tableau sur stdout, la progression
  sur stderr (#63) ; largeur comptée en `char`, sans crate ; les échecs sous le tableau avec la cause et le « à faire »
  de #64, et une ligne « 2 vidéos sur 12 en échec » ; le CSV laissé (#115) dans le bilan, l'extrait refait en openh264
  reste une ligne au fil de l'eau ; pas d'état « sautée » avant #70 ; la durée lisible de `summary::header::duration` ;
  le rappel des réglages avancés changés en pied de bilan (décision du lot 8 : le rappel va à #65).
- Le modèle du résultat (bibliothèque) et la mise en forme du tableau (binaire) dans deux modules distincts, testés.
  Codes de sortie inchangés. Textes par Fluent, dans les deux langues, nombres par le formateur de #114.
- Pièges de #178 : pas d'égalité exacte sur la sortie d'erreur ; imports des tests réservés à un système ;
  `#[cfg(test)]` seulement sur les aides de tests qui font `unwrap`.

### #56 — Les usages courants en tête de l'aide (worktree `extraits`, branche `rust/56-tldr`)

3 points. **Plan court d'abord** (`rapports/plan-56.md`), avec la maquette du tl;dr, de `-h` et de `--help` en français
et en anglais, mot pour mot, puis arrêt pour validation. Relecture : `lot8-relectures/gardes.md` § #56. Décision 6.
- Décidé : les exemples portent le nom de la commande lancée (aujourd'hui `batdetect-rs`, `batdetect` après #106).
- Recommandations de la relecture, à reprendre dans le plan : clés Fluent et une fonction `usage` dans le module de
  textes, sans module à part ; sans aucun argument, le tl;dr sur stdout avec le code 0 (`-o` sans vidéo reste une
  erreur d'usage, code 2) ; seul : tl;dr et « --help pour tout » ; `-h` : tl;dr et options essentielles ; `--help` :
  tout, plus les fichiers produits (noms réels livrés par #86, #103, #115) et un renvoi au README ; les exemples sur le
  modèle de `tldr` (une ligne « description : », puis la commande), dans `before_help` et `after_help` comme prévu à
  #113 ; plus d'exemple de mode rapide, `--workers` ni `doctor` (#58). La section « Utilisation » du README va à #106.
- Le look de l'aide (#113, #115) se garde : couleurs, une ligne par option, défauts sur leur ligne, majuscule.
- Pièges de #178 ; `#[cfg(test)]` seulement sur les aides de tests qui font `unwrap`.

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
- **#159, compte rendu (13 octobre, 19 h 45)** : trois pistes gardées (chemin identité en 1:1, −4,6 % sur `092-1440` ;
  noyau plié, −1,4 % sur la 092 ; blocs masqués, −1,9 % sur `osd`), deux écartées (champs de `Rounding`, taps contigus
  et inverse sur 32 bits). Le moteur générique passe de +7,9 % à +6,3 % de CPU par rapport au 3:2 : on s'arrête là
  (la qualité prime sur le temps). Choix de l'agent acceptés, dont le refus d'une retouche de revue qui changeait le
  code machine produit. `mise run check` classé lourd dans `MESURES.md`, et les deux agents prévenus. Issue #176 créée
  (comparaison hors plages abîmées ; Backlog, 2 points, P3).
- **#159, vérification de la tour (14 octobre, 7 h)** : `compare local` sur la branche identique à main (966 pistes, 963
  appariées dont 958 concordantes, 3 perdues, 9 ajoutées, aucune vidéo sous 98 %). En Review.
- **Nuit du 13 au 14 octobre** : `extraits` s'est arrêté sur deux erreurs réseau (21 h 22 et 22 h 57) au milieu de
  #115 ; relancé le 14 à 6 h 55. CI de main après #175 verte, en 15 min 26 s.
- **Aide, majuscule (14 octobre, 7 h 30)** : chaque texte d'aide d'option ou d'argument commence par une majuscule, en
  français comme en anglais (« Vidéos, ou dossiers… », « Print the version »). Fait dans la branche de #115, en
  reliquat de #114. Planche dans `rapports/planche-115/`.
- **#159 fusionnée (PR #177, 14 octobre, 7 h 14)** : la file `qualite` est terminée.
- **#115, compte rendu (14 octobre, 8 h)** : décisions de l'agent acceptées (forme du CSV et sa limite, notée pour #83 ;
  pas de nouveau tableau sur un CSV étranger au nom courant ; le banc garde ses minuscules ; ménage en deux temps,
  `create_new`). Retouches : le CSV s'écrit par un fichier partiel renommé sans écraser, et le ménage retire aussi les CSV
  partiels ; `summary` prend la convention `.partial` de `files::names`, et un test la lie à celle de `media`.
  Commentaires : casse sur APFS et NTFS → #91 ; fichier ouvert sous Windows → #64 ; empreinte du CSV → #83. La tour a
  relancé `local_outputs` : vert, 18,5 s.
- **Worktree `qualite` fermé (14 octobre, 8 h 15)** : sous-espace herdr fermé, `git worktree remove` sans `--force`
  (worktree propre, PR #177 fusionnée), lien de mémoire retiré. La branche `rust/159-moteur` reste, à supprimer par
  l'utilisateur s'il le souhaite. `GRAPPES` ne garde que `extraits`.
- **#115, retouches (14 octobre, 8 h 45)** : le CSV s'écrit par `.<nom>.partial` renommé par `persist_noclobber`
  (`tempfile` passe en dépendance de `files`), et le ménage retire les CSV partiels ; une seule convention `.partial`
  dans `files::names`, un test la lie à celle de `media`. `local_outputs` vert, 17,8 s. En Review.
- **#178, CI rouge (14 octobre, 9 h 15)** : sous Windows, un import inutilisé (le seul test qui s'en sert est réservé à
  Unix) fait échouer clippy `-D warnings`. Sous macOS, deux tests de `command.rs` exigent une sortie d'erreur exacte,
  alors que le runner virtualisé de GitHub y écrit « IOServiceMatchingfailed for: AppleM2ScalerParavirtDriver ».
  Invisible sur le Mac de l'utilisateur. Correctif demandé à l'agent, dans la même branche.
- **#115 fusionnée (PR #178, 14 octobre, 7 h 52)** après le correctif de CI.
- **#179 créée et lancée (14 octobre, 8 h)** à la demande de l'utilisateur : séparer tests rapides et tests lents en
  tâches de CI parallèles (2 points, P2), et mesurer `slow_clips` (172 s sous macOS) sans le réduire. Worktree `ci`
  ouvert par le lanceur, en parallèle de la file `commande`.
- **#86 et #91 lancées (14 octobre, 8 h)** dans `extraits`, branche `rust/86-sortie` depuis 44153bb.
- **#179, compte rendu (14 octobre, 8 h 30)** : choix de l'agent acceptés (noms `rust`/`rust-windows` gardés, tâches
  lentes `rust-slow` et `rust-windows-slow`, répartition des étapes, LFS retiré des tâches rapides, docs/05). Au merge,
  la tour ajoute `rust-slow (macos-latest)`, `rust-slow (ubuntu-latest)` et `rust-windows-slow` au ruleset, puis purge
  les anciennes générations du cache (10,24 Go pour 32 entrées, au-delà du quota), liste montrée avant. Issues créées :
  #180 (réduire `slow_clips`, 2 points, P2, À faire, lancée dans `ci` après #179) et #181 (action composite pour la clé de
  FFmpeg et la mise en place Windows, 2 points, P3, Backlog). En Review.
- **#86 et #91, compte rendu (14 octobre, 8 h 45)** : deux prémisses fausses de la tour corrigées par l'agent (clé non
  UTF-8 par octets bruts et non par `to_string_lossy` ; la v0.1.1 écrivait dans `out/` relatif, pas `~/out/`). Décisions
  acceptées (sonde dans le dossier existant le plus proche, défaut de `-o` sur sa ligne, forme du refus). Règle de casse
  gardée sous Linux. Retouche : `writable` dans `CLAUDE.md`. Laissés : une même vidéo donnée deux fois est traitée
  deux fois, et le refus affiche un chemin relatif.
- **Après #182 et #183 (14 octobre, 11 h 50)** : ruleset à 12 vérifications requises (les trois tâches lentes
  ajoutées). Cache purgé de 18 entrées (6,5 Go : anciennes générations de main et PR fusionnées), 19 restent. CI de
  main après #179 : 7 min 51 s contre 15 min 16 s ; macOS 4 min 33 s (rapide) et 4 min 55 s (lent). Sur les PR #182
  et #183, `rust-slow (macos-latest)` a pris 7 à 9 min, cache froid (aucune entrée `-slow` sur main avant le merge de
  #182). Les plus longues sont maintenant `rust-reference` (7 min 48 s) et `rust-windows-slow` (7 min 46 s). #180 et
  #63 lancées.
- **#184 créée (14 octobre, 12 h)** à la demande de l'utilisateur : « tu le fais une première fois au début puis tu
  utilises ces résultats partout ». Les tests de référence traitent la 092 entière cinq fois (468 s en CI) ; un seul
  passage aux défauts, partagé, plus la détection à 1440 (gardée). 3 points, P2. Plan d'abord, dans `ci` après #180,
  relu par tech-architect et rust-expert ; code après la fusion de #63 (signature de `process`).
- **#180, compte rendu (14 octobre, 12 h 15)** : `slow_clips` de 107 à 115 s à 59 à 76 s sur le M1 Max (−27 % de CPU),
  similarité identique au bit près. Décisions acceptées. Retouche : `#[cfg(test)]` inutiles retirés ; le contrôle des
  pts des extraits que la tour avait demandé était fondé sur une prémisse fausse (extraits et zooms sont réécrits à la
  cadence de l'encodeur ; seule la vidéo annotée garde les pts de la source) : docs/11 est corrigée à la place.
- **#63, compte rendu (14 octobre, 12 h 20)** : étapes sur la sortie d'erreur, une ligne par étape quand elle est
  redirigée, réécrite sur place dans un terminal. Décisions acceptées (échec d'affichage qui arrête le lot, ligne de
  stdout gardée jusqu'à #65, cause au-dessus de « — échec », effacement par caractères). Retouches : `progress` et
  `status` dans `CLAUDE.md`, `#[cfg(test)]` retirés de `tests/command.rs`.
- **`#[cfg(test)]` dans `tests/` (14 octobre, 12 h 40)** : la tour l'avait dit inutile ; il sert à clippy, qui ne permet
  `unwrap` qu'en `cfg(test)`. Il n'est inutile que sur une aide sans `unwrap`. Retiré seulement de celles-là
  (`lines_naming`, `video_lines`, le support de similarité).
- **#184, plan (14 octobre, 13 h)** : validé. Un binaire `crates/cli/tests/reference_092.rs`, un `OnceLock` calculé une
  fois (passage aux défauts ∥ détection à 1440), `process` coupé en `process` + `process_detected` public, dépendance
  de test `cli` → `compare`, vérification N1 (réglages de la 092 = défauts) dans le test de concordance, recoupement
  des deux chemins de lecture perdu sur la 092 (gardé sur l'extrait). Estimation : `rust-reference` de 468 s à 300 à
  330 s. #184 fusionnée avant #64 ; l'agent de #64 est prévenu.
- **#64, plan (14 octobre, 13 h 15)** : validé. Une phrase pour « pas une vidéo » et la copie tronquée ; table validée
  mot pour mot, sauf « trop de pistes », dont « c'est probablement du bruit » est retiré (juger le bruit revient à
  Manon) : « regarder l'image résumé ; si ce sont de vraies traces, relancer avec un --max-tracks plus grand ». Choix de
  forme acceptés (trois lignes, « à faire : », lecture refusée et nom illisible à part, sans renvoi à docs/01, quota non
  reconnu côté FFmpeg). Codé sans attendre #184 : la branche sera mise à jour par l'utilisateur à la PR.
- **#184, compte rendu (14 octobre, 15 h 30)** : un binaire `reference_092.rs`, deux passages sur la 092 au lieu de
  cinq ; en local 116 s → 67 s, CPU 759 s → 437 s, pic 1,66 Go. Décisions acceptées (trois résultats séparés, causes
  gardées, `processed_in_removed_folder`). Retouche : `compare` dans la ligne de `cli` de `CLAUDE.md`. Issue #187 créée
  (dossiers laissés dans `/tmp` par `slow_command` et `slow_extract`, 1 point, P3, Backlog).
- **Ménage (14 octobre, 15 h 45)** : maquettes de rust-expert et dossiers de compilation d'une ancienne session `ci`
  supprimés (2,1 Go) ; dans `rapports/`, planches et mesures des issues fusionnées, démo de la 092, essai du lot 3,
  listes et journaux de CI (9,7 Go → 2,9 Mo). Restent les comptes rendus, plans, relectures et petites planches.
- **#64, compte rendu (14 octobre, 16 h 45)** : trois lignes par échec (phrase, « à faire : », « détail : »), doublon
  « failed » disparu, crochet de panique, `io_kind` dans `media`, `NoDecoder`. Choix de l'agent acceptés ; « format non
  géré » gardé pour une vidéo trop courte. Retouches : « si ce sont de vrais passages » (vocabulaire du projet, la tour
  avait écrit « traces »), `failure` dans `CLAUDE.md`. Le test du fichier ouvert, le crochet et la branche Windows ne
  seront vérifiés que par la CI Windows.
- **#184 fusionnée (PR #188, 14 octobre, 17 h)** : `rust-reference` 468 s → 258 s sur la PR ; les cinq noms de tests
  vérifiés identiques en local (les journaux de CI ne se téléchargeaient pas : `results-receiver` injoignable). #64
  commitée, fusion avec main simulée sans conflit. #187 et #181 ajoutées au lot pour le worktree `ci` ; #187 lancée.
- **#64 fusionnée (PR #189, 14 octobre, 18 h 58)** ; #65 lancée le 15 octobre.
- **#187, compte rendu (15 octobre)** : plus aucun dossier dans le TMPDIR après `rust:slow` (2 avant). Choix de l'agent
  acceptés. Retouches : `command.rs` (`FLIGHT`) et `settings_agreement.rs` passent à `CARGO_TARGET_TMPDIR` ; `panicked`
  partagé dans `cli/tests/support/`.
- **#181 (15 octobre)** : choix de l'agent acceptés. Retouche du chemin du cache FFmpeg sous Unix abandonnée : la tour
  l'avait dite sans effet sur la clé, mais `actions/cache` calcule la version sur le texte brut du chemin, et FFmpeg
  aurait été reconstruit. `~/.cache` reste en dur sous Unix (juste, puisque `XDG_CACHE_HOME` y vaut `$HOME/.cache`).
- **#65, compte rendu (15 octobre)** : bilan réel validé sur la planche. Choix acceptés (« avec le détail écrit pendant
  le traitement », « — » pour une durée impossible) ; retouche : `tally` et `board` dans `CLAUDE.md`.
- **Label `lot: 8`** créé et posé sur les 15 issues fermées par un merge du lot (règle ajoutée au skill le 15 octobre).
- **#65 (PR #191) et #181 (PR #192) fusionnées (15 octobre, 7 h 25)**. #192 : CI verte, aucune nouvelle entrée de cache
  `cargo-` ni `ffmpeg-` sur la PR, donc les clés n'ont pas bougé (succès exacts, FFmpeg non reconstruit). #56 lancée.
- **#56, plan (15 octobre)** : validé, avec un sixième exemple (deux soirées d'un coup). Choix de l'agent acceptés (format
  tldr, noms d'exemple, pied sous l'usage seul et `-h`, lien vers le README). `chemins-d-or.md` écrit par la tour, à
  valider par l'utilisateur ; la passe de QA se fera dans le worktree `ci`, après #56.
- **#56, compte rendu (15 octobre)** : disposition des fichiers produits sur deux lignes acceptée (clap cassait les
  colonnes). Retouches : noms de tranche, de partie et `batdetect/` tirés de `files::names` ; module `style` qui coupe
  les cycles `texts`/`errors` ↔ `arguments` ; `CLAUDE.md`. Le repli sans retrait à 80 colonnes n'est pas mesuré.
- **Pause (15 octobre, vers 16 h 50)** : PR #193 (#56) ouverte, CI en cours. À la reprise : lire sa CI, poser le label
  `lot: 8` sur #56 au merge, puis la passe de QA dans `ci` avec `chemins-d-or.md` (non relu par l'utilisateur), puis la
  clôture du lot.
- **Passe de QA du lot 8 (15 octobre)** : 58 lignes, 53 OK, 3 KO, 2 à constater ; pics de mémoire sous 1,2 Go. KO rejoués
  par la tour : 5b (raisons anglaises de nos propres refus) → commentaire sur #55 ; A4 (même vidéo sous deux écritures,
  né de #86 et #65) → #194, corrigée avant la clôture à la demande de l'utilisateur ; B7 (dossier d'extraits vide,
  antérieur au lot, #112) → #195 (P3, Backlog). Sept frictions → #196 (P3, Backlog). Cahier mis à jour.
- **#194, compte rendu (15 octobre)** : dédoublonnage par chemin réel, racines réelles ; rejoué par la tour (`. ./v.mp4`
  et deux casses sur APFS : une seule vidéo). Choix de l'agent acceptés (première écriture gardée, fichier absent,
  dédoublonnage sensible à la casse). En Review.
- **Clôture (15 octobre)** : #194 fusionnée (PR #197) ; copies de la QA supprimées ; worktrees `ci` et `extraits` fermés ;
  journal écrit ; leçons du lot réunies dans `retours.md` ; skill retouché (état de PR relu, ce que le local ne voit
  pas, retouche vérifiée dans le code, planches lourdes au merge, QA d'une commande, mesure du dossier des comptes
  rendus, budget de mesure dans la consigne commune, `board-field.py` patient après un ajout au board).
