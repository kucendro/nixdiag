{ lib }:
nodes:
let
  q = builtins.toJSON;
  names = builtins.attrNames nodes;
  units = n: builtins.attrValues nodes.${n}.nixdiag.facts.topology.units;
  connections = n: lib.concatMap (u: u.connections) (units n);
  entries =
    n:
    builtins.filter (e: e.port != null && !(e.udp or false)) (
      lib.concatMap (u: u.expose ++ u.connections) (units n)
    );
  claimed = n: lib.unique (lib.concatMap (u: u.ports) (units n) ++ map (e: e.port) (entries n));
  other = n: lib.findFirst (m: m != n) null names;
  lan =
    n:
    q (
      lib.remove "" [
        nodes.${n}.networking.primaryIPAddress
        nodes.${n}.networking.primaryIPv6Address
      ]
    );
  each = f: lib.concatMap f names;

  connection =
    n: c:
    if c.name != null && c.port != null then
      [ "through(${n}, ${q c.name}, ${toString c.port})" ]
    else
      lib.optional (lib.hasInfix "://" c.to) "answers(${n}, ${q c.to})";

  serves =
    n:
    let
      s = nodes.${n}.services.nixdiag.serve;
    in
    lib.optional s.enable "serves(${n}, ${q s.virtualHost}, ${q names})";

  scope =
    n: e:
    lib.optional (other n != null)
      "reaches(${other n}, ${n}, ${toString e.port}, ${if e.scope != null then "True" else "False"})";
in
builtins.readFile ./checks.py
+ lib.concatLines (
  lib.unique (
    [ "start_all()" ]
    ++ each (n: [ "${n}.wait_for_unit(\"multi-user.target\")" ])
    ++ each (n: map (p: "listens(${n}, ${toString p})") (claimed n))
    ++ each (n: lib.concatMap (connection n) (connections n))
    ++ each serves
    ++ each (n: lib.concatMap (scope n) (entries n))
    ++ each (n: [ "listens_only(${n}, ${lan n}, ${q (claimed n)})" ])
  )
)
