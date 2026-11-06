# gappalib18.nix; build with: nix --extra-experimental-features
# 'nix-command flakes' build --impure --no-link --print-out-paths -f gappalib18.nix
let
  pkgs = (builtins.getFlake "github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe").legacyPackages.x86_64-linux;
in
pkgs.coqPackages_8_20.gappalib.overrideAttrs (old: {
  version = "1.8.0";
  name = "coq8.20-gappalib-1.8.0";
  src = pkgs.fetchurl {
    url = "https://gitlab.inria.fr/gappa/coq/-/archive/gappalib-coq-1.8.0/coq-gappalib-coq-1.8.0.tar.gz";
    hash = "sha256-xSoI9ZHAMbkkfG3Wnajgu7bhCkglNIBngSrz4YnmkA8=";
  };
})
