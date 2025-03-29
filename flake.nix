{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      rev = self.shortRev or self.dirtyShortRev or "dirty";
      supportedSystems =
        function:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
          "aarch64-linux"
        ] (system: function nixpkgs.legacyPackages.${system});
    in
    {
      overlays.default = final: prev: {
        btrfs-auto-snapshot = final.callPackage ./package.nix { inherit rev; };
      };

      packages = supportedSystems (pkgs: rec {
        btrfs-auto-snapshot = pkgs.callPackage ./package.nix { inherit rev; };
        default = btrfs-auto-snapshot;
      });
    };
}
