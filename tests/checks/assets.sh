set -euo pipefail

for f in "$fresh"/*.svg; do
  cmp -s "$f" "$assets/$(basename "$f")" || {
    echo "assets/$(basename "$f") is stale; run just assets"
    exit 1
  }
done
echo "assets: $(ls "$fresh" | wc -l) match"
