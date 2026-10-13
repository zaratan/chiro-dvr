# Lot 7 : moteur générique, banc, extraits et image résumé en Rust — consignes

Préparé le 12 octobre 2026 (zaratan). Relectures contre le code, en lecture seule, sur `main` 8638592 :
`/Users/zaratan/Projects/dvr-wt/rapports/lot7-relectures/{moteur,banc,extraits,resume}.md`.

| File | Enchaînement | Points |
|---|---|---|
| 1 `moteur` | #144 (3, seule d'abord) → #149 (3) → #150 (5) → #151 (5) → #107 (5) → #108 (5) → #109 (5) | 31 |
| 2 `extraits` | #110 (3) → #112 (8) → #103 (8) | 19 |
| 3 `resume` | #104 (8, petite PR `draw` commune d'abord) → #127 (3) → #157 (densité, à estimer) | 11 + ? |

#144 part seule et se fusionne avant le départ des files 2 et 3 (`types`, `tracking`, `cli/src/process.rs` partagés).
Le banc peut glisser au lot 8 s'il ne tient pas. Chaque issue a sa branche depuis `origin/main` après `fetch`.

## Décisions de l'utilisateur (12 octobre 2026)

Transverses :
1. **Ordre** : #144 d'abord, fusionnée ; puis les trois files. #104 commence par une petite PR des ajouts communs de
   `draw` (`from_rgba`, rectangle plein, `draw::style`), fusionnée vite, que #112 reprend.
2. **Plage pleine** (suite du 11 octobre) : la détection lit Y brut avec la même table quelle que soit l'étiquette
   (`tv`, `pc`, `yuvj420p`) ; la table propre aux sources `pc` de #98 est retirée. À l'affichage, extraits et image
   résumé gardent la plage pleine (contraste 0,86 de la v0.1.1, ciel non écrêté), jugée à l'œil par l'utilisateur sur
   3 pistes faibles (amplitude < 25) avant la fusion de #112, et sur l'image résumé avant celle de #104.
3. **Critères** : on liste les écarts à la v0.1.1, on n'exige pas l'égalité. Zooms au ras d'un seuil (70 pistes sur
   966), temps de légende tirés des `pts` (13 sur 966), réglages non par défaut à 98 % dans les deux sens. Les extraits
   se comparent aux images sources passées par la même conversion, pas à la v0.1.1. L'image résumé se vérifie avec les
   pistes de la référence. Docs : le Rust s'écrit dans docs/11 ; docs/02, 05, 08, README et lire-les-resultats
   changent à #106.
4. **Taille** : tout est gardé (61 points). Points corrigés : #144 3, #98 13 (découpée), #112 8, #109 5, #127 3.

Moteur :
5. #144 élargie à `Span` (constructeurs infaillibles), `Track::with_id`, `engine::damage` ; `cli::process::Error::Exclusion`
   disparaît si `exclude` devient infaillible. 3 points.
6. Médiane sans borne arbitraire : tampons dimensionnés par la configuration (61 échantillons pour 2 s au pas de 1).
7. #98 commence par un banc d'essai : `tools/reference_v011.py` enregistre la v0.1.1 elle-même sur 16 réglages, sur
   l'extrait et sur une vidéo 720×540 tirée de l'extrait (sous LFS, commande ffmpeg écrite dans docs/11, ajoutée à la
   CI) ; `compare` lit la configuration du `video.json` ; critère 98 % dans les deux sens, écarts listés et jugés. Aux
   défauts, rien ne bouge (`slow_extract` au bit près, 963 / 3 / 9 sur les 26 vidéos).
8. #98 découpée en trois sous-issues : #149 banc d'essai, 3 ; #150 géométrie et filtres (taille, réduction, noyau, disque),
   5 ; #151 fenêtre et masques (médiane au-delà de 16, OSD, k = 0, tests portés), 5. La couture d'injection passe dans
   #107.

