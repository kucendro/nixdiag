set -euo pipefail

paths=$(jq '.hosts.nixdiag.paths | length' "$closures")
bytes=$(jq '[.hosts.nixdiag.paths[].narSize] | add' "$closures")
echo "nixdiag runtime closure: $((bytes / 1048576)) MiB across $paths paths"

if [ "$bytes" -gt $((600 * 1024 * 1024)) ]; then
  echo "runtime closure passed 600 MiB; re-measure and raise the ceiling on purpose"
  exit 1
fi
