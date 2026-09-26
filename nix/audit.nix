{ pkgs, adapters }:
let
  reads = pkgs.writeText "nixdiag-reads.json" (
    builtins.toJSON (builtins.mapAttrs (_: a: a.reads) adapters)
  );
in
pkgs.writeShellApplication {
  name = "nixdiag-audit";
  runtimeInputs = with pkgs; [
    curl
    brotli
    jq
  ];
  text = ''
    channels=("$@")
    if [ "$#" -eq 0 ]; then channels=(nixos-unstable nixos-25.05); fi
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    {
      echo "| adapter | read | channel | status | path |"
      echo "|---|---|---|---|---|"
      for channel in "''${channels[@]}"; do
        curl -fsSL "https://channels.nixos.org/$channel/options.json.br" \
          | brotli -d | jq -c 'keys | map({ key: ., value: true }) | from_entries' > "$tmp/known.json"
        jq -r --arg channel "$channel" --slurpfile known "$tmp/known.json" -f ${./audit.jq} ${reads}
      done
    } | tee "$tmp/report.md"
    ! grep -q '| broken |' "$tmp/report.md"
  '';
}
