# Lot 9 — la bascule vers le Rust (préparé le 15 octobre 2026, `main` @ f364698)

## Composition

Choisie par l'utilisateur le 15 octobre, après la clôture du lot 8.

| Issue | Points | Priorité | Note |
|---|---|---|---|
| #202 données hors du Mac | 3 | P2 | dépôt privé `chiro-dvr-data` + seau Scaleway + `mise run rust:data` |
| #198 (a) garde et validation à l'aveugle | 3 | P2 | sous-issue de #106 |
| #199 (b) retrait du Python, renommage, CI | 3 | P2 | après (a) et le tag `python-final` |
| #200 (c) docs | 5 | P2 | après (b) |
| #201 (d) profil de docs/09 | 3 | P3 | en parallèle, mesures sous le verrou |
| #205 découper `mise run check` (créée le 16 octobre) | 2 | P2 | grappe `bascule`, après #199 |
| #55 messages des options, refus avant calcul | 5 | P2 | corps à réécrire pour le Rust |
| #195 dossier d'extraits vide | 1 | P3 | |
| #196 frictions de la QA du lot 8 | 3 | P3 | cases 1 et 2 vers #55 |

#105 fermée le 15 octobre (ses trois sous-issues livrées au lot 8). #106 reste la parente, en Backlog.
Créées le 15 octobre (#198 à #202) ; #55, #195, #196 amendées en tête. Textes : `dvr-wt/rapports/lot9-issues/`, script `dvr-wt/rapports/lot9-apply.py`.

## Grappes proposées

| Grappe | File | Raison |
|---|---|---|
| `bascule` | #198 → #199 → #205 → #200 | (b) touche tout ; (c) décrit ce que (b) laisse |
| `donnees` | #202 → #201 | docs et mise.toml ; mesures de (d) sous le verrou, en parallèle de la garde |
| `commande` | #195 → #55 → #196 | #55 et #196 partagent `fr.ftl`, `en.ftl`, `texts.rs`, `messages.rs`, `main.rs`, `jobs.rs` ; #195 n'a que `process.rs` |

Conflits connus : `mise.toml` (données, (b)), docs/11 (toutes), `tests/command.rs` (#195, #55). Le retard se mesure
avant chaque Review.

## Prémisses corrigées (relecture du 15 octobre, `rapports/lot9-relectures/commande.md`)

- #55 : le refus avant calcul des chemins absents et des non-vidéos **n'est pas fait** ; l'inverse est figé par 4 tests
  (`tests/command.rs:205-217`, `:238-246`, `:362-371`, `jobs/tests.rs:217-223`). Déjà faits : plus de bloc usage (une
  ligne), suggestion d'option proche, nom de l'option dans les bornes. `ffmpeg absent` est caduc (FFmpeg statique).
  7 messages de `fr.ftl` interpolent de l'anglais, 3 joignables depuis la ligne de commande ; 8 causes anglaises
  distinctes derrière `invalid-value-because`.
- #195 : vrai ; un troisième cas laisse aussi un dossier vide (aucun encodeur, `process/tests.rs:109-116`). Le critère
  sur le ménage de #115 est déjà tenu.
- #196 : les sept cases sont vraies ; la case 5 (Ctrl-C) demande un gestionnaire de signal qui n'existe pas et sa
  promesse de reprise dépend de #70 ; la case 7 se raréfie avec #55.
- #106 : les chiffres de la relecture du lot 8 tiennent (112 `.py`, 5 tâches Python en CI dont 4 requises, docs/09 sans
  le mot Rust, `--against` sans contrôle de la vidéo).

## Décisions prises pendant le lot

1. 15 octobre, préparation : #105 fermée ; #106 redécoupée en quatre sous-issues (a) 3, (b) 3, (c) 5, (d) 3 ; l'issue des
   données entre en tête du lot ; #55, #195, #196 entrent.
2. #55 : refus en bloc avant calcul, code 2, une ligne par chemin avec sa raison (renverse 4 tests) ; ligne d'usage
   retirée de nos refus d'entrées ; raisons des nombres à nous, traduites, avec la plage, conseil du point pour une
   virgule ; `RegionShape` traduit par la commande ; table fermée de raisons d'E/S ; `--osd-region` à valeur négative
   refusée par son propre message ; refus injoignables qui nomment l'option ; `render` porte le réglage refusé typé ;
   corps réécrit pour le Rust, 5 points ; cases 1 et 2 de #196 rattachées.
3. #196 : « — trop de pistes » en fin de ligne ; majuscules des douze descriptions ; Ctrl-C avec `ctrlc` et une ligne
   « lot interrompu à la vidéo k/N ; relancer la même commande refait le lot » ; `-V` = `--version` ; le bilan ne titre
   que les racines qui existent.
4. #195 : `extraits/` créé seulement après l'essai d'encodeur réussi et avec au moins un extrait ; 2 tests inversés ;
   l'essai d'encodeur à 0 piste reste.
5. Trois grappes : `bascule`, `donnees`, `commande`.
6. #202 : `local_outputs.rs` lit `outputs/{fr,en}/video_092_original_3m24-4m05/` (122 Mo, 8 mp4 de 15 Mo), que
   l'issue laissait hors du dépôt de données. Décidé : ces sorties de l'extrait entrent dans le dépôt en **git
   ordinaire** (pas LFS) ; les trois vidéos de QA restent en LFS.
7. #198 : `--against` tire le nombre d'images des lignes d'`injected.jsonl.gz` (présent dans les bancs v0.1.1 comme
   Rust), pas de `bench-run.json`, que seule la v0.1.1 écrit ; l'écart à la consigne est noté dans docs/11.
