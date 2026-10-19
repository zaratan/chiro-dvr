# batdetect — guide pour Claude

Détection et suivi de chauves-souris dans des vidéos de jumelles thermiques. Rust, FFmpeg lié en statique.
Mode d'emploi : [README.md](README.md). Construire, vérifier, publier : [CONTRIBUTING.md](CONTRIBUTING.md).
Base de connaissances : [docs/](docs/README.md), à lire avant de toucher à un réglage ([docs/02-methode.md](docs/02-methode.md)).

```bash
mise run check                    # rust:static et rust:test, 20 s à 1 min : vert avant de rendre la main
cargo run --release -p batdetect -- in/video_092_original.mp4 -o out # référence : 5 min, 14 pistes, ~20 s de calcul (docs/09)
```

Un binaire `-dirty` (changements non commités, `--version`) ne saute jamais une vidéo déjà faite : la relance de la référence mesure toujours le code courant (#70).
Un binaire propre la saute.

## Rôles

Manon, naturaliste, confirme les passages à l'œil.
Son verdict sur la 092, rendu le 9 octobre 2026, est la vérité terrain du test `slow` ([docs/03](docs/03-resultats-092.md)).
Sur 14 pistes, 13 sont des chauves-souris. Seule la piste lente de 3:29 n'en est pas une.
Mieux vaut un faux positif facile à écarter qu'un passage manqué.
Une question sur ce qui est ou n'est pas une chauve-souris lui remonte. Elle ne se tranche pas dans le code.

## Règles

- **Jamais d'écriture dans git** : ni `add`, ni `commit`, ni `stash`, ni `push`. L'utilisateur construit l'historique lui-même.
- **Tout changement de détection ou de suivi se mesure sur `video_092_original.mp4`** avant d'être rendu.
  On relève le nombre de pistes et les temps de début, et on met à jour [docs/03-resultats-092.md](docs/03-resultats-092.md).
- **`in/` et `out/` sont hors git.** `in/` contient les vidéos de l'utilisateur : ne rien y supprimer ni modifier.
- **L'extrait `tests/fixtures/video_092_original_3m24-4m05.mp4` est coupé sans réencodage** sur une image-clé de l'original (image source 6150, 30,03 i/s).
  Le réencoder changerait les détections.
- **Seules trois vidéos de `tests/fixtures/` sont suivies par Git LFS** (`.gitattributes`) : les deux de la 092 et `video_092_original_3m24-4m05_720x540.mp4`.
  Cette dernière est l'extrait réduit une fois par la commande de [docs/11](docs/11-migration-rust.md#réglages), et ne se refait jamais.
  Après un clone, lancer `git lfs pull`. Sinon les tests `slow` et `reference` échouent sur un pointeur (garde dans la crate `reference`).
  Les cinq petites fixtures du Rust sont des fichiers git ordinaires de quelques Ko.
  Ce sont `full_range.mp4`, `full_range_midway.mp4`, `two_gops.mp4`, `spot.mp4` et son remux en plage pleine `spot_full_range.mp4`.
  Elles ont été faites une fois par la commande de [docs/11](docs/11-migration-rust.md#tests).
  Les détections de l'extrait par le Rust (`tests/fixtures/rust/`) sont aussi des fichiers ordinaires.
  `compare --frames` les écrit, et chaque système doit les retrouver au bit près.
  La 092 entière `video_092_original.mp4` est une copie exacte de `in/`.
  Ses tests (`tests/reference_*.rs`) tournent par `mise run rust:reference` et ont leur propre tâche de CI, `rust-reference`.
  La tâche `rust-windows-slow` ne récupère que l'extrait. Les tâches `rust-slow` et `rust-settings` récupèrent l'extrait et sa 720×540.
  Les gardes de détection portent sur la référence v0.2.0 (`tests/fixtures/reference-v0.2.0/`, #276).
  Ce sont des écarts figés par vidéo et par cas. Ils comptent les images qui diffèrent et les détections perdues et ajoutées.
  Côté pistes, ils comptent les pistes appariées, concordantes, perdues et ajoutées. Ils valent 0 jusqu'à #21.
  Après, ils ne comptent que des détections faibles ajoutées et leurs pistes, avec leur cause (docs/11).
  Une piste s'apparie à ±5 images de début et ±3 points.
  Les seuls faits affirmés sans référence sont ceux de Manon sur la 092 (`reference_092`) : ses 13 passages et leurs débuts.
  À côté, seuls l'objet lent de 3:29 et la cible connue de 3:42.10 sont tolérés (docs/03).
  L'écart entre Mac et Linux de 4 images et 2 points avait été mesuré contre la v0.1.1.
  Celle-ci ne garde plus que l'aval, les sorties, le format v1 et les entrées (témoin ffprobe).
- **Les vidéos se copient depuis les jumelles en USB**, jamais par l'export Stream Vision 2.
  Cet export divise le débit par 12 ([docs/01-contexte.md](docs/01-contexte.md)).
- **Docs synchronisées** : un changement de comportement met à jour `docs/` dans la même série de changements.
  Il ne touche que le document qui possède cette vérité (table de [docs/README.md](docs/README.md#qui-tient-quoi)).
- **Coupure des lignes** : dans les `.md`, on coupe à la fin d'une phrase.
  Une ligne regroupe des phrases jusqu'à ~140 caractères, et une phrase plus longue reste entière.
  Jamais de coupure à une largeur fixe (#256).

## Code

**Un module, une responsabilité, ses tests.** Mieux vaut beaucoup de petits fichiers clairs qu'un gros fichier à plusieurs rôles.

- Code, identifiants et README en anglais, docs en français. Une valeur magique devient une constante nommée.
- Les docs pour la naturaliste (`docs/premiere-soiree.md`, `docs/lire-les-resultats.md`, le README) ne contiennent rien de technique.
  Ni commande de reproduction, ni `<details>`, ni script, ni nom de crate ou de fichier source.
  Les captures se refont par `tools/docs-images.sh` ([docs/images/README.md](docs/images/README.md)).
- Tests sans commentaire : le nom du test porte le pourquoi.
  Pas de mock de FFmpeg, ni de couture qui remplace un processus dans le code : images fabriquées ou vraie vidéo.
  Un outil du système se teste par un vrai exécutable factice (script `sh` sur le `PATH` passé au test).
- `tests/fixtures/reference/` tient la référence v0.1.1 en fichiers git ordinaires.
  Elle couvre l'extrait, la 092 et la 092 à 1440 pixels de travail (`video_092_original_width_1440/`, #151).
  Son dossier `settings/` a ses 16 cas à d'autres réglages de détection (`--detect`, docs/11).
  Son `README.md` donne le format, le dossier des 26 vidéos (`BATDETECT_VIDEOS_DIR`) et les versions.
  Elle ne se réenregistre que depuis le tag `python-final` (`tools/reference_v011.py`, qui exige `src/` identique à `v0.1.1`).
- `tests/fixtures/reference-v0.2.0/` tient la référence v0.2.0 du Rust (#250). Elle a les mêmes cas que la v0.1.1, plus ses deux bancs.
  Les 26 vidéos sont hors dépôt (`BATDETECT_REFERENCE_V020_DIR`, facultative pour `rust:data`).
  Son `README.md` donne le format, la provenance et la recette. Elle ne se réenregistre que par `record`, depuis un clone propre.
- `packaging/` : `NOTICE` et `licenses/` décrivent le binaire Rust livré (code MIT, FFmpeg LGPL 2.1, #224).
  `mise run notice` y réécrit la liste des crates.
  `release.yml` publie sur un tag `vX.Y.Z`, derrière les gardes de `release-guards.sh` (docs/05).
- Toutes les coordonnées, distances et surfaces hors de la détection sont en pixels de la vidéo d'origine.
  Seule la détection travaille à `work_width`.

## Rust

Le Python est retiré depuis #199 (dernier état au tag `python-final`).
La chaîne de construction, la charte et les tests sont dans [docs/11](docs/11-migration-rust.md). L'histoire de la réécriture est dans git.

```bash
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:ffmpeg  # FFmpeg 7.1.1 + openh264, une fois par poste
mise run rust:static      # fmt, clippy, règles de la charte, cargo deny, cargo shear
mise run rust:test        # tests rapides, sans couverture (dans check)
mise run rust:coverage    # tests rapides, couverture ≥ 95 % (dans check:ci)
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run check:ci  # rust:coverage, rust:slow, rust:settings : joué par la CI
# git push : le hook pre-push de lefthook lance mise run check (pre-commit : rustfmt seul)
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:slow       # l'extrait LFS, le test de mémoire (hors couverture)
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:settings   # les 16 cas de réglages, sous Linux seulement en CI
/Users/zaratan/Projects/dvr-wt/verrou lourd mise run rust:reference  # la 092 entière
/Users/zaratan/Projects/dvr-wt/verrou lourd cargo run --profile ci -p compare --bin compare -- local  # écarts à la référence v0.2.0, ou v0.1.1 par --reference (extract, 092, 092-1440, settings, local)
/Users/zaratan/Projects/dvr-wt/verrou lourd cargo run --profile ci -p compare --bin compare -- local --frames B --same-as A  # mêmes détections qu'un passage --frames A, hors plages abîmées
/Users/zaratan/Projects/dvr-wt/verrou lourd cargo run --release -p bench -- VIDÉO --against DOSSIER --rejoin-hidden 0 --reach-margin 0  # garde du banc contre un banc de la v0.1.1, sans recollage ni portée
BATDETECT_REFERENCE_DIR=… BATDETECT_VIDEOS_DIR=… mise run rust:data  # référence et 26 vidéos sur une autre machine (seau, sommes dans tests/fixtures/reference/SHA256SUMS)
```

Avant de rendre : `mise run check`.
Le lourd (`check:ci`, `rust:reference`, `local_*`) se lance une seule fois, et seulement si le diff peut changer son résultat.
La CI en joue les morceaux sur chaque PR.
Un temps ou une mémoire publiés passent par `verrou chrono /usr/bin/time -l` au calme (`/Users/zaratan/Projects/dvr-wt/MESURES.md`).
Le hook pre-push de `lefthook.yml` lance `mise run check` (pre-commit : rustfmt seul).
Sans cache FFmpeg pour la branche, `LEFTHOOK=0 git push` le saute.
Ne pas lancer `lefthook run` ni `lefthook install` : ils réécrivent les hooks du dossier git commun à tous les worktrees.

Windows (MSVC, `/MT`, `+crt-static`) ne se construit que dans les tâches CI `rust-windows` et `rust-windows-slow`.
FFmpeg y est construit par `packaging/ffmpeg-windows.sh` sous MSYS2 ([docs/11](docs/11-migration-rust.md#windows)).

Le workspace est virtuel : `members = ["crates/*"]` et la toolchain de `rust-toolchain.toml`.
Il a treize crates depuis #71, aux dépendances à sens unique :

- `types` : vocabulaire commun (`Rational`, `FrameIndex`, `Pts`, `FrameRate`, `Timeline`, `Detection`, `Measures`, `FrameDetections`, `Span`, `EdgeLosses`, `Track`, `Language`, `SourceSize`, `WorkScale`).
  Ne dépend d'aucune autre crate du workspace.
- `media` → `types` : seule frontière avec FFmpeg et seule crate avec `unsafe` (`frame`, `decode`, `encode`, `convert`, `library`).
  `convert` passe du YUV au RGBA et retour, en BT.709. `decode::placed` se place sur une image-clé par `pts`.
  `payload` (feature `testing`) abîme un mp4 pour les tests. `fingerprint`, sous la même feature, est la seule empreinte d'image des tests.
  Elle est stable d'un système et d'une version de Rust à l'autre : les tests figés l'enregistrent.
- `reference` → `types` : chemins des fixtures et garde LFS.
  `location` donne les emplacements hors dépôt, dont `reference_video` pour les vidéos de la référence.
  La crate lit et écrit les références v0.1.1 et v0.2.0 et leurs bancs (`bench`). Elle apparie les détections et les pistes.
- `engine` → `media`, `types` (#116). Il porte la couture du banc (`detect_injected`).
- `tracking` → `types` (#120).
- `files` → `types` (#122). `params.json` marque un dossier terminé.
  `process` l'écrit en dernier (`files::params`, par `.partial`), et le ménage (`files::previous`) l'efface en premier (#66).
  `results` relit un dossier de sortie pour la page : taille des PNG par `picture`, échecs de `.batdetect/echecs/` par `failures` (#71).
- `page` (#71) : la page de résultats d'une soirée en HTML, depuis un modèle sans disque (`model`).
  Les libellés passent par le trait `Labels`, et le CSS et le JS sont embarqués. Elle ne dépend d'aucune crate.
  Les verdicts se saisissent dans la page (#228). `verdicts.csv` est à côté d'elle, lu en octets par `files::verdicts` et embarqué.
  La lecture, le rattachement et le brouillon sont en JS (`assets/csv.js`, `assets/store.js`).
  `mise run page:js` les teste par `node --test`, dans `check`. L'image résumé y est redessinée en SVG sur le fond de la vidéo (`scene`).
  Elle a sa ligne (`caption`) et sa frise en HTML (`strip`). Tout suit la mise en page de `params.json`, jamais recalculée (#259).
- `draw` : dessin pur sur une toile RGBA opaque, avec la police Roboto embarquée, la palette et les tailles.
  Elle ne dépend d'aucune crate du workspace.
- `render` → `media`, `draw`, `types` (#110, #112). `source` choisit par `pts` les images des extraits.
  Sur une plage abîmée, le rendu reprend sur la dernière image-clé saine, avec un seul fil de décodage.
  `windows` fait les fenêtres et les groupes.
  Les boîtes, `#N` et les traces des extraits, des zooms et de la vidéo annotée prennent la couleur de la piste sur l'image résumé (`ink`, `overlay`, `picture`, #13).
  `colored` réunit pistes et couleurs, de longueurs contrôlées à la construction (#287).
  Chaque groupe a son encodeur (`media::encode::fanout`).
  Les groupes tournent en parallèle (`clips`, `schedule`, `group`, `ahead`, `encoders`).
  Les zooms ralentis sont en openh264 (`zoom`, `work`, #103). La vidéo annotée suit les `pts` de la source et se lit par tronçons (#103).
  `drawn` (feature `testing`) rend les images dessinées sans encodeur, pour les tests figés (#198).
- `summary` → `draw`, `types`, `files` (#104) : l'image résumé sans FFmpeg.
  Elle a ses tranches, couleurs, flèches, pastilles, bandeau et frise du temps, et s'écrit en PNG.
  `pages` rend aussi le fond et la géométrie de chaque piste pour la page (#259).
- `compare` → `engine`, `media`, `tracking`, `reference`, `types` (#119) : outil de développement.
  Ses binaires `compare` et `record` ne sont pas livrés. `record` enregistre la référence v0.2.0 depuis un commit propre (#250).
- `bench` → `engine`, `tracking`, `reference`, `media`, `types` (#100) : banc de mesure, binaire `batdetect-bench` (`--targets`, `--replay`, `--against`).
  Sa feature `testing` porte la vidéo bruitée et la cible lente de ses tests (`noisy`).
- `cli` (paquet `batdetect`, binaire `batdetect`) → `engine`, `tracking`, `files`, `summary`, `render`, `draw`, `media`, `page`, `types`, et `compare` pour ses tests (#184).
  Ses modules :
  - `written` : page à la racine de chaque dossier de sortie.
    Elle s'écrit au début du lot puis à chaque fin de vidéo, seulement si elle est à nous (#71).
  - `ledger` : échec d'une vidéo, gardé dans `.batdetect/echecs/` pour la page et effacé à la réussite (#71).
  - `refusals` : entrées refusées du lancement, dans `.batdetect/refus.json` pour la page (#267).
  - `options` : réglages de la commande.
  - `process` : une vidéo de bout en bout. `tracked` en est le suivi, partagé avec les tests figés.
  - `reuse` : vidéo déjà faite, sautée d'après son `params.json` (#70).
  - `inventory` : inventaire au départ (#62).
    Il décide la reprise, ouvre les autres vidéos et refuse celles qui ne s'ouvrent pas, puis annonce la durée.
  - `space` : place à prévoir au lancement et par vidéo, regroupée par volume, sans disque (#61).
  - `checkup` : modèle de `doctor` (#58). Il essaie l'encodeur, lit la place libre par `disk` et vérifie le droit d'écrire.
  - `disk` : place libre et volume d'un dossier, rien sous Windows (#61).
    La place libre se lit par `statfs` sous macOS et par `statvfs` sous les autres Unix.
  - `eta` : heure de fin de la soirée, sans horloge (#68). Les secondes par image des vidéos calculées s'appliquent aux images restantes.
  - `consent` : question avant de traiter les vidéos valides quand des entrées sont refusées (`--skip-refused`).
  - `language`.
  - `listing` : pistes de `params.json`, avec temps, couleur, extraits écrits (#66) et trace (#259).
  - `overview` : image résumé et fond.
  - `sheets` : mise en page des images résumé dans `params.json` (#259).
  - `settings` : réglages de `params.json`.
  - `outcome` : boucle des vidéos, seul `catch_unwind` du code livré (#113).
  - `writable` : sonde d'écriture de chaque dossier de sortie avant tout calcul (#86).
  - `access` : table fermée des raisons d'un chemin refusé, sans texte (#55).
  - `jobs` : vidéos et dossiers de sortie (#55). Toutes les entrées fautives sont refusées ensemble avant tout calcul.
  - `progress` : étapes d'une vidéo annoncées par `process` (#63).
  - `failure` : cause d'un échec, sans texte (#64).
  - `tally` : modèle du bilan de fin de soirée (#65).
  - `flight`, dans les tests seulement : vidéo fabriquée où un carré clair traverse un fond sombre (#112).
    Son fichier est dans `tests/support/` et sert aussi aux tests d'intégration.
  - `sparse`, dans les tests sous Unix seulement : vraie vidéo allongée à 10 To en fichier épars pour les contrôles de place (#61).
    Son fichier est aussi dans `tests/support/`.

  Modules du binaire :
  - `admission` : plan, inventaire, contrôle d'écriture, contrôle de place et question des refus (#61).
  - `status` : vidéo et étape en cours sur la sortie d'erreur, redessinée sur un terminal (#63). Il affiche aussi l'heure de fin (#68).
  - `board` : mise en forme du bilan (#65).
  - `commands` : aiguillage de `doctor` et `completion` avant le parseur (#58, #59).
  - `completion` : script zsh par `clap_complete` (#59).
  - `doctor` : lignes de `batdetect doctor` (#58).
  - `clock` : durée en h, min et s pour le bilan et l'annonce (#62).
  - `terminal` : largeur du terminal par `tcgetwinsize`, rien sous Windows (#68).
  - `notify` : sous macOS dans un terminal, notification par `osascript` et page de résultats ouverte par `open` après le bilan (#69, #71).
    Ce sont les seuls sous-processus du binaire.
  - `sheet` : libellés de la page par Fluent (#71).
  - `style` : couleurs et nom de l'option d'aide (#56).
  - `parsing` : lecture des valeurs d'options, avec une erreur typée qui porte l'option et sa plage (#55).
  - `arguments` : ligne de commande par clap (#123). Il construit l'aide sur deux niveaux et lit les options en une requête.
  - `errors` : erreurs de clap mises en forme dans chaque langue (#114). Il écrit aussi les refus sur la sortie d'erreur.
  - `messages` : lignes du terminal tirées des modèles sans texte de la bibliothèque (#95).
    Ce sont les refus, l'annonce, les étapes, les échecs et la version.
  - `texts` : catalogues Fluent embarqués et `Texts`, qui forme chaque message dans la langue choisie (#114).
    `Texts` porte aussi les libellés de la bibliothèque, du tableau et de l'image résumé (`Wording`, `Headings`, `Labels`).
  - `numbers` : écriture des nombres dans chaque langue (#114).
    Le français prend la virgule décimale et sépare les milliers par U+202F. L'anglais prend le point.

  Les usages courants sont en tête de l'aide et s'affichent sans argument (#56).
  La version est gravée par `build/engraved.rs`, partagé avec `compare` (#295).
  Les textes sont en Fluent dans `locales/{fr,en}.ftl` : langue de l'aide, des erreurs, des lignes et du bandeau (#114).

Charte (23 règles numérotées, lints et raisons dans [docs/11](docs/11-migration-rust.md#charte)) :

- Rust idiomatique, sans imiter ni Python, ni numpy, ni OpenCV. Un écart de détection se mesure par appariement, puis se juge.
- Lints dans `[workspace.lints]` seulement. Zéro `#[allow]` ou `#[expect]`. Un lint gênant se résout en réécrivant le code.
- Erreurs typées par `thiserror`, avec leur cause. Ni `unwrap`, ni `expect`, ni `panic!`, ni `println!` hors des tests. Aucun résultat jeté.
- Aucun type de `ffmpeg_next` hors de `media`. Configurations validées à la construction.
- Un module, ses tests dans `src/<module>/tests.rs`.
  Le niveau d'un test se choisit par le nom de son fichier : `tests/slow_*.rs`, `tests/settings_*.rs`, `tests/reference_*.rs`, `tests/local_*.rs` (les 26 vidéos, jamais en CI), `tests/hardware_*.rs` (VideoToolbox).
- Une dépendance se déclare une fois dans `[workspace.dependencies]`.
- **Commentaires** : la règle du zéro commentaire vaut aussi en Rust, avec deux exceptions.
  Chaque bloc `unsafe` contient une seule opération et porte une ligne `// SAFETY:` qui dit pourquoi le contrat tient.
  Un `///` d'une ligne est permis sur un élément public dont le nom ne dit pas le contrat externe. Au plus 2 %.
