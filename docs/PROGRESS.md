# Progress log

Legend: ✅ implemented and verified by automated tests · 🟡 implemented,
verified only partially (see note) · ⬜ not started.

## Stage 1 – working keyboard over Wayland ✅
* layer-shell surface, input-method-v2 commit/preedit/delete, virtual
  keyboard keys and fallback typing, `sm.puri.OSK0`, auto show on focus —
  e2e `keyboard_appears_on_focus`, `typing_letters`.
* arm64 build runs on the FuriPhone (surface, fractional scale) 🟡 typing on
  device not yet exercised (requires `glissando-trial` at the phone).

## Stage 2 – layouts and UI ✅/🟡
* en/de/ru letters, symbols ×2, numpad, phone, digits, email/url bottom rows ✅ (unit tests)
* key preview, long-press popups, hints, shift/caps lock/chording ✅ unit, 🟡 popups not e2e tested
* suggestion strip, toolbar, one-handed mode, number row, height setting 🟡 (no e2e)
* emoji / clipboard / quick-menu panels 🟡 (render path unit-tested, no e2e yet)
* themes light/dark/system, own vector icons ✅

## Stage 3 – prediction, correction, dictionaries ✅
* trie dictionaries from AOSP + Leipzig, spatial fuzzy correction,
  completion, bigram prediction, personal model, multilingual detection
  (unit tests in glissando-lang).
* auto-correct + undo, smart punctuation, double-space period, recompose on
  backspace (engine tests).

## Stage 4 – glide typing ✅
* own decoder; e2e `glide_typing`; benchmark in ARCHITECTURE.md.
* ⬜ evaluation on real human traces (FUTO swipe dataset), ⬜ optional FUTO backend.

## Stage 5 – remaining Gboard features
* space cursor + glide delete ✅ e2e `space_cursor_and_glide_delete`
* password privacy ✅ e2e `password_field_is_private`
* ⬜ settings application (GTK4/libadwaita) – config file + quick menu exist
* ⬜ emoji search UI (data + search function exist), emoji suggestions in strip
* ⬜ voice input (local, e.g. whisper.cpp/Vosk) as optional module
* ⬜ landscape layout tuning, split layout

## Stage 6 – performance
* measured first render 23–37 ms on device (cache build); ⬜ pre-render shift states, ⬜ partial damage

## Stage 7 – device integration ✅/🟡
* user-prefix deploy, `glissando-trial`, `glissando-session` with automatic
  fallback, seccomp network restriction ✅ scripts written; trial on device ⬜
* ⬜ Debian package with Phosh-OSK alternative

## Privacy ✅
* `tests/privacy/check.sh`: no networking crates, no DNS imports, e2e under
  socket guard with zero attempts, e2e in empty network namespace.