8. #195 : décisions de l'agent acceptées (condition sur `outputs.clips`, erreur de dossier après l'ouverture de la
   source, `--annotated` dans le test à 0 piste, tournure de docs/11).
9. #202 (tour, d'après `crates/reference/src/location.rs` : la variable nomme le dossier qui contient `data/`,
   `bench/`…) : la racine du dépôt de données **est** `$BATDETECT_REFERENCE_DIR`, arbre à plat (`data/`, `bench/`,
   `summary/`, `logs/`, `outputs/`, `qa/`, `SHA256SUMS`, `README.md`) ; pas de sous-dossier `reference-v0.1.1/`.
10. #198 (tour, mandat de la consigne) : plan de mesures validé tel qu'écrit (`rapports/plan-198.md`, 56 min de verrou :
    compilation 5, référence 5, `compare local` 20, garde du banc 2, 092 chronométrée 2, sorties d'aveugle 10, `check`
    12) ; une seule passe de la 092 ; la 089-091 pour l'image résumé à périodes hors analyse. `--against` : 3 tests,
    137 verts dans `bench`.
11. #195 : PR #203 (8c5241f) ; `rust/55-messages` empilée sur ce commit, PR non fusionnée.
12. Seau `chiro-dvr-videos/img_0000` : envoi fini le 15 octobre à 20 h 40 (26 mp4, 10,845 Gio, 2 h 40 depuis un hôtel) ;
    `rclone check` à 20 h 41 : 0 différence, 26 fichiers ; `donnees` autorisé à lire le seau.
13. #55 : D1 à D6 confirmées (ni usage ni pied sous les refus d'entrées et de dossiers ; usage retirée des refus de
    valeur ; borne basse seule ; conseil du point pour `--threshold` et `--clip-margin` ; un dossier sans vidéo parmi
    d'autres refuse tout ; `allow_hyphen_values` sur `--osd-region`) ; les textes de `plan-55.md` (1.1 à 2.16) validés
    tels quels par l'utilisateur.
14. #198, mesures 0 à 3 (15 octobre, 21 h) : `rust:reference` 14 pistes ; `compare local` 963 / 3 / 9, 089 à 98,19 %,
    les 17 écarts de pistes identiques à #123 ; garde du banc identique à #109. Mesure 4 en attente du calme : Steam
    tient ~100 % de CPU ; l'utilisateur le quitte, l'agent fait les sorties d'aveugle en attendant.
15. #202, revirement (21 h 30) : l'utilisateur ne voulait ni LFS ni dépôt de données ; l'option cochée à 19 h disait
    « vidéos de QA en LFS » et la tour n'a pas relu ce choix après le « git ordinaire » de 20 h. Décidé : **tout dans
    le seau** (`reference-v0.1.1/` à côté d'`img_0000/`), `SHA256SUMS` dans le dépôt public, `rust:data` en rclone
    seul, pas de `qa/`. Le dépôt GitHub `chiro-dvr-data` (créé vide, push interrompu) et le `.git` local de 1,4 Go
    sont à supprimer par l'utilisateur.
16. #198, validation à l'aveugle (22 h) : six cas « aussi bien » (l'utilisateur voit peu ou pas de différence entre
    A et B) ; images résumé 097 et 089-091 : rien à redire. Mesure 4 : la 092 entière par la commande Rust en
    19,0 s réels, 128,5 s CPU, 1,09 Go au pic, 14 pistes. Partie 4 lancée : tests figés sur les six cas.
17. #198 (tour, d'après CLAUDE.md « `fingerprint` est la seule empreinte d'image des tests ») : `media::fingerprint`
    passe à un FNV-1a 64 bits écrit à la main, stable d'une version de Rust à l'autre (le `DefaultHasher` ne se fige
    pas), plus `of_rgba` pour les PNG ; aucune dépendance ajoutée. L'empreinte figée côté render porte le `YuvFrame`
    dessiné juste avant l'encodeur. À confirmer au premier passage en CI sur les trois systèmes.
18. Seau `reference-v0.1.1/` : téléversé par l'utilisateur (200 fichiers, 157 Mio, 10 min 39) ; `rclone check` le
    16 octobre à 0 h 26 : 0 différence, 200 fichiers. `donnees` lancé sur la vérification de `rust:data`.