Banc :
9. Garde de bascule (#109) : liste des cibles perdues et gagnées jugée par l'utilisateur, échec automatique seulement
   sous la borne basse de l'intervalle ; mêmes cibles rejouées des deux côtés, avec la visibilité enregistrée par le
   Python.
10. Injection (#107) : gris visé arrondi vers le bas, puis Y le plus proche par la table (1 niveau du Python sur 100 %
    des 471 046 pixels enregistrés).
11. Générateur aléatoire écrit dans le dépôt (SplitMix64 + xoshiro256**), sans dépendance. Pas de cache : `--targets`
    et `--replay` rejouent des cibles et un suivi enregistrés. `--workers` retiré.
12. Enregistrements du banc non commités : tests `local_`, cas fabriqués en CI. Tableau Rust contre Python dans docs/11.

Extraits :
13. Reprise exacte après une zone abîmée, 1 fil de décodage par groupe ; égalité au pixel avec une lecture séquentielle
    sur la fenêtre 6580–6684 de la 103.
14. Au plus 8 sessions VideoToolbox, openh264 au-delà, attribué par voie. Un encodeur qui casse : la fenêtre est relue
    et refaite en openh264, VideoToolbox écarté pour la suite de la vidéo.
15. Vidéo annotée aux `pts` de la source ; openh264 avec une image-clé par seconde environ, pour toutes les sorties ;
    `--zoom` et `--annotated` entrent dans #103.
16. Corrections d'issues : #103 dépend de #112 ; #102 se ferme avec #112 ; « docs/02, docs/05 » retiré de #102 et #112 ;
    critère de #110 durci ; `fast_run.py:104-108` → `:38`, `:107-114` ; `docs/11:1209-1210` nomme #103.

Image résumé :
17. #127 : frise du temps sous l'image (périodes hors analyse et illisibles) ; plages illisibles datées dans le bandeau
    comme les hors analyse (3 puis « + N autres »). Faite sur le Rust, après #104.
18. PNG par la crate `png` (MIT/Apache, tire `png` et `fdeflate`), sortie RGB ; `draw` reste sans format de fichier.
19. Fond : médiane des 25 images en YUV dans `engine`, puis une conversion BT.709 en plage pleine. Masques OSD : #104
    prépare l'entrée (rectangles), #151 les branche.
20. Libellés gardés : « hors analyse », « illisible ».

## Décisions prises en cours de lot

- #144 (12 octobre) : `tracking::Error::Track` est gardée comme garde-fou à la frontière piste en cours → piste ; seul un
  bogue de l'association ou de la fusion l'atteint. Son nom ou sa doc et docs/11 le disent.
- #144 fin (12 octobre, accepté) : constructeurs de `Span` (`single`, `hull`, `widened`, `intersection`), `unstable_spans`
  infaillible, `Span::frame_count` sans débordement. `engine::detection::Error::Detections` (inatteignable) va à #150.
- #104 plan (12 octobre, validé) : étape 1 réduite à ce dont #104 a besoin (`paste`), sans `from_rgba` ni `draw::style` ;
  fond sans les images gardées qui tombent dans une plage abîmée (toutes gardées s'il n'en reste aucune) ;
  `tools/summary_v011.py` neuf ; résumé après `params.json` ; étape 2 depuis `origin/main` après fusion de l'étape 1.
  Bornes des masques : commentaire sur #151. État de docs/11 confié à #149.
- #149 (12 octobre) : `serde` dans `engine` pour `DetectSettings` et `Region` (`Deserialize`, noms v0.1.1).
- #110 (12 octobre) : `media::encode` gagne un intervalle d'images-clés optionnel (`with_key_interval`), défaut inchangé ;
  le test l'emploie (GOP 10), #112 le règle à une seconde environ pour les sorties.
- #110 fin (12 octobre, accepté) : `Placed` sans `FrameIndex`, technique `mdat` partagée par la feature `testing` de `media`,
  règle « image-clé en erreur → image-clé d'avant » gardée. Empreinte en cinq copies : #154 (Backlog).
- **Affichage (12 octobre, choix de l'utilisateur à l'œil sur les fonds 092, 122, 124)** : la plage pleine laissait un voile
  (ciel à Y = 16 affiché à 10-15). Variante (c) retenue : noir calé sur 16, blancs non écrêtés, Y' = (Y − 16) × 255 / 239.
  Vaut pour l'image résumé (#104) et les extraits (#112). La détection lit toujours Y brut. Remplace le « juger à l'œil »
  de la décision 2.
- #149 (12 octobre) : le cas `max_area_500` ne discriminait pas (plus grande tache de l'extrait 488,25 px²) ; réenregistré à 400.
- #104 (12 octobre, choix de l'utilisateur) : placement des pastilles amélioré dans #104 (zéro recouvrement et zéro hors
  cadre visés, pastille près de sa piste) ; v0.1.1 266 recouvrements sur 8 vidéos, Rust avant 349 (rayon 29 en Roboto 600).
- #112 (12 octobre) : écrit avec `SourceRange::Full`, passe à la variante (c) après la fusion de #104, avant ses mesures.
- #112 plan (12 octobre, validé) : jetons VideoToolbox par groupe, départ dans l'ordre ; trace aux centres décimaux (lissée) ;
  une erreur de `Source` dans un groupe fait échouer les extraits de la vidéo ; CLAUDE.md : la carte des crates ; ordre
  CSV → `params.json` → résumé → extraits → garde ; extraits pour l'œil dans `rapports/planche-112/`.
- #104 fin (12 octobre) : placement 40 anneaux × 16 directions (0 recouvrement), trait de rappel plus fin, rayon 29 à trois
  chiffres, `FullBlackAt16` dans `media::convert`, fond sans les plages abîmées élargies. Temps de l'image résumé à mesurer
  étape par étape avant fusion (092 : 17,7 s et 106 s CPU contre 11,8 s et 101 s à #123 ; temps réel très bruité ce jour).
- #150 plan (12 octobre, validé) : type de mesures sans numéro dans `types` (dégelé depuis #144) ; `exp` standard avec tests
  de marge ; second arrêt pour juger le banc ; 092 à 1440 → #151 ; pas d'optimisation 3:2 sans chiffres.
- #157 (12 octobre, créée avec l'utilisateur) : image résumé coupée au nombre de pistes (au plus N par image, consécutives),
  N choisi à l'œil sur la 097 et la 099 (15, 20, 25) ; après #127 dans la file `resume`. #104 se fusionne tel quel.
- #112 2a (12 octobre, validée à l'œil) : boîte géante d'une image sur la 092 piste 13 (image 57, 3:51), identique en v0.1.1 ;
  l'utilisateur veut borner la boîte dessinée à la taille habituelle de la piste (affichage seulement), dans #112.
- #150 banc (12 octobre, validé) : 12 cas comparés, 76 pistes concordantes ; `sigma_0` sous 98 % dans la bande d'affichage
  colorée seulement (table GRAY contre BGR2GRAY), figé hors de la bande, à revoir à #151 (commentaire) ; petits écarts acceptés ;
  CPU +17,6 % (réduction générique 1,83 ms contre 0,53 ms) → optimisation générique, cible +10 %.
- #112 2b (12 octobre, validée) : boîte dessinée bornée à 4 fois la médiane de chaque côté sur la piste, recentrée (33 boîtes
  sur 33 865, 7 visiblement réduites) ; groupes en parallèle ; reprise en openh264 si VideoToolbox casse. Sous-phase 3
  après commit et mise à jour depuis main par l'utilisateur.
