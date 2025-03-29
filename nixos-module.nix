{
  config,
  lib,
  pkgs,
  btrfs-simple-snapshot,
  ...
}:

let
  name = "btrfs-simple-snapshot";
  cfg = config.services.${name};
in
{
  options.services."${name}" = {
    enable = lib.mkEnableOption "Enable ${name} service";

    package = lib.mkOption {
      type = lib.types.package;
      default = btrfs-simple-snapshot;
      description = "The package to use for ${name}";
    };

    tasks = lib.mkOption {
      type = lib.types.listOf (
        lib.types.submodule {
          options = {
            pre-cmd = lib.mkOption {
              type = lib.types.str;
              default = "";
              example = ''
                # mount script
                $${pkgs.coreutils}/bin/mkdir -p /var/tmp/btrfs-simple-snapshot/root
                $${pkgs.util-linux}/bin/mount -o noatime -o compress=zstd /dev/disk/by-label/nixroot /var/tmp/btrfs-simple-snapshot/root
              '';
              description = "Command to run before snapshot";
            };

            mount-point = lib.mkOption {
              type = lib.types.path;
              example = "/var/tmp/btrfs-simple-snapshot/root";
              description = "Mount point for the subvolume";
            };

            subvolume = lib.mkOption {
              type = lib.types.str;
              example = "home";
              description = "Path to the subvolume relative to mount point";
            };

            post-cmd = lib.mkOption {
              type = lib.types.str;
              default = "";
              example = ''
                # unmount script
                $${pkgs.util-linux}/bin/umount /var/tmp/btrfs-simple-snapshot/root
                $${pkgs.coreutils}/bin/rm -r /var/tmp/btrfs-simple-snapshot/root
              '';
              description = "Command to run after snapshot";
            };

            snapshot-path = lib.mkOption {
              type = lib.types.str;
              default = ".snapshots";
              description = "Path where snapshots are stored";
              example = "backups";
            };

            snapshot = {
              enable = lib.mkEnableOption "Take snapshot when service is triggered";
              args = lib.mkOption {
                default = { };
                type = lib.types.submodule {
                  options = {
                    readonly = lib.mkOption {
                      type = lib.types.bool;
                      default = false;
                      example = true;
                      description = "Make snapshot readonly";
                    };
                    prefix = lib.mkOption {
                      type = lib.types.nullOr lib.types.str;
                      default = null;
                      example = "my-fav-subvol";
                      description = "Prefix for snapshot name (null value defaults to subvolume name)";
                    };
                    suffix-format = lib.mkOption {
                      type = lib.types.str;
                      default = "%Y-%m-%d-%H%M%S";
                      example = "backup-%Y-%m-%d";
                      description = "Format for suffix (with datetime) of snapshot name";
                    };
                  };
                };
                description = "Configuration of CLI arguments for ${name} snapshot";
              };
            };
            cleanup = {
              enable = lib.mkEnableOption "Perform cleanup of old snapshots when service is triggered";
              args = lib.mkOption {
                default = { };
                type = lib.types.submodule {
                  options = {
                    keep-count = lib.mkOption {
                      type = lib.types.nullOr lib.types.int;
                      default = null;
                      description = "Minimum number of snapshots to preserve";
                    };
                    keep-since = lib.mkOption {
                      type = lib.types.nullOr lib.types.str;
                      default = null;
                      description = "Minimum age of snapshots to preserve";
                    };
                  };
                };
                description = "Configuration for CLI arguments for ${name} clean";
              };
            };
          };
        }
      );
      default = [ ];
      description = "List of subvolumes and their configuration to manage";
    };
    config = {
      verbose = lib.mkEnableOption "Enable verbose log output";
      interval = lib.mkOption {
        type = lib.types.str;
        example = "daily";
        description = "Interval for systemd timer. Refer https://www.freedesktop.org/software/systemd/man/latest/systemd.time.html#Calendar%20Events or {manpage}`systemd.timer(5)`.";
        default = "weekly";
      };
    };
  };

  config = lib.mkIf cfg.enable {

    assertions = [
      {
        assertion = cfg.tasks != [ ];
        message = "No snapshot or cleanup tasks defined for ${name} service.";
      }
      {
        assertion = lib.all (task: task.snapshot.enable || task.cleanup.enable) cfg.tasks;
        message = "${name} service is disabled for both snapshot and cleanup. Enable at least one of them.";
      }
      {
        assertion = lib.all (
          task:
          task.cleanup.enable
          -> (task.cleanup.args.keep-count != null || task.cleanup.args.keep-since != null)
        ) cfg.tasks;
        message = "For cleanup, each task must have either `keep-count` or `keep-since` or both set.";
      }
    ];

    environment.systemPackages = [ cfg.package ];

    systemd.timers."${name}" = {
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = cfg.config.interval;
        Persistent = true;
      };
    };

    systemd.services."${name}" =
      let
        extra_args = if cfg.config.verbose then "--verbose" else "";
        trigger = pkgs.writeShellScriptBin "${name}-trigger" (
          lib.concatStrings (
            map (
              task:
              let
                action = if task.snapshot.enable then "snapshot" else "clean";
                snapshot_args =
                  if task.snapshot.enable then lib.cli.toGNUCommandLineShell { } task.snapshot.args else "";
                cleanup_args =
                  if task.cleanup.enable then lib.cli.toGNUCommandLineShell { } task.cleanup.args else "";
                snapshot_path = "--snapshot-path ${task.snapshot-path}";
              in
              ''
                ${task.pre-cmd}
                ${cfg.package}/bin/${name} ${action} ${task.mount-point} ${task.subvolume} ${snapshot_args} ${cleanup_args} ${snapshot_path} ${extra_args}
                ${task.post-cmd}
              ''
            ) cfg.tasks
          )
        );
      in
      {
        description = "${name} service for managing btrfs subvolume snapshots";
        serviceConfig = {
          ExecStart = "${trigger}/bin/${name}-trigger";
          Restart = "no";
          RemoveIPC = "yes";
          ProtectSystem = "yes";
          PrivateTmp = true;
        };
      };
  };
}
