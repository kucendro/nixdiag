{ self }:
{
  imports = [
    ./module/serve.nix
    ./module/history.nix
    (import ./module/timer.nix { inherit self; })
  ];
}
