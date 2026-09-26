{ lib }:
rec {
  loopback = host: lib.hasPrefix "127." host || host == "localhost" || host == "::1";

  url =
    s:
    let
      authority = builtins.head (lib.splitString "/" (lib.last (lib.splitString "://" s)));
      m = builtins.match "(\\[[^]]*]|[^:]*)(:([0-9]+))?" authority;
      host = if m == null then authority else builtins.head m;
      port = if m == null then null else builtins.elemAt m 2;
    in
    {
      host = lib.removeSuffix "]" (lib.removePrefix "[" host);
      port =
        if port != null then
          lib.toInt port
        else if lib.hasPrefix "https://" s then
          443
        else
          80;
    };

  flagValues =
    flag: flags:
    if flags == [ ] then
      [ ]
    else
      let
        f = builtins.head flags;
        rest = builtins.tail flags;
      in
      if f == flag && rest != [ ] then
        [ (builtins.head rest) ] ++ flagValues flag (builtins.tail rest)
      else if lib.hasPrefix "${flag}=" f then
        [ (lib.removePrefix "${flag}=" f) ] ++ flagValues flag rest
      else
        flagValues flag rest;

  flagValue =
    flag: flags:
    let
      v = flagValues flag flags;
    in
    if v == [ ] then null else lib.last v;

  addrScope =
    open: addr:
    let
      a = lib.removeSuffix "]" (lib.removePrefix "[" addr);
      v4 = builtins.match "([0-9]+)\\.([0-9]+)\\.[0-9]+\\.[0-9]+" a;
      o1 = if v4 == null then -1 else lib.toInt (builtins.head v4);
      o2 = if v4 == null then -1 else lib.toInt (builtins.elemAt v4 1);
      guarded = if open then "public" else null;
    in
    if lib.hasPrefix "127." a || a == "::1" then
      null
    else if (o1 == 100 && o2 >= 64 && o2 <= 127) || lib.hasPrefix "fd7a:115c:a1e0" a then
      "mesh"
    else if
      o1 == 10
      || (o1 == 192 && o2 == 168)
      || (o1 == 172 && o2 >= 16 && o2 <= 31)
      || lib.hasPrefix "fc" a
      || lib.hasPrefix "fd" a
    then
      "lan"
    else
      guarded;

  scopeOf =
    open: addrs:
    let
      scopes = map (addrScope open) addrs;
    in
    lib.findFirst (s: builtins.elem s scopes) null [
      "public"
      "lan"
      "mesh"
    ];
}
