{ pkgs, ... }:
let
  tls = pkgs.runCommand "nixdiag-vm-tls" { nativeBuildInputs = [ pkgs.openssl ]; } ''
    mkdir $out
    openssl req -x509 -newkey rsa:2048 -nodes -days 3650 -subj /CN=vm \
      -keyout $out/key.pem -out $out/cert.pem
  '';
in
{
  environment.systemPackages = [ pkgs.curl ];
  environment.etc."vm/cert.pem".source = "${tls}/cert.pem";
  environment.etc."vm/key.pem".source = "${tls}/key.pem";
}
