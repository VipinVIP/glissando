# Architecture

```
crates/
  glissando-core     layouts (TOML), geometry/hit testing, touch state machine, text utils   (no deps but serde/toml)
  glissando-lang     mmap trie dictionaries, spatial fuzzy search, ranking, personal model,
                     language detection, typing engine (composing, auto-correct, smart punctuation)
  glissando-swipe    glide decoder (trie-pruned alignment + shape/location matching)
  glissando          the keyboard: Wayland client, cairo/pango renderer, D-Bus, haptics,
                     panels (emoji, clipboard, quick menu), config
  glissando-testkit  text-input-v3 test client and virtual pointer driver for e2e tests
data/                layouts, emoji (Unicode/CLDR), dictionaries (built by tools/build-dicts.py)
packaging/           cross build container, device scripts (trial, session, deploy)
tests/               e2e (headless sway) and privacy checks
```

## Why an independent keyboard

* **Squeekboard** (GTK3, Rust+C): mature protocol handling but no
  prediction, a drawing model built around static YAML layouts and a GTK4
  port still pending. Reaching Gboard-level interaction would mean
  rewriting most of it.
* **Stevia** (GTK4, C/GObject): completion via presage/hunspell, emoji,
  cursor gestures; no glide typing. Its widget tree and GObject plumbing make
  per-frame animation (trails, popups) and multi-language ranking awkward.
* **Own client**: the platform surface is small and well defined
  (layer-shell + input-method-v2 + virtual-keyboard-v1 + `sm.puri.OSK0`),
  so we implement it directly with `wayland-client` and draw with cairo/pango
  (already on every Phosh system, handles fonts, colour emoji and
  fractional scaling). All keyboard logic is plain Rust, unit testable and
  shared with the e2e harness.

## Runtime structure

One thread, one glib main loop: the Wayland fd, timers (long press,
repeat, animations), D-Bus (`sm.puri.OSK0`, feedbackd) and the config file
monitor all dispatch on it. Rendering caches the static key layer and only
paints dynamic parts (pressed keys, previews, popups, trail, strip) per
frame; frames are paced by `wl_surface.frame`.

Input flows: `wl_touch`/`wl_pointer` → `Keyboard` (keyboard.rs) →
`TouchMachine` (taps, rollover, long press, repeat, glide detection,
space/backspace scrubbing) → `Engine` (glissando-lang) → `Op`s
(commit/preedit/delete/key) → `zwp_input_method_v2` batches or
`zwp_virtual_keyboard_v1` keys. Without an input-method client (shown
manually) text is typed through a dynamically generated XKB keymap.

## Language model

* Dictionary: character trie, BFS laid out, 16 bytes/node, memory mapped
  (`.gld`, see `dict.rs`), word frequencies on the AOSP 0–255 log scale,
  subtree max frequency for best-first completion, bigram table sorted by
  predecessor.
* Correction: bounded weighted edit distance over the trie (substitution,
  omission, insertion, transposition, doubled letters, diacritics) with
  per-tap key probabilities from the touch position (Gaussian spatial
  model).
* Ranking: −cost + log frequency + personal boost + bigram context +
  language prior. Multilingual typing: all enabled languages of the same
  script are active at once; an online detector shifts priors towards the
  language being written.
* Personalisation: per-language TSV in `$XDG_DATA_HOME/glissando/user/`
  (0600): learned words, personal dictionary (pinned), blocked words, word
  pair counts. Never fed from password/PIN/sensitive fields.

## Glide typing

`glissando-swipe::ShapeDecoder`: resamples the trace (48 points, key-width
units); walks the dictionary trie keeping only prefixes whose letters the
finger passed in order (pass radius 0.95 kw) starting/ending near the
gesture ends; scores survivors by a monotonic letter-to-path alignment,
location and shape distances to the ideal key polyline and a length
penalty; combines with frequency, personal and context scores. Layout and
language agnostic (works for ЙЦУКЕН, QWERTZ, letters in popups such as ё/ъ/ß
fold onto their base key).

Evaluation on **real human gestures** (FUTO swipe dataset, MIT licence,
held-out `test.jsonl`, 3000 in-vocabulary English words, QWERTY,
`cargo run --release -p glissando-swipe --example eval_futo`): top-1
87.7 %, top-3 95.1 % without context; 88.4 % / 95.4 % with the previous word
as context. Parameters were tuned on the `dev.jsonl` split only (from
75.6 % / 86.9 % before tuning).

Synthetic gestures (`--example bench`, 0.25 kw noise): en 98.0 % / 100 %,
de 96.3 % / 100 %, ru 97.0 % / 98.0 %. Latency: three-stage pruning (O(L)
bound → alignment for 700 → full score for 120) gives p95 ≈ 1.5 ms on
x86-64; on the phone p50 2.5 ms / p95 10 ms on a big core before the last
optimisation step.

Optional future backend: FUTO Swipe (`swipe-library`, ExecuTorch) as a
separate plugin package, because its weights carry their own licence
(attribution requirement) and the runtime is a large C++ dependency.
