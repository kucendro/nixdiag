{
  lib,
  d2,
  mdbook,
  woff2,
  runCommand,
  makeWrapper,
}:
let
  bare =
    if (d2.override.__functionArgs or { }) ? withImageSupport then
      d2.override { withImageSupport = false; }
    else
      d2;

  fonts = runCommand "mdbook-open-sans" { nativeBuildInputs = [ woff2 ]; } ''
    mkdir $out
    for w in regular 600 700 italic; do
      cp ${mdbook.src}/crates/mdbook-html/front-end/fonts/open-sans-v17-all-charsets-$w.woff2 $out/$w.woff2
      woff2_decompress $out/$w.woff2
      rm $out/$w.woff2
    done
  '';
in
runCommand "d2-${bare.version}"
  {
    nativeBuildInputs = [ makeWrapper ];
    meta.mainProgram = "d2";
  }
  ''
    makeWrapper ${lib.getExe bare} $out/bin/d2 \
      --set D2_FONT_REGULAR ${fonts}/regular.ttf \
      --set D2_FONT_SEMIBOLD ${fonts}/600.ttf \
      --set D2_FONT_BOLD ${fonts}/700.ttf \
      --set D2_FONT_ITALIC ${fonts}/italic.ttf
  ''
