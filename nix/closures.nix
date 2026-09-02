{ pkgs, lib }:
let
  perHost =
    name: drv:
    pkgs.runCommand "nixdiag-closure-${name}.json"
      {
        __structuredAttrs = true;
        exportReferencesGraph.closure = [ drv ];
        nativeBuildInputs = [ pkgs.jq ];
        preferLocalBuild = true;
        allowSubstitutes = false;
      }
      ''
        out=''${outputs[out]}
        jq -c --arg host ${lib.escapeShellArg name} \
          '{ ($host): { paths: (.closure | map({ path, narSize }) | sort_by(.path)) } }' \
          < "$NIX_ATTRS_JSON_FILE" > "$out"
      '';
in
{
  mkClosures =
    toplevels:
    let
      files = lib.mapAttrsToList perHost toplevels;
    in
    if files == [ ] then
      pkgs.writeText "nixdiag-closures.json" ''{"schema":1,"hosts":{}}''
    else
      pkgs.runCommand "nixdiag-closures.json"
        {
          nativeBuildInputs = [ pkgs.jq ];
          preferLocalBuild = true;
        }
        ''
          jq -s '{ schema: 1, hosts: (add // {}) }' ${lib.escapeShellArgs files} > $out
        '';
}
