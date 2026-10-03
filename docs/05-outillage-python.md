# Outillage Python

## Pourquoi Python

Mesuré sur la 092 : décodage + réduction 1,3 ms par image, médiane du fond 9,3 ms,
seuil + taches 0,26 ms. Le calcul lourd est déjà en C (numpy, OpenCV, ffmpeg) ; une
réécriture en Rust ne gagnerait que sur la médiane. L'écosystème (OpenCV, suivi,
YOLO) est natif en Python. La question se reposera si l'outil doit être distribué en
binaire unique à des naturalistes sans Python.

## Correspondances

| Besoin | Ruby | TS | Rust | Ici |
| --- | --- | --- | --- | --- |
| Version du langage | rbenv | mise | rustup | **mise** (`python = "3.14.8"`) |
| Paquets, environnement, lancement | bundler | pnpm | cargo | **uv** |
| Manifeste | Gemfile | package.json | Cargo.toml | `pyproject.toml` |
| Lockfile | Gemfile.lock | pnpm-lock.yaml | Cargo.lock | `uv.lock` |
| Dépendances installées | vendor | node_modules | target | `.venv/` |
| Lint | rubocop | eslint | clippy | **ruff check** |
| Format | rubocop | prettier | rustfmt | **ruff format** |
| Types | sorbet | tsc | compilateur | **basedpyright** strict |
| Tests | rspec | vitest | cargo test | **pytest** |
| Hooks | overcommit | husky | — | **lefthook** |
| Scripts | rake | scripts npm | alias cargo | **tâches mise** |

## Choix et raisons

- **Python et uv fixés par mise**, comme bun et pnpm sur les autres projets. uv a
  interdiction d'utiliser ou de télécharger son propre Python
  (`python-preference = "only-system"`, `python-downloads = "never"`) : une seule
  source de vérité, `mise.toml`.
- **basedpyright** plutôt que mypy (plus proche de `strictTypeChecked`, s'installe sans
  Node) ou ty (d'Astral, encore en bêta 0.0.83 en septembre 2026).
- **ruff** avec une liste explicite de familles de règles, pas `ALL` : `ALL` active les
  règles de docstrings, contraires à la règle zéro commentaire, et change à chaque
  montée de version.
- **lefthook** plutôt que prek ou pre-commit : binaire unique, `{staged_files}` natif,
  et les hooks lancent `uv run ruff`, donc la version de `uv.lock`.
- **Extrait réel dans git**, pas en LFS : 5,3 Mo une fois pour toutes.
- **Pas de seuil de couverture** (`mise run coverage` pour la voir) : un pourcentage
  bloquant n'a pas de sens sur un outil de cette taille.
- **CI** : `jdx/mise-action` installe les versions de `mise.toml`, actions épinglées
  par SHA (audit zizmor propre).

## Pièges rencontrés

- `.python-version` à `3.14` laissait uv prendre le premier 3.14 trouvé (3.14.7 de
  mise au lieu de 3.14.8). D'où la version exacte, et dans `mise.toml` seulement.
- Les stubs d'OpenCV renvoient des types larges (`MatLike`) : chaque retour de `cv2`
  passe par `np.asarray(..., dtype=...)` pour retrouver un type numpy précis.
- Les paramètres de test d'une dataclass mélangeant `int` et `float` ne se typent pas
  avec `**kwargs` : les tests de config invalide passent des lambdas.
- `uv run` synchronise `.venv` tout seul avant de lancer : passé l'installation,
  `uv sync` n'est presque jamais nécessaire à la main.
