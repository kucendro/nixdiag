/**
  Edge node: mesh control plane and the fleet's reverse proxy.
*/
{
  imports = [
    ../../modules/common.nix
    ../../modules/mesh.nix
    ../../modules/web.nix
  ];
}
