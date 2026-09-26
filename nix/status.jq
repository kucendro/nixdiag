def worst:
  if any(. == "broken") then "broken"
  elif any(. == "renamed" or . == "unaudited") then "renamed"
  else "ok" end;
(group_by(.ref) | map(max_by(.date))) as $latest
| ($latest | map(select(.eval != "ok" or ([.audit[].status] | worst) != "ok") | .ref)) as $bad
| {
    schemaVersion: 1,
    label: "adapters",
    message: (
      if $latest == [] then "no runs yet"
      elif $bad == [] then "ok on \($latest | map(.ref) | join(", "))"
      else "check \($bad | join(", "))" end),
    color: (if $latest == [] then "lightgrey" elif $bad == [] then "brightgreen" else "red" end)
  }
