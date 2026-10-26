# Running glissando on the FuriPhone

Nothing here needs root or touches system files. Everything lives in
`~/.local/opt/glissando` on the phone.

## Build and copy

```bash
packaging/cross/build-arm64.sh          # rootless podman, Debian forky arm64 libraries
python3 tools/build-dicts.py            # once: data/dict/*.gld
packaging/device/deploy.sh furios       # ssh host from ~/.ssh/config
```

## Try it (temporary)

On the phone (SSH or terminal):

```bash
~/.local/opt/glissando/bin/glissando-trial start 30   # 30 minutes
~/.local/opt/glissando/bin/glissando-trial stop       # back to the system keyboard now
```

`start` stops `mobi.phosh.OSK.service` and runs glissando as the transient
user unit `glissando-trial` with a run-time limit. Its `ExecStopPost`
restarts the system keyboard whenever glissando ends: stop, crash, timeout
or logout. Logs: `journalctl --user -u glissando-trial`.

## Use it as the keyboard (per user, reversible)

```bash
~/.local/opt/glissando/bin/glissando-session enable
~/.local/opt/glissando/bin/glissando-session disable  # restores Stevia/Squeekboard
```

`enable` writes one drop-in, `~/.config/systemd/user/mobi.phosh.OSK.service.d/glissando.conf`,
that points the existing keyboard service at `glissando-launch`. The
launcher starts the system keyboard instead if glissando crashed three
times within five minutes or is missing. Removing the file (from any shell,
also via SSH) restores the original behaviour.

## Emergency rollback without a working keyboard

* via SSH: `glissando-session disable` or `rm -r ~/.config/systemd/user/mobi.phosh.OSK.service.d && systemctl --user daemon-reload && systemctl --user restart mobi.phosh.OSK.service`
* a hardware keyboard or the FuriOS terminal is not needed: three crashes
  already switch back automatically, and a reboot keeps the fallback logic.

## Verified on the device

| Check | Status |
|---|---|
| arm64 binary starts, binds globals, creates the layer surface | done (2026-10-26) |
| fractional scale 1.74, 413 px logical width | done |
| first full render 23–37 ms on the phone CPU | measured |
| seccomp socket restriction under systemd user units | done |
| typing through input-method-v2 with Stevia stopped (`glissando-trial`) | **not yet verified** – needs someone at the phone |
| glide typing, haptics strength, clipboard on device | **not yet verified** |
