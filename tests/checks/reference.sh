set -euo pipefail
source "$(dirname "$0")/lib.sh"

diff_manifest docs "$reference" "$docs"
d2_compiles "$docs"
no_store_paths "$docs"
