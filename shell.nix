# Backwards-compatible shell.nix for users without flakes enabled
# Prefer using: nix develop (with flakes)
#
# Usage: nix-shell

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
}).shellNix
