# Privacy

glissando is private by construction:

* **No network code.** No HTTP/TLS/DNS crates; the binary imports no name
  resolution functions. When run as a session keyboard or trial, systemd
  forbids every socket family except `AF_UNIX`/`AF_NETLINK`
  (`RestrictAddressFamilies`), so even a bug could not reach the network.
* **No telemetry, analytics, crash upload, accounts, remote flags or
  identifiers.**
* **No logging of typed text.** `GLISSANDO_DEBUG=1` logs state changes only
  (activate, surface size, frame times). `GLISSANDO_DEBUG_INPUT=1` logs touch
  coordinates and must only be used in test sessions.
* **Sensitive fields** (content purpose password/PIN, hints
  `sensitive_data`/`hidden_text`): no composing text, no suggestions, no
  auto-correction, no learning, no glide typing.
* **Personal data stays local and is erasable:**

  | Data | Location | Erase |
  |---|---|---|
  | learned words, personal dictionary, word pairs | `~/.local/share/glissando/user/*.tsv` (0600) | quick menu → "Delete learned words", or delete the files |
  | recent emoji | `~/.local/state/glissando/recent-emoji` | delete the file |
  | clipboard history | memory only, expires after the configured hours | strip → clipboard → Clear; turning history off |
  | pinned clipboard items | `~/.local/state/glissando/pinned-clips` (0600) | unpin, or delete the file |

  Clipboard entries offered with a password-manager hint MIME type are
  never recorded. Words containing digits or `@` are never learned.
* **Licences and provenance** of bundled data: `data/dict/SOURCES.md`
  (AOSP word lists, Apache-2.0; Leipzig corpora, CC BY 4.0) and
  `data/emoji/LICENSE-UNICODE` (Unicode emoji/CLDR data).

## Verification

`tests/privacy/check.sh` (run by CI and before releases):

1. dependency graph contains no networking crates,
2. the binary imports no `getaddrinfo`/`gethostbyname`/…,
3. the complete end-to-end suite passes with an `LD_PRELOAD` socket guard
   that refuses and records any non-local socket or DNS lookup — the record
   must be empty,
4. the end-to-end suite passes inside an empty network namespace.

On the device: `systemd-run --user -p RestrictAddressFamilies="AF_UNIX AF_NETLINK"`
was verified to block `AF_INET` while glissando runs normally.
