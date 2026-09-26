diff_manifest() {
  local want=$1 reference=$2 docs=$3 seen=0
  while read -r build path; do
    case "$build" in
    "") continue ;;
    docs | closures) ;;
    *)
      echo "MANIFEST: unknown build '$build' for $path"
      exit 1
      ;;
    esac
    [ "$build" = "$want" ] || continue
    [ -e "$docs/$path" ] || {
      echo "MANIFEST lists $path, but the build did not write it"
      exit 1
    }
    diff -u "$reference/$path" "$docs/$path"
    seen=$((seen + 1))
  done < <(sed 's/#.*//' "$reference/MANIFEST")
  [ "$seen" -gt 0 ] || {
    echo "no MANIFEST entries for '$want'"
    exit 1
  }
  echo "$want: $seen snapshots match"
}

no_store_paths() {
  if grep -rIqE '/nix/store/[a-z0-9]{32}-' "$1"; then
    echo "generated docs contain a store path; that would retain Nix references:"
    grep -rIoE '/nix/store/[a-z0-9]{32}-[^ `"]*' "$1" | head
    exit 1
  fi
}
