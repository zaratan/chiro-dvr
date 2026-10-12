# Lot 6 : fondations Rust (#94, #95, V, #116 à #123, #96, #111, #101) et docs de méthode (#72, #75, #76, #78) — consignes

Lot choisi le 10 octobre 2026 : 18 issues, 57 points au board plus V (3) et #95 passée de 5 à 8, soit 63 points. Fin du lot : la
commande Rust `batdetect-rs` sort les pistes de la 092 en CSV, appariées à la référence v0.1.1 ; le moteur (#97) et le
suivi (#99) sont complets. Relectures contre le code : `~/Projects/dvr-wt/rapports/lot6-relectures/` (docs,
construction, moteur, suivi, référence et sorties, architecture, charte Rust).

## Files

Trois worktrees au plus en même temps. Chaque issue a sa branche depuis `origin/main` après `fetch`.

| File | Enchaînement | Fin vers (points) |
|---|---|---|
| 1 `moteur` | #95 (8) → #116 → #117 → #118 → #119 | 21 |
| 2 `reference` puis `suivi` | #94 pendant #95 → V (après #94 et #95) → #120 → #121 → #122 → #123 (attend #119) | 22 |
| 3 `docs` puis `windows`, `sorties` | #76 → #75 → #78 → #72 pendant #95 → #96 → #111 → #101 | 21 |

Première vague : `moteur` (#95), `reference` (#94), `docs` (#76). Les consignes des issues suivantes s'écrivent quand leur
file arrive, sur le code fusionné ; les décisions ci-dessous les fixent déjà.

## Décisions de l'utilisateur (10 octobre 2026)

1. **But : détecter aussi bien que la v0.1.1, ou mieux** — pas l'identité au Python. On apparie (détections, pistes) avec
   tolérances et on juge chaque écart. Pas d'imitation des arrondis Python, de `CAP_PROP_FPS` ni de l'octet du CSV.
2. **Neuf crates** (architecte, révisé) : `types`, `media` (seule avec `unsafe` : décodage, encodage, conversion FFmpeg),
   `reference` (chemins de test, garde LFS, format de la référence, appariement), `engine`, `tracking`, `files`, `draw`,
   `compare` (outil : lance moteur et suivi, liste les écarts), `cli` (paquet `batdetect`, binaire `batdetect-rs`).
   Plus tard : `render`, `summary`, `bench`, `i18n`.
3. **Nouvelle issue V, 3 points** : vocabulaire commun (`types`) et `reference` (format #94, lecture et écriture,
   appariement). Après #94 et #95. Après sa fusion, `types` et `reference` sont gelées pour le lot ; un besoin imprévu
   passe par une petite PR fusionnée d'abord.
4. **Trois files** (tableau ci-dessus).
5. **CI : l'utilisateur pousse à chaque essai.** La tour prépare la commande et lit la CI elle-même.
6. **Lints : `pedantic` + restrictions** (bloc de la charte). Sept lints coupés dans la table du workspace seulement.
7. **Contrôles** : cargo-deny, cargo-shear, règles par grep, proptest ciblé.
8. **Revues** : dès 5 points (#95, #118, #121, #111, #96), plan relu par `rust-expert` et `tech-architect`, validé par
   l'utilisateur avant le code. Sur toute issue Rust : revue finale `rust-expert` et `lead-engineer-reviewer`.
9. **Les écarts de détection se jugent à l'œil, par l'utilisateur** : `compare` liste chaque piste en plus ou en moins
   avec vidéo et temps. Aucun « pire » sans explication.
10. **docs/11** porte tout le Rust (une section par crate et par sujet). docs/10, docs/02, le README et la partie Python de
    CLAUDE.md restent vrais pour le Python livré et sont réécrits à #106.
11. **#94 : tout, données d'abord.** Passe 1 : données de détection (extrait, 092, 26 vidéos). Passe 2 : sorties
    complètes en deux langues et bancs. Critère : relancer redonne les mêmes détections et pistes hors images réparées ;
    les mp4 sont hors comparaison.
12. **Référence des 26 vidéos** : `~/Projects/dvr-wt/reference-v0.1.1/`, variable `BATDETECT_REFERENCE_DIR`. Extrait et
    092 commités sous `tests/fixtures/reference/`.
13. **#101 : planche de rendus** (2 ou 3 polices OFL, extraits lissés ou non, sur une image de la 092) ; l'utilisateur
    choisit.
14. **#111** : le repli de VideoToolbox se choisit sur mesures (logiciel Apple contre openh264 : SSIM et vitesse sur
    l'extrait). openh264 : débit ou QP fixé pour un SSIM ≥ 0,977 (VideoToolbox q65), sans option.
15. **Critères ajoutés** : #118 ≥ 98 % de détections retrouvées dans les deux sens sur l'extrait ; #117 comparé en local
    sur 093, 122 et 124 aux tables de #94 (124 : une image de plus attendue) ; #119 et #96 : détections identiques au bit
    près entre macOS, Linux et Windows (fichier de détections de l'extrait commité), un écart remonte à l'utilisateur.
16. **#75** corrige `docs/10:354-355` (verdict 13 sur 14) ; l'utilisateur relit la section « en simple » avant la fusion.
17. **#123** : `-o` obligatoire. Test de la 092 : Linux en CI, Mac en local par la tour avant Review.
18. **GitHub « Require branches to be up to date before merging »** : à activer (main n'a aucune protection, mesuré).
19. **#96** : 5 essais CI par méthode (FFmpeg compilé → vcpkg → BtbN ; arrêt avant BtbN, lié en dynamique), priorité P2.

**Décisions techniques de la tour** (à contredire si besoin) :
- Images par seconde : cadence moyenne du flux (`avg_frame_rate`, API sûre de ffmpeg-next), repli sur `r_frame_rate` si
  0/0. Mesuré : 092 `avg_frame_rate` = 90110000/3000927 = 30,02738820371172, la valeur de la v0.1.1 ;
  `r_frame_rate` = 10000/333. Les temps viennent des `pts` (#35).
- Profil `ci` : tests avec `overflow-checks` et `debug-assertions`. Les temps se mesurent en `release`.
- CSV de #122 : mêmes colonnes et même sens que la v0.1.1 ; la forme exacte reste à #84.

## Prémisses corrigées par les relectures

- **#94** : le test `slow` fige aussi les 4 pistes de l'extrait (`tests/test_end_to_end.py:200-205`). La liste à
  enregistrer manquait : toutes les images (vides comprises), l'ordre des détections dans l'image, les plages instables
  en numéros d'image, les points de chaque piste, les `pts` et la base de temps, le patch de gris des cibles du banc.
- **#95** : la route `FFMPEG_DIR` de ffmpeg-sys-next ne lie ni openh264 ni la bibliothèque C++ → route pkg-config
  statique. `av_guess_frame_rate` rend `r_frame_rate`, pas la cadence de la v0.1.1, et n'a pas d'enveloppe sûre.
- **#116** : la lecture va dans `media` (FFI), pas dans `engine`. Le décodeur expose aussi l'image complète YUV.
- **#117** : l'extrait et la 092 n'ont aucune image abîmée (0 erreur, 0 saut) : le critère d'origine n'y testait rien.
- **#118** : les tests Python sont à l'échelle 3 et sans filtre ; ils ne se transposent pas tels quels. Le prototype
  vérifie le réseau de médiane jusqu'à 16 échantillons seulement.
- **#119** : seuil « à moins de 3 px » (inégalité stricte), pas « au plus ».
- **#120** : `Span` est créé par V ; l'exclusion reçoit des plages, pas une sonde.
- **#122** : #12 se ferme avec #123 (nombre de fils réel) ; #35 n'y entre que pour le CSV.
- **#123** : le test Python `test_bats_confirmed_by_the_naturalist_are_tracked` compte 3:36 comme une chauve-souris ; c'est
  la piste lente (verdict du 9 octobre, `docs/03:38-47`). Le test Rust ne reprend pas cette affirmation.
- **#76** : #57, #85 et #87 sont fusionnées : le bandeau dit « N pistes », la console française « pistes » et « points= »,
  le CSV s'appelle `<nom>_pistes.csv`. La colonne `hits` reste.
- **#72** : les dix docs font 2 659 lignes (2 587 dans l'issue).
- **#75** : le tableau de docs/10 nomme 16 modules (17 dans l'issue).
- **#101** : les extraits ne sont pas lissés (LINE_8) ; deux polices Hershey ; palette dans `style.py`, en BGR.
- **#96** : son second critère (empreinte) dépend de #119, pas de #97.

## Charte Rust (envoyée avec chaque consigne Rust, vérifiée par la tour avant Review)

But : détecter au moins aussi bien que la v0.1.1, en Rust idiomatique. Ne recopie ni le Python ni le prototype ligne à
ligne.

**Outillage**
1. Toolchain de `rust-toolchain.toml` (1.99.0), édition 2024. `cargo fmt --all --check` ne sort rien.
2. `cargo clippy --workspace --all-targets --locked -- -D warnings` sans avertissement. Les lints se règlent dans
   `[workspace.lints]` du `Cargo.toml` racine, nulle part ailleurs.
3. Zéro `#[allow]`, `#[expect]`, `cfg_attr(…allow…)`, même si clippy le suggère. Un lint gênant se résout en réécrivant
   le code. S'il paraît faux, arrête-toi et écris à la tour.
4. Chaque crate a `[lints] workspace = true`. Seule `media` a sa propre table : la copie exacte de celle du workspace,
   sans `unsafe_code`.

**Pas d'imitation**
5. N'imite ni Python, ni numpy, ni OpenCV : pas de `round_ties_even` pour copier `round()`, pas d'ordre d'opérations
   recopié. Écris le calcul simple et juste. L'écart se mesure par appariement avec tolérances (`reference`), puis se juge.

**Sûreté et erreurs**
6. `unsafe` seulement dans `media`. Un bloc contient une seule opération et porte une ligne `// SAFETY:` qui dit pourquoi
   le contrat tient.
7. Aucun type de `ffmpeg_next` ne sort de `media`. Une fonction `extern "C"` ne peut pas paniquer.
8. Chaque bibliothèque a son `enum` d'erreur (`thiserror`), qui garde la cause (`#[source]`). Ni `anyhow`, ni
   `Box<dyn Error>`, ni erreur `String`. Une erreur par module dans `media` (`decode::Error`, `encode::Error`).
9. Une entrée refusée (vidéo abîmée, format non géré, réglage hors bornes) rend une erreur typée, jamais une panique.
   `unwrap`, `expect`, `panic!`, `todo!` et `dbg!` sont refusés hors des tests.
10. Une panique signale un bug. Seule la commande la rattrape, une fois par vidéo. Les fils passent par
    `std::thread::scope` ou rayon ; jamais de fil détaché.
11. Aucun résultat jeté : ni `let _ =` sur un `Result`, ni `.ok();`.
12. Ni `println!` ni `eprintln!` : `writeln!` sur un flux verrouillé, erreur traitée. Tout texte vu par l'utilisateur vit
    dans un seul module de la commande (Fluent à #114).

**Types et calcul**
13. L'index d'image (ordre de décodage) et le `pts` ont chacun leur type (`FrameIndex`, `Pts`). Pas de sentinelle :
    `Option<Pts>`.
14. Hors de la détection, tout est en pixels de la vidéo d'origine. Seul `WorkScale` passe des pixels de travail aux
    pixels d'origine.
15. Conversions : `From` quand elle existe. Un `as` qui perd de l'information suit un `round` ou un `clamp` visible, ou
    vit dans une fonction de conversion nommée.
16. Même entrée, même sortie, quel que soit le nombre de fils : pas d'itération sur `HashMap`/`HashSet` qui atteint une
    sortie, pas de somme flottante parallèle.
17. Une configuration se construit par une fonction qui valide et rend un `Result`. Les comptes non nuls sont des
    `NonZero*`.
18. Boucles chaudes : aucune allocation par pixel ni par image, tampons réutilisés. Itérateurs, `zip`, `chunks_exact`
    plutôt que des indices.

**Forme et tests**
19. Un module, une responsabilité. Ses tests vont dans `src/<module>/tests.rs` (`#[cfg(test)] mod tests;`). Pas de
    `mod.rs`. Fonctions de 100 lignes et 7 arguments au plus.
20. `pub(crate)` par défaut, `Debug` sur tout type public. Pas de `pub use` dans `media/src/lib.rs`.
21. Zéro commentaire, sauf `// SAFETY:` et une ligne `///` sur un élément public quand le nom ne suffit pas (contrat
    externe). Au plus 2 % des lignes ajoutées.
22. Tests sans commentaire, nommés par le pourquoi. `unwrap`, `expect` et `panic!` y sont permis. Les flottants se
    comparent avec une tolérance. Pas de faux FFmpeg : vraie vidéo, ou vidéo fabriquée par openh264. Couverture d'au
    moins 95 %. Tests lents dans `tests/slow_*.rs` (`#[ignore = "slow"]`), tests de la 092 dans `tests/reference_*.rs`
    (`#[ignore = "reference"]`). proptest là où un oracle simple existe.
23. Une dépendance se déclare une fois dans `[workspace.dependencies]`, puis `x.workspace = true`. Une dépendance hors de
    la liste de l'issue se demande à la tour. `cargo deny check` et `cargo shear` sont verts.

Contrôles de la tour (C1 à C16, détail dans `rapports/lot6-relectures/charte-rust.md`) : toolchain, fmt, clippy, aucun
`allow`/`expect` par grep, table de `media` égale à celle du workspace sans `unsafe_code`, aucun `ffmpeg_next` hors de
`media`, dépendances par le workspace, un seul `catch_unwind`, taux de commentaires ≤ 2 %, aucune imitation
(`round_ties_even|opencv|numpy|python|pairwise`), couverture ≥ 95 %, tests, deny, shear, paquets ajoutés au `Cargo.lock`.

---

## #95 — Construction macOS et Linux

Issue : `gh issue view 95 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/moteur`, branche `rust/95-construction` depuis `origin/main`. 8 points : **plan d'abord**
(`/Users/zaratan/Projects/dvr-wt/rapports/plan-95.md`), relu en parallèle par `rust-expert` et `tech-architect`, puis
arrête-toi : la tour le fait valider par l'utilisateur. Ensuite, arrête-toi à la fin de chaque sous-phase.

Pose le workspace, FFmpeg et l'outillage dont toutes les issues Rust partiront. Décisions prises (elles priment sur la
piste de l'issue) :

- **Workspace virtuel** à la racine, `resolver = "3"`, `members = ["crates/*"]`, `[workspace.package]` (édition 2024,
  `rust-version = "1.99"`, licence MIT, `publish = false`). `[workspace.dependencies]` déclare d'avance : ffmpeg-next
  (codec, format, software-scaling), thiserror, serde (derive), serde_json, rayon, tiny-skia, ab_glyph, clap, dhat,
  tempfile, proptest.
- **Crates créées ici, sans crate vide** : `types` avec `Rational` seul ; `media` (`frame` : `YuvFrame` ; `decode` et
  `encode` minimaux ; signatures `Decoder::open/next` et `Encoder::create/push/finish` extensibles, jamais renommées) ;
  `reference` (chemin d'une fixture depuis son `CARGO_MANIFEST_DIR`, garde contre un pointeur LFS) ; `cli` : paquet
  `batdetect`, binaire `batdetect-rs` (`[[bin]]`), qui ne fait que `--version` avec la ligne de licence de FFmpeg. Les
  autres crates (`engine`, `tracking`, `files`, `draw`, `compare`) naissent avec leurs issues.
- **Fichiers de la charte** (texte dans `/Users/zaratan/Projects/dvr/.claude/tower-control/lot-6.md` et
  `/Users/zaratan/Projects/dvr-wt/rapports/lot6-relectures/charte-rust.md`, section B) : table `[workspace.lints]`,
  `clippy.toml`, `rustfmt.toml`, `rust-toolchain.toml` (1.99.0, clippy, rustfmt, llvm-tools), `deny.toml`. `media`
  recopie la table sans `unsafe_code`. Vérifie chaque ligne avec clippy 1.99 (mesurée en 1.91 par le relecteur).
- **Profils** : `release` (`lto = "fat"`, `codegen-units = 1`, `panic = "unwind"`) ; `ci` (`inherits = "release"`,
  `lto = false`, `codegen-units = 16`, `overflow-checks` et `debug-assertions`). Les temps se mesurent en `release`.
- **FFmpeg 7.1.1 exactement** (archive ffmpeg.org, sha256) et **openh264, dernier tag 2.x** (sha256), construit par son
  Makefile. Configure de lecture repris de `rust-proto/archive/ffmpeg-7.1.1-lgpl/configure.txt` (branche
  `exploration/rust`, `git show`), plus `--enable-libopenh264 --enable-encoder=libopenh264,h264_videotoolbox
  --enable-muxer=mp4 --pkg-config-flags=--static`. `MACOSX_DEPLOYMENT_TARGET=14.0`. Aucun `--enable-gpl` ni
  `--enable-nonfree`.
- **Liaison par pkg-config en statique**, pas `FFMPEG_DIR` : la route `FFMPEG_DIR` de ffmpeg-sys-next 9.0.1 n'émet que
  les `av*` et oublie openh264 et la bibliothèque C++ (`build.rs:1159-1188`). En statique, les dépendances sont dans
  `Libs:` (`Libs.private` est vide).
- **Scripts** : `packaging/ffmpeg-unix.sh` et un fichier commun de versions et de sommes (`packaging/ffmpeg-versions`) ;
  #96 ajoutera le sien pour Windows sans toucher à la clé de cache Unix. Bash portable.
- **Cache local partagé** : `${XDG_CACHE_HOME:-$HOME/.cache}/batdetect/ffmpeg/<empreinte des scripts>-<os>-<arch>/`,
  construit dans un dossier temporaire puis `mv` atomique. Dans chaque worktree, un lien `.ffmpeg` ignoré par git, visé
  par `.cargo/config.toml` (`[env]`, `relative = true`) : `cargo` seul marche. La construction de FFmpeg passe sous
  `lockf /Users/zaratan/Projects/dvr-wt/mesure.lock`.
- **Cible processeur** : rien à poser sur macOS (`apple-m1` est le défaut d'`aarch64-apple-darwin`) ; `x86-64-v2` pour
  `x86_64-unknown-linux-gnu`, Rust seulement.
- **Tests à trois niveaux, choisis par le nom du fichier** : rapides (`cargo test --workspace --locked`) ;
  `tests/slow_*.rs` marqués `#[ignore = "slow"]` (extrait LFS) ; `tests/reference_*.rs` marqués
  `#[ignore = "reference"]` (092 entière). Pose un premier test de chaque niveau : décodage de l'extrait (slow), de la
  092 (reference). Ajouter un test ne doit plus jamais demander de toucher la CI ni `mise.toml`.
- **Tâches mise** : `rust:ffmpeg`, `rust:static` (fmt, clippy, règles C4 à C11, deny, shear), `rust:test` (une seule
  passe de tests sous `cargo llvm-cov --profile ci … --fail-under-lines 95`), `rust:slow`, `rust:reference`. `check`
  ajoute `rust:static`, `rust:test`, `rust:slow` sans retirer les tâches Python. Outils épinglés dans `mise.toml`
  (cargo-llvm-cov, cargo-deny, cargo-shear).
- **CI** : une tâche Rust sur macOS et une sur Linux (clippy sur les deux), cache FFmpeg par `runner.os`, `runner.arch`
  et l'empreinte des scripts. Sur Linux, nasm téléchargé épinglé par sha256 et mis en cache (`docs/05` : plus
  d'`apt-get`). La tâche `reference` Rust tourne sous Linux. Ne touche pas `release.yml`.
- **Essai VideoToolbox sur le runner macOS** : encoder une seconde avec `allow_sw=0`, puis `allow_sw=1`. Résultat écrit
  dans docs/11 : il dit si #19 peut se fermer par la CI.
- **Push** : tu ne pousses pas. Quand tout est vert en local, écris le compte rendu ; la tour fait pousser l'utilisateur,
  lit la CI et te renvoie ce qui casse.
- **Docs** : crée `docs/11-migration-rust.md`, une section par sujet (construction, licences de FFmpeg et d'openh264,
  lints coupés et pourquoi, essai VideoToolbox) et par crate. `CLAUDE.md` : une section Rust (commandes, carte des neuf
  crates, charte, exception `// SAFETY:` à la règle des commentaires). `docs/05` : le paragraphe « une réécriture en Rust
  ne gagnerait presque rien » (l. 5-11) est daté et renvoie à docs/11 ; `rust-toolchain.toml` est la seconde source de
  versions (l. 31-34) ; nombre de tâches CI (l. 61-64) ; décision de l'utilisateur, en passant : l. 42, l'extrait LFS pèse
  60 240 812 octets, pas 5,3 Mo. `README.md` : prérequis de développement (l. 18-30) et ce que fait
  `check` (l. 157). `docs/README.md` : une ligne pour docs/11 (la grappe docs refond ce tableau en parallèle : conflit
  d'une ligne attendu).

**Ne touche pas** à `src/batdetect`, `docs/10`, `docs/02`, `packaging/NOTICE` (#93), `release.yml`. Pas de type partagé
au-delà de `Rational` : ils arrivent par V.

Attendu :
- `mise run check` vert en local (Python inchangé : 559 tests collectés dont 4 lents, `uv run pytest --collect-only -q`) ; contrôles C1 à C16 de la charte verts.
- `cargo test --workspace --locked --test 'slow_*' -- --include-ignored` : l'extrait décode **1 230 images**, cadence
  moyenne **10000/333**, 1440×1080, `yuv420p`, plage `tv`.
- Un encodage openh264 d'une seconde, relu (nombre d'images, taille).
- `batdetect-rs --version` affiche la ligne LGPL de FFmpeg ; `otool -L` sans `libav*` ; `minos` ≤ 14.0 (`vtool -show`).
- Après push par l'utilisateur : CI verte sur macOS et Linux ; essai VideoToolbox consigné.
Mesuré le 10 octobre 2026 par `ffprobe -count_frames` sur `tests/fixtures/video_092_original_3m24-4m05.mp4` (main
dc6968f) : 1 230 images, `r_frame_rate` = `avg_frame_rate` = 10000/333, 1440×1080, `yuv420p`, `tv`, `bt709`. Poste :
stable 1.91.0 et nightly 1.93 installées, 1.99.0 disponible (`rustup check`) ; absents : nasm, meson, ninja,
cargo-llvm-cov, cargo-deny, cargo-shear ; présents : pkgconf 3.0.7, make, clang 21, CLT 27.

---

## #94 — Référence v0.1.1

Issue : `gh issue view 94 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/reference`, branche `rust/94-reference` depuis `origin/main` (Python : la consigne projet
s'applique).

Un script `tools/reference_v011.py` enregistre une fois ce que la v0.1.1 détecte, pour que le Rust s'y compare sans
relancer le Python. Il refuse de tourner si `git diff --quiet v0.1.1 -- src` échoue. Ajoute `tools` à l'`include` de
basedpyright (`pyproject.toml:49`). Décisions prises (elles priment sur l'issue) :

- **Passe 1, données** — pour l'extrait, la 092 et les 26 vidéos de `~/dossier sans titre/img_0000/` :
  - par image, **toutes les images, vides comprises** : numéro, `pts`, image-clé, erreur de décodage vue par ffprobe
    (`-threads 1`, arguments de `probe.py`) ;
  - par vidéo : `CAP_PROP_FPS`, `avg_frame_rate`, `r_frame_rate`, base de temps, nombre d'images, taille, sha256 ;
  - les détections brutes avant exclusion, **dans leur ordre** dans chaque image, flottants écrits par `repr` ;
  - les plages abîmées **et instables** en numéros d'image ;
  - les pistes avec **tous leurs points**, et le CSV ;
  - les versions (Python, numpy, OpenCV, ffprobe).
  Format : JSON Lines, une ligne par image ; gzip pour les 26 vidéos ; pas de pickle. Il sera lu par le Rust (V).
- **Passe 2, sorties et bancs** :
  - sorties complètes en français (`LANG=fr_FR.UTF-8`) puis en anglais (sans locale), encodeur par défaut
    (VideoToolbox), pour l'extrait, la 092, la 097, la 125, et la 089+090+091 concaténées sans réencodage. Vérifie
    d'abord la concaténation par ffprobe : 27 033 images attendues, aucun saut ni erreur aux jonctions ;
  - bancs de la 092, graine 1 : transit par défaut (9 classes × 30) et chasse
    (`--motions hunt circle --amplitudes -60 -30 --per-class 60`). Enregistre les cibles tirées (forme, trajectoire,
    amplitude, **patch de gris**) et les résultats. `bench.json` normalisé : code `v0.1.1`, chemin relatif.
- **Emplacements** : extrait et 092 sous `tests/fixtures/reference/` (données seulement, fichiers git ordinaires, pas de
  vidéo ni de PNG) ; tout le reste sous `~/Projects/dvr-wt/reference-v0.1.1/`. Documentation (format, versions,
  sommes, emplacement, variable `BATDETECT_REFERENCE_DIR`) dans `tests/fixtures/reference/README.md`, **pas dans
  docs/11** (créé en parallèle par #95).
- **N'écris jamais dans `~/dossier sans titre/`** : ses dossiers `video_*` contiennent les sorties du 8 octobre aux anciens
  noms, que la v0.1.1 effacerait.
- **Mesures lourdes** : chaque `batdetect`, `batdetect-bench`, `ffprobe` ou `ffmpeg` sur une vidéo entière passe par
  `lockf /Users/zaratan/Projects/dvr-wt/mesure.lock`, une commande par appel. Durée attendue : environ 55 min pour la
  passe 1 et 50 min pour la passe 2. Environ 10 Go.

**Ne touche pas** à `src/` ni aux tests Python. Pas de `docs/11`.

Attendu :
- Relancer la passe 1 sur l'extrait et la 092 redonne les mêmes détections et pistes ; sur une vidéo à images réparées
  (103), les mêmes hors plages abîmées. Les mp4 sont hors comparaison.
- Pistes enregistrées de la 092 : 14, débuts [222, 1019, 1550, 1614, 2180, 3271, 3688, 4343, 4714, 4923, 6294, 6855, 6933,
  7122], points [20, 40, 7, 34, 35, 32, 17, 13, 10, 44, 290, 17, 24, 27] (`tests/test_reference_092.py:18-19`). Extrait :
  4 pistes, débuts [144, 705, 783, 972], points [290, 17, 24, 27].
- Un tableau par vidéo dans le compte rendu : images, détections, pistes, plages abîmées et instables, temps de calcul.
Mesuré le 10 octobre 2026 : 26 vidéos (`ls`), 11 565 045 370 octets ; tag `v0.1.1` = c9948d2, `src/` et `tests/`
identiques à main ; 318 Go libres ; `~/Projects/dvr-wt/reference-v0.1.1` n'existe pas encore. Les 966 pistes Python sur
les 26 vidéos viennent du prototype (`rust-proto/archive/comparison-26/resultats.md`) : à remesurer ici.

---

## Grappe docs — #76 → #75 → #78 → #72

Worktree : `~/Projects/dvr-wt/docs`. Une branche par issue depuis `origin/main` (ou sur la précédente si elle n'est pas
encore fusionnée). Les docs de **méthode** décrivent ce qui reste vrai en Rust : ni ffprobe, ni `--mode quick`/`normal`,
ni tranches ou `--workers`, ni OpenCV, ni largeurs de travail, ni seuils chiffrés. Repère-toi par titre de section : les
numéros de ligne des issues ont glissé (docs/02 : +5). Ne touche ni `README.md` (#81, #95) ni `CLAUDE.md` (#95).

### #76 — Glossaire (branche `docs/76-glossaire`)

Issue : `gh issue view 76 --repo zaratan/chiro-dvr --comments`.
Crée `docs/lire-les-resultats.md` : titre, introduction (règle de sensibilité et distinction piste / passage), section
« Glossaire ». La section « Ce que l'outil ne voit pas » arrive avec #78, entre l'introduction et le glossaire : ne la
crée pas vide.
Le glossaire a 16 entrées (celles de l'issue moins « mode »), en ordre alphabétique, une à deux phrases chacune, avec un
renvoi à docs/10 pour les notions de méthode. Dis **où** on voit la notion (« sur l'image résumé », « dans le tableau des
pistes », « dans l'extrait »), sans nom de colonne, de dossier ni de ligne de console : ils changent en Rust (#79, #81,
#84, #86). « Garde-fou du nombre de pistes » sans chiffre. « Verdict » : le jugement de Manon, que l'outil n'utilise pas.

Prémisse corrigée : le bandeau dit déjà « N pistes » (`src/batdetect/output/summary.py:52`), la console française
« pistes » et « points= ». N'écris pas que le bandeau dit encore « passages ».

Attendu : `docs/lire-les-resultats.md` existe, 16 entrées, aucun nom de colonne CSV ni de dossier de sortie (grep `hits`,
`extraits/`, `clips/` → rien) ; `mise run check` vert.

### #75 — Les techniques en simple (branche `docs/75-techniques`)

Issue : `gh issue view 75 --repo zaratan/chiro-dvr --comments`.
Section « ## Les techniques en simple » d'environ 40 lignes dans `docs/10-logique-de-detection.md`, entre « L'idée en une
phrase » et le schéma, ton impersonnel comme le reste de docs/10. Mots du glossaire de #76 (« point », pas
« détection »). Renvois par ancre vers les sections de méthode (§2, §4, §7, §8, §9), pas vers §1 ni §5 bis (Python).
« Ce que ça rate » : une phrase, sans numéro d'issue (ils vont dans #78). Un renvoi depuis `docs/lire-les-resultats.md`.

Décision de l'utilisateur : corrige aussi `docs/10:354-355` (« 14 pistes sur la 092, dont deux passages confirmés ») selon
le verdict du 9 octobre : 13 chauves-souris sur 14 pistes, la piste lente de 3:29 n'en est pas une (`docs/03:38-47`).
L'utilisateur relira la section avant la fusion : termine-la, puis arrête-toi.

Attendu : la section existe, aucune occurrence de ffprobe, OpenCV, `quick`, `workers` ni de seuil chiffré dans ses lignes ;
`docs/10:354` dit 13 sur 14 ; `mise run check` vert.
Mesuré le 10 octobre 2026 : docs/10 fait 374 lignes ; son tableau `:22-33` nomme 16 modules.

### #78 — Ce que l'outil ne voit pas (branche `docs/78-limites`)

Issue : `gh issue view 78 --repo zaratan/chiro-dvr --comments`.
Section « Ce que l'outil ne voit pas » dans `docs/lire-les-resultats.md`, tableau à trois colonnes « ce que vous voyez →
ce qui a pu se passer → quoi faire », vouvoiement, l'issue entre parenthèses dans la dernière colonne (#9, #11, #21 à
#24, #26 à #28, toutes ouvertes). « Quoi faire » donne des gestes (revoir la vidéo d'origine à tel moment, regarder
l'extrait zoomé), jamais d'options (`--unstable-pad`, `--noise-factor`, `--target-sigma`, `--mode` disparaissent en Rust).
Pas de ffprobe pour les images illisibles. Conseils de prise de vue : `docs/01:91-94` en entier (calibration NUC
comprise), présentés comme venant de la littérature, non validés sur les jumelles de Manon. La règle de sensibilité
reformulée sans contredire `docs/02:8-11`, avec l'exemple de la piste lente de 3:29.

Attendu : 11 hypothèses de `docs/10:364-374` couvertes ou écartées avec raison dans le compte rendu ; `mise run check`
vert.

### #72 — Sommaire par public (branche `docs/72-sommaire`)

Issue : `gh issue view 72 --repo zaratan/chiro-dvr --comments`.
`docs/README.md` : section « Pour utiliser l'outil » en tête (lien vers `lire-les-resultats.md`), puis le tableau
existant sous « Pour comprendre ou modifier l'outil » avec une colonne « Pour qui » : 01 et 06 les deux, 10 les deux,
02-05 et 07-09 développeur. Colonne « Contenu » inchangée. Pas de lien vers `CONTRIBUTING.md` (#74) ni
`premiere-soiree.md` (#80). Garde le titre « Base de connaissances » (cité par `README.md:184` et `CLAUDE.md`).
Conflit d'une ligne attendu avec #95 (ligne docs/11) : la seconde branche fusionnée l'ajoute au nouveau format.

Attendu : `docs/README.md` a deux sections et une colonne « Pour qui » ; aucun lien mort (vérifie chaque lien) ;
`mise run check` vert.
Mesuré le 10 octobre 2026 : `docs/README.md` 20 lignes ; les dix docs 2 659 lignes (2 587 dans l'issue).

## Décisions prises en cours de lot (10-11 octobre 2026)

- #94 : pas de test pour `tools/reference_v011.py` (validé par ses sorties). Repérages : cache du banc → #100, jumelles
  fusionnées → #124, stabilité et longueur → #120, CLAUDE.md (`tools/`, `tests/fixtures/reference/`) → reliquat de #124.
- #95 : cargo-shear avec liste `ignored` qui se vide seule ; ab_glyph retiré (RUSTSEC-2026-0192), la pile de polices se
  choisit dans le plan de #101 ; essai VideoToolbox bloquant en CI (#19 se ferme par la CI) ; `rust-reference` attend
  les deux systèmes ; cache de `target/` en CI, clé avec l'empreinte de FFmpeg ; lefthook + `cargo fmt` → reliquat de #124.
- #75 : titre « La chaîne de traitement » au-dessus du schéma. #78 : lignes #42 (début et fin) et #25, introduction
  raccourcie ; bandeau → issue #127 ; copie USB → #80. #72 : phrase d'accueil pour la naturaliste.

## Vague 2 (11 octobre 2026) — #124 V, #96, #111

Décision de l'utilisateur : V, #96 et #111 partent ensemble ; un 4e worktree `moteur` démarre #116 à la fusion de V.
#95 est fusionnée (PR #130) : la charte vit dans `docs/11-migration-rust.md` (§ Charte, 23 règles) et la section Rust de
`CLAUDE.md` ; la règle de main exige aussi les tâches Rust.

**Bloc commun aux issues Rust.** La consigne projet vaut pour les vidéos, les mesures et le verrou. Le code suit la
charte de docs/11 et la section Rust de CLAUDE.md ; la tour lance les contrôles C1 à C16 avant Review. Avant de rendre :
`mise run check` vert (il contient `rust:static`, `rust:test`, `rust:slow`) ; `lockf … mise run rust:reference` si
l'issue touche la lecture ou la détection. Plans (≥ 5 points) relus par `rust-expert` et `tech-architect` ; revue finale
`rust-expert` et `lead-engineer-reviewer`. Pas de push : la tour fait pousser l'utilisateur et lit la CI. Les tâches
Python de la CI ne tournent en PR que si `src/`, `tests/`, `tools/`, `pyproject.toml`, `uv.lock`, `mise.toml` ou
`.github/` changent. Relectures : `~/Projects/dvr-wt/rapports/lot6-relectures/`.

### #124 — Vocabulaire commun et lecture de la référence (worktree `suivi`, branche `rust/124-vocabulaire`)

Complète `types` et `reference` : c'est le contrat entre les grappes, gelé après la fusion.
- `types` : `FrameIndex`, `Pts`, `Timeline` (pts de chaque image, base de temps, cadence), `Detection` (champs en `f64`,
  pixels de la vidéo d'origine), `FrameDetections` (une liste par image, images vides comprises, ordre gardé), `Span`
  (bornes comprises), `Track` et ses mesures (corde, longueur du trajet, surface maximale, amplitude de crête, virage
  médian), `Language`, `WorkScale`, `SourceSize`. Un type n'y entre que s'il est produit par une crate et lu par au moins
  deux autres.
- `reference` : lecture et écriture des quatre fichiers de `tests/fixtures/reference/README.md` (format écrit par #94 :
  c'est la spécification), `.gz` compris ; chemin des 26 vidéos par `BATDETECT_REFERENCE_DIR` ; appariement des
  détections (même image, à moins de 3 px, inégalité stricte) et des pistes (début à ±5 images, points à ±3), avec la
  liste des écarts (vidéo, temps) à juger. Sens repris de `rust-proto/archive/comparison-26/analyze.py:47-90` (branche
  `exploration/rust`), pas son code.
- Les points des pistes jumelles fusionnées ne sont pas des détections brutes (63 sur 610 dans la 092) : pas
  d'affirmation « points ⊂ détections ».
- Reliquats décidés par l'utilisateur : deux lignes de `CLAUDE.md` pour `tools/` et `tests/fixtures/reference/` ;
  `cargo fmt` sur les fichiers Rust indexés dans `lefthook.yml` ; la section « Référence v0.1.1 » de docs/11 renvoie au
  README, plus une section par crate pour ce que tu ajoutes.
- Dépendances : `serde`, `serde_json` (et un décompresseur gzip si besoin : à demander à la tour) ; retire de la liste
  `ignored` de cargo-shear ce que tu utilises.
Attendu : extrait et 092 lus (402 et 780 détections brutes, 4 et 14 pistes ; débuts de la 092 [222, 1019, 1550, 1614, 2180,
3271, 3688, 4343, 4714, 4923, 6294, 6855, 6933, 7122]) ; relus puis réécrits, mêmes valeurs ; la référence s'apparie à
elle-même à 100 % ; une détection déplacée de 2,9 px est appariée, de 3,0 px non ; en local, test `#[ignore = "local"]`
avec `BATDETECT_REFERENCE_DIR` : 26 vidéos, 966 pistes. Mesuré le 11 octobre par lecture des fixtures et des 26 dossiers.

### #96 — Construction Windows (worktree `windows`, branche `rust/96-windows`)

8 points, temps limité : **plan d'abord** (`rapports/plan-96.md`), relu, puis arrêt pour validation.
- Script à part, `packaging/ffmpeg-windows.sh` (bash sous MSYS2), qui partage `packaging/ffmpeg-versions` sans changer la
  clé de cache Unix. MSVC : `msys2/setup-msys2` et `ilammy/msvc-dev-cmd` épinglés par SHA ; masquer le `link.exe` de
  coreutils ; noms `.a` contre `.lib` ; FFmpeg et openh264 en `/MT`, Rust en `+crt-static` ; `x86-64-v2` ; liaison par
  pkg-config qui lit `Libs` ; `LIBCLANG_PATH` pour bindgen ; garde du multifil (`w32threads`).
- Tâche CI `windows-latest` en fin de `ci.yml`, LFS en `enableCrossOsArchive` ; `.gitattributes` : `eol=lf` pour
  `tests/fixtures/reference/**`.
- Au plus 5 essais CI par méthode : FFmpeg construit par nous → vcpkg → BtbN. On s'arrête **avant** BtbN (lié en
  dynamique, contre la décision « statique »). Chaque essai est poussé par l'utilisateur : prépare-le et relis-le en
  local avant de le demander.
Attendu : sur le runner Windows, l'extrait décode 1 230 images à 10000/333 ; un encodage openh264 d'une seconde se relit ;
`dumpbin /dependents` sans DLL de FFmpeg ni `vcruntime140.dll`. L'identité des détections avec macOS et Linux viendra
avec #119. Aucun moyen local : la CI est le seul banc d'essai.

### #111 — Encodage dans le processus (worktree `encodage`, branche `rust/111-encodage`)

5 points : **plan d'abord** (`rapports/plan-111.md`), relu, puis arrêt pour validation. Étend `media::encode` (#95).
- VideoToolbox q65 (`global_quality`) ; mesurer sur l'extrait SSIM et vitesse du **logiciel Apple** (`allow_sw=1`) et
  d'**openh264**, et t'arrêter avec les chiffres : l'utilisateur choisit le repli sur Mac. openh264 reste le repli ailleurs.
- openh264 : débit ou QP fixé par mesure pour un SSIM d'au moins 0,977 (VideoToolbox q65, `docs/09:76`), sans option.
- Repli : essai au démarrage, puis à l'ouverture de chaque extrait ; liste ordonnée d'encodeurs injectable (couture de
  test, pas de faux FFmpeg).
- Cadence en rationnel exact (10000/333, et ÷4 pour le ralenti). Nombre d'encodeurs simultanés borné par la mémoire
  mesurée de chaque encodeur en 1440×1080, pas par 6.
- Plage et espace de couleur des sorties : openh264 n'écrit pas la plage (#95). Propose dans le plan ce qu'on écrit
  (plage limitée, bt709 comme la source ?) : décision de l'utilisateur.
Attendu : chiffres mesurés sous le verrou (SSIM, vitesse, mémoire) pour VideoToolbox q65, logiciel Apple et openh264 ;
un extrait de l'extrait encodé et relu par chacun ; fichier partiel jamais visible. Mesuré au lot 4 : VideoToolbox q65 =
SSIM 0,977 (`docs/09:76`), à remesurer avec le moteur Rust.

## Vague 3 (11 octobre 2026) — #120 (suivi), #116 (moteur)

#124 est fusionnée (PR #133) : `types` et `reference` sont gelées pour le lot ; un besoin imprévu passe par une petite
PR fusionnée d'abord. Bloc commun Rust : celui de la vague 2.

### #120 — Exclusion et stabilité (worktree `suivi`, branche `rust/120-exclusion`)

Crée la crate `tracking` (dépend de `types` seulement) : modules `spans` (images couvertes, détections vidées en gardant
les images), `stability` (`StabilityConfig` : `max_blobs` 20, `pad_s` 1,0 ; limite de foule tirée de la médiane du
nombre de détections sur **toutes** les images, vides comprises ; plages instables), `exclusion` (reçoit les détections,
les plages abîmées toutes faites, le nombre d'images signalées et la cadence ; rend les détections gardées et les
plages). Pas de contrôle du nombre d'images ici (garde de #116), pas de sonde ffprobe.
- Python à lire (pas à recopier) : `src/batdetect/spans.py`, `stability.py`, `exclusion.py` et leurs tests
  (`test_spans.py` 3, `test_stability.py` 10 portables — les 3 qui appellent `detect_frames` attendent #118 —,
  `test_exclusion.py` 6 à adapter). Relecture : `lot6-relectures/suivi.md` (#120).
- Commentaire de l'issue : la stabilité dépend de la longueur de la vidéo (médiane sur toute la vidéo) ; la
  concaténation 089-091 ignore 27,9 s contre 28,9 s en morceaux.
Attendu : sur l'extrait et la 092 (fixtures), aucune plage instable et toutes les détections gardées (402 et 780) ; en
local sur les 26 vidéos (`tests/local_*.rs`, `BATDETECT_REFERENCE_DIR`), plages instables et `detections_kept` égales à
la référence pour chaque vidéo — par exemple 093 : 210 gardées, 1 plage / 153 images ; 094 : 541, 8 / 1 773 ; 103 : 1 344,
16 / 6 637 ; 120 : 410, 0 / 0. Tout écart est listé (vidéo, plage) et expliqué, pas corrigé par imitation du Python.
Mesuré par #94 (`reference-v0.1.1/data/*/video.json`, rapport-94.md).

### #116 — Lecture et gardes (worktree `moteur`, branche `rust/116-lecture`)

Étend `media::decode` (#95) et crée la crate `engine` (module `input` : le seul point d'entrée du moteur, qui porte les
refus). 3 points, pas de plan.
- `media::decode` expose par image : `FrameIndex`, `Option<Pts>`, image-clé, drapeaux d'erreur bruts, erreur de
  `send_packet` ; et pour le flux : base de temps, nombre d'images annoncé, cadence moyenne (déjà en place). Réglage du
  nombre de fils (0 = automatique). Couture « paquet → paquet » avant `send_packet`, pour le canari de #117.
- Lecture anticipée dans un fil borné (`std::thread::scope`), comme `src/batdetect/prefetch.py` (4 images) ; aucune
  allocation par image.
- Après un `Eof`, lire l'erreur d'entrée-sortie du contexte : certains démultiplexeurs rendent EOF sur une lecture
  tronquée (commentaire de l'issue).
- `engine::input` refuse : plage pleine (`yuvj420p` ou `pc`, lus comme `Full` par `media` depuis #95), format autre que
  4:2:0, 0 image annoncée, et un nombre d'images décodées différent du nombre attendu (annoncé moins les images perdues
  aux sauts de `pts`, somme de Δpts/pas − 1). Chaque refus est une erreur typée qui nomme la vidéo.
- Python à lire : `video.py`, `prefetch.py`, `damage.py:74-76` ; tests `test_video.py` 7, `test_prefetch.py` 7,
  `test_damage.py:67`, `test_parallel.py:160`. Relecture : `lot6-relectures/moteur.md` (#116).
Attendu : l'extrait et la 092 donnent 1 230 et 9 011 images, cadences égales à `stream.avg_frame_rate` de leur
`video.json` ; en local sur les 26 vidéos (`BATDETECT_REFERENCE_DIR`, `BATDETECT_VIDEOS_DIR`), le nombre d'images
décodées égale `counts.frames` de la référence pour chacune — dont 103 : 8 998, 120 : 8 996, 122 : 8 998 (images perdues
aux sauts) — et la garde ne refuse aucune des 26. `rust:reference` sous le verrou.

## Notes pour la suite (journal)

- **Au dégel de `types`** : donner à `types` des constructeurs infaillibles (filtrer un `FrameDetections` sans le
  recopier ni le revalider ; `Span` déjà valide). Les variantes d'erreur impossibles de `tracking` (`Error::Detections`,
  `Error::Span`) disparaîtront avec. Repéré par #120, validé par l'utilisateur.
- **#118** : reliquat — porter les 3 tests de saut de luminosité de `test_stability.py` qui appellent `detect_frames`.
- **#96** : `rust-windows` à ajouter aux tâches requises de main après fusion.
- **#118 → #119** (11 octobre 2026, accepté par l'utilisateur) : `detect_input` public, `rayon` dépendance d'`engine`, `tracking` en dépendance de test d'`engine` pour `local_tracks.rs` — à déplacer dans `compare` à #119, puis retirer la dépendance de test. Critère des 15 s mesuré en `release` (11,5 s à P2).
- **#118 fin** (12 octobre 2026, accepté par l'utilisateur) : ordre des taches au balayage (pas l'ordre par blocs d'OpenCV), lots de 32 et lecture anticipée de 36, tampons gardés ; seuil des 98 % gardé (l'identité stricte de #119 couvre la régression) ; 098 0:41 perdue et 7 ajoutées : ce sont les 9 divergences du prototype déjà jugées acceptables par Manon le 10 octobre (#97), constaté à #123.
- **#119 fin** (12 octobre 2026, accepté par l'utilisateur) : saut modéré « rien d'allumé, rien d'exclu », sauts en `slow`, `media` dans `compare`, tests de comparaison déplacés dans `compare`, fixture `tests/fixtures/rust/`. Hors périmètre : #144 (types infaillibles, `powi(2)`), commentaire sur #123 (erreurs sans nom de vidéo), chemin de fixture retouché par l'agent.
- **#123 fin** (12 octobre 2026, accepté par l'utilisateur) : `cli` bibliothèque et binaire, messages minimaux et langue par l'environnement (`en:fr` → anglais), écarts de `params.json` listés dans docs/11, code 2 pour les erreurs d'usage. #12 se ferme avec #123 (fils réels) ; la version tirée de git va à #105 (commentaire). Panique non rattrapée : commentaire sur #113. Retouches : test du dossier illisible, tests de commande accélérés. Les 17 écarts de pistes sont les 9 divergences du prototype jugées par Manon le 10 octobre (#97).
- **#146 fin** (12 octobre 2026, accepté) : binaire contrôlé construit avec le workspace, repli du cache en PR seulement, clé calculée dans `ffmpeg-key`, `--no-renames`. Quota de cache (11,6 Go) laissé à l'éviction de GitHub.

## Vague 4 (11 octobre 2026) — #121 (suivi)

### #121 — Suivi (worktree `suivi`, branche `rust/121-suivi`)

5 points : **plan d'abord** (`rapports/plan-121.md`), relu par `rust-expert` et `tech-architect`, puis arrêt pour
validation. Module `tracking::track` (crate créée par #120) ; `Track` et ses mesures viennent de `types` (gelé).
- Appariement image par image, fusion des pistes jumelles, filtres de fin (`min_hits`, `min_travel`, virage médian),
  `TrackConfig` validée à la construction. Python à lire, pas à recopier : `src/batdetect/track.py` (168 lignes) et
  `tests/test_track.py` (27 tests, deux paramétrés) ; relecture `lot6-relectures/suivi.md` (#121).
- But : pistes appariées à la référence avec les tolérances de `reference` (début ±5 images, points ±3), pas
  l'égalité exacte ; pas d'imitation des arrondis Python. Les écarts sont listés (vidéo, temps) et jugés par
  l'utilisateur.
- Entrée de test : les détections brutes de la référence passées par `tracking::exclude` (#120), sans attendre #118.
- Commentaire de l'issue : `Detection::distance` passe par le `hypot` du système (un ulp d'écart avec Python sur 16 %
  des paires), sans effet aux tolérances ; à garder en tête si un seuil tombe sur une distance exacte.
Attendu : extrait et 092 (fixtures) : 4 et 14 pistes appariées à la référence, aucun écart ; en local sur les 26 vidéos :
966 pistes de référence, nombre de paires, pistes seules de chaque côté, chaque écart listé avec vidéo et temps.
Mesuré par #94 (966 pistes) et #124 (appariement de la référence avec elle-même : 100 %).

### #101 — Socle de dessin (worktree `encodage`, branche `rust/101-dessin`)

Crée la crate `draw` (pure : toile RGBA, police embarquée, palette, tailles) et le module `media::convert` (conversion
RGB ↔ YUV 4:2:0 par swscale, **matrice BT.709, plage limitée**, dans les deux sens : décision du 11 octobre, #111).
Trois temps, avec arrêts :
1. **Pile de polices** : `ab_glyph` est exclu (sa dépendance `ttf-parser` est déclarée non maintenue, RUSTSEC-2026-0192 ;
   on n'ignore pas l'avis). Propose une pile maintenue (par exemple `skrifa`, suggérée par l'avis, avec le tracé par
   tiny-skia) et sa licence ; nouvelle dépendance = accord de la tour.
2. **Planche de rendus**, puis arrêt : 2 ou 3 polices libres (OFL) avec chiffres à largeur fixe et glyphes accentués
   précomposés, sur une vraie image de la 092 (une piste de l'extrait), avec les boîtes et traces des extraits **lissées
   et non lissées**, et un en-tête d'image résumé accentué (« hors analyse », « illisible »). PNG dans
   `/Users/zaratan/Projects/dvr-wt/rapports/planche-101/` : l'utilisateur choisit sur l'image.
3. **Code**, après le choix : primitives (rectangle 1-2 px, polyligne 2 px, texte, ligne 1 px lissée, disque et contour,
   polyligne d'épaisseur variable, polygone plein et contour, mesure de texte, assombrissement 0,6) ; palette des 12
   couleurs de `src/batdetect/output/style.py:11-28` stockée en RGB, avec un test qui épingle chacune au BGR inversé du
   Python ; tailles de `style_for` recalées sur la nouvelle police (`text_px` = hauteur des chiffres, taille du `#N`
   figée par un test) ; texte normalisé en NFC, glyphe de repli ; dessin sur fond opaque (alpha prémultiplié de
   tiny-skia).
- Prémisses corrigées : les extraits de la v0.1.1 ne sont pas lissés (LINE_8) ; deux polices Hershey (SIMPLEX pour le
  `#N`, DUPLEX pour le résumé) ; palette dans `style.py`, en BGR. Relecture : `lot6-relectures/reference-sorties.md` (#101).
Attendu : planche validée par l'utilisateur ; aller-retour RGB → YUV → RGB à 2 niveaux au plus sur des aplats alignés
sur des blocs 2×2 ; licence de la police dans `NOTICE` à #93 (noté dans docs/11) ; `mise run check` vert.
- **#117** : type `DecodeErrors` dans `media` au lieu des bits bruts (décision du 11 octobre) ; pertes aux bords
  rendues par #116 → plages abîmées ; règle 16 vérifiée hors plages abîmées.
- **#118** : en Python, le fil lecteur appliquait aussi l'injection et la réduction ; si #118 veut ce parallélisme,
  ajouter une étape de préparation côté lecture (repérage de #116).
- **#123** : unifier `tracking::Error::Padding(f64)` avec la forme `Setting { setting, value }` en écrivant les messages (repérage de #121).

## Vague 5 (11 octobre 2026) — #122 (suivi)

### #122 — CSV, params.json et noms selon la langue (worktree `suivi`, branche `rust/122-tableaux`)

Crée la crate `files` (dépend de `types`) : noms des fichiers produits selon la langue, CSV des pistes, `params.json`
(générique sur `Serialize`), format des temps. Elle reçoit un `types::Language` ; la lecture de la langue du système
viendra avec la commande (#123) puis Fluent (#114).
- **Noms** : les deux tables de `src/batdetect/output/names.py:28-29` (`extraits`/`piste`/`_pistes`/`_resume`/`_annotee`
  et `clips`/`track`/`_tracks`/`_summary`/`_annotated`), `params.json` non traduit, `_zoom` commun ; numéro de piste
  sur la largeur du nombre de pistes (2 chiffres au moins) ; le temps du nom d'extrait est passé par l'appelant (en
  secondes, tiré des `pts` à #102). Sans le ménage ni les anciens noms (#115).
- **CSV** : mêmes 12 colonnes, même sens et même forme que la v0.1.1 (en-têtes anglais, virgule, point décimal, CRLF,
  sans guillemets, même contenu en français et en anglais) ; la forme définitive reste à #84. `max_area_px` entier. Les
  temps viennent des `pts` (`Timeline`, partie CSV de #35), format `m:ss.cc` de `src/batdetect/output/timefmt.py:6-9`.
- **`params.json`** : clés de la v0.1.1 pour les réglages fournis par l'appelant, sans `mode` ni `ffprobe` ; avec la
  version du programme (fixée à la compilation) et la version de FFmpeg liée (#12). Le nombre de fils réel arrive avec
  #123, qui fermera #12.
- Tests unitaires sur des pistes fabriquées ; test sur les fixtures : le CSV écrit depuis les pistes de la référence
  comparé **par les valeurs** au `tracks.csv` de la v0.1.1.
- Python à lire : `output/tables.py`, `output/names.py`, `output/timefmt.py` et leurs tests ; relecture
  `lot6-relectures/suivi.md` (#122).
Attendu : extrait 4 lignes, 092 14 lignes, 12 colonnes ; toutes les colonnes égales à la v0.1.1 sauf les temps, dont
chaque écart est listé (image, temps v0.1.1, temps par les `pts`) — sur l'extrait, au plus l'image 1050 (34,96 contre
34,97, calcul de la relecture, non mesuré) ; sur la 092, des écarts de l'ordre du centième vers la fin sont possibles
(cadence moyenne 30,027 de la v0.1.1 contre les vrais `pts`) : à mesurer, pas à supposer. `mise run check` vert.
- **Au dégel de `types`** (suite) : `Track::path_length` rend `-0.0` pour une piste d'un seul point (somme vide) ;
  `files` l'écrit `0.0` (repérage de #122).
- **#123** : porter le test Python des clés de `params.json` par configuration ; ajouter `files` et `tracking` à
  `[workspace.dependencies]` en les consommant. **#101/#103** : test des seuils de zoom dans `params.json`.

## Vague 6 (11 octobre 2026) — #117 (moteur)

### #117 — Images abîmées et canari (worktree `moteur`, branche `rust/117-abimees`)

3 points, pas de plan. Dans `media::decode` et `engine` (module `damage`).
- **`DecodeErrors`** : un type dans `media` pour les drapeaux d'erreur de décodage, au lieu des bits bruts (décision du
  11 octobre). Une image est en erreur si ses drapeaux ne sont pas vides. Le bit `CONCEALMENT_ACTIVE` dépend d'un cfg de
  ffmpeg-next : le canari tourne sur chaque système (CI macOS, Linux, Windows).
- **`engine::damage`** : images en erreur, images après un saut de `pts` et **pertes aux bords** rendues par #116 →
  plages abîmées (`Vec<Span>`) jusqu'à l'image-clé suivante, élargies d'une marge passée par l'appelant (la demi-fenêtre
  du fond ; `DetectConfig` arrive avec #118). Python à lire : `src/batdetect/damage.py:13-71`, tests `test_damage.py`
  (15, sans l. 67 ni 87) et `test_probe.py` l. 36, 45, 50, 57.
- **Canari** : par la couture de #116 (`Packet::shrink`), corrompre l'extrait à quelques positions — une image-clé, une
  image P au milieu d'un GOP (GOP de 15), une dans le dernier GOP, deux images consécutives — et figer l'ensemble exact
  des images signalées. L'ensemble de ffprobe, mesuré une fois sur une copie tronquée sur disque (arguments de
  `src/batdetect/probe.py:12-25`, version d'ffprobe écrite dans docs/11), est inscrit en constante : celui du Rust doit le
  contenir.
- **Règle 16 hors plages abîmées** : vérifier que les détections (après #118) ou à défaut les tables par image ne
  dépendent pas du nombre de fils hors des plages abîmées.
- Commentaires de l'issue : canari du multifil à l'exécution (lire le nombre et le type de fils du décodeur) ; `linesize`
  négatif non géré par `data()` de ffmpeg-next.
- Docs dans docs/11 (pas docs/10). Relecture : `lot6-relectures/moteur.md` (#117).
Attendu : extrait et 092 : aucune image abîmée, aucune plage (comme la référence). En local (`BATDETECT_REFERENCE_DIR`,
`BATDETECT_VIDEOS_DIR`) : 093, 122 et 124 comparées aux tables de #94 (`ffprobe.error_frames`, `gap_frames`,
`damaged_spans`) — la 124 doit donner une image de plus que ffprobe (310 contre 309), écart attendu du décodeur ; puis
les 26 vidéos, chaque écart listé (vidéo, image) et expliqué. `rust:reference` sous le verrou.
- **Plage des sources** : lues comme pleines pour l'affichage (décision du 11 octobre). Le refus des sources marquées `pc`/`yuvj420p` par `engine::input` (#116) est à réexaminer avec #98 : la détection lit Y brut quelle que soit l'étiquette.
- **124, image 2197 (~1:13)** : exclusion de 1,4 s gardée (image réparée par le décodeur sans message, `CONCEALMENT_ACTIVE` seul) ; l'utilisateur doit confirmer à l'œil.

## Vague 7 (11 octobre 2026) — #118 (moteur)

### #118 — Détection (worktree `moteur`, branche `rust/118-detection`)

5 points : **plan d'abord** (`rapports/plan-118.md`), relu par `rust-expert` et `tech-architect`, arrêt pour validation.
Dans `engine` : préparation (gris à la largeur de travail, filtre à la taille de la cible), fond médian (réseau de tri),
bruit par pixel, seuil relevé, taches (fermeture, composantes), `DetectConfig` validée, pipeline parallèle, 25 images
YUV gardées pour l'image résumé, sortie : vidéo, table par image, `FrameDetections`, plages abîmées (#117).
- Réglages par défaut seulement, 1440×1080 seulement (refus de #116) ; plus de 16 échantillons de médiane refusés
  jusqu'à #98 (le réseau n'est vérifié que jusqu'à 16).
- Pas d'imitation d'OpenCV ni de numpy : le calcul simple et juste, les écarts mesurés par `reference` et jugés.
  Les tests Python sont à l'échelle 3 et sans filtre : écrire les tests Rust à l'échelle réelle (1,5).
- Prototype `rust-proto/fast/` (branche `exploration/rust`) : se porte, ne se recopie pas ; sa réservation
  `with_capacity(above.len() * offsets.len())` (`blobs.rs:70`) est à remplacer par un chemin dense au-delà d'un seuil de
  pixels allumés ; mémoire vérifiée par `dhat` en test (pas de `GlobalAlloc`).
- Règle 16 : mêmes détections quel que soit le nombre de fils, plages abîmées exclues.
- Reliquats : les 3 tests de saut de luminosité de `test_stability.py` qui appellent `detect_frames` ; si le
  parallélisme le demande, une étape de préparation côté lecture (repérage de #116).
- Python à lire : `detect.py`, `median.py`, `noise.py`, `video.py:67-72`, `parallel.py:60-115` et leurs tests ;
  relecture `lot6-relectures/moteur.md` (#118). Docs dans docs/11.
Attendu : extrait et 092 : au moins **98 % des détections appariées dans les deux sens** à la référence
(`reference`, moins de 3 px, même image), hors plages ignorées ; en local sur les 26 vidéos, le taux par vidéo dans les
deux sens et chaque écart groupé (vidéo, temps) à juger. Temps de détection de la 092 en profil `release` sous le verrou
(`/usr/bin/time -l`), à côté du Python (118,7 s de détection, 134 s de commande complète) et du prototype (19 s).
Repères du prototype (non remesurés) : 092 780 contre 781 détections, 100 % et 99,87 % ; la 089 est la plus basse des
26 à 98,19 %.

## Vague 8 (12 octobre 2026) — #119 (moteur)

### #119 — Comparaison et identité entre systèmes (worktree `moteur`, branche `rust/119-comparaison`)

2 points, sans plan relu. Crate `compare` → `engine`, `tracking`, `reference`, `types` : l'outil de développement
(binaire, non livré) qui lance le moteur sur l'extrait, la 092 ou les 26 vidéos (`BATDETECT_REFERENCE_DIR`,
`BATDETECT_VIDEOS_DIR`) et liste les taux dans les deux sens et les écarts groupés (vidéo, temps), détections et pistes.
- Reprendre ce que #118 a posé dans `engine/tests` : `support/agreement.rs`, `local_detection.rs`, `local_tracks.rs`
  passent dans `compare` (code de bibliothèque, tests locaux) ; `tracking` quitte les dépendances de test d'`engine` ;
  docs/11 et l'arête de test de CLAUDE.md suivent.
- Écrire les détections au format de la référence (`frames.jsonl`, `reference::frames::Frames::write`) depuis
  `engine::detection::detect`.
- **Identité entre systèmes** : le `frames.jsonl` de l'extrait, produit sur le Mac, est commité (fichier texte, pas
  LFS) ; un test `slow_*` de `compare` le redemande au bit près. Les tâches `rust` (macOS, Ubuntu) et `rust-windows`
  le lancent déjà. Avant de figer, relève les opérations flottantes de la chaîne dont le résultat dépend de la libm du
  système (`exp`, `ln`, `powf`… ; `sqrt` et les quatre opérations sont exactes) et dis si l'une peut faire basculer
  une détection. Un écart en CI remonte à l'utilisateur : ne pas le corriger seul.
- Seuils inchangés : 98 % dans les deux sens, à moins de 3 px (strict), plages abîmées et instables de la référence
  ignorées dans les deux sens.
- Reliquat : les 3 tests de saut de luminosité de `tests/test_stability.py` (l. 101-125 : saut global, grand saut
  ignoré autour du saut, saut modéré), à l'échelle réelle 1,5 ; ils passent par `engine` puis `tracking`. Le moyen
  de fournir les images (vidéo synthétique encodée par `media`, ou entrée du détecteur) est à toi, dis-le.
- Temps : la 092 est mesurée à #118 (11,2 s en `release`). Ne le refaire que si un fichier d'`engine` change.
- Note de #124 : `reference::detection_matching` alloue à chaque image ; ne la toucher que si le temps de `compare`
  sur les 26 vidéos le demande, chiffre à l'appui.
Attendu : `mise run check` vert, `rust:slow` et `rust:reference` sous le verrou, l'outil lancé sur les 26 vidéos avec
les mêmes chiffres qu'à #118 (963 pistes appariées, 3 perdues, 9 ajoutées ; la 089 à 98,19 %).

## Vague 9 (12 octobre 2026) — #123 (suivi)

### #123 — Commande de bout en bout (worktree `suivi`, branche `rust/123-commande`)

3 points, sans plan relu. Le binaire `batdetect-rs` (crate `cli`, paquet `batdetect`) : vidéos en entrée, un dossier
par vidéo sous `-o` avec le CSV des pistes et `params.json`. Rien d'autre (ni image résumé, ni extraits).
- **`-o` obligatoire** : sans lui, la commande refuse de démarrer et le dit (le défaut se décide avec #86 et #113).
- Chaîne : `engine::detection::detect` → `tracking::exclude` (plages abîmées et instables) → `track_detections` →
  `files` (CSV, `params.json`, noms selon la langue). La langue vient du système (#122 l'a laissée à la commande) ;
  messages minimaux par `messages.rs` : une ligne par vidéo traitée (nom, nombre de pistes), une par échec.
- **Plusieurs vidéos** : un échec n'arrête pas les autres (`tests/test_end_to_end.py:130`), le code de sortie le dit, et
  le message nomme la vidéo (`engine::detection::Error` est `transparent`, commentaire de #119 sur #123).
- **`params.json`** : le nombre de fils réel (ferme #12) ; porter le test Python des clés par configuration.
- Unifier `tracking::Error::Padding(f64)` avec la forme `Setting { setting, value }` (repérage de #121).
- `files` et `engine` passent par `[workspace.dependencies]`, consommés.
- **Tests** : `slow` sur l'extrait sur les trois systèmes (`test_end_to_end.py:189` et `:201` : 4 pistes) ; le test
  Python qui compte 3:36 comme une chauve-souris est périmé (c'est la piste lente de 3:29, `docs/03:38-47`), le Rust
  ne reprend pas l'affirmation. `reference` sur la 092 : Linux en CI ; la tour la relance sur le Mac avant la Review.
- **Comparaison** : les pistes que la commande écrit, appariées à la référence (`reference`, début ±5 images,
  points ±3) sur les 26 vidéos. Attendu : les chiffres de #118 et #119 (966, 963 appariées, 3 perdues, 9 ajoutées).
  Toute autre divergence est listée (vidéo, temps) pour l'utilisateur. Le moyen (lecture du CSV par `compare`, ou
  autre) est à toi, dis-le.
- Temps et mémoire de la commande sur la 092, en `release`, sous le verrou (`/usr/bin/time -l`), à côté des 11,2 s de
  détection seule (#118) et des 134 s de la commande Python.
- Docs : `docs/03` (la 092 avec le Rust), `docs/11`, CLAUDE.md si une arête change.
- Python à lire : `cli.py`, `jobs.py`, `arguments.py`, `output/tables.py` et leurs tests ; relecture
  `lot6-relectures/suivi.md` (#123).
Attendu : `mise run check` vert, `rust:slow` et `rust:reference` sous le verrou, la comparaison des 26 vidéos.

## Vague 10 (12 octobre 2026) — #146 (ci), en parallèle de #123

### #146 — Temps de la CI Rust (worktree `ci`, branche `rust/146-ci`)

Sans plan relu. Les cinq leviers de l'issue, mesures du run 38205056197 (PR #145) comme « avant ».
- `rust-reference` sans `needs: rust`.
- **`memory_flash.rs` devient un test `slow_`** (décision de l'utilisateur) : il tourne sur les trois systèmes sans
  l'instrumentation de la couverture. La couverture doit rester ≥ 95 % sans lui ; docs/11 et CLAUDE.md suivent.
- **Clé de repli du cache `target`** : seule l'empreinte de `Cargo.lock` peut varier. L'image du runner, l'empreinte
  FFmpeg et la toolchain restent dans le préfixe de repli, sinon une bibliothèque FFmpeg périmée serait liée sans
  recompilation ; réordonner la clé en conséquence. Relève la taille des caches (`gh api repos/zaratan/chiro-dvr/actions/caches`)
  et dis si le repli la fait grossir d'une PR à l'autre.
- **Windows** : clippy (82 s) et `cargo build -p batdetect` (60 s). Mesure d'abord sur un run au cache chaud (PR #143,
  run 38181305786) ce qui reste quand rien ne recompile, puis propose ; clippy reste sous Windows (code `cfg(windows)`).
- **Filtre Python** (`changes`) : `tests/fixtures/rust/` et `tests/fixtures/reference/` ne déclenchent plus les
  tâches Python (aucun `.py` ne les lit ; `tools/` reste dans le filtre).
- Ne renomme aucune tâche : les vérifications requises de `main` portent leurs noms (`rust (macos-latest)`,
  `rust (ubuntu-latest)`, `rust-reference`, `rust-windows`, `static`, `fast-tests`, `slow-tests`, `reference`).
- Ne touche ni `crates/cli`, ni `tracking`, ni `files` (#123 y travaille en parallèle).
- Les journaux de CI se lisent par `gh run view <run> --repo zaratan/chiro-dvr --job <id> --log` (lecture seule).
  `actionlint` si l'outil est disponible, sinon dis-le.
Attendu : `mise run check` vert, `rust:slow` sous le verrou (avec le test de mémoire), la liste des tests passés par
système relevée sur le run de #145 pour que la tour la compare à celle de la PR, et le temps attendu par levier. Le
temps « après » se lit sur la CI de la PR que l'utilisateur ouvrira.
