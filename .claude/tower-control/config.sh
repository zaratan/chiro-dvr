ISSUES_REPO="zaratan/chiro-dvr"
PROJECT_OWNER="zaratan"
PROJECT_NUMBER=4
WORKTREES_DIR="$HOME/Projects/dvr-wt"
BASE_BRANCH="origin/main"
BOOTSTRAP='mise trust && mise install && uv sync && mise run rust:link'
declare -A REPO_CHECK=([_]='test -d .venv')
