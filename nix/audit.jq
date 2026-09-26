def known: . as $p | $known[0] | has($p);
def settings:
  split(".") as $s
  | [range(1; $s | length)]
  | map($s[:.])
  | any(.[-1] == "settings" and (join(".") | known));
def status:
  map(select(known)) as $hits
  | if .[0] | known then ["ok", .[0]]
    elif $hits != [] then ["renamed", $hits[0]]
    elif any(settings) then ["unaudited", .[0]]
    else ["broken", .[0]] end;
to_entries[]
| .key as $adapter
| .value
| to_entries[]
| .key as $read
| (.value | status) as [$status, $path]
| if $format == "json" then { adapter: $adapter, read: $read, ref: $ref, status: $status, path: $path }
  else "| \($adapter) | \($read) | \($ref) | \($status) | `\($path)` |" end
