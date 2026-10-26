#!/usr/bin/env python3
"""Build data/emoji/emoji.tsv from Unicode emoji-test.txt and CLDR annotations.

Sources (Unicode License v3, see data/emoji/LICENSE-UNICODE):
  https://unicode.org/Public/emoji/<ver>/emoji-test.txt
  https://github.com/unicode-org/cldr-json  (cldr-annotations-full, cldr-annotations-derived-full)

Output columns: emoji, group, then one keyword column per language
(keywords separated by '|', first entry is the CLDR short name).
Only fully-qualified emoji are kept; skin tone variants are folded into
their base emoji (column 'tones' lists the variants).
"""
import json, sys, urllib.request, pathlib

EMOJI_VER = "16.0"
CLDR_VER = "46.0.0"
LANGS = ["en", "de", "ru"]
ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "data" / "emoji" / "emoji.tsv"
CACHE = ROOT / ".devtools" / "emoji-cache"

def fetch(url, name):
    CACHE.mkdir(parents=True, exist_ok=True)
    p = CACHE / name
    if not p.exists():
        with urllib.request.urlopen(url, timeout=60) as r:
            p.write_bytes(r.read())
    return p.read_text(encoding="utf-8")

TONES = {chr(c) for c in range(0x1F3FB, 0x1F400)}

def main():
    test = fetch(f"https://unicode.org/Public/emoji/{EMOJI_VER}/emoji-test.txt", f"emoji-test-{EMOJI_VER}.txt")
    ann = {}
    for lang in LANGS:
        for kind in ["annotations", "annotationsDerived"]:
            pkg = "cldr-annotations-full" if kind == "annotations" else "cldr-annotations-derived-full"
            url = f"https://raw.githubusercontent.com/unicode-org/cldr-json/{CLDR_VER}/cldr-json/{pkg}/{kind}/{lang}/annotations.json"
            data = json.loads(fetch(url, f"{kind}-{lang}-{CLDR_VER}.json"))
            entries = data[kind]["annotations"] if kind in data else data["annotationsDerived"]["annotations"]
            for e, v in entries.items():
                names = v.get("tts", []) + v.get("default", [])
                d = ann.setdefault(e.replace("️", ""), {})
                cur = d.setdefault(lang, [])
                for n in names:
                    if n not in cur:
                        cur.append(n)
    rows, base_index = [], {}
    group = ""
    for line in test.splitlines():
        if line.startswith("# group:"):
            group = line.split(":", 1)[1].strip()
            continue
        if not line or line.startswith("#"):
            continue
        cps, rest = line.split(";", 1)
        if "fully-qualified" not in rest.split("#")[0]:
            continue
        if group == "Component":
            continue
        e = "".join(chr(int(c, 16)) for c in cps.split())
        base = "".join(ch for ch in e if ch not in TONES)
        if base != e and base in base_index:
            rows[base_index[base]]["tones"].append(e)
            continue
        key = e.replace("️", "")
        kw = ann.get(key, {})
        base_index[e] = len(rows)
        rows.append({"e": e, "group": group, "kw": kw, "tones": []})
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("# emoji\tgroup\ttones\t" + "\t".join(LANGS) + f"\t# Unicode emoji {EMOJI_VER}, CLDR {CLDR_VER}\n")
        for r in rows:
            cols = [r["e"], r["group"], " ".join(r["tones"])]
            for lang in LANGS:
                cols.append("|".join(k.replace("\t", " ").replace("|", "/") for k in r["kw"].get(lang, [])))
            f.write("\t".join(cols) + "\n")
    print(f"wrote {len(rows)} emoji to {OUT}")

if __name__ == "__main__":
    sys.exit(main())
