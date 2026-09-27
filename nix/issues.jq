def body($adapter):
  [ "\($maintainers[0][$adapter] // [] | map("@" + .) | join(" ")) `nix/adapters/\($adapter).nix` needs a look.",
    "",
    "| Ref | Read | Status | Path |",
    "|---|---|---|---|",
    (.[] | "| \(.ref) | \(.read) | \(.status) | `\(.path)` |"),
    "",
    "**renamed**: only this later candidate exists, put it first. **broken**: no candidate exists, add the new path. **unaudited**: a freeform `settings` hides the path." ]
  | join("\n");
(map(.ref as $ref | .audit[] | select(.status != "ok") | .ref = $ref)
 | group_by(.adapter)
 | map(.[0].adapter as $a | { key: "\($a) adapter", value: body($a) })
 | from_entries) as $want
| ($open[0] | map({ key: .title, value: . }) | from_entries) as $have
| ($want | to_entries[] | .key as $title | .value as $body | $have[$title] as $issue
   | if $issue == null then ["issue", "create", "--label", "monitor", "--title", $title, "--body", $body]
     elif ($issue.body | gsub("\r"; "")) != $body then ["issue", "edit", $issue.number, "--body", $body], ["issue", "comment", $issue.number, "--body", $body]
     else empty end),
  (select(all(.audit != [])) | $have | to_entries[] | select($want[.key] == null)
   | ["issue", "close", .value.number, "--comment", "Every nixpkgs head is ok again."])
| map(tostring)
