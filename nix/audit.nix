{ pkgs, adapters }:
let
  reads = pkgs.writeText "nixdiag-reads.json" (
    builtins.toJSON (builtins.mapAttrs (_: a: a.reads) adapters)
  );
  options = pkgs.writeText "nixdiag-options.nix" ''
    ref:
    let
      n = builtins.getFlake ref;
    in
    (import (n.outPath + "/nixos/release.nix") { nixpkgs = n; }).options
  '';
in
pkgs.writeShellApplication {
  name = "nixdiag-audit";
  runtimeInputs = with pkgs; [
    curl
    brotli
    jq
    nix
  ];
  text = ''
    format=table
    if [ "''${1:-}" = --json ]; then
      format=json
      shift
    fi
    refs=("$@")
    if [ "$#" -eq 0 ]; then refs=(nixos-unstable nixos-25.05); fi
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    options() {
      case "$1" in
      nixos-* | nixpkgs-unstable) curl -fsSL "https://channels.nixos.org/$1/options.json.br" | brotli -d ;;
      *) cat "$(nix build --impure --no-link --print-out-paths --expr "import ${options} \"$1\"")/share/doc/nixos/options.json" ;;
      esac
    }
    {
      if [ "$format" = table ]; then
        echo "| adapter | read | ref | status | path |"
        echo "|---|---|---|---|---|"
      fi
      for ref in "''${refs[@]}"; do
        options "$ref" | jq -c 'keys | map({ key: ., value: true }) | from_entries' > "$tmp/known.json"
        jq -rc --arg ref "$ref" --arg format "$format" --slurpfile known "$tmp/known.json" -f ${./audit.jq} ${reads}
      done
    } | tee "$tmp/report"
    ! grep -qw broken "$tmp/report"
  '';
}
