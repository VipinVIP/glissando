# glissando

A private, offline on-screen keyboard for Phosh (FuriOS, Mobian, postmarketOS)
with the ergonomics of a modern phone keyboard: glide typing, prediction and
auto-correction, multilingual typing, cursor control on the space bar, glide
delete, emoji and clipboard panels — without any network access.

Status: early but working. See [docs/PROGRESS.md](docs/PROGRESS.md).

## Features

* Wayland native: `zwp_input_method_v2` + `zwp_virtual_keyboard_v1` +
  layer-shell; implements Phosh's `sm.puri.OSK0` D-Bus interface, so it is a
  drop-in replacement for Squeekboard/Stevia.
* Layouts: English (QWERTY), German (QWERTZ), Russian (ЙЦУКЕН), symbols,
  number pad, phone/PIN pads, email/URL variants; easy TOML format.
* Typing: multitouch rollover, key preview, long-press alternates and
  digit/symbol hints, auto-capitalisation, double-space period, smart
  punctuation after suggestions, backspace undoes auto-correction.
* Language: dictionaries for en/de/ru with bigram prediction, spatial
  typo correction, personal dictionary learned on device, simultaneous
  multilingual typing with automatic language detection.
* Gestures: glide typing with trail, space-bar cursor control, glide delete
  on backspace, optional trackpad on long-press space.
* Panels: emoji (2000+, categories, recents, search data), clipboard history
  with retention and pinning, quick menu (one-handed mode, number row, theme,
  height, glide toggle, delete learned data).
* Light/dark themes following the system, haptics via feedbackd with
  adjustable strength, fractional scaling.

## Build and test

```bash
cargo build --release
cargo test                      # unit tests
python3 tools/build-dicts.py    # dictionaries (downloads public sources once)
tools/devenv/fetch-sway.sh      # headless sway for e2e tests (no root)
dbus-run-session -- tests/e2e/e2e.py
tests/privacy/check.sh          # no-network guarantees
packaging/cross/build-arm64.sh  # aarch64 build for the phone
```

Installing and trying it on a phone safely: [docs/DEVICE.md](docs/DEVICE.md).

## Documentation

* [Architecture](docs/ARCHITECTURE.md)
* [Privacy](docs/PRIVACY.md)
* [Gboard behaviour study](docs/research/gboard.md), [FuriOS input stack](docs/research/furios.md)

## Licence

GPL-3.0-or-later. Bundled data: AOSP word lists (Apache-2.0), Leipzig
Corpora Collection bigrams (CC BY 4.0), Unicode emoji and CLDR annotations
(Unicode License v3). Details in `data/*/`.
