# Reference behaviour: modern phone keyboards

glissando aims at the interaction quality users know from mainstream phone
keyboards such as Gboard. This page lists the user-visible behaviour we
target. It is based on using such keyboards and on their public settings
screens; glissando contains no code, assets or data from them.

## Feature checklist

Keys: optional number row (also for password fields), emoji key or
language switch key, comma/period keys, touch & hold for digits and symbols
with hints on the keys, key pop-up, adjustable touch & hold duration.

Appearance: font size, light and dark themes, optional key borders,
adjustable keyboard height, one-handed mode (left/right).

Glide typing: glide typing with a fading trail, glide delete (slide left from
the delete key to remove whole words), glide cursor control (slide on the
space bar to move the cursor), optional trackpad mode on long-press space.

Corrections: auto-correction (undone by the next backspace),
auto-capitalisation, word and next-word suggestions, auto-space after a
suggestion, double-space period, smart punctuation (a period or comma after
an auto-space replaces it: "word, ").

Expression: emoji panel with categories, recents and search; emoji
suggestions while typing.

Clipboard: history with retention time, pinned items.

Personalisation: learned words, personal dictionary, delete learned data.

## Deliberately absent

Usage statistics upload, crowd-sourced/federated training, audio donation,
accounts, cloud sync, remote configuration, contacts access. glissando has
no network access at all; see `docs/PRIVACY.md`.

## Interaction parameters in glissando

| Behaviour | glissando default (`TouchConfig`) |
|---|---|
| Glide start | leave the start key and travel 0.5 key widths, or 1.5 key widths regardless |
| Glide trail | 5 px trail fading out over 280 ms |
| Scrub start (space/backspace) | 18 px |
| Cursor step | 11 px per character |
| Glide delete | first word after 18 px, one more word per 36 px |
| Row tolerance while scrubbing | 25 px + one key unit |
| Long press | 300 ms (150–1000 configurable) |

Wayland input methods cannot select text, so glide delete previews the
words in the suggestion strip and deletes them on release.

## Suggestion strip

Three slots with the strongest candidate in the middle (bold when it will be
applied by auto-correction) and the verbatim word on one side. After a glide
the strip shows alternatives and backspace removes the whole glided word.
