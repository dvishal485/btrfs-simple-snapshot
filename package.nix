{
  lib,
  rustPlatform,
  installShellFiles,
  btrfs-progs,
  makeBinaryWrapper,
  rev ? "dirty",
}:

let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  pname = "btrfs-simple-snapshot";
  runtimeDeps = [ btrfs-progs ];
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
    makeBinaryWrapper
  ];

  buildInputs = [ btrfs-progs ];

  preFixup = ''
    mkdir completions
    $out/bin/${pname} completions bash > completions/${pname}.bash
    $out/bin/${pname} completions zsh > completions/${pname}.zsh
    $out/bin/${pname} completions fish > completions/${pname}.fish

    installShellCompletion completions/*
  '';

  postFixup = ''
    wrapProgram $out/bin/${pname} --prefix PATH : ${lib.makeBinPath runtimeDeps}
  '';

  meta = {
    description = "Create and manage Btrfs snapshots automatically";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ imsick ];
  };
}
