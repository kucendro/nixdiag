def worst:
  if any(. == "broken") then "broken"
  elif any(. == "renamed" or . == "unaudited") then "renamed"
  else "ok" end;
def glyph: if . == "ok" then "✓" elif . == "broken" then "✗" else "⚠" end;
def cells: join(" | ");
$meta[0] as $m
| (map(.ref) | unique) as $refs
| "# Adapters\n",
  "| Adapter | Role | Maintainers | Reads |",
  "|---|---|---|---|",
  ($m | to_entries[]
   | "| `\(.key)` | \(.value.role) | \(.value.maintainers | join(", ")) | \(.value.reads | to_entries | map("`\(.value[0])`") | join(", ")) |"),
  (if $refs == [] then "\nNo monitor runs yet." else empty end),
  ($refs[] as $ref
   | (map(select(.ref == $ref)) | sort_by(.date) | .[-12:]) as $runs
   | "\n## \($ref)\n",
     "| | \($runs | map(.date) | cells) |",
     "|---|\($runs | map("---") | join("|"))|",
     "| vm | \($runs | map(if .vm == null then "–" elif .vm == "ok" then "✓" else "✗" end) | cells) |",
     ($m | keys[] as $a
      | "| `\($a)` | \($runs | map([.audit[] | select(.adapter == $a) | .status] | if . == [] then "–" else worst | glyph end) | cells) |"),
     "\nLast run at nixpkgs `\($runs[-1].rev[:7])`.")
