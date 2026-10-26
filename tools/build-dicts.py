#!/usr/bin/env python3
"""Build data/dict/<lang>.gld from openly licensed sources.

Unigrams: AOSP LatinIME word lists (Apache-2.0), as redistributed by
OpenBoard v1.4.5. Bigrams and missing frequent words: Leipzig Corpora
Collection news corpora (CC BY 4.0, https://wortschatz.uni-leipzig.de).
Downloads are cached in .devtools/dict-cache; provenance is written to
data/dict/SOURCES.md.

    tools/build-dicts.py [en de ru]
"""
import gzip, math, os, subprocess, sys, tarfile, urllib.request
from collections import defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, ".devtools", "dict-cache")
OUT = os.path.join(ROOT, "data", "dict")
AOSP = "https://raw.githubusercontent.com/openboard-team/openboard/v1.4.5/dictionaries/{}_wordlist.combined.gz"
LEIPZIG = "https://downloads.wortschatz-leipzig.de/corpora/{}.tar.gz"
LANGS = {
    "en": ("en_US", "eng_news_2024_300K"),
    "de": ("de", "deu_news_2024_300K"),
    "ru": ("ru", "rus_news_2024_300K"),
}
MAX_SUCC = 16


def fetch(url):
    os.makedirs(CACHE, exist_ok=True)
    p = os.path.join(CACHE, url.rsplit("/", 1)[1])
    if not os.path.exists(p):
        print("downloading", url)
        tmp = p + ".part"
        urllib.request.urlretrieve(url, tmp)
        os.rename(tmp, p)
    return p


def aosp_words(path):
    words = {}
    with gzip.open(path, "rt", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line.startswith("word="):
                continue
            kv = dict(x.split("=", 1) for x in line.split(",") if "=" in x)
            if kv.get("not_a_word") == "true" or kv.get("possibly_offensive") == "true":
                continue
            w, fr = kv["word"], int(kv.get("f", "0"))
            if fr > 0:
                words[w] = max(words.get(w, 0), fr)
    return words


def leipzig(corpus):
    path = fetch(LEIPZIG.format(corpus))
    words, co = {}, []
    with tarfile.open(path) as t:
        for m in t.getmembers():
            if m.name.endswith("-words.txt"):
                for line in t.extractfile(m).read().decode("utf-8").splitlines():
                    p = line.split("\t")
                    if len(p) >= 3:
                        words[int(p[0])] = (p[1], int(p[2]))
            elif m.name.endswith("-co_n.txt"):
                for line in t.extractfile(m).read().decode("utf-8").splitlines():
                    p = line.split("\t")
                    if len(p) >= 3:
                        co.append((int(p[0]), int(p[1]), int(p[2])))
    return words, co


def is_word(w):
    return any(c.isalpha() for c in w) and not any(c.isdigit() for c in w) and len(w) <= 40 and " " not in w


def build(lang):
    aosp_name, corpus = LANGS[lang]
    words = aosp_words(fetch(AOSP.format(aosp_name)))
    lwords, co = leipzig(corpus)
    total = sum(c for _, c in lwords.values()) or 1
    # Add frequent corpus words the 2014 AOSP list lacks (scaled to its log range).
    added = 0
    lower_known = {w.lower() for w in words}
    for _, (w, c) in lwords.items():
        if c < 40 or not is_word(w) or w in words:
            continue
        if w[0].isupper() and w.lower() in lower_known:
            continue  # sentence-initial capital of a known word
        f = int(max(1, min(200, 255 + 16 * math.log10(c / total) + 40)))
        words[w] = f
        added += 1

    def form(w):
        if w in words:
            return w
        l = w.lower()
        return l if l in words else None

    succ = defaultdict(list)
    for a, b, c in co:
        if c < 3 or a not in lwords or b not in lwords:
            continue
        wa, wb = form(lwords[a][0]), form(lwords[b][0])
        if wa and wb and is_word(wa) and is_word(wb):
            succ[wa.lower()].append((wb, c))
    bigrams = []
    for a, lst in succ.items():
        tot = sum(c for _, c in lst)
        lst.sort(key=lambda x: -x[1])
        seen = set()
        for b, c in lst[:MAX_SUCC]:
            if b in seen:
                continue
            seen.add(b)
            p = c / tot
            f = int(max(1, min(255, 255 + 48 * math.log10(p))))
            bigrams.append((a, b, f))
    os.makedirs(OUT, exist_ok=True)
    wpath = os.path.join(CACHE, f"{lang}.words.tsv")
    bpath = os.path.join(CACHE, f"{lang}.bigrams.tsv")
    with open(wpath, "w", encoding="utf-8") as f:
        for w, fr in words.items():
            f.write(f"{w}\t{fr}\n")
    with open(bpath, "w", encoding="utf-8") as f:
        for a, b, fr in bigrams:
            f.write(f"{a}\t{b}\t{fr}\n")
    dictc = os.path.join(ROOT, "target", "release", "glissando-dictc")
    if not os.path.exists(dictc):
        subprocess.run(["cargo", "build", "--release", "-p", "glissando-lang", "--bin", "glissando-dictc"], cwd=ROOT, check=True)
    subprocess.run([dictc, "--words", wpath, "--bigrams", bpath, "--out", os.path.join(OUT, f"{lang}.gld")], check=True)
    return {"lang": lang, "aosp": AOSP.format(aosp_name), "corpus": LEIPZIG.format(corpus),
            "words": len(words), "added": added, "bigrams": len(bigrams)}


def main():
    langs = sys.argv[1:] or list(LANGS)
    info = [build(l) for l in langs]
    with open(os.path.join(OUT, "SOURCES.md"), "w") as f:
        f.write("# Dictionary sources\n\nGenerated by `tools/build-dicts.py`; the `.gld` files are not committed.\n\n")
        f.write("| lang | words | bigrams | unigram source (Apache-2.0) | corpus (CC BY 4.0) |\n|---|---|---|---|---|\n")
        for i in info:
            f.write(f"| {i['lang']} | {i['words']} (+{i['added']} from corpus) | {i['bigrams']} | {i['aosp']} | {i['corpus']} |\n")
        f.write("\nLeipzig Corpora Collection: D. Goldhahn, T. Eckart, U. Quasthoff (2012): Building Large Monolingual "
                "Dictionaries at the Leipzig Corpora Collection: From 100 to 200 Languages. LREC 2012. "
                "Licensed CC BY 4.0. AOSP word lists © The Android Open Source Project, Apache License 2.0.\n")
    for i in info:
        print(i)


if __name__ == "__main__":
    main()
