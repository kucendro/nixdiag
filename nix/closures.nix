# Per-host system closure sizes.
#
# Kept separate from the facts projection because the provenance differs:
# facts.json is pure evaluation, while nar sizes exist only for *realised*
# paths. That is also why this is its own file with its own schema rather than
# extra fields on the facts model — `mkFacts` structurally cannot produce it.
#
# `__structuredAttrs = true` is load-bearing. The classic exportReferencesGraph
# text format carries the graph only (path / empty deriver / nrefs / refs) and
# no sizes at all; structured attrs replace the path list with PathInfo objects
# that expose narSize. Same technique as pkgs.closureInfo, which we do not
# reuse because it discards per-path sizes into a nix-store --load-db blob.
{ pkgs, lib }:
let
  # name -> the host's system.build.toplevel
  perHost =
    name: drv:
    pkgs.runCommand "nixdiag-closure-${name}.json"
      {
        __structuredAttrs = true;
        exportReferencesGraph.closure = [ drv ];
        nativeBuildInputs = [ pkgs.jq ];
        # Pins the jq to the machine invoking the build. That is right when the
        # docs are built where the systems are (a build host, a CI runner), and
        # it is why that placement matters: Nix does not schedule on input
        # locality, so without this it could ship an entire system closure to a
        # remote builder to run a few lines of jq. The flip side is that
        # invoking the build somewhere else copies every measured closure to
        # that machine -- see the `closures` notes in nix/lib.nix.
        preferLocalBuild = true;
        allowSubstitutes = false;
      }
      ''
        # structuredAttrs exposes outputs as a bash array rather than $out
        out=''${outputs[out]}
        # sort_by keeps the merged file byte-identical across runs
        jq -c --arg host ${lib.escapeShellArg name} \
          '{ ($host): { paths: (.closure | map({ path, narSize }) | sort_by(.path)) } }' \
          < "$NIX_ATTRS_JSON_FILE" > "$out"
      '';
in
{
  # { hostName = toplevel; ... } -> closures.json
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
