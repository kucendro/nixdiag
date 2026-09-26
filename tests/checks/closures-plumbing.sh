set -euo pipefail

jq -e '.schema == 1' "$closures" >/dev/null
jq -e '.hosts.demo.paths | length > 0' "$closures" >/dev/null
jq -e '.hosts.demo.paths | all(has("path") and has("narSize"))' "$closures" >/dev/null
jq -e '.hosts.demo.paths == (.hosts.demo.paths | sort_by(.path))' "$closures" >/dev/null
