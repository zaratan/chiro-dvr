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
