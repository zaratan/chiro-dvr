# Lot 1 : consignes par issue (brouillon, avant décisions et mesures de départ)

Grappe `decodage` : #20 puis #1. Grappe `detection` : #18, puis #7, puis #8.
#14 n'est pas envoyée à un agent : décision de l'utilisateur.
Les « Attendu » marqués « à mesurer » le seront sur `main` après le commit, avant lancement.

## #20 — Documentation en dérive

Issue : `gh issue view 20 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/decodage`, branche `docs/20-derive` depuis `origin/main`.

Corrige les cinq endroits relevés par l'issue et le titre du premier tableau de
`docs/03-resultats-092.md`. Vérifie chaque valeur dans le code avant de l'écrire.
**Ne touche à aucun fichier hors de `docs/`.**

Attendu : `grep -n "10 (défaut)\|par cœur\|12 pistes" docs/` ne rend plus rien ;
`docs/08` cite `median.py` ; `mise run check` vert.

## #1 — Erreurs de décodage h264

Issue : `gh issue view 1 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/decodage`, branche `bug/1-decodage` depuis `origin/main`.

Décisions prises : (à remplir après réponse de l'utilisateur : repérage, neutralisation,
signalement).
**Ne change pas de décodeur pour la détection. Ne cherche pas pourquoi les fichiers sont
abîmés** : l'utilisateur compare une seconde copie depuis les jumelles.

Attendu : 092 inchangée (14 pistes, mêmes débuts) ; sur
`/Users/zaratan/dossier sans titre/img_0000/video_122.mp4`, 195 images en erreur repérées,
et nombre de pistes avant et après (avant : à mesurer sur `main`).

## #18 — Test de référence trop lâche

Issue : `gh issue view 18 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/detection`, branche `tests/18-reference` depuis `origin/main`.

Resserre `tests/test_end_to_end.py` : débuts et nombre de points de chaque piste de l'extrait.
Décision prise : tolérances (à remplir). **Ne touche à aucun réglage ni à `src/`.**

Attendu : le test passe sur `main` et échoue pour chacun des cinq réglages dégradés de
l'issue (`threshold` 35 et 45, `min_area` 18, `merge_radius` 0, `max_gap` 1, `min_hits` 10),
tableau à l'appui. Valeurs de départ : à mesurer sur `main`.

## #7 — Seuil par pixel et filtrage à la taille de la cible

Issue : `gh issue view 7 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/detection`, branche `produit/7-seuil-par-pixel`, après le
merge de #18.

B2 et D1 de `docs/07-ameliorer-la-detection.md`. Plan relu puis validé avant de coder.
Décision prise : critère de réussite et valeurs par défaut (à remplir).

Attendu : tableau du banc avant et après par classe de cible ; pistes de la 092 avant et
après, appariées une à une. Chiffres de départ : à mesurer sur `main`.

## #8 — Extrémités des pistes

Lancée après le merge de #7 ; consigne écrite à ce moment-là, sur ses résultats.
