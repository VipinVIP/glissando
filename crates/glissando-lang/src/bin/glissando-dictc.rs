//! Dictionary compiler: `glissando-dictc --words W.tsv [--bigrams B.tsv] [--combined X.combined] --out D.gld`
//!
//! `W.tsv`: `word<TAB>freq` (freq 1..=255, AOSP log scale).
//! `B.tsv`: `word1<TAB>word2<TAB>freq`.

use glissando_lang::dict::{Dict, DictBuilder};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut b = DictBuilder::default();
    let mut out = None;
    let mut i = 1;
    while i < args.len() {
        let v = args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--words" => {
                for line in std::fs::read_to_string(&v).expect("read words").lines() {
                    if let Some((w, f)) = line.split_once('\t') {
                        b.add_word(w, f.trim().parse().unwrap_or(0));
                    }
                }
            }
            "--bigrams" => {
                for line in std::fs::read_to_string(&v).expect("read bigrams").lines() {
                    let f: Vec<&str> = line.split('\t').collect();
                    if f.len() == 3 {
                        b.add_bigram(f[0], f[1], f[2].trim().parse().unwrap_or(0));
                    }
                }
            }
            "--combined" => b.add_combined(&std::fs::read_to_string(&v).expect("read combined"), 1),
            "--out" => out = Some(v),
            other => {
                eprintln!("unknown argument {other}");
                std::process::exit(2);
            }
        }
        i += 2;
    }
    let Some(out) = out else {
        eprintln!("usage: glissando-dictc --words W.tsv [--bigrams B.tsv] --out D.gld");
        std::process::exit(2);
    };
    let bytes = b.build();
    let d = Dict::from_bytes(bytes.clone()).expect("valid output");
    std::fs::write(&out, &bytes).expect("write");
    println!("{out}: {} words, {} nodes, {} bytes", d.word_count(), d.node_count(), bytes.len());
}
