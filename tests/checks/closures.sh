set -euo pipefail
source "$(dirname "$0")/lib.sh"

diff_manifest closures "$reference" "$docs"
grep -q '| Closure |' "$docs/wiki/src/hosts.md"
no_store_paths "$docs"
