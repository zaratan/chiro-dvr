# Chemins d'or de batdetect-rs

Cahier de la passe de QA de fin de lot. Chaque ligne vaut par son résultat constaté (fichier présent, nombre, texte lu),
jamais par l'absence d'erreur. Un agent le joue sur le dernier commit de `main`, sans rien corriger : il note OK ou KO
avec le détail, ligne par ligne, dans `rapports/qa-<lot>-suivi.md`.

## 0. Préparation

- Binaire : `cargo build --release --locked -p batdetect --bin batdetect-rs`, sous le verrou de `MESURES.md`, dans le
  worktree de la passe, sur `origin/main` à jour.
- **Ne jamais écrire dans `in/` ni dans `~/dossier sans titre/`** : sans `-o`, les résultats vont à côté des vidéos. Les
  vidéos se copient d'abord dans `dvr-wt/qa/` (hors git), et toutes les commandes s'y lancent :
  - `dvr-wt/qa/nuit-1/` : `video_092_original.mp4` (copie de `in/`, 5 min, 14 pistes attendues) et l'extrait
    `video_092_original_3m24-4m05.mp4` (copie de `tests/fixtures/`) ;
  - `dvr-wt/qa/nuit-2/` : `video_129.mp4` (copie de `~/dossier sans titre/img_0000/`, 3 plages abîmées) ;
  - `dvr-wt/qa/pieges/` : `notes.mp4` (un texte renommé), `tronquee.mp4` (les 20 premiers Mo de l'extrait),
    `Vol.mp4` et `sous/vol.mp4` (deux copies de l'extrait) ;
  - pour les chemins d'argent : `video_124.mp4` et `video_099.mp4` (copies de `~/dossier sans titre/img_0000/`).
- Chaque passage lourd (092 entière, 129) se lance sous le verrou, une commande par appel.
- Langue : `LANG=fr_FR.UTF-8` ou `LANG=C`, toujours explicite.
- Après la passe : `dvr-wt/qa/` se supprime en entier.

## 1. Une soirée, sans `-o`, en français

`batdetect-rs nuit-1 nuit-2`
- Sortie d'erreur : une ligne par étape et par vidéo (`[k/3] … — détection`, `suivi`, `image résumé`, `extraits`),
  puis `— N pistes`.
- Sortie standard : le bilan, un tableau par dossier `nuit-1/batdetect` et `nuit-2/batdetect`, colonnes vidéo, durée,
  pistes, non analysé, calcul.
- `nuit-1/batdetect/video_092_original/` : `video_092_original_pistes.csv` (14 pistes), `video_092_original_resume.png`,
  `params.json` (version, réglages), `extraits/` (14 `piste_NN_XmYYs.mp4`, et les zooms de la règle `auto`).
- La 129 : « non analysé » non nul dans le bilan, et les plages abîmées visibles sur l'image résumé.
- Code de sortie 0.

## 2. La même soirée, relancée en anglais dans le même dossier

`LANG=C batdetect-rs nuit-1 nuit-2`
- Plus aucun nom français dans `batdetect/` (`_tracks.csv`, `_summary.png`, `clips/`, `track_NN_…`) ; aucun fichier
  partiel.
- Bilan en anglais, nombres avec le point.

## 3. Avec `-o`

`batdetect-rs nuit-1 -o ../sortie`
- Rien de nouveau dans `nuit-1/` ; les résultats sont dans `../sortie/video_092_original/` et `…_3m24-4m05/`.
- `batdetect-rs pieges/Vol.mp4 pieges/sous/vol.mp4 -o ../sortie2` : deux dossiers distincts, chacun avec ses extraits
  (homonymes à la casse près, #91).

## 4. Les échecs

`batdetect-rs pieges/notes.mp4 pieges/tronquee.mp4 pieges/absente.mp4 nuit-1/video_092_original_3m24-4m05.mp4`
- Chaque échec : trois lignes au fil de l'eau (phrase, « à faire : », « détail : »), puis `— échec`.
  - `absente` : « fichier introuvable » ;
  - `notes` et `tronquee` : « ce fichier n'est pas une vidéo lisible, ou sa copie est incomplète ».
- L'extrait est traité malgré les échecs.
- Bilan : « 3 vidéos sur 4 en échec », avec phrase et « à faire » sans le détail. Code 1.

## 5. Les refus avant calcul

- Un dossier en lecture seule (`chmod 555` sur une copie de `nuit-1`) : « aucune vidéo n'est traitée, les résultats
  ne peuvent pas s'écrire dans : … », code 2, aucun calcul lancé.
- `--threshold -1`, `--osd-region 0,0,1`, `--tresh 3` : un refus qui nomme l'option, en français, code 2.

## 6. Trop de pistes

`batdetect-rs nuit-1/video_092_original.mp4 --max-tracks 5`
- CSV et image résumé écrits, pas d'extraits ; « à faire : regarder l'image résumé ; si ce sont de vrais passages… ».
- Bilan : section « Trop de pistes », code 1.

## 7. Un tableau annoté

Ajouter une colonne « verdict » à `nuit-1/batdetect/video_092_original_3m24-4m05/…_pistes.csv`, puis relancer sur
l'extrait.
- Le CSV annoté est intact ; la ligne « laissé tel quel … le nouveau tableau n'est pas écrit » apparaît au fil de l'eau
  et dans le bilan ; l'image résumé et les extraits sont refaits.

## 8. L'aide

- `batdetect-rs` seul : les usages courants, code 0, sur la sortie standard.
- `-h` : l'essentiel ; `--help` : tout, avec les réglages avancés sous leur titre, chaque défaut sur sa ligne, la
  majuscule en tête de chaque texte, les fichiers produits.
- Dans un vrai terminal : les couleurs ; avec `NO_COLOR=1` ou redirigée : aucune.
- Les deux langues.
- `--version` : la version tirée de git.

## 9. Les options de rendu

`batdetect-rs nuit-1/video_092_original_3m24-4m05.mp4 -o ../sortie3 --annotated --zoom all`
- La vidéo annotée entière, un zoom par piste en pleine taille ; les extraits gardent les images de la source.

## 10. Un arrêt brutal

Interrompre (Ctrl-C) pendant les extraits de la 092, puis relancer la même commande.
- Après la relance : aucun `.partial` dans `batdetect/`, et les mêmes fichiers qu'en 1.

## 11. Les réglages changés

`batdetect-rs nuit-1/video_092_original_3m24-4m05.mp4 -o ../sortie4 --threshold 11`
- `params.json` porte `threshold` 11 ; le bilan finit par « Réglages changés : --threshold 11 ».

## Chemins d'argent

Des variantes des chemins d'or. Quand l'attendu n'est pas écrit (« à constater »), l'agent note ce qu'il voit, et la
tour classe : comportement voulu, défaut du lot, ou défaut plus ancien.

### A. Les entrées

1. Un dossier vide, puis un dossier qui ne contient que des `.txt` et des `.jpg` : « aucune vidéo trouvée », code 2.
2. Un dossier avec un sous-dossier de vidéos : seul le premier niveau est lu, et le sous-dossier `batdetect/` d'une
   passe précédente n'est jamais repris comme entrée.
3. Des extensions en majuscules (`VIDEO_092.MP4`), un nom avec espaces et accents (`nuit d'été/vidéo 1.mp4`), et le
   même nom en NFD (copié depuis le Finder) : à constater ; les noms de sortie gardent l'accent.
4. La même vidéo donnée deux fois, sous deux écritures (`nuit-1` et `nuit-1/…mp4`, puis `a2` et `./a2/…mp4`) :
   traitée une seule fois, un seul titre dans le bilan (#194, après la QA du lot 8).
5. Des chemins relatifs et absolus mêlés ; `-o` relatif ; `-o` vers un dossier qui n'existe pas (créé) ; `-o` à
   l'intérieur du dossier des vidéos ; `-o` vers un fichier (refusé avant calcul, code 2).

### B. Les vidéos

6. Une vidéo très courte (les 3 premières secondes de l'extrait, coupées sans réencodage sur une image-clé) : code 0,
   « 0 piste », CSV, image résumé et `params.json` écrits (constaté au lot 8). Une coupe hors image-clé est vue abîmée.
7. Une vidéo sans aucune piste (`--threshold 200`) : bilan à « 0 piste », image résumé écrite, pas de dossier
   d'extraits vide (#195).
8. La 124 (65 plages abîmées) : traitée, « non analysé » important dans le bilan, plages visibles sur l'image résumé.
9. Une vidéo chargée (la 099) : plusieurs images résumé par partie (`_1_sur_N`), puis une relance avec
   `--max-tracks 0` : aucune limite, extraits écrits.
10. La 092 avec `--work-width 1440` : 14 pistes, et `params.json` le dit.

### C. Les options

11. `--zoom none`, `--zoom auto`, `--zoom all` sur l'extrait : 0, la règle, puis un zoom par piste.
12. `--clip-margin 0` et `--clip-margin 10` : les durées des extraits suivent.
13. `--osd-region` répétée deux fois : les deux zones dans `params.json`.
14. `--threshold 11.5` accepté, `--threshold 11,5` refusé avec un message qui nomme l'option.
15. Les options avant ou après les vidéos ; `--` avant un nom de vidéo qui commence par un tiret.

### D. La langue et le terminal

16. `LANG` absent : la langue du Mac ; `LANGUAGE=fr` avec `LANG=C` : le français ; `LANG=de_DE.UTF-8` : l'anglais.
17. Sortie standard redirigée dans un fichier, sortie d'erreur au terminal : le bilan dans le fichier, la progression
    à l'écran, sans couleur dans le fichier.
18. `| head -1` sur la sortie standard : le lot s'arrête (décidé à #113), sans panique.
19. `NO_COLOR=1` : aucune couleur ; `CLICOLOR_FORCE=1` redirigé : les couleurs.

### E. Les relances et les pannes

20. Un dossier de sortie de la v0.1.1 (anciens noms `split/`, `.tracks.csv`, `_boxes.mp4`) : retirés par le ménage.
21. Une relance qui donne moins de parties d'image résumé : les parties en trop sont retirées.
22. Ctrl-C pendant la détection (et non pendant les extraits), puis relance : rien de partiel, mêmes fichiers.
23. Disque plein : une image disque de 50 Mo (`hdiutil create -size 50m`) montée comme `-o` : « le disque est plein »,
    avec le dossier ; puis la vidéo suivante est tentée.

## Contrôles transverses

- Chaque commande : son code de sortie (0, 1 ou 2) noté et conforme à l'attendu.
- Le bilan seulement sur la sortie standard ; la progression, les échecs et les refus sur la sortie d'erreur.
- Rien d'écrit hors des dossiers de sortie : `find dvr-wt/qa -newer <repère>` avant et après, et le dossier temporaire du
  système inchangé.
- Le pic de mémoire de la 092 et de la 124 (`/usr/bin/time -l`) : sous 4 Go, loin des 16 Go de la machine de Manon.

## Lignes du lot

Une ligne par issue du label `lot: <n>`, tirée de l'Attendu de sa consigne dans `lot-<n>.md`.