- #127 planche (12 octobre, jugée) : repères de départ gardés ; durée totale illisible gardée sur « + N autres » ; fond des
  bandes distinct (une bande vide ne doit pas ressembler à une barre pleine) ; plages du bandeau réunies quand elles se
  touchent à la seconde (la frise et params.json gardent le détail).
- #127 fin (12 octobre, accepté) : teintes 150/230, bande vide en cadre, graduation 1/2/5/10 min, plages hors analyse aussi
  réunies à la seconde (089 : 1,7 s analysées incluses dans « 0:00 - 0:06 », sens prudent), total sur « + N autres » des
  illisibles seulement. Suggestions de revue cosmétiques jetées.
- #150 fin (12 octobre, accepté) : noyau à 4 096 parts par plus forts restes, disque arrondi moitié vers le haut, gris
  source gardé en entier, optimisations génériques (+7,5 % CPU). Hors périmètre : #159 (vitesse), #160 (doublons de
  `reference`), commentaire #151 (borne de `merge_radius`), retouches (branche morte de `blur_row_by`, message « inf »).
- #157 (12 octobre, choix de l'utilisateur) : N = 20 ; paquets égaux (21 → 11 + 10) ; couleurs attribuées par paquet ; repères
  des autres pistes en gris sur la frise ; noms `_3_sur_8` / `_3_of_8`. Anciennes images : commentaire sur #115.
- #151 plan (12 octobre, validé) : marge des masques ⌈4σ⌉ (retour à 3σ si un cas OSD passe sous 98 %) ; `merge_radius` ≤ 25 px
  source (1 s par image d'éclair) ; tous les refus retirés ; référence v0.1.1 de la 092 à 1440 commitée, testée en CI.
- #112 sous-phase 3 (12 octobre, extraits validés à l'œil) : variante (c), SSIM 0,966-0,968 contre les sources (seuils
  0,963 VT / 0,964 openh264), 092 en 18,9 s, 2,2 Go au plus avec les services VideoToolbox ; correctifs CI (chemin non UTF-8,
  ligne parasite de VideoToolbox).