19. #198 (tour, d'après la consigne projet « aucune vidéo ajoutée au dépôt : il est public » et le niveau `local` de
    docs/11) : les cas 2 et 4, dans l'extrait LFS, deviennent `render/tests/slow_frozen.rs` (CI, trois systèmes, à
    confirmer au premier passage : tiny-skia SIMD et swscale n'ont jamais été comparés au pixel près entre systèmes) ;
    les cas 1, 3, 5, 6 et les deux PNG, `local_` (jamais en CI) ; aucun fond médian des scènes n'entre dans le dépôt.
    Budget : 5 min de verrou en plus (61 min).
20. #198 : les tests figés des six cas vivent dans `cli` et dessinent les **pistes du Rust** (détection, suivi, dessin,
    couture `testing` de render sans encodeur) : on fige exactement les images jugées ; un changement de détection
    casse ces empreintes, à refiger en le disant. Choix de l'utilisateur, contre la recommandation de la tour (render,
    pistes v0.1.1).
21. #202 : `local_command` (`crates/cli/tests/local_command.rs:78`) lisait les noms des 113 `*_zoom.mp4` des sorties de
    la 097 et de la 125 (709 Mo). Décidé : une liste de noms versionnée dans `tests/fixtures/reference/`, le test la
    lit ; rien de plus au seau.
22. #55 : les cinq décisions de l'agent acceptées (valeur vide → message existant ; sixième raison « inaccessible » ;
    refus injoignables nommés par `match` exhaustif ; `MIN_WORK_WIDTH` pub ; `RegionShape` inchangé dans `engine`).
23. #202, vérification (1 h) : le rapatriement des 11 Go depuis le seau ne passe pas à l'hôtel (16 Mo en deux passes,
    0 Ko/s) ; la tour avait de plus mis ce téléchargement sous le verrou de calcul, à tort (24 min de blocage).
    Décidé : copie locale des 26 vidéos dans le dossier cible, `rust:data` à 0 téléchargé, relance à vide, `local_*`,
    `check` ; le chrono du rapatriement complet reste à mesurer depuis une bonne connexion.
24. #55 : vérifiée par la tour, Review ; #204 créée (Backlog, 1, P3) pour les `._*.mp4` exFAT.
25. #55, retouche avant commit (1 h 25) : le refus en bloc devient une question quand des entrées fautives côtoient des
    valides et que l'entrée standard est un terminal (« Traiter quand même les N vidéos valides ? (O/n) », Entrée =
    oui ; oui → valides traitées, refus au bilan, code 1 ; non → code 2) ; sans terminal, refus en bloc, code 2 ;
    option `--skip-refused` = oui sans question ; toutes fautives → refus en bloc. Textes à relire avant de figer.

## Consignes

Consigne commune : `~/.claude/skills/tower-control/assets/consigne-commune.md` + `consigne-projet.md`.

## #198 — la garde de la bascule

