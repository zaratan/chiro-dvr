# Outillage Python

## Pourquoi Python

Mesuré sur l'original de la 092 ([09](09-profilage.md)) : décodage 1,1 ms par image
(8,6 ms de CPU, le décodeur occupe presque tous les cœurs), réduction 0,8 ms, médiane du
fond 0,7 ms (tri par comparaisons vectorisé en uint8, contre 12,6 ms avec `np.median`), résidu et
taches 1,9 ms. Le calcul lourd est déjà en C (numpy, OpenCV, ffmpeg) et le décodage
domine : une réécriture en Rust ne gagnerait presque rien. L'écosystème (OpenCV, suivi,
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
- **Extrait réel en LFS** (5,3 Mo) : GitHub refusait le fichier dans git. La tâche
  `slow-tests` de la CI ne télécharge que lui, et le garde en cache.
- **092 entière en LFS** (454 Mo, accord de la naturaliste) : test à marqueur `reference`,
  exclu de `pytest` par défaut, dans une tâche de CI parallèle qui garde l'objet LFS en
  cache pour ne pas entamer le quota de téléchargement LFS de GitHub à chaque passage.
- **Vidéo absente** : sans `git lfs pull`, une vidéo de test n'est qu'un pointeur de
  quelques octets. `tests/conftest.py` arrête alors les tests `slow` et `reference` avec
  ce message, au lieu d'un `VideoError` peu lisible.
- **Tolérances des tests sur vraie vidéo** : nombre de pistes exact, début ±5 images,
  points ±3. La même vidéo ne donne pas les mêmes pistes sur Mac arm64 et Linux : jusqu'à
  4 images de décalage sur un début et 2 points d'écart (mesuré en octobre 2026, OpenCV
  5.0.0). Sur l'extrait, ces tolérances attrapent `threshold` 35 et 45 ; sur la 092
  entière, tous les réglages dégradés essayés sauf `merge_radius` 0, absorbé par la
  fusion des jumelles.
- **Seuil de couverture à 95 %**, mesuré sur les tests rapides seuls (`mise run test:fast`,
  97 % en octobre 2026) : sous ce seuil, la CI échoue. Le rapport s'affiche dans le résumé
  de la tâche `fast-tests`.
- **pytest-xdist** sur les tests rapides (`-n auto`) : 26 s → 12 s en local sur 4
  processus, mesuré en octobre 2026. Les tests `slow` et `reference` restent séquentiels.
