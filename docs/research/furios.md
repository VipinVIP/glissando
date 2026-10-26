# FuriOS / Phosh input stack (observed 2026-10-26)

Device: FuriPhone FLX1s+, kernel 4.19.325-furiphone-radon, FuriOS 14.0
("forky", Debian testing based, Halium 12L vendor), 8 cores (Cortex-A55
cluster reported first), 7.6 GB RAM.

| Component | Version |
|---|---|
| phoc | 0.44.0-1+furios7 (wlroots based compositor) |
| phosh | 0.55.0-1+furios0 |
| phosh-osk-stevia | 0.54.0-2 (active keyboard, selected manually) |
| squeekboard | 1.43.1-2+furios1 (FuriLabs fork, installed, inactive) |
| feedbackd | 0.8.9 (`org.sigxcpu.Feedback.Haptic.Vibrate(s, a(du))` available) |
| glib / cairo / pango | 2.88.1 / 1.18.4 / 1.57.1 |

## How Phosh runs the keyboard

* `update-alternatives` group **Phosh-OSK**: link
  `/usr/share/applications/sm.puri.OSK0.desktop` with dependent
  `/usr/lib/systemd/user/mobi.phosh.OSK.service`. Stevia (priority 30) and
  Squeekboard (25) register alternatives.
* `mobi.phosh.OSK.service`: `Type=dbus`, `BusName=sm.puri.OSK0`,
  `ExecStart=/usr/bin/phosh-osk-stevia --allow-replacement`,
  `Restart=on-failure`.
* Phosh talks to the keyboard via D-Bus `sm.puri.OSK0` at `/sm/puri/OSK0`:
  method `SetVisible(b)`, property `Visible`.
* Text goes through Wayland `zwp_input_method_v2` (only one input method per
  seat; a second one receives `unavailable`), raw keys through
  `zwp_virtual_keyboard_v1`, the surface is a `zwlr_layer_surface_v1` with
  namespace `osk`. Stevia additionally uses `zwlr_data_control_v1`
  (clipboard), `zphoc_device_state_v1` and foreign-toplevel.
* phoc offers fractional scaling: on this phone the keyboard is configured
  413 logical px wide at scale 1.74 (720 px panel).
* `org.gnome.desktop.a11y.applications screen-keyboard-enabled` is `true`.

## Swipe typing at FuriLabs

FuriLabs announced FUTO Swipe based glide typing for FuriOS 14.0.4 (blog
"Swipe Keyboard"): encoder-only use of the FUTO models, English and
Spanish, learning dictionary. The code is not in the public FuriLabs
`squeekboard` (branch `forky`) or `phosh-osk-stub` forks as of this date.

FUTO's `swipe-library` (C++, GPL-3.0, ExecuTorch runtime) and the model
weights (FUTO Model Weights License 1.0: redistribution allowed with a
visible "powered by FUTO Swipe" notice, not sublicensable) are a candidate
optional backend; see `docs/ARCHITECTURE.md`.

## Decision

glissando is an independent keyboard that speaks exactly the same
protocols and D-Bus contract as Stevia/Squeekboard, so Phosh needs no
changes and the system keyboard can be swapped in and out per user without
root (systemd user drop-in), see `docs/DEVICE.md`.