Issue : `gh issue view 198 --repo zaratan/chiro-dvr` (sous-issue de #106 ; lis aussi #106 et docs/11, section de la garde, l. 2686-2700).
Worktree : `/Users/zaratan/Projects/dvr-wt/bascule`, branche `rust/198-garde` depuis `origin/main`.

Quatre parties, dans cet ordre, avec un arrêt après la première et après la troisième.

1. **`--against` vérifie la vidéo** (`crates/bench/src/against.rs`) : nom tiré de `bench.json`, nombre d'images tiré de
   `bench-run.json` ; refus typé (`thiserror`) sinon, avec un test. Puis écris ton plan de mesures dans
   `rapports/plan-198.md` : chaque commande sous le verrou, son temps estimé, ce qu'elle prouve. **Arrête-toi** : la
   tour valide le budget avant que tu prennes le verrou.
2. **Les mesures**, une commande par `lockf`, au calme (critère de `MESURES.md`) : `mise run rust:reference` ;
   `cargo run --profile ci -p compare -- local` ; la garde du banc rejouée contre les bancs enregistrés de la v0.1.1
   (`$BATDETECT_REFERENCE_DIR/bench/{transit,hunt}`) ; temps et pic de mémoire de la 092 par la commande telle qu'elle
   est (`/usr/bin/time -l cargo run --release -p batdetect --bin batdetect-rs -- …`, sortie dans `out/` du worktree).
   Chaque chiffre va dans docs/11 avec la date et le commit. Un écart par rapport au lot 7 (963 appariées, 3 perdues,
   9 ajoutées, 089 à 98,19 % ; 14 pistes ; transit 0/0 ; chasse 2 perdues, 1 gagnée sur 584) se liste vidéo par vidéo
   avec le temps de la piste, sans le juger.
3. **La validation à l'aveugle**, préparée dans `/Users/zaratan/Projects/dvr-wt/rapports/aveugle-198/` : six cas sur
   extraits et zooms (piste en bordure avec zoom, passage rapide, piste courte, piste lente 3:29 de la 092, boîte
   interpolée, zoom ×4), choisis parmi les cinq vidéos dont la v0.1.1 a ses sorties
   (`$BATDETECT_REFERENCE_DIR/outputs/fr/` : 089-091, 092, extrait, 097, 125). Deux fichiers par cas, `cas-k-A.mp4`
   et `cas-k-B.mp4`, A ou B tiré au sort ; la correspondance dans `correspondance.md`, que seule la tour lit. Les deux
   images résumé (la 097 ; une image avec des périodes hors analyse) en clair, côte à côte v0.1.1 et Rust. Un
   `lisez-moi.md` : pour chaque cas, le temps de la piste et la question « la cible se voit-elle aussi bien ? ».
   **Arrête-toi** : l'utilisateur juge, la tour te rend les verdicts.
4. **Les tests figés** sur les cas validés : empreinte des images dessinées (RGBA, avant encodage) dans les tests de
   `render`, pixels des PNG dans ceux de `summary`, par `media::fingerprint`. Jamais une empreinte de mp4. Verdicts et
   mesures dans docs/11 (section de la garde), plus les lignes « jusqu'à #106 » que la garde rend fausses.

**Ne touche pas** au Python, à `mise.toml`, à `ci.yml`, à `release.yml`, ni aux noms des binaires : c'est la deuxième
partie de #106, dans cette même grappe, après toi. Pas de README ni de docs autres que docs/11 : troisième partie.

Attendu : `cargo test -p bench` refuse un banc rejoué sur une autre vidéo ; les quatre mesures écrites dans docs/11 ;
six verdicts écrits ; les tests figés passent (`mise run rust:test`, puis la CI sur trois systèmes) ; `mise run check` vert.
Chiffres de départ : ceux du lot 7 ci-dessus, sur c1432f6 (docs/11:2686-2688) ; les remesurer est l'objet de l'issue.
Budget de verrou : à décrire dans le plan ; ordre de grandeur attendu 40 min (référence 5, local 20, banc 10, 092 3).

## #202 — les données hors du Mac

Issue : `gh issue view 202 --repo zaratan/chiro-dvr`.
Worktree : `/Users/zaratan/Projects/dvr-wt/donnees`, branche `rust/202-donnees` depuis `origin/main`.

Tu prépares, tu ne pousses rien : le dépôt de données est créé et poussé par l'utilisateur.

1. **L'arbre du dépôt de données**, dans `/Users/zaratan/Projects/dvr-wt/chiro-dvr-data/` (hors de tout dépôt git,
   tu n'y lances aucune commande git) : `reference-v0.1.1/{data,bench,summary,logs}` copiés depuis
   `$BATDETECT_REFERENCE_DIR` ; `qa/` avec `video_099.mp4`, `video_124.mp4`, `video_129.mp4` copiées depuis
   `~/dossier sans titre/img_0000/` ; `.gitattributes` qui met `qa/*.mp4` en LFS ; `SHA256SUMS` (`shasum -a 256`) de
   tout l'arbre et des 26 vidéos du dossier source ; `README.md` : provenance (tag `v0.1.1`, commit c9948d2),
   recette de réenregistrement, emplacement du seau, date et résultat du `rclone check`. **Arrête-toi** : l'utilisateur
   crée le dépôt privé `zaratan/chiro-dvr-data` et pousse l'arbre.
2. **`mise run rust:data`** dans `mise.toml` : exige `BATDETECT_REFERENCE_DIR` et `BATDETECT_VIDEOS_DIR` (aucun défaut) ;
   refuse un `BATDETECT_VIDEOS_DIR` égal à `in/` du dépôt ou à `~/dossier sans titre/img_0000`, ou qui contient des
   mp4 absents de `SHA256SUMS` ; `git clone` ou `git pull` puis `git lfs pull` du dépôt de données ; `rclone copy
   chiro-dvr:chiro-dvr-videos/img_0000 "$BATDETECT_VIDEOS_DIR" --include "*.mp4"` ; vérification de chaque SHA-256 ;
   ne retélécharge que ce qui manque ou diffère ; une ligne de bilan par emplacement.
3. **Docs** : `tests/fixtures/reference/README.md` (emplacements et récupération), docs/11 (la ligne du niveau `local`,
   l. 436), une ligne dans `CLAUDE.md` (section Rust).

**Ne touche pas** à `~/.config/rclone/rclone.conf`, ni à `in/`, ni à `~/dossier sans titre/` ; jamais `rclone sync`
ni `rclone move`, seulement `copy` et `check`. Le seau se remplit encore depuis le Mac : attends le mot de la tour
avant tout `rclone check`, et ne lance aucune lecture du seau pendant l'envoi.

Attendu : `rust:data` vers deux dossiers vides de `dvr-wt/` reproduit les deux emplacements, 0 écart de SHA-256,
temps noté ; relancé aussitôt, 0 octet téléchargé ; `cargo test --workspace --locked --profile ci --test 'local_*' --
--include-ignored` vert sur les copies, sous le verrou, une fois ; `rclone check` à 0 différence.
Mesuré le 15 octobre par la tour : `data/` 34,1 Mo, `bench/` 2,9 Mo, `summary/` 0,1 Mo, `logs/` 0,2 Mo ; 26 mp4,
10,77 Go ; 099 453 Mo, 124 436 Mo, 129 407 Mo.

## #195 — plus de dossier d'extraits vide

Issue : `gh issue view 195 --repo zaratan/chiro-dvr` (son premier paragraphe porte la décision du lot).
Worktree : `/Users/zaratan/Projects/dvr-wt/commande`, branche `rust/195-extraits-vides` depuis `origin/main`.

`crates/cli/src/process.rs` (`render_clips`, l. 257-270) : `extraits/` n'est créé qu'après l'essai d'encodeur réussi
et quand au moins un extrait va s'écrire. Les deux tests qui figent le dossier vide (`process/tests.rs:92-107`,
`:109-116`) s'inversent ; un test d'arborescence pour 0 piste dans `tests/command.rs`, près de `:220-236`.

