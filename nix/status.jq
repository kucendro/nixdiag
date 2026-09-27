def adapters(states): [.audit[] | select(.status as $s | states | index($s)) | .adapter] | unique;
def verdict:
  adapters(["broken"]) as $broken
  | adapters(["renamed", "unaudited"]) as $renamed
  | if .vm == "failed" then ["vm failed", "red"]
    elif $broken != [] then ["broken: \($broken | join(", "))", "red"]
    elif $renamed != [] then ["renamed: \($renamed | join(", "))", "yellow"]
    else ["ok", "brightgreen"] end;
def badge(name; text; color): { schemaVersion: 1, label: name, message: text, color: color };
(group_by(.ref) | map(max_by(.date))) as $latest
| ($latest | map(select((verdict | .[1]) != "brightgreen") | .ref)) as $bad
| { "status.json": badge("adapters";
      (if $latest == [] then "no runs yet" elif $bad == [] then "ok on \($latest | map(.ref) | join(", "))" else "check \($bad | join(", "))" end);
      (if $latest == [] then "lightgrey" elif $bad == [] then "brightgreen" else "red" end)) }
  + ($latest | map({ key: "status/\(.ref).json", value: (verdict as [$text, $color] | badge(.ref; $text; $color)) }) | from_entries)
