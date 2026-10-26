# Changelog

## Unreleased

- Words no longer split in two after moving the cursor in Qt apps and other
  clients that do not report why the text changed: edits are recognised by
  comparing the text, late reports of our own typing are ignored
- Backspace with a selection deletes the selection with a key press
- A field without surrounding text no longer inherits the previous field's
- Terminal keys: Esc, Tab, Ctrl, Alt and arrows above the letters in
  terminals (and when typing without an input-method client); Ctrl and Alt
  apply to the next key, arrows repeat while held

## 0.1.0 – first preview

- Wayland keyboard for Phosh: input-method-v2, virtual keyboard fallback,
  `sm.puri.OSK0`, layer-shell, fractional scaling, portrait and landscape
- Layouts: English (QWERTY), German (QWERTZ), Russian (ЙЦУКЕН), symbols,
  number/phone/PIN pads, email and URL variants
- Glide typing with trail; space-bar cursor control; glide delete
- Suggestions, next-word prediction, spatial auto-correction with undo,
  multilingual typing with language detection, on-device learning
- Emoji panel with search and suggestions, clipboard history, quick menu,
  one-handed mode, number row
- Local voice typing (whisper.cpp) as an optional package
- GTK4/libadwaita settings app (EN/DE/RU)
- Debian packages registering a non-default Phosh-OSK alternative
- Privacy: no network code, kernel-enforced network ban, automated checks
