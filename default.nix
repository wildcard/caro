# Backwards-compatible default.nix for users without flakes enabled
# Prefer using: nix build (with flakes)
#
# Usage: nix-build

(import (
  let
    lock = builtins.fromJSON (builtins.readFile ./flake.lock);
    flake-compat = lock.nodes.flake-compat.locked;
  in fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/${flake-compat.rev}.tar.gz";
    sha256 = flake-compat.narHash;
  }
) {
  src = ./.;
}).defaultNix
