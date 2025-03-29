{
  lib,
  rustPlatform,
  installShellFiles,
  btrfs-progs,
  rev ? "dirty",
}:

let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  pname = "btrfs-simple-snapshot";
  version = "${cargoToml.package.version}-${rev}";
in
rustPlatform.buildRustPackage {
  inherit pname;
  inherit version;
  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.intersection (lib.fileset.fromSource (lib.sources.cleanSource ./.)) (
      lib.fileset.unions [
        ./src
        ./Cargo.toml
        ./Cargo.lock
      ]
    );
  };

  useFetchCargoVendor = true;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    installShellFiles
  ];

  buildInputs = [ btrfs-progs ];

  preFixup = ''
    mkdir completions
    $out/bin/${pname} completion bash > completions/${pname}.bash
    $out/bin/${pname} completion zsh > completions/${pname}.zsh
    $out/bin/${pname} completion fish > completions/${pname}.fish

    installShellCompletion completions/*
  '';

  meta = {
    description = "Create and manage Btrfs snapshots automatically";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ imsick ];
  };
}
