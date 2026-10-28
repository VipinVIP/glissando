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
* ✅ evaluation on real human traces (FUTO dataset): 87.7 % top-1 / 95.1 % top-3 held-out; ⬜ optional FUTO neural backend (deferred, see ARCHITECTURE.md).

## Stage 5 – remaining features
* space cursor + glide delete ✅ e2e `space_cursor_and_glide_delete`
* password privacy ✅ e2e `password_field_is_private`
* ✅ settings application (GTK4/libadwaita, en/ru/de), runs on the phone (launcher entry, user scope)
* 🟡 emoji search (query in the strip, typed or glided) and emoji suggestions while typing — builds, unit tests pass, no dedicated e2e test yet
* ✅ local voice typing (glissando-voice + whisper.cpp), e2e `voice_input`, pipeline verified on the phone with a recorded sample
* ✅ landscape: row-pitch normalised touch model, lower rows; full e2e suite passes with `E2E_LANDSCAPE=1`; ⬜ split layout
* ✅ terminal keys (Esc, Tab, Ctrl, Alt, arrows) for terminal fields, e2e `terminal_keys`
* ✅ apps without automatic showing (wlr-foreign-toplevel app_id), settings page, e2e `no_auto_show_in_listed_apps`; 🟡 Stevia import only checked against its schema, not on the device
* ✅ editing in clients without a text change cause (Qt): edits detected from the text, e2e `editing_with_cursor_moves_qt`; 🟡 not yet tried with real Qt/Chromium apps on the device
* ✅ preedit kept while GTK deletes a selected text or the app changes it, preedit committed by phoc on touch recognised, e2e `typing_over_a_selected_search`; traced on the device in GNOME Maps

## Stage 6 – performance
* measured on device: key frame ~1 ms, cached shift ~2 ms, glide decode p50 2.5 ms (big core); ✅ shift states pre-rendered; ⬜ partial damage

## Stage 7 – device integration ✅/🟡
* user-prefix deploy, `glissando-trial`, `glissando-session` with automatic
  fallback, seccomp network restriction ✅ scripts written; trial on device ✅ (by the user)
* ✅ Debian packages (`packaging/debian/build-deb.sh`) with a non-default Phosh-OSK alternative; install simulated on the device

## Privacy ✅
* `tests/privacy/check.sh`: no networking crates, no DNS imports, e2e under
  socket guard with zero attempts, e2e in empty network namespace.
