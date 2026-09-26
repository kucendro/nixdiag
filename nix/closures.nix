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
    {
      toplevels,
      served ? [ ],
    }:
    let
      files = lib.mapAttrsToList perHost toplevels;
    in
    if files == [ ] then
      pkgs.writeText "nixdiag-closures.json" ''{"schema":1,"hosts":{},"served":[]}''
    else
      pkgs.runCommand "nixdiag-closures.json"
        {
          nativeBuildInputs = [ pkgs.jq ];
          preferLocalBuild = true;
          served = builtins.toJSON served;
        }
        ''
          jq -s --argjson served "$served" '{ schema: 1, hosts: (add // {}), served: $served }' \
            ${lib.escapeShellArgs files} > $out
        '';
}
