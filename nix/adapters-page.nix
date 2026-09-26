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
    records | jq -f ${./status.jq} > $out/status.json
  ''