**Ne touche pas** à l'essai d'encodeur à 0 piste (`params.json` enregistre la liste), ni au ménage de #115, déjà bon.

Attendu : sur une copie de l'extrait dans `out/`, `--threshold 200` et `--max-tracks 1` donnent un dossier de vidéo
sans `extraits/`, code 0 puis 1, CSV et image résumé présents ; `cargo test -p batdetect --profile ci` vert ;
`mise run check` vert.
Mesuré le 15 octobre par la tour sur 22b41f9 (passe de QA, ligne B7) : `extraits/` présent et vide dans les deux cas.

## #55 — messages des options et refus avant calcul

Issue : `gh issue view 55 --repo zaratan/chiro-dvr --comments`. Le paragraphe « Décision du 15 octobre 2026 (lot 9) »
en tête porte les décisions ; le corps en dessous décrit le Python et ne vaut que pour le comportement à obtenir ; les
cinq commentaires listent les cas Rust avec leurs lignes. La relecture de la tour :
`/Users/zaratan/Projects/dvr-wt/rapports/lot9-relectures/commande.md`, section #55 (table « demande / état / preuve »).
Worktree : `/Users/zaratan/Projects/dvr-wt/commande`, branche `rust/55-messages` (créée par la tour).

Deux parties, avec un arrêt entre les deux.

