{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      pname = "btrfs-auto-snapshot";
    in {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        inherit pname;
        version = "0.1.0";
        src = ./.;

        useFetchCargoVendor = true;
        cargoLock = {
           lockFile = ./Cargo.lock;
         };

        nativeBuildInputs = [
          pkgs.installShellFiles
        ];

        buildInputs = [ pkgs.btrfs-progs ];

        preFixup = ''
          mkdir completions
          $out/bin/${pname} completion bash > completions/${pname}.bash
          $out/bin/${pname} completion zsh > completions/${pname}.zsh
          $out/bin/${pname} completion fish > completions/${pname}.fish

          installShellCompletion completions/*
        '';

        meta = {
          description = "Create and manage Btrfs snapshots automatically";
          license = pkgs.lib.licenses.mit;
          maintainers = with pkgs.lib.maintainers; [ imsick ];
        };
      };
    };
}
