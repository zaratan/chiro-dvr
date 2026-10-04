# Lot 1 : consignes par issue (valeurs de départ mesurées sur `main` 2600887)

Trois worktrees sous `~/Projects/dvr-wt/` : `decodage` (#20, puis #14, puis #1), `reference` (#18),
`detection` (#7 : plan d'abord, code après le merge de #18 ; puis #8).
Rapports et plans : `~/Projects/dvr-wt/rapports/`.
Départ : `mise run check` vert (315 tests), `pytest -m slow` 2 verts, 092 à 14 pistes et 0 s ignorée.

## #20 — Documentation en dérive

Issue : `gh issue view 20 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/decodage`, branche `docs/20-derive` depuis `origin/main`.

Corrige les endroits relevés par l'issue et le titre du premier tableau de `docs/03-resultats-092.md`.
Vérifie chaque valeur dans le code avant de l'écrire. `docs/08` cite déjà `median.py` : rien à y faire.
Les « 12 pistes » de `docs/03-resultats-092.md:9` et de `docs/07` décrivent l'export, ils restent.
**Ne touche à aucun fichier hors de `docs/`.**

Attendu : `grep -n "10 (défaut)\|par cœur" docs/*.md` ne rend plus rien (3 lignes aujourd'hui :
`docs/03-resultats-092.md:113`, `docs/09-profilage.md:126` et `:152`) ; `docs/02-methode.md:18` ne
présente plus la détection comme parallèle par défaut ; `docs/06-questions-ouvertes.md:3` dit
14 pistes ; `mise run check` vert (315 tests au départ).

## #14 — Licence

Issue : `gh issue view 14 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/decodage`, branche `docs/14-licence` depuis `origin/main`.

Décision prise par l'utilisateur : licence MIT, 2026, titulaire « Denis <Zaratan> Pasin » ;
l'accord de la naturaliste pour l'extrait est obtenu. Ajoute `LICENSE`, le champ `license` de
`pyproject.toml` et une ligne dans le README. **Rien d'autre.**

Attendu : `LICENSE` au texte MIT standard ; `uv sync --locked` passe encore ; `mise run check` vert.

## #1 — Erreurs de décodage h264

Issue : `gh issue view 1 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/decodage`, branche `bug/1-decodage` depuis `origin/main`.

Décisions prises par l'utilisateur (en commentaire de l'issue) : repérage par une passe ffmpeg
avant la détection ; détections ignorées de l'image en erreur jusqu'à l'image-clé suivante ; ces
images sortent aussi du calcul de stabilité ; signalement à part des mouvements, dans la sortie et
dans `params.json` ; pas d'option pour désactiver. Issue de 8 points : plan relu puis validé.
**Ne change pas de décodeur pour la détection. Ne cherche pas pourquoi les fichiers sont
abîmés** : l'utilisateur compare une seconde copie depuis les jumelles.

Départ mesuré sur la 122 : 25 pistes, 206 s ignorées sur 300 en 29 plages « unstable », qui
contiennent toutes une image en erreur ; 168 des 195 images en erreur sont dans une plage.

Attendu : 092 inchangée (14 pistes, mêmes débuts, 0 s ignorée) ; sur
`/Users/zaratan/dossier sans titre/img_0000/video_122.mp4`, 195 images en erreur repérées,
puis pistes et secondes ignorées avant et après, en séparant mouvement et erreur de décodage.

## #18 — Test de référence trop lâche

Issue : `gh issue view 18 --repo zaratan/chiro-dvr --comments` (décisions en commentaire).
Worktree : `~/Projects/dvr-wt/reference`, branche `tests/18-reference` depuis `origin/main`.

**Étape 1, mesure seule, sans rien modifier.** Pour l'extrait du dépôt et pour la 092 entière, relève
par piste l'image de début et le nombre de points avec les réglages par défaut, puis avec chaque
réglage dégradé de l'issue : `threshold` 35 et 45, `min_area` 18, `merge_radius` 0, `max_gap` 1,
`min_hits` 10. Écris le tableau des écarts dans `~/Projects/dvr-wt/rapports/tolerances-18.md`, avec
la tolérance la plus large qui fait encore échouer chaque réglage dégradé, et ta recommandation.
Puis arrête-toi : l'utilisateur valide les tolérances. Ne les choisis pas toi-même.

Départ, réglages par défaut. Extrait : 4 pistes, débuts 4,80 s, 23,48 s, 26,21 s, 32,50 s ; 287, 16,
21 et 18 points. 092 entière : 14 pistes, 0 s ignorée, débuts 0:07.39, 0:33.94, 0:51.62, 0:53.75,
1:12.67, 1:49.23, 2:02.82, 2:24.67, 2:36.99, 2:43.95, 3:29.61, 3:48.29, 3:51.02, 3:57.32.

**Étape 2, après validation.** Assertions resserrées sur l'extrait dans `tests/test_end_to_end.py` ;
la 092 entière copiée dans `tests/fixtures/` et suivie par Git LFS (ligne écrite à la main dans
`.gitattributes`), vérifiée sur ses 14 pistes par un test à marqueur dédié ; une tâche de CI séparée
et parallèle, avec `lfs` et cache de la vidéo. C'est l'exception décidée par l'utilisateur à la règle
« aucune vidéo ajoutée au dépôt ». La 089 et la 090 sont hors périmètre.
**Ne touche ni à `src/` ni à un réglage.**

Attendu de l'étape 1 : le tableau pour les 6 réglages dégradés et les 2 vidéos ; `git status --short` vide.

## #7 — Seuil par pixel et filtrage à la taille de la cible : phase de plan

Issue : `gh issue view 7 --repo zaratan/chiro-dvr`.
Worktree : `~/Projects/dvr-wt/detection`, branche `produit/7-seuil-par-pixel` depuis `origin/main`.

Plan seul, **aucune modification du worktree**. B2 et D1 de `docs/07` ; lire aussi `docs/08` et
`docs/09`. Mesurer d'abord le départ : le banc avec les réglages par défaut, et les 14 pistes de la
092. Écrire `~/Projects/dvr-wt/rapports/plan-7.md` en sous-phases mesurables, le faire relire par
`tech-architect` et `lead-engineer-reviewer` en parallèle, puis s'arrêter.

Décisions prises par l'utilisateur : aucune valeur par défaut ne change sans sa validation, tableau
avant et après à l'appui. Une piste nouvelle sur la 092 ne se juge pas dans le code : les verdicts
de la naturaliste manquent (#6). Mémoire : 16 Go au plus. `tests/test_end_to_end.py` est resserré
par #18, mergée avant le code.
**Ne traite ni les extrémités de pistes ni l'hystérésis** (D2, D3, E3) : c'est #8.

Attendu : `plan-7.md` avec le tableau du banc au départ par classe de cible, les 14 pistes de départ,
les avis des deux relecteurs ; `git status --short` vide.

## #8 — Extrémités des pistes

Lancée après le merge de #7 ; consigne écrite à ce moment-là, sur ses résultats.
