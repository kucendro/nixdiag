set -euo pipefail
source "$(dirname "$0")/lib.sh"

diff_manifest docs "$reference" "$docs"
jq -e '.meta.schema and .totals.hosts' "$docs/api/v1/snapshot.json" >/dev/null
for f in "$docs"/api/v1/*.json; do
  grep -q 'Auto-generated' "$f" || {
    echo "no marker: $f"
    exit 1
  }
done
no_store_paths "$docs"
