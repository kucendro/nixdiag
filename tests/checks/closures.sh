set -euo pipefail
source "$(dirname "$0")/lib.sh"

diff_manifest closures "$reference" "$docs"
grep -qE '^\| Closure +\|' "$docs/wiki/src/hosts.md"
no_store_paths "$docs"