- Banc (12 octobre, choix de l'utilisateur) : #107, #108, #109 dans une seule branche `rust/100-banc` et une seule PR, plan
  d'ensemble, un arrêt par issue, vérification lourde à la fin seulement.
- Vérification de la tour (12 octobre, remarque de l'utilisateur) : seulement ce que la CI ne voit pas (`compare local` sur les
  26 vidéos, diff), et une remesure au calme d'un temps annoncé quand il compte. Plus de `check`/`slow`/`reference` en double.
- #112 fin (12 octobre, accepté) : sortie mp4 ouverte en FFI dans `media` (5 `unsafe`, chemins non UTF-8), seuils SSIM 0,963/0,964,
  extraits gardés si la vidéo échoue, pas de budget global d'encodeurs. Entrée non UTF-8 : commentaire sur #93. Rendu de
  la 099 (≈ 50 s sur 64,3 s) à mesurer étape par étape avant le commit (demande de l'utilisateur).
- #151 banc (12 octobre, validé) : 16 cas, 93 pistes concordantes ; `window_2s` traité comme `sigma_0` (98 % hors de la bande
  d'affichage) ; `merge_radius` ≤ 12 px source (1 s par image d'éclair à toutes les largeurs) ; tampons de médiane sur la pile
  jusqu'à 64 échantillons (+7 % CPU sinon). 092 à 1440 : 14/14, 28 s contre 277 s en v0.1.1.
- #112 rendu (12 octobre, décision de l'utilisateur) : dans #112 avant le commit, sessions rendues en route (plafond 8, pas de
  départ hors de l'ordre) et conversion par zones (simulé : 099 49,8 → 28,4 s). Limite de sessions du M3 de Manon à mesurer avec #10.
- #112 correctif du rendu (13 octobre) : planche 4 validée à l'œil ; gain mesuré sous charge (099 69,8 → 63,2 s, −9 %) loin du
  simulé (−43 %) ; mesure au calme (charge < 2) demandée avant les revues, avec l'explication de l'écart.
- #151 fin (13 octobre, accepté) : choix de forme acceptés ; fenêtre absurde → commentaire #113 ; blocs masqués → #159.
- Banc plan (13 octobre, validé) : appariement dans `bench::matching`, `reference::bench` pour les fichiers ; garde par
  `local_guard.rs` et `--against DOSSIER` ; `TrackSettings`/`StabilitySettings` dans `tracking`.
- #107 (13 octobre, validé) : égalité de `nearest_luma` alternée (biais +0,001), contraste effectif entier, une graine par cible
  (SplitMix64 depuis `--seed`), constructeurs qui refusent l'absurde ; 986 patchs à un niveau au plus.
- #112 refonte de la nuit (13 octobre, acceptée) : un encodeur VideoToolbox par groupe (`Fanout`), décodage multi-fils hors
  images en erreur (26 vidéos sans images B, vérifié par la tour), pas d'openh264 en appoint ; 099 58,7 → 26,9 s au calme.
- #108 (13 octobre, validé) : test exact (bench.json redonné cible par cible sur les deux bancs) ; run.json, `Injected`,
  rapport tolérant, détections chronométrées, test de bout en bout en 96 × 72. Le test ne distingue pas un rayon de 16 à 24 px.
- #103 plan (13 octobre, validé) : zoom et vidéo annotée en travaux à part, boîte bornée dans le zoom, zoom auto par défaut ;
  zooms en openh264 (règle fixe) et zoom plus petit (accepté pour Manon par l'utilisateur), taille proposée sur la planche.
- Banc fin (13 octobre, accepté) : garde transit 0/0, chasse 2 perdues 1 gagnée sur 584 ; banc Rust 31,6 s contre 243 s ;
  choix de la commande acceptés ; vidéo de test en double → #154, vérifier la vidéo de `--against` → #106.
- #103 (13 octobre, utilisateur) : zooms en pleine taille 1440×1080 (mieux pour les objets diffus) ; la taille réduite est
  abandonnée ; zooms en openh264, mesure (a) openh264 contre (b) tout VideoToolbox.
- **Reliquats pour #112** : regrouper les fenêtres chevauchantes (l'appelant passe l'enveloppe du groupe à `serve`) ;
  accesseur des images-clés dans `engine` (aujourd'hui calculées dans `damage.rs:27-31`) ; fenêtres bornées à la
  dernière image ; test `local` moteur → `Source` en fils automatiques ; `media::encode` écrit en 0600 (relevé par #104).

## Prémisses corrigées (détail dans les relectures)

- Les images de `~/dossier sans titre/video_*` sont les sorties du 8 octobre (anciens noms), pas la v0.1.1 ; même code de
  dessin et de détection. La vraie v0.1.1 : `reference-v0.1.1/outputs/{fr,en}/` (extrait, 092, 097, 125).
- #98 : sur `main`, plus de 16 échantillons est déjà refusé par une erreur typée (`config.rs:105`) ; `threshold`,
  `min_area`, `max_area`, `noise_factor` sont déjà génériques ; figés : `work_width` (3:2), `target_sigma`,
  `merge_radius` (jamais lu), OSD absent, borne de 16. Règle 19 : constructeur à plus de 7 arguments.
- Banc : `reference` apparie à 3 px un à un, le banc à 24 px avec pureté 0,5 et plusieurs pistes par cible ; aucun
  lecteur des fichiers du banc ; `batdetect-rs` n'a aucune option de réglage (#109 en écrit environ 26). #108 a un test
  exact à portée : métriques Rust sur les détections et cibles du Python = `bench.json` cible par cible.
- Extraits : `FrameIndex` est un compteur d'images rendues (faux après un placement) ; `Converter` n'est pas `Send` ;
  au plus 13 fenêtres ouvertes à la fois (096, 099) ; un seul extrait traverse une perte d'images (103, piste 5).
- Image résumé : « la police ne dessine que l'ASCII » est faux ; le fond est déjà gardé par `engine::summary_frames`.

## Consignes

### #144 — Types infaillibles (worktree `moteur`, branche `rust/144-types`)

3 points, sans plan relu. Seule en vol : les files `extraits` et `resume` partent après sa fusion.
- Constructeurs infaillibles dans `types` : `FrameDetections::without(&[Span])` (ou équivalent) qui filtre sans
  revalider ; `Span` (`single`, `hull`, `widened` ou ce que le code demande) ; `Track::with_id` pour la renumérotation.
- Retirer les variantes d'erreur devenues inatteignables : `tracking::Error::{Detections, Span, Track}` selon ce qui
  reste atteignable, `compare::Error::Kept`, `engine::damage::Error`, `cli::process::Error::Exclusion` si `exclude`
  devient infaillible. Une variante qui reste doit pouvoir arriver : dis laquelle et par quel chemin.
- `Track::path_length` ne rend plus `-0.0` pour une piste d'un point ; garder le `+ 0.0` de `files/src/table.rs:57`
  (il sert aussi aux valeurs arrondies à `-0.0`).
- `types/src/scale.rs:52` : `work * (factor * factor)` au lieu de `powi(2)` ; retoucher docs/11:1271-1272.
- Ne touche pas `engine::detection::Error::Detections` (#98 réécrit `detection.rs`).
- Relecture : `lot7-relectures/moteur.md` § #144.
Attendu : `mise run check` vert ; `rust:slow` (dont `slow_extract` au bit près) et `rust:reference` sous le verrou ;
aucune variante d'erreur inatteignable dans `types`, `tracking`, `compare`, `engine::damage`, `cli`. docs/11 à jour
(section `types`, `spans`).

### #110 — Images choisies par pts (worktree `extraits`, branche `rust/110-images`)

3 points, sans plan relu. Part après la fusion de #144. Nouvelle crate `render` (→ `media`, `types` seulement), module
`source` ; `media::decode` gagne un placement sûr (`Input::seek` = `avformat_seek_file`, `flush`, ffmpeg-next 9 :
aucun `unsafe` nouveau).
- Après un placement, ne rends pas de `FrameIndex` depuis le décodeur (c'est un compteur d'images rendues,
  `decode.rs:366-367`) : le numéro vient de la `Timeline` par le `pts` (recherche inverse dans `render::source`, pour ne
  pas toucher `types`).
- Image de reprise : la dernière image-clé ≤ début de fenêtre qui n'est pas elle-même en erreur (répéter tant qu'elle
  l'est), tirée des images-clés et des erreurs du moteur ; pas de marge fixe (celle de 30 du prototype supposait des GOP
  de 15). 1 fil de décodage par groupe de fenêtres chevauchantes, décodé une fois ; le parallélisme viendra des groupes.
- Placement raté (conteneur sans index) : si la première image servie dépasse le début voulu, relire depuis le début.
- Prototype à porter, pas à recopier : `exploration/rust:rust-proto/fast/src/bin/framesrc.rs` ; `fast_run.py:38`,
  `:107-114`, `:124`. Vidéo de test qui perd des images : la technique de `engine/src/testing.rs:43-52` (à partager
  proprement, pas à dupliquer).
- `batdetect-rs` ne change pas : bibliothèque et tests. CLAUDE.md et docs/11 gagnent la crate `render`.
- #150 peut toucher `media::decode` : garde tes changements de `decode.rs` petits et isolés.
- Relecture : `lot7-relectures/extraits.md` § Constats communs et § #110.
Attendu : en CI, sur une vidéo openh264 1440×1080 qui perd des images, chaque fenêtre sert exactement les images
[début..=fin] de la `Timeline`, dans l'ordre, sans manque ni doublon, pixels égaux à une lecture séquentielle à un fil ;
`slow` : fenêtres tirées au hasard dans l'extrait, même égalité ; `local` sous le verrou : 103 fenêtre 6580–6684, et les
fenêtres de la 123 et de la 124 dans des plages abîmées, égalité au pixel. `mise run check` vert.

### #104 — Image résumé (worktree `resume`, branche `rust/104-draw` puis `rust/104-resume`)

8 points. **Plan d'abord** (`rapports/plan-104.md`), relu par `rust-expert` et `tech-architect`, puis arrêt pour
validation. Part après la fusion de #144.
- **Étape 1, PR à part, fusionnée vite** (branche `rust/104-draw`) : les ajouts de `draw` dont #104 et #112 ont tous deux
  besoin (`Canvas::from_rgba`, rectangle plein, ce que le plan identifie), `draw` dans `[workspace.dependencies]`.
  `draw::style` reste tel quel ; ce qui est propre au résumé va dans `summary`. Arrêt après cette étape.
- **Étape 2** (branche `rust/104-resume`, depuis `origin/main` après la fusion de l'étape 1) : nouvelle crate `summary`
  (→ `draw`, `types`, `files` pour `frame_seconds`), un module par rôle (géométrie, couleurs, pastilles, flèches,
  marqueur, légende, tranches, composition, PNG) ; 57 tests Python à porter (`tests/test_summary.py` et voisins),
  sauf `test_header_stays_ascii…` (`:111`), à inverser.
- Fond : médiane des 25 images en YUV dans `engine::summary_frames` (sélection sur un tampon de 25 octets réutilisé ;
  nombre pair : demi-somme arrondie au plus proche, sans imiter numpy), puis une conversion
  `Converter::new(w, h, SourceRange::Full)` BT.709. Aucune image gardée : erreur typée.
- PNG par la crate `png` 0.18.1 (MIT/Apache, tire `fdeflate`), RGB 8 bits ; elle relit l'image dans le test d'aller-retour.
- Tranches de 10 min, durée et temps de légende par les `pts` (`files::time::frame_seconds`, `spans_in_seconds`) ; durée
  « 3 min 22 s » et horloge `m:ss` tronquée à écrire. Découpage testé sur une `Timeline` fabriquée de 27 033 `pts`.
- Masques OSD : `summary` accepte une liste de rectangles, vide tant que #151 n'est pas faite.
- Textes du bandeau : trait `Labels` dans `summary`, implémenté dans `bin/batdetect-rs/messages.rs` (FR/EN, pluriels) ;
  libellés v0.1.1 gardés. `cli/process.rs` : l'étape dans sa propre fonction, appelée par une ligne (#112 et #103 y
  ajoutent les leurs).
- Pas de test SSIM du fond : une mesure unique dans docs/11 (fond Rust converti en plage limitée pour la mesure
  seulement, luminance, hors traits, contre le PNG v0.1.1 de la 092).
- Relecture : `lot7-relectures/resume.md` § #104. Images v0.1.1 : `reference-v0.1.1/outputs/{fr,en}/` ; celles de
  `~/dossier sans titre/video_*` sont du 8 octobre (même code), lecture seule.
Attendu : en CI, `summary` nourri des pistes de la référence (extrait, 092 ; 26 vidéos en local) donne, contre la
v0.1.1, mêmes numéros, même indice de couleur, même tranche, mêmes lignes d'en-tête, écarts de `pts` listés ; la
concaténation en local sous le verrou (2 images, `_000m-010m` et `_010m-016m`, 21 pistes dans la 1re, « + 1 autre ») ;
aucune pastille hors cadre ni recouverte. À l'œil, par l'utilisateur : la 092, la 097, la concaténation et une vidéo
illisible (fond en plage pleine, Roboto, lissage, largeur du bandeau). `mise run check` vert.

### #149 — Banc d'essai à 16 réglages (worktree `moteur`, branche `rust/149-reglages`)

3 points, sans plan relu. Après la fusion de #144. Le texte de l'issue fait foi (décisions du 12 octobre).
- `tools/reference_v011.py` : une option de configuration de détection, écrite dans `video.json` ; il refuse toujours de
  tourner si `src/` diffère du tag. Le Python tourne sous le verrou, un cas par appel.
- La vidéo 720×540 tirée de l'extrait : commande ffmpeg dans docs/11, sous LFS (`.gitattributes`), récupérée par la CI
  (`ci.yml` : tâches qui tirent l'extrait). CLAUDE.md : le paragraphe LFS.
- 16 cas (liste dans l'issue), références en `frames.jsonl.gz` à côté de celles de #94 ; `reference` les relit.
- `DetectSettings` → `DetectConfig` validée (règle 19) ; `compare` lit la configuration du `video.json`. Les cas que le
  moteur ne sait pas encore faire sont refusés par une erreur typée et listés.
- Relecture : `lot7-relectures/moteur.md` § #98 (méthode, niveaux 0 à 3).
Attendu : aux défauts, rien ne bouge (`slow_extract` au bit près, `rust:reference` 14 pistes, `compare local` 963 / 3 /
9) ; 16 références commitées ; par cas faisable, 98 % dans les deux sens et écarts listés. `mise run check` vert.

### #112 — Boîtes, traces et extraits (worktree `extraits`, branche `rust/112-extraits`)

8 points. **Plan d'abord** (`rapports/plan-112.md`), relu par `rust-expert` et `tech-architect`, puis arrêt pour
validation ; puis arrêt après chaque sous-phase. Part après la fusion de #110 (PR #155). Le texte de l'issue fait foi
(décisions du 12 octobre), avec les points suivants.
- **Mesurer d'abord** le coût de `media::convert` par image (`ACCURATE_RND | BITEXACT`, deux copies par sens) : il
  décide de la forme du parallélisme. `Converter` n'est pas `Send` : un par fil (`for_each_init` de rayon) ; dans un
  groupe, `push` en parallèle sur les encodeurs actifs (openh264 ne monte qu'avec des fils).
- **Affichage** : variante (c) choisie par l'utilisateur pour l'image résumé (noir calé sur 16, blancs non écrêtés) ;
  reprendre le réglage que #104 pose dans `media::convert` (ou s'aligner sur lui si #104 n'est pas encore fusionnée :
  demander à la tour).
- **Encodeurs** : au plus 8 sessions VideoToolbox, openh264 au-delà, attribué par voie ; un encodeur qui casse en
  cours d'écriture → la fenêtre est relue et refaite en openh264, VideoToolbox écarté pour la suite de la vidéo, le
  message nomme l'extrait. openh264 avec une image-clé par seconde environ (`with_key_interval`, #110). Les fichiers
  vidéo s'écrivent avec les droits habituels de l'utilisateur (aujourd'hui 0600 : voir comment #104 l'a réglé pour le PNG).
- **Fenêtres** (reliquats de #110) : regrouper les fenêtres chevauchantes (l'appelant passe l'enveloppe du groupe à
  `render::source::serve`) ; bornées à la dernière image ; accesseur des images-clés dans `engine` (aujourd'hui calculées
  dans `damage.rs:27-31`) ; noms d'extraits par `files::time::frame_seconds` (#35).
- **Dessin** : toutes les pistes présentes sur l'image, dans chaque extrait (`overlay.py:76-85`), dessiné une fois par
  image et partagé ; couleurs et `#N` déjà dans `draw` ; constantes propres aux extraits dans un module à part (pas dans
  `draw::style`). Ce que #112 seul demande à `draw` (par exemple `from_rgba`), #112 l'ajoute.
- **Commande** : le dossier des extraits est effacé et recréé avant le rendu (le ménage entre langues reste à #115) ;
  garde `max_tracks` = 300 portée ici ; `RenderConfig` dans `params.json` (`box_pad`, `trail_s`, `clip_margin_s`,
  `max_tracks`, encodeurs employés) ; l'étape dans sa propre fonction de `cli/src/process.rs`. Aucune option nouvelle.
- **Vérifier** : pixels hors dessins par SSIM masqué contre les images sources de mêmes `pts` passées par la même
  conversion (seuil à partir des 0,979 de #111) ; boîte qui suit la cible après un trou (vidéo fabriquée, modèle
  `cli/tests/support/flight.rs`) ; échec VideoToolbox forcé par une couture d'encodeurs ; test `local` moteur → `Source`
  en fils automatiques. Sous le verrou : temps du rendu de la 092, mémoire sur la 125 et sur la 096 ou la 099
  (13 fenêtres).
- **À l'œil, par l'utilisateur** (extraits Rust contre `reference-v0.1.1/outputs`, et `~/dossier sans titre` pour la
  123) : 092 piste 11 (3:29, lente), 092 piste 3 (0:51, faible, en bordure), un extrait dense de la 097, un extrait qui
  traverse des images abîmées (097 ou 123), la 103 extrait 3:40.
- Docs : docs/11 seulement. #102 se ferme avec cette issue.
- Relecture : `lot7-relectures/extraits.md` § Constats communs et § #112.
Attendu : `mise run check` vert ; `rust:slow` et `rust:reference` sous le verrou ; les vérifications ci-dessus.

### #150 — Géométrie et filtres (worktree `moteur`, branche `rust/150-geometrie`)

5 points. **Plan d'abord** (`rapports/plan-150.md`), relu par `rust-expert` et `tech-architect`, puis arrêt pour
validation. Le texte de l'issue fait foi (décisions du 12 octobre), avec ses commentaires (rayon de fusion, erreur
`Detections` inatteignable).
- Toute taille de vidéo, réduction à n'importe quelle `work_width`, noyau depuis `target_sigma`, disque depuis
  `merge_radius` (la constante `HANDLED_MERGE_RADIUS` disparaît), plage : Y brut avec la même table quelle que soit
  l'étiquette (le refus `FullRange` est levé). La variante (c) de l'affichage (#104) ne concerne pas la détection.
- Banc d'essai de #149 : chaque cas que cette issue débloque (`size_720x540`, `width_*`, `sigma_*`, `merge_radius_0`,
  `quick` si possible) passe à 98 % dans les deux sens, écarts listés (`compare settings`) ; aux défauts rien ne bouge
  (`slow_extract` au bit près, 092 à 14 pistes, `compare local` 963 / 3 / 9).
- Invariance de plage : le même flux remuxé avec `-bsf:v h264_metadata=video_full_range_flag=1` redonne les mêmes
  détections au bit près.
- `engine::detection::Error::Detections` (inatteignable, note de #144) : à retirer en réécrivant `detection.rs`.
- Tests rapides : les vidéos fabriquées peuvent être petites maintenant (la couverture ralentit beaucoup les tests
  1440×1080) ; garde le test de mémoire à jour.
- Temps de la 092 aux défauts, en `release` sous le verrou, avant et après (11,2 s à #118).
- Relecture : `lot7-relectures/moteur.md` § #98.
Attendu : `mise run check` vert ; `rust:slow` et `rust:reference` sous le verrou ; `compare settings` et `compare local`.

### #127 — Frise du temps et plages illisibles datées (worktree `resume`, branche `rust/127-frise`)

3 points, sans plan relu. Après #104 (fusionnée, PR #158). Le texte de l'issue fait foi (décisions du 12 octobre).
- **Frise du temps** sous l'image, sur toute sa largeur, dans chaque tranche : une bande « hors analyse » et une bande
  « illisible », de deux teintes du bandeau, graduée en minutes ; elle montre où sont les périodes, de 1 à 87 plages
  (la 121). Les pistes peuvent y avoir un repère de départ si cela reste lisible : propose, ne l'impose pas.
- **Bandeau** : les plages illisibles datées comme les hors analyse, 3 puis « + N autres » ; libellés FR et EN dans
  `messages.rs` (énumération `HeaderLine` de #104).
- Dans `summary` (module à part, par exemple `timeline`) ; temps tirés des `pts` (`files::time`), plages par
  `spans_in_seconds` comme `params.json`.
- #157 (coupe au nombre de pistes) vient juste après dans cette file : garde la frise indépendante du nombre de pistes
  par image (une image coupée garde la frise de sa plage de temps, ou de la tranche entière : propose).
- Planche pour l'œil, sous `rapports/planche-127/` : la 121 (87 plages illisibles), la 122, la 096 et la 092 (aucune).
- Docs : docs/11 ; README et lire-les-resultats changent à #106.
- Relecture : `lot7-relectures/resume.md` § #127.
Attendu : `mise run check` vert ; les vidéos sans période ignorée ne changent pas (mêmes données qu'à #104) ; la
planche jugée par l'utilisateur.

### #157 — Image résumé coupée au nombre de pistes (worktree `resume`, branche `rust/157-densite`)

Sans plan relu si l'agent l'estime à 3 points ou moins ; sinon plan d'abord. Après #127 (fusionnée, PR #161). Le texte
de l'issue fait foi (décisions du 12 octobre, mesures de densité).
- Couper au nombre : au plus N pistes consécutives par départ dans chaque image, numérotation de la vidéo entière, fond
  commun, même mise en page ; bandeau avec la plage de temps du paquet et sa position (« 2 / 8 »).
- La frise de #127 reste celle de la tranche de 10 minutes (décidé à #127) ; les repères de départ de la frise : ceux du
  paquet seulement, ou tous avec ceux du paquet en évidence (propose).
- Noms de fichiers par `files::names`, FR et EN (propose) ; le ménage des anciens noms reste à #115.
- Planche pour l'œil dans `rapports/planche-157/` : la 097 et la 099 à N = 15, 20 et 25, puis arrêt pour que
  l'utilisateur choisisse N. Ensuite N en constante nommée, docs/11, tests, revues.
Attendu : `mise run check` vert ; aucune vidéo de N pistes ou moins ne change (mêmes données qu'à #127) ; aucune pastille
recouverte ni hors cadre (test local de #104).

### #151 — Fenêtre et masques (worktree `moteur`, branche `rust/151-masques`)

5 points. **Plan d'abord** (`rapports/plan-151.md`), relu par `rust-expert` et `tech-architect`, puis arrêt pour
validation ; second arrêt quand le banc d'essai est mesuré, pour que l'utilisateur juge les écarts. Le texte de l'issue
fait foi (décisions du 12 octobre), avec ses commentaires.
- **Médiane sans borne arbitraire** : tampons dimensionnés par la configuration ; test 0-1 exhaustif jusqu'à 20, puis
  proptest contre un tri ; test de mémoire à jour pour 61 échantillons.
- **Zones OSD** : une tache sous le masque n'est jamais vue ; niveau moyen hors du masque ; le masque ne se teste que pour
  les candidats au-dessus du plancher. Bornes : le Python coupe en `[floor(x0·w), ceil(x1·w))`, `PixelBox` inclut ses
  bornes (commentaire de #104). Brancher aussi les masques dans le fond de l'image résumé (`summary` accepte déjà une
  liste de rectangles depuis #104).
- **Réglages** : unifier `engine::settings::DetectSettings` et la copie privée de `cli/src/settings.rs:24`, pour que
  `params.json` écrive `osd_regions` (commentaire de #149) ; borner `merge_radius` à la validation (commentaire de #150).
- Raccourci de calcul pour k = 0 ; les tests restants de `test_detect.py` (OSD, flou nul, seuil sans bruit), à l'échelle
  réelle.
- **Banc d'essai** : les 4 cas encore refusés (`osd`, `width_1440_osd`, `step_1`, `window_2s`) passent à 98 % dans les deux
  sens ; `sigma_0` devrait alors passer en entier (commentaire sur cette issue) : mesure-le et dis-le.
- **092 à 1440 de travail** (reporté de #150) : une référence v0.1.1 de la 092 à `work_width` 1440, enregistrée sous le
  verrou par `tools/reference_v011.py`, comparée par `compare`, et chronométrée en `release` (temps CPU contre `main`).
- Aux réglages par défaut, rien ne bouge (`slow_extract` au bit près, 092 à 14 pistes, `compare local` 963 / 3 / 9).
Attendu : `mise run check` vert ; `rust:slow`, `rust:reference`, `compare settings` et `compare local` sous le verrou.

### #107, #108, #109 — Banc de mesure en Rust, une seule PR (worktree `moteur`, branche `rust/100-banc`)

Décision de l'utilisateur (12 octobre) : les trois issues dans une branche et une PR (`Closes #107`, `#108`, `#109`, et
la parente #100), pour une seule CI et une seule vérification lourde. Après #151. 15 points.
**Plan d'ensemble d'abord** (`rapports/plan-100.md`), relu par `rust-expert` et `tech-architect`, puis arrêt pour
validation. Ensuite **un arrêt à la fin de chaque issue** (choix de l'agent posés à l'utilisateur, vérification légère
de la tour : statique et tests rapides). Vérification lourde (092, 26 vidéos, banc d'essai, banc complet) à la fin
seulement. Les textes des issues font foi (décisions du 12 octobre), avec la relecture `lot7-relectures/banc.md`.
- **#107, fausses chauves-souris** : nouvelle crate `bench`, `synthetic` en module ; la couture d'injection (un crochet
  entre la lecture et la copie du plan Y, qui ne touche pas les images du fond du résumé) ; injection au gris visé
  arrondi vers le bas puis Y le plus proche par la table (1 niveau du Python sur 100 % des pixels enregistrés) ;
  générateur SplitMix64 + xoshiro256** écrit dans le dépôt, sans dépendance ; transit (Bézier) et chasse.
- **#108, collecte, appariement, comptes** : `reference` étendue à l'appariement du banc (24 px, pureté 0,5, plusieurs
  pistes par cible) et aux fichiers du banc (`bats.jsonl.gz`, `injected.jsonl.gz`, `bench.json`) ; test exact : les
  métriques Rust sur les détections et cibles enregistrées par le Python redonnent `bench.json` cible par cible ;
  `ignored_spans()` et `Span::contains` suffisent (pas d'ensemble d'images ignorées dans `tracking`).
- **#109, rapport, commande, garde** : la commande du banc (binaire de `bench`) avec ses réglages (environ 26 options ;
  `--workers` retiré) ; pas de cache : `--targets` et `--replay` rejouent des cibles et un suivi enregistrés ; garde de
  bascule : liste des cibles perdues et gagnées jugée par l'utilisateur, échec automatique seulement sous la borne basse
  de l'intervalle de Wilson, mêmes cibles rejouées des deux côtés avec la visibilité enregistrée par le Python ; tableau
  Rust contre Python dans docs/11 (docs/08 change à #106).
- Enregistrements du banc non commités : tests `local_`, cas fabriqués en CI. Temps d'un passage de banc Rust, en
  `release` sous le verrou, contre les 243 s du Python (docs/08).
Attendu : `mise run check` vert ; aux réglages par défaut rien ne bouge ; le test exact de #108 ; la garde de #109 sur
les enregistrements Python, avec la liste des cibles perdues et gagnées pour l'utilisateur.

### #103 — Zoom, ralenti, vue brute, vidéo annotée (worktree `extraits`, branche `rust/103-zoom`)

8 points. **Plan d'abord** (`rapports/plan-103.md`), relu par `rust-expert` et `tech-architect`, puis arrêt pour
validation ; arrêt à chaque sous-phase, et planche pour l'œil. Le texte de l'issue fait foi (décisions du 12 octobre).
- Deux sorties : `_zoom.mp4` (la « vue brute » et le ralenti sont le mécanisme du zoom) et la vidéo annotée ; options
  `--zoom` et `--annotated` sur `batdetect-rs`, pour que l'utilisateur les essaie.
- Vidéo annotée aux `pts` de la source (en phase avec le CSV après une perte d'images) ; elle dessine sur toutes les
  images : réutiliser la conversion par zones et l'encodeur par groupe de #112 ; image-clé environ une par seconde.
- Critère « mêmes zooms » : les bascules au ras d'un seuil (70 pistes sur 966) sont listées et jugées, pas interdites.
  Seuils de zoom dans `params.json` (`RenderConfig`), test des seuils (note de #101/#103 au lot 6).
- Affichage : variante (c), comme les extraits. Boîte bornée de #112 dans le zoom aussi.
- Planche pour l'œil dans `rapports/planche-103/` : zooms de la 092 (pistes lente et faible), de la 097 dense, et un
  extrait de vidéo annotée de la 092 et de la 103 (à travers une perte d'images).
- Temps et mémoire au calme (critère de `MESURES.md`) de la commande avec `--zoom` et `--annotated` sur la 092 et la 099.
- Vérifications selon la consigne projet : le lourd une seule fois, à la fin.
- Relecture : `lot7-relectures/extraits.md` § #103.
