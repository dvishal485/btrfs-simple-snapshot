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
        btrfs-simple-snapshot = final.callPackage ./package.nix { inherit rev; };
      };

      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        import ./nixos-module.nix {
          inherit lib config pkgs;
          btrfs-simple-snapshot = self.packages.${pkgs.system}.btrfs-simple-snapshot;
        };

      packages = supportedSystems (pkgs: rec {
        btrfs-simple-snapshot = pkgs.callPackage ./package.nix { inherit rev; };
        default = btrfs-simple-snapshot;
      });
    };
}
