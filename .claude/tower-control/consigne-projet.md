Dépôt unique : issues, board et code dans `zaratan/chiro-dvr`. Tu travailles dans ton
worktree, sous `~/Projects/dvr-wt/`. Ton compte rendu s'écrit dans
`/Users/zaratan/Projects/dvr-wt/rapports/rapport-<N>.md`, jamais dans le worktree.

**Vérifications avant de rendre**

- **Rust : le lourd une seule fois, à la fin, et seulement ce que le changement peut toucher.** La CI de la PR relance
  `static`, `test`, `slow` et `reference` sur trois systèmes ; ne fais pas une seconde CI.
  - En travaillant et à chaque arrêt de sous-phase : `mise run rust:static` et les tests de la crate touchée
    (`cargo test -p <crate> --profile ci`). Rien de lourd, sauf si l'arrêt sert justement à mesurer (banc d'essai,
    extraits ou images pour l'œil, temps).
  - Une fois, avant le compte rendu : `mise run check`, puis seulement les commandes lourdes que le diff peut changer :
    `rust:reference` et `compare local` si la détection ou le suivi aux réglages par défaut peuvent bouger, `compare
    settings` si les réglages du moteur bougent, le test `local_` de la crate touchée.
  - Après une retouche de revue qui ne touche que docs, tests ou noms : `rust:static` et les tests de la crate, pas de
    nouveau passage lourd.
- `mise run check` vert (statique, tests, couverture, `slow`, `settings`).
**Mesures : lourd en partagé, chronométré en exclusif**

- Plusieurs agents mesurent sur la même machine. Toute commande lourde (`batdetect-rs`, `compare` ou un banc sur une
  vidéo entière, `ffprobe`/`ffmpeg` sur une vidéo entière, tests `slow_*`, `settings_*`, `reference_*`, `local_*`,
  `mise run check`, compilation froide) passe par `/Users/zaratan/Projects/dvr-wt/verrou lourd <commande>` : les
  lourds tournent ensemble. Tout temps ou pic de mémoire qui sera publié passe par
  `/Users/zaratan/Projects/dvr-wt/verrou chrono /usr/bin/time -l <commande>` : il attend que plus rien de lourd ne
  tourne et bloque les lourds pendant sa durée. Une commande par appel. Un téléchargement ou un envoi réseau ne prend
  aucun verrou. Règle complète et critère de calme dans `/Users/zaratan/Projects/dvr-wt/MESURES.md`. Les séries durent
  plus longtemps : n'écourte rien pour compenser.
- Avant chaque traitement d'une vidéo entière, vérifie qu'il reste au moins 20 Go libres
  (`/bin/df -g .` : `df` est un alias de `duf` sur ce poste) ; au-delà de 200 pistes, supprime les extraits produits et garde le CSV.
- Un réglage bon sur la 092 et au banc n'est pas un réglage : avant de le proposer, lance-le
  sur toutes les vidéos de `in/` sans défaut connu et donne le nombre de pistes de chacune.
- Un verdict à l'œil se consigne avec le temps de la piste (mm:ss.ii), jamais son numéro, qui
  change à chaque réglage.
- Les temps réels sont invalides si la machine s'est mise en veille : publie le temps CPU
  (`/usr/bin/time -l`) à côté.
- Un test fondé sur une vidéo fabriquée est « à confirmer au premier passage en CI » dans le
  compte rendu : le ffmpeg BtbN 9.0.1 du runner Linux ne reproduit pas tout ce que le ffprobe 9.0.2 de Homebrew fait sur Mac.
- Le compte rendu ne s'écrit qu'à la toute fin, quand plus aucune mesure ne tourne en arrière-plan.

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
- `docs/10-logique-de-detection.md` explique la logique étape par étape (outil, choix,
  limite). Toute issue qui change une étape de la détection ou du suivi met à jour sa
  section dans la même issue : l'outil, la raison du choix, la limite, et la ligne du
  tableau des hypothèses si elle change. Le compte rendu dit quelles sections ont bougé.
- Ce qui est ou n'est pas une chauve-souris ne se tranche pas dans le code : c'est une
  question pour la naturaliste, à faire remonter.
- La qualité de détection passe avant le temps de calcul ; 16 Go de mémoire est la limite.

**Issues de 8 points** : avant de coder, écris ton plan dans
`/Users/zaratan/Projects/dvr-wt/rapports/plan-<N>.md`, fais-le relire par `tech-architect`
et `lead-engineer-reviewer` en parallèle, puis arrête-toi : la tour le fait valider.
Ensuite, arrête-toi à la fin de chaque sous-phase du plan avec ses chiffres.
