Dépôt unique : issues, board et code dans `zaratan/chiro-dvr`. Tu travailles dans ton
worktree, sous `~/Projects/dvr-wt/`. Ton compte rendu s'écrit dans
`/Users/zaratan/Projects/dvr-wt/rapports/rapport-<N>.md`, jamais dans le worktree.

**Vérifications avant de rendre**

- `mise run check` vert (lint, format, types, tests).
- Si tu touches à `parallel.py`, `video.py`, `detect.py`, `track.py`, `stability.py` ou
  `median.py` : `uv run pytest -m slow`, puis la référence
  `uv run batdetect /Users/zaratan/Projects/dvr/in/video_092_original.mp4 -o out/ref`,
  avec le nombre de pistes et les temps de début, avant et après. La consigne de l'issue
  donne les chiffres de départ.
- Tout changement de détection ou de suivi se mesure aussi au banc, avant et après :
  `uv run batdetect-bench /Users/zaratan/Projects/dvr/in/video_092_original.mp4`
  (docs/08-banc-de-mesure.md).

**Vidéos : lecture seule**

- `/Users/zaratan/Projects/dvr/in/` et `/Users/zaratan/dossier sans titre/img_0000/`
  contiennent les vidéos de l'utilisateur : ne rien y écrire, supprimer ni déplacer.
  Toute sortie va dans `out/` de ton worktree (hors git), par `-o`.
- `tests/fixtures/video_092_original_3m24-4m05.mp4` ne se réencode pas et ne se remplace pas.
- Aucune vidéo ajoutée au dépôt : il est public.

**Règles du dépôt** (détail dans `CLAUDE.md`)

- Zéro commentaire, zéro `noqa`, zéro `type: ignore`. Code en anglais, docs en français.
- Un module, une responsabilité, son fichier de tests. Pas de mock d'OpenCV ni de subprocess.
- Un changement de comportement met à jour `docs/` dans la même issue ; si les 14 pistes
  de la 092 changent, `docs/03-resultats-092.md` aussi.
- Ce qui est ou n'est pas une chauve-souris ne se tranche pas dans le code : c'est une
  question pour la naturaliste, à faire remonter.
- La qualité de détection passe avant le temps de calcul ; 16 Go de mémoire est la limite.

**Issues de 8 points** : avant de coder, écris ton plan dans
`/Users/zaratan/Projects/dvr-wt/rapports/plan-<N>.md`, fais-le relire par `tech-architect`
et `lead-engineer-reviewer` en parallèle, puis arrête-toi : la tour le fait valider.
Ensuite, arrête-toi à la fin de chaque sous-phase du plan avec ses chiffres.