- **CI en quatre tâches parallèles** : `static` (lint, format, types, sans ffmpeg ni LFS),
  `fast-tests` (tests rapides et couverture), `slow-tests` (l'extrait LFS) et `reference`
  (la 092 entière). Chacune a sa tâche mise (`static`, `test:fast`, `test:slow`) ;
  `mise run check` enchaîne les trois premières en local.
- **ffmpeg statique, plus d'`apt-get`.** Le 8 octobre 2026, la tâche `fast-tests` de la
  PR #50 a atteint son délai de 10 min pendant `apt-get install ffmpeg`, à cause d'un miroir
  Azure lent. L'étape prenait d'habitude 22 à 35 s. L'action `setup` installe désormais
  ffmpeg et ffprobe :
  - source : la build statique Linux x86-64 de **BtbN/FFmpeg-Builds**, release
    `autobuild-2026-08-31-13-27`, ffmpeg **n9.0.1**, variante `gpl` (les tests encodent
    en `libx264`) ;
  - pourquoi BtbN : c'est une release GitHub, et ses builds de fin de mois sont gardées
    (depuis novembre 2024), alors que les builds quotidiennes disparaissent au bout de
    deux semaines. johnvansickle ne publie que la dernière version sous une URL fixe, et
    hors de GitHub ;
  - vérification : le sha256 est écrit dans l'action et contrôlé avant l'extraction ;
  - installation : `curl`, `sha256sum -c`, `tar`, aucun code tiers exécuté ;
  - cache : `actions/cache`, clé faite du nom et du sha. Les binaires sont copiés dans
    `/usr/local/bin`, pas ajoutés à `GITHUB_PATH`, que zizmor signale.

  Version proche du ffmpeg 9 des Mac. Les tests `slow`, passés dans un conteneur Linux
  amd64 avec ces binaires, restent dans leurs tolérances. `awalsh128/cache-apt-pkgs-action`
  avait été écartée plus tôt : à cache vide, elle exécute un script d'apt-fast pris sur
  `master`, non épinglé. La release (`macos-latest`) garde `brew install ffmpeg`.
- **CI** : `jdx/mise-action` installe les versions de `mise.toml`, actions épinglées
  par SHA. L'installation commune est dans `.github/actions/setup` ; zizmor la signale à
  chaque appel local (`self-repository`, niveau le plus bas), rien d'autre.

## Distribution par Homebrew

- **Binaire PyInstaller plutôt que formule Python.** Mesuré en octobre 2026 (#30) :
  - le binaire (`--onedir`) pèse 63 Mo à télécharger et 147 Mo installé. Une formule sur
    `python@3.14`, `opencv` et `numpy` de Homebrew ajoute 91 formules à celles de ffmpeg,
    environ 3,2 Go ;
  - le binaire donne sur la 092 un `tracks.csv` identique à l'octet à celui du dépôt.
    La formule donne les mêmes 14 pistes, mais des `hits` et surfaces un peu différents :
    l'OpenCV de Homebrew décode par son propre ffmpeg ;
  - ses versions sont celles d'`uv.lock`, alors que celles de Homebrew changeraient les
    résultats sans nouvelle version de batdetect.

  Limites : Mac Apple Silicon seulement, macOS 14 minimum (les extensions de numpy 2.5.3
  l'exigent).
- **Release sur tag `vX.Y.Z`** (`.github/workflows/release.yml`). La version n'est pas lue
  du tag : la release échoue si le tag diffère de `pyproject.toml`, si le commit n'est pas
  sur `main` ou si sa CI n'a pas réussi. La release est créée en brouillon et rendue
  visible seulement une fois l'archive envoyée : le tap ne voit jamais de release sans
  archive. Pas de préversion pour l'instant.
- **`mise run package`** construit le binaire, avec `freeze_support()` en tête de `main`
  (sans lui, un binaire gelé relance la commande dans chaque processus de `--workers`), et
  joint les licences. **`mise run package:check`** contrôle :
  - le macOS minimum de chaque fichier (14 au plus, comme la formule) ;
  - que le FFmpeg embarqué est celui que nomme `packaging/NOTICE` ;
  - que chaque bibliothèque embarquée y est listée ;
  - que dépôt, binaire et binaire `--workers 2` donnent sur l'extrait le même
    `tracks.csv`. Cela prouve l'emballage, pas les 14 pistes : celles-ci restent la tâche
    du test `reference`.
- **Formule `batdetect` du tap `zaratan/homebrew-bat-tools`**, de la même forme que
  `chiro` pour que `bump-formulae.yml` la mette à jour. `preserve_rpath` empêche Homebrew
  de réécrire et de re-signer la centaine de bibliothèques du binaire. Sans lui,
  l'installé ne serait plus l'archive contrôlée en CI.
- **Double licence.** Le code est sous MIT. Le binaire est sous GPL v3 : la roue
  `opencv-python-headless` embarque sur macOS le FFmpeg 7.1.1 de Homebrew, compilé avec
  `--enable-gpl --enable-version3` et lié à x264, x265, rubberband et vidstab, alors que son
  `LICENSE-3RD-PARTY.txt` annonce la LGPL. L'archive joint :
  - la GPL v3 ;
  - le `LICENSE` MIT ;
  - les licences de la roue et de CPython ;
  - `packaging/NOTICE` : une ligne par bibliothèque (formule Homebrew, licence, historique
    de la formule) et les sources de FFmpeg. Seules 16 bibliothèques sur 100 gardent leur
    préfixe de construction, donc leur version Homebrew : pour les autres, le NOTICE
    renvoie à l'historique de la formule.

  Quand la roue change, `package:check` échoue sur les bibliothèques absentes du NOTICE :
  il faut alors le compléter à la main.
- **`ffprobe` vient de Homebrew** (`depends_on "ffmpeg"`), pas du binaire. Il repère les
  plages abîmées : « identique au dépôt » ne vaut donc qu'à `ffprobe` égal, et une montée
  de ffmpeg chez Homebrew peut changer les plages ignorées.
- **`macos-latest`** construit la release. Le jour où il change de version de macOS, uv
  peut prendre des roues plus récentes : la garde du macOS minimum arrête alors la release
  plutôt que de livrer un binaire qui planterait sur macOS 14.

## Pièges rencontrés

- `.python-version` à `3.14` laissait uv prendre le premier 3.14 trouvé (3.14.7 de
  mise au lieu de 3.14.8). D'où la version exacte, et dans `mise.toml` seulement.
- Les stubs d'OpenCV renvoient des types larges (`MatLike`) : chaque retour de `cv2`
  passe par `np.asarray(..., dtype=...)` pour retrouver un type numpy précis.
- Les paramètres de test d'une dataclass mélangeant `int` et `float` ne se typent pas
  avec `**kwargs` : les tests de config invalide passent des lambdas.
- `uv run` synchronise `.venv` tout seul avant de lancer : passé l'installation,
  `uv sync` n'est presque jamais nécessaire à la main.
