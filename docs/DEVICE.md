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

## Voice typing

```bash
packaging/voice/build-whisper.sh        # whisper.cpp for arm64 (once)
packaging/device/deploy.sh --voice furios
```

Installs `whisper-cli` and the multilingual Whisper base model (q8_0, 82 MB,
checksum verified) into the user prefix. The microphone appears in the
suggestion strip; tap it, speak, and pause (or tap again). On the FLX1s an
11 s utterance is recognised in about 4 s. Recording uses `parecord`; the
audio stays in memory and in a private file under `$XDG_RUNTIME_DIR` that is
deleted immediately.

## System-wide installation (Debian packages)

```bash
packaging/debian/build-deb.sh           # dist/glissando_*.deb, dist/glissando-voice_*.deb
```

On the phone:

```bash
sudo apt install ./glissando_0.2.0_arm64.deb ./glissando-voice_0.2.0_arm64.deb
# switch (takes effect after re-login, or restart the keyboard service):
sudo update-alternatives --set Phosh-OSK /usr/share/glissando/io.github.x_vlad_x.Glissando.desktop
systemctl --user daemon-reload && systemctl --user restart mobi.phosh.OSK.service
# back to Stevia:
sudo update-alternatives --set Phosh-OSK /usr/share/phosh-osk-stevia/mobi.phosh.Stevia.desktop
systemctl --user daemon-reload && systemctl --user restart mobi.phosh.OSK.service
```

The package registers a Phosh-OSK alternative with priority 20 (Stevia has
30), so installing it never changes the active keyboard by itself. The
system unit has the same network ban and crash fallback as the user setup.
`apt-get install -s` of both packages was checked on the device.

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
| typing through input-method-v2 with Stevia stopped (`glissando-trial`) | done by the user (2026-10-26), no errors in the journal |
| local voice pipeline (recording via PulseAudio under the network ban, whisper.cpp) | done with a recorded sample; live speech at the phone not yet tried |