1. **Le refus en bloc avant tout calcul** (`crates/cli/src/jobs.rs`, `main.rs`). La planification relève **tous** les
   chemins fautifs au lieu de s'arrêter au premier (`videos_of` l. 37, `SharedFolder` l. 103) : absent, fichier nommé
   dont l'extension n'est pas une vidéo (les cinq extensions reconnues citées), dossier vide, dossier illisible,
   plusieurs vidéos vers le même dossier de sortie. Une erreur typée qui porte la liste ; la commande écrit une ligne
   d'erreur puis une ligne par chemin avec sa raison traduite, code 2, aucun dossier `batdetect/` créé, pas de ligne
   d'usage. Les quatre tests qui figent l'échec au tour de la vidéo (`tests/command.rs:205-217`, `:238-246`,
   `:362-371`, `src/jobs/tests.rs:217-223`) s'inversent ; `failure::Missing` et `NotAVideo` restent pour un fichier qui
   disparaît en cours de lot. « aucune vidéo trouvée » liste les dossiers donnés et les extensions reconnues (case 2
   de #196). **Arrête-toi** avec, dans `rapports/plan-55.md`, la table de tous les messages nouveaux ou changés des
   deux parties, en français et en anglais, texte exact : la tour les fait relire avant que tu les figes dans les tests.
2. **Les textes** (`locales/{fr,en}.ftl`, `bin/batdetect-rs/{errors,messages,texts,arguments}.rs`, `src/writable.rs`) :
   les huit causes anglaises derrière `invalid-value-because` (bibliothèque standard, clap `is not in 0..=…`) deviennent
   des phrases à nous qui nomment l'option et la plage ; une virgule décimale reçoit le conseil d'écrire le point ;
   `RegionShape` est traduit par la commande, `engine` expose une erreur typée sans texte de langue ; les raisons de
   `jobs::Error` et les genres d'E/S des dossiers refusés passent par une table fermée de raisons traduites (refusé,
   pas un dossier, introuvable, lecture seule, disque plein), sur le modèle de `failure.rs` (case 1 de #196) ;
   `--osd-region` accepte une valeur qui commence par un tiret pour la refuser avec son propre message ; les refus
   injoignables (`arguments.rs:285-298`, `:318-327`) nomment l'option ; `render` porte le réglage refusé sous forme
   typée, le nom `clip_margin_s` en dur disparaît (`messages.rs:44`). La ligne d'usage ne suit plus que les erreurs
   de syntaxe ; le pied dit `<nom du binaire> --help`, tiré du nom réel (il change à #199).

**Ne touche pas** aux cases 3 à 7 de #196 (après toi, même grappe), ni au bilan (#65), ni aux causes d'échec en cours
de calcul (#64) hors ce que le point 1 déplace. `failure-detail` (l. 50) et `clips-redone` (l. 8) gardent leur texte
anglais : décidé à #64.

Attendu, en français (`LANG=fr_FR.UTF-8`), sur une copie de l'extrait dans `out/` et un dossier vide `out/vide/` :
- `batdetect-rs absente.mp4 README.md out/vide extrait.mp4 -o out/s` : code 2, `out/s` absent, sortie d'erreur = une
  ligne d'erreur puis trois lignes (chemin, raison), sans ligne d'usage, sans mot anglais ;
- `--max-tracks many`, `--threshold 11,5`, `--osd-region 0,0,1`, `--osd-region -0.1,0,1,1`, `--min-hits 0` : code 2,
  chaque refus nomme l'option, raison en français ; la virgule reçoit le conseil du point ;
- `--tresh 3` : la suggestion `--threshold` et la ligne d'usage, comme aujourd'hui ;
- `fr.ftl` n'interpole plus d'anglais hors `failure-detail` et `clips-redone` (`grep -c` sur les sept lignes relevées) ;
- `cargo test -p batdetect --profile ci` et `mise run check` verts.
Mesuré le 15 octobre par la tour (QA du lot 8, 22b41f9) et par la relecture (f364698) : `--osd-region 0,0,1` donne un
cadre français avec « expected four fractions X0,Y0,X1,Y1 » ; `--max-tracks many` « invalid digit found in string » ;
`absente.mp4` est accepté puis échoue à son tour, code 1 ; 7 lignes de `fr.ftl` interpolent de l'anglais.

## Incidents

- 15 octobre, 20 h 16 et 20 h 29 : le guetteur a rendu « ARRÊTÉ sans <fichier> » alors que le fichier (rapport-195.md,
  plan-198.md) venait d'être écrit, à la même minute que l'arrêt de l'agent. À regarder à la clôture (`stat`, minute
  de l'arrêt).
- 20 h 55 : le guetteur a rendu « ARRÊTÉ » sur `bascule` qui attendait, à dessein, son `compare local` de 20 min en
  shell de fond : l'arrêt de dix minutes sans fichier ne tient pas compte d'un shell qui tourne. Relancé.
- 16 octobre, 0 h 50 : `board-field.py` a rendu `KeyError: 204` deux fois sur une issue tout juste créée, avant et après
  `gh project item-add` : le listing du projet est resté en retard plus d'une minute (six relances de 10 s). Champs
  posés par l'identifiant d'item rendu par `item-add`. À faire : `board-field.py` accepte un id d'item, ou `lot9-apply.py`
  sert à toute issue créée en chemin.

26. 16 octobre, 2 h 30 : verrou à deux niveaux (`dvr-wt/verrou lourd|chrono`, `flock` partagé/exclusif), `MESURES.md` et la
    consigne projet réécrits ; réseau hors verrou. #205 créée (2 points, P2, « À faire », grappe `bascule` après #199) :
    `check` rapide en local, lourd en CI ; 21 min de `check` mesurées ce soir.

## #196 — les frictions de la QA du lot 8

Issue : `gh issue view 196 --repo zaratan/chiro-dvr`. Le paragraphe « Décision du 15 octobre 2026 (lot 9) » en tête
porte les décisions, case par case. Les cases 1 et 2 sont faites par #55 (branche `rust/55-messages`, sur laquelle tu
pars) : ne les refais pas, coche-les dans le compte rendu en renvoyant à #55. La relecture de la tour :
`/Users/zaratan/Projects/dvr-wt/rapports/lot9-relectures/commande.md`, section #196 (fichiers et lignes par case).
Worktree : `/Users/zaratan/Projects/dvr-wt/commande`, branche `rust/196-frictions` (créée par la tour).

Cinq cases, chacune avec son test, dans cet ordre :
- **Case 3** : la fin de ligne de « trop de pistes » dit « — trop de pistes » (le mot du bilan, `fr.ftl:143`), plus
  « — échec » (`status.rs:85-90`, `messages.rs:93-98`).
- **Case 4** : majuscule en tête des douze descriptions des fichiers produits (`fr.ftl:181-186`, `en.ftl:166-171`) ;
  le test des majuscules (`texts/tests.rs:676-688`) s'étend aux identifiants `file-*`.
- **Case 6** : `-V` et `--version` affichent la même chose, ligne FFmpeg comprise (`arguments.rs:225-229`,
  `:247-253`) ; un test pour les deux dans `tests/version.rs`.
- **Case 7** : le bilan ne titre que les racines qui existent (`board.rs:29-35`, `tally.rs:16-24,63-65`) ; une vidéo
  en échec reste sous « vidéos en échec ». Après #55, le cas ne vient plus que d'une vidéo abîmée ou disparue en
  cours de lot : fabrique-le ainsi dans le test.
- **Case 5** : un gestionnaire de signal (crate `ctrlc`, déclarée une fois dans `[workspace.dependencies]`, passée par
  `cargo deny`) et une ligne sur la sortie d'erreur à l'interruption : « lot interrompu à la vidéo k/N ; relancer la
  même commande refait le lot » (les deux langues), code 130 inchangé, rien d'autre ne change (le ménage de #115 rend
  la relance propre). Sans promesse de reprise : c'est #70. Teste-la en envoyant SIGINT au processus de la commande
  sur une copie de l'extrait.

Reliquat de #55 à prendre ici : dans `fr.ftl`, devant « ; », les anciens messages mettent tantôt une espace insécable,
tantôt une espace ordinaire ; uniformise sur l'insécable, comme devant « : », et dis dans le compte rendu combien de
messages ont changé.

**Ne touche pas** au bilan hors la case 7 (#65), ni aux causes d'échec (#64), ni à la reprise d'un lot interrompu (#70).

Attendu, en français, sur une copie de l'extrait dans `out/` :
- `--max-tracks 1` : la ligne de progression finit par « — trop de pistes », code 1 ;
- `--help` : les douze descriptions des fichiers produits commencent par une majuscule, dans les deux langues ;
- `-V` et `--version` : sortie identique, deux lignes ;
- une vidéo tronquée (`head -c 20000000` de l'extrait) à côté d'une vidéo saine : le bilan n'a qu'un titre de
  dossier, la tronquée sous « vidéos en échec » ;
- Ctrl-C pendant la détection : la ligne « lot interrompu à la vidéo 1/1 … », code 130, aucun `.partial` après
  relance ;
- `cargo test -p batdetect --profile ci` vert ; `mise run check` vert, une fois, sous le verrou.
Mesuré le 15 octobre par la tour (QA du lot 8, 22b41f9) : « — échec » sur trop de pistes ; 12 descriptions en
minuscule ; `-V` une ligne, `--version` deux ; titre `pieges/batdetect` pour un dossier jamais créé ; rien après un
Ctrl-C.

## #199 — le retrait du Python

Issue : `gh issue view 199 --repo zaratan/chiro-dvr` (sous-issue de #106 ; la garde #198 est passée : parité remesurée,
six cas jugés « aussi bien », tests figés). La relecture de la tour : `/Users/zaratan/Projects/dvr-wt/rapports/lot8-relectures/commande.md`,
section #106, « Ce que la bascule demande » et « Le retrait » (listes de fichiers et de tâches).
Worktree : `/Users/zaratan/Projects/dvr-wt/bascule`, branche `rust/199-retrait` (créée par la tour sur le commit de #198).

Ce qui part, ce qui reste et ce qui se renomme est dans l'issue ; les points qui ne s'y lisent pas :
- `python` reste dans `mise.toml` pour `rust:rules` seul (décision du lot 8) ; `uv`, les tâches Python, `lefthook`
  `ruff`/`format` et les trois premières étapes de `check` partent. `check` commence au Rust.
- `ci.yml` : les cinq tâches Python partent (`changes`, `static`, `fast-tests`, `slow-tests`, `reference`) ;
  `.github/actions/setup` perd `uv` et le FFmpeg de BtbN. Les quatre contrôles requis du ruleset sont retirés par
  l'utilisateur au merge : dis dans le compte rendu lesquels.
- `release.yml` : une seule étape, qui échoue avec « publication impossible avant #93 ».
- Binaires : `batdetect` et `batdetect-bench` (`[[bin]]`, `src/bin/…`, `CARGO_BIN_EXE_…`, `--bin`, `otool`, `ldd` dans
  `ci.yml`, les textes d'aide et de pied qui lisent `CARGO_BIN_NAME`). Version du workspace 0.0.0 + `git describe`.
- `tests/fixtures/video_092_original.mp4` (LFS) et `tests/fixtures/rust/` restent ; `tests/fixtures/reference/` reste.
- La PR dit `Closes #16` et `Closes #52` en plus de `Closes #199` ; le compte rendu dit ce que #45 perd.
- Docs : seulement ce que le retrait casse (commandes de CLAUDE.md et de docs/11, section Python de CLAUDE.md
  retirée) ; la réécriture est #200.

Reliquats de #198, à prendre ici parce que les fichiers sont dans ta branche : `cli/tests/local_clips.rs:42-57`
recopie le suivi de `process_detected`, passe par `process::tracked` ; docs/11 l. 21 « les extraits sont en cours
(#112) » est périmé.

**Préalable au merge, par l'utilisateur** : le tag `python-final` sur le dernier commit de `main` qui a le Python ; la
tour le lui rappelle avant la PR. Ne le pose pas toi-même.

**Ne touche pas** à `packaging/NOTICE`, `GPL-3.0.txt`, `release-notes.sh` (restent jusqu'à #93), ni à la détection.

Attendu : `find . -name '*.py' -not -path './target/*'` ne rend rien ; `mise run check` vert ; `batdetect --version` et
`batdetect-bench --help` répondent ; `mise run rust:reference` à 14 pistes (une fois, `verrou lourd`) ; `git grep -n
"batdetect-rs\|batdetect-bench-rs"` ne rend que docs/11 (histoire) et le CHANGELOG ; la CI de la PR verte sur les
huit tâches Rust.
Mesuré le 15 octobre par la tour sur f364698 : 112 fichiers `.py` ; 5 tâches Python en CI dont 4 requises ;
`batdetect-rs` dans 9 fichiers suivis, `batdetect-bench-rs` dans 7.

## #201 — le profil du Rust dans docs/09

Issue : `gh issue view 201 --repo zaratan/chiro-dvr` (sous-issue de #106). Les chiffres de bout en bout existent déjà
dans docs/11, section « Mesure de la bascule (#198) » : la 092 entière par la commande en 19,0 s réels, 128,5 s de CPU,
1,09 Go au pic (v0.1.1 : 135,4 s, 436,6 s, 0,70 Go). Ton travail est de dire **où** passent ces 19 s et ces 128 s.
Worktree : `/Users/zaratan/Projects/dvr-wt/donnees`, branche `rust/201-profil` (créée par la tour sur le commit de #202).

- Lis d'abord `/Users/zaratan/Projects/dvr-wt/MESURES.md` : le verrou a deux niveaux depuis cette nuit. Tes relevés de
  temps passent par `verrou chrono`, le profileur par `verrou lourd`.
- Écris ton plan de mesures dans `rapports/plan-201.md` avant de rien lancer : huit mesures au plus, une heure de
  verrou au plus, chacune avec ce qu'elle prouve. **Arrête-toi** : la tour valide le budget.
- Mesures prévues : temps par étape sur la 092 entière et sur l'extrait (le protocole d'étapes de #63 les sépare
  déjà : détection, suivi, image résumé, extraits, zooms), total réel et CPU, pic de mémoire (`/usr/bin/time -l`),
  nombre de fils actifs ; un profileur d'échantillonnage, choisi par toi et nommé dans la doc, pour les points chauds
  du décodage et de la détection ; binaire `release`, `LANG=fr_FR.UTF-8`, sorties dans `out/` de ton worktree,
  copie de la 092 depuis `in/` jamais modifiée.
- docs/09 réécrite pour le Rust, dans la même forme que docs/09 aujourd'hui : protocole (verrou, calme, commande
  exacte), machine, date et commit de chaque chiffre, et ce que les chiffres disent ; docs/README si sa ligne change.
- **Aucun changement de code.** Un point chaud trouvé devient une proposition dans le compte rendu, avec sa mesure ;
  la tour en fait une issue.

Attendu : chaque chiffre de docs/09 a sa commande, sa date, son commit et sa machine ; le temps et le pic de la 092
recoupent ceux de #198 au bruit près (dis l'écart) ; `git diff --stat` ne touche que docs/09 et docs/README ;
`mise run rust:static` vert (pas de `check` : rien de code).
Mesuré le 15 octobre par #198 (f364698, au calme) : 19,0 s réels, 128,5 s CPU, 1,09 Go ; par la CI, `rust:reference`
traite la 092 en 2 passages.
27. #198 (3 h 50) : vérifiée par la tour (12 fichiers + 6 nouveaux, +387 −29, 1 `///`, 0 `allow`, aucun `cfg(unix)`
    touché, fusion sans conflit, 2 derrière), Review ; cinq décisions de l'agent acceptées (cas 2 et 4 figés deux fois,
    PNG en français, lignes « jusqu'à #106 » gardées, piste courte 097 1:54, `#[cfg(test)]` sur les aides à `unwrap`).
    Mesures : 53 min de verrou pour 56 prévues ; `check` 18 min. Question ouverte : si `slow_frozen` diffère sous
    Linux ou Windows, chercher l'étape qui diffère, pas une constante par système.
28. #202 : PR #207 (b38748a) ; `rust/201-profil` empilée dessus. #55 : correctif Windows (49a466c) dans `fix55`, CI
    relancée. #196 lancée sur 043134f ; les espaces de `fr.ftl` tout en insécable (convention existante, tour).
29. 16 octobre, 4 h 05 : `/tc-delegate on` (session) : la tour commite, pousse, ouvre et merge les PR des branches du
    lot ; format relevé : titre court en anglais sans corps, PR « Closes #N », merge commit. #198 commitée 9bd7915,
    PR #208. `rust/199-retrait` empilée sur 9bd7915, consigne envoyée. `python-final` reste à l'utilisateur.
30. 4 h 40 : #207 mergée par la tour (0c6c270, #202 fermée, `lot: 9`, question du chrono du rapatriement commentée) ;
    #206 mergée par l'utilisateur (21754f1, #55 fermée, `lot: 9`) ; worktree `fix55` retiré. #208 (#198) CLEAN, merge
    programmé après la CI de `main`. Caches Python du worktree `bascule` supprimés par la tour pour #199.
31. 5 h 05 : #208 mergée par la tour (42ca4d5, #198 fermée, `lot: 9`) après les CI de `main` de #206 et #207 ; CI de
    `main` verte sur 42ca4d5 : `slow_frozen` passe sur les trois systèmes, la question ouverte de #198 est close.
    `rapports/aveugle-198/` (165 Mo) à supprimer, proposé à l'utilisateur.
