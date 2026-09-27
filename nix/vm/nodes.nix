{
  lib,
  adapters,
  bases,
  options,
  mkDocs,
}:
let
  inherit (lib) head tail concatStringsSep;
  helpers = import ../adapters/lib.nix { inherit lib; };
  fail = msg: throw "nixdiag vm: ${msg}";

  exists =
    let
      go =
        node: segs:
        if segs == [ ] then
          true
        else if lib.isOption node then
          go (node.type.getSubOptions [ ]) (if head segs == "<name>" then tail segs else segs)
        else
          node ? ${head segs} && go node.${head segs} (tail segs);
    in
    path: go options (lib.splitString "." path);

  place =
    segs: v:
    if segs == [ ] then
      v
    else if head segs == "<name>" then
      lib.mapAttrs (_: place (tail segs)) v
    else
      { ${head segs} = place (tail segs) v; };

  write =
    what: candidates: v:
    place (lib.splitString "." (
      lib.findFirst exists (fail "no option for ${what}: ${concatStringsSep ", " candidates}") candidates
    )) v;

  enableOf = name: a: a.enable or [ "services.${name}.enable" ];

  under =
    read: path:
    read == [ ]
    || (
      path != [ ] && (head read == "<name>" || head read == head path) && under (tail read) (tail path)
    );

  hits = reads: path: lib.any (r: under (lib.splitString "." r) path) reads;

  strings =
    v:
    if builtins.isString v then
      [ v ]
    else if builtins.isList v then
      lib.concatMap strings v
    else if builtins.isAttrs v then
      lib.concatMap strings (builtins.attrValues v)
    else
      [ ];

  hosts =
    a:
    map (s: (helpers.url s).host) (
      builtins.filter (lib.hasInfix "://") (strings (a.tests.probe or { }))
    );

  moduleOf =
    name: a:
    let
      reads = lib.concatLists (builtins.attrValues a.reads) ++ enableOf name a;
      minimum = a.tests.minimum or { };
      bad = builtins.filter (hits reads) (
        lib.collect builtins.isList (lib.mapAttrsRecursive (p: _: p) minimum)
      );
    in
    if bad != [ ] then
      fail "${name} minimum sets what it reads: ${concatStringsSep ", " (map (concatStringsSep ".") bad)}"
    else
      {
        imports = [
          (write "${name} enable" (enableOf name a) true)
          minimum
        ]
        ++ lib.mapAttrsToList (r: write "${name}.${r}" a.reads.${r}) (a.tests.probe or { });
      };

  placed = lib.mapAttrs (
    name: a:
    let
      vms = a.tests.vm or [ ];
      unknown = builtins.filter (v: !(bases ? ${v})) vms;
    in
    if vms == [ ] then
      fail "${name} has no tests.vm"
    else if unknown != [ ] then
      fail "${name} names unknown vm: ${concatStringsSep ", " unknown}"
    else
      a
  ) adapters;

  on = vm: lib.filterAttrs (_: a: builtins.elem vm a.tests.vm) placed;
in
sv:
let
  vms = builtins.filter (vm: builtins.elem sv bases.${vm}.stateVersions && on vm != { }) (
    builtins.attrNames bases
  );
  missing = builtins.filter (h: !(builtins.elem h vms)) (
    lib.concatMap hosts (lib.concatMap (vm: builtins.attrValues (on vm)) vms)
  );
in
if missing != [ ] then
  fail "probe names vm with no node at ${sv}: ${concatStringsSep ", " (lib.unique missing)}"
else
  lib.genAttrs vms (vm: {
    imports = [
      bases.${vm}.module
      ../module
      ./kit.nix
      (import ./docs.nix { inherit mkDocs; })
      { system.stateVersion = sv; }
    ]
    ++ lib.mapAttrsToList moduleOf (on vm);
  })
