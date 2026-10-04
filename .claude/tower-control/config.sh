ISSUES_REPO="zaratan/chiro-dvr"
PROJECT_OWNER="zaratan"
PROJECT_NUMBER=4
WORKTREES_DIR="$HOME/Projects/dvr-wt"
BASE_BRANCH="origin/main"
BOOTSTRAP='mise trust && uv sync'
declare -A REPO_CHECK=([_]='test -d .venv')
declare -A GRAPPES=([decodage]="docs/20-derive" [detection]="tests/18-reference")
