{
  pkgs,
  adapters,
  data,
}:
pkgs.runCommand "nixdiag-adapters"
  {
    nativeBuildInputs = [ pkgs.jq ];
    info = builtins.toJSON (
      builtins.mapAttrs (_: a: {
        inherit (a) role maintainers reads;
      }) adapters
    );
    passAsFile = [ "info" ];
  }
  ''
    mkdir $out
    records() { find ${data} -mindepth 2 -maxdepth 2 -name '*.json' -exec cat {} + | jq -s .; }
    records | jq -r --slurpfile meta "$infoPath" -f ${./adapters.jq} > $out/adapters.md
    mkdir $out/status
    records | jq -f ${./status.jq} > badges.json
    for k in $(jq -r 'keys[]' badges.json); do jq --arg k "$k" '.[$k]' badges.json > "$out/$k"; done
  ''
