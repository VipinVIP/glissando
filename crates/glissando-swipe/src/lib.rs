//! Glide typing decoder.
//!
//! The built-in decoder is layout and language agnostic. It walks the
//! dictionary trie and keeps only prefixes whose letters the finger passed
//! in order (a cheap geometric feasibility test), then scores the surviving
//! words with
//!
//! * a monotonic alignment of each letter's key to the gesture path,
//! * the location distance between the resampled gesture and the ideal
//!   polyline through the word's keys,
//! * the scale/translation invariant shape distance of both,
//!
//! combined with word frequency, personal history and context. All
//! coordinates are in key-width units.

use glissando_lang::dict::Dict;
use glissando_lang::model::{logf, Language};
use std::collections::HashMap;

pub const RESAMPLE: usize = 48;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pt {
    pub x: f32,
    pub y: f32,
}

impl Pt {
    fn d(self, o: Pt) -> f32 {
        ((self.x - o.x).powi(2) + (self.y - o.y).powi(2)).sqrt()
    }
}

/// Key centres of the letters on the current layout, in key-width units.
#[derive(Debug, Clone, Default)]
pub struct KeyMap {
    keys: HashMap<char, Pt>,
}

impl KeyMap {
    pub fn new(keys: impl IntoIterator<Item = (char, f32, f32)>) -> KeyMap {
        let mut m = HashMap::new();
        for (c, x, y) in keys {
            for l in c.to_lowercase() {
                m.insert(l, Pt { x, y });
            }
        }
        KeyMap { keys: m }
    }

    /// Where a word character has to be swiped (folding letters that live
    /// in long-press popups onto their base key).
    pub fn pos(&self, c: char) -> Option<Pt> {
        let l = c.to_lowercase().next().unwrap_or(c);
        if let Some(p) = self.keys.get(&l) {
            return Some(*p);
        }
        let base = match l {
            'ё' => 'е',
            'ъ' => 'ь',
            'ß' => 's',
            _ => glissando_core::text::fold_char(l),
        };
        self.keys.get(&base).copied()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GestureCandidate {
    pub word: String,
    pub score: f32,
    pub lang: usize,
}

pub struct DecodeParams {
    /// Radius around a key centre that counts as passing the key.
    pub pass_radius: f32,
    /// Allowed distance of the first/last letter from the gesture ends.
    pub end_radius: f32,
    pub max_results: usize,
    pub max_expansions: usize,
}

impl Default for DecodeParams {
    fn default() -> Self {
        DecodeParams { pass_radius: 0.95, end_radius: 1.35, max_results: 8, max_expansions: 400_000 }
    }
}

/// Resample a polyline to `n` equidistant points.
pub fn resample(points: &[Pt], n: usize) -> Vec<Pt> {
    if points.is_empty() {
        return Vec::new();
    }
    let total: f32 = points.windows(2).map(|w| w[0].d(w[1])).sum();
    if total <= 1e-4 || points.len() == 1 {
        return vec![points[0]; n];
    }
    let step = total / (n - 1) as f32;
    let mut out = Vec::with_capacity(n);
    out.push(points[0]);
    let mut acc = 0.0;
    let mut prev = points[0];
    let mut i = 1;
    while i < points.len() && out.len() < n - 1 {
        let cur = points[i];
        let d = prev.d(cur);
        if acc + d >= step && d > 0.0 {
            let t = (step - acc) / d;
            let q = Pt { x: prev.x + t * (cur.x - prev.x), y: prev.y + t * (cur.y - prev.y) };
            out.push(q);
            prev = q;
            acc = 0.0;
        } else {
            acc += d;
            prev = cur;
            i += 1;
        }
    }
    while out.len() < n {
        out.push(*points.last().unwrap());
    }
    out
}

fn normalize(points: &[Pt]) -> Vec<Pt> {
    let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let (mut cx, mut cy) = (0.0, 0.0);
    for p in points {
        minx = minx.min(p.x);
        miny = miny.min(p.y);
        maxx = maxx.max(p.x);
        maxy = maxy.max(p.y);
        cx += p.x;
        cy += p.y;
    }
    let n = points.len() as f32;
    cx /= n;
    cy /= n;
    let s = (maxx - minx).max(maxy - miny).max(0.5);
    points.iter().map(|p| Pt { x: (p.x - cx) / s, y: (p.y - cy) / s }).collect()
}

fn mean_dist(a: &[Pt], b: &[Pt]) -> f32 {
    a.iter().zip(b).map(|(p, q)| p.d(*q)).sum::<f32>() / a.len().max(1) as f32
}

/// Remove jitter and very close duplicate points of a raw trace.
pub fn clean_trace(raw: &[(f32, f32)], min_step: f32) -> Vec<Pt> {
    let mut out: Vec<Pt> = Vec::with_capacity(raw.len());
    for &(x, y) in raw {
        let p = Pt { x, y };
        if out.last().map(|l| l.d(p) >= min_step).unwrap_or(true) {
            out.push(p);
        }
    }
    if let (Some(&(x, y)), Some(last)) = (raw.last(), out.last()) {
        if last.x != x || last.y != y {
            out.push(Pt { x, y });
        }
    }
    out
}

struct Prepared {
    path: Vec<Pt>,
    norm: Vec<Pt>,
    start: Pt,
    end: Pt,
    length: f32,
}

/// Letter positions of a word; consecutive duplicates collapse. `None` if
/// the word contains letters that cannot be glided on this layout.
fn word_keys(word: &str, keys: &KeyMap) -> Option<Vec<Pt>> {
    let mut v: Vec<Pt> = Vec::new();
    for c in word.chars() {
        if c == '\'' || c == '’' || c == '-' {
            continue;
        }
        let p = keys.pos(c)?;
        if v.last().map(|l| l.d(p) > 1e-3).unwrap_or(true) {
            v.push(p);
        }
    }
    Some(v)
}

/// Monotonic alignment of letters to path points: mean squared distance.
fn alignment_cost(letters: &[Pt], path: &[Pt]) -> f32 {
    let n = path.len();
    let l = letters.len();
    if l == 0 {
        return f32::MAX;
    }
    let mut prev = vec![f32::MAX; n];
    // First letter anchored to the start region.
    for (i, p) in path.iter().enumerate().take(n / 4 + 1) {
        let d = letters[0].d(*p);
        prev[i] = d * d + i as f32 * 0.01;
    }
    for k in 1..l {
        let mut cur = vec![f32::MAX; n];
        let mut best = f32::MAX;
        for i in 0..n {
            best = best.min(prev[i]);
            if best < f32::MAX {
                let d = letters[k].d(path[i]);
                cur[i] = best + d * d;
            }
        }
        prev = cur;
    }
    // Last letter anchored to the end region.
    let tail = prev[n - 1 - n / 6..].iter().copied().fold(f32::MAX, f32::min);
    tail / l as f32
}

pub struct ShapeDecoder {
    pub params: DecodeParams,
}

impl Default for ShapeDecoder {
    fn default() -> Self {
        ShapeDecoder { params: DecodeParams::default() }
    }
}

impl ShapeDecoder {
    fn prepare(&self, trace: &[Pt]) -> Option<Prepared> {
        if trace.len() < 2 {
            return None;
        }
        let length: f32 = trace.windows(2).map(|w| w[0].d(w[1])).sum();
        let path = resample(trace, RESAMPLE);
        let norm = normalize(&path);
        Some(Prepared { start: trace[0], end: *trace.last().unwrap(), path, norm, length })
    }

    /// Words whose letters the gesture visits in order, from one dictionary.
    fn feasible(&self, dict: &Dict, keys: &KeyMap, g: &Prepared, out: &mut Vec<(u32, u8)>) {
        let r = self.params.pass_radius;
        let path = &g.path;
        // Precompute, per key position, the path indices within radius.
        let mut near_cache: HashMap<(i32, i32), Vec<bool>> = HashMap::new();
        let mut near = |p: Pt| -> Vec<bool> {
            let k = ((p.x * 100.0) as i32, (p.y * 100.0) as i32);
            near_cache.entry(k).or_insert_with(|| path.iter().map(|q| q.d(p) <= r).collect()).clone()
        };
        // DFS state: node, next usable path index, last key position.
        let root = dict.node(0);
        let mut stack: Vec<(u32, usize, Option<Pt>, u8)> = Vec::new();
        for (cid, child) in dict.children(&root) {
            if let Some(p) = keys.pos(child.ch) {
                if p.d(g.start) <= self.params.end_radius {
                    stack.push((cid, 0, Some(p), 1));
                }
            }
        }
        let mut expansions = 0;
        while let Some((id, idx, last, depth)) = stack.pop() {
            expansions += 1;
            if expansions > self.params.max_expansions {
                break;
            }
            let node = dict.node(id);
            if node.freq > 0 && depth >= 2 {
                if let Some(lp) = last {
                    if lp.d(g.end) <= self.params.end_radius {
                        out.push((id, node.freq));
                    }
                }
            }
            if depth >= 24 {
                continue;
            }
            for (cid, child) in dict.children(&node) {
                if child.max_freq == 0 {
                    continue;
                }
                if child.ch == '\'' || child.ch == '’' || child.ch == '-' {
                    stack.push((cid, idx, last, depth));
                    continue;
                }
                let Some(p) = keys.pos(child.ch) else { continue };
                if last.map(|l| l.d(p) < 1e-3).unwrap_or(false) {
                    // Double letter: same key, same place on the path.
                    stack.push((cid, idx, last, depth + 1));
                    continue;
                }
                let nm = near(p);
                if let Some(j) = (idx..path.len()).find(|&j| nm[j]) {
                    stack.push((cid, j, Some(p), depth + 1));
                }
            }
        }
    }

    fn shape_cost(&self, letters: &[Pt], g: &Prepared) -> f32 {
        let tmpl = resample(letters, RESAMPLE);
        let loc = mean_dist(&g.path, &tmpl);
        let shape = mean_dist(&g.norm, &normalize(&tmpl));
        let align = alignment_cost(letters, &g.path);
        let tlen: f32 = letters.windows(2).map(|w| w[0].d(w[1])).sum();
        let len_pen = ((g.length - tlen).abs() / (tlen + 1.0)).min(2.0);
        2.4 * align + 1.3 * loc + 1.6 * shape + 0.8 * len_pen
    }

    /// Decode a gesture. `trace` is in key-width units. `active` are
    /// (language index, log prior) pairs; `prev` is the previous word.
    pub fn decode(
        &self,
        trace: &[Pt],
        keys: &KeyMap,
        langs: &[Language],
        active: &[(usize, f32)],
        prev: Option<&str>,
    ) -> Vec<GestureCandidate> {
        let Some(g) = self.prepare(trace) else { return Vec::new() };
        let mut scored: Vec<GestureCandidate> = Vec::new();
        for &(li, prior) in active {
            let lang = &langs[li];
            let mut words: Vec<(String, u8)> = Vec::new();
            if let Some(d) = &lang.dict {
                let mut ids = Vec::new();
                self.feasible(d, keys, &g, &mut ids);
                words.extend(ids.into_iter().map(|(id, f)| (d.word(id), f)));
            }
            for (w, u) in lang.user.words.iter() {
                if u.blocked || !(u.pinned || u.count >= glissando_lang::user::LEARN_THRESHOLD) {
                    continue;
                }
                if w.chars().count() >= 2 && !words.iter().any(|(x, _)| x == w) {
                    words.push((w.clone(), 0));
                }
            }
            for (w, f) in words {
                if lang.user.is_blocked(&w) {
                    continue;
                }
                let Some(letters) = word_keys(&w, keys) else { continue };
                if letters.is_empty() {
                    continue;
                }
                if letters[0].d(g.start) > self.params.end_radius || letters.last().unwrap().d(g.end) > self.params.end_radius {
                    continue;
                }
                let cost = self.shape_cost(&letters, &g);
                let freq = if f > 0 { logf(f) } else { lang.dict_logf(&w).unwrap_or(-6.0) };
                let ctx = context_bonus(lang, prev, &w);
                let score = -cost * 2.2 + freq + lang.user.boost(&w) + ctx + prior;
                scored.push(GestureCandidate { word: w, score, lang: li });
            }
        }
        scored.sort_by(|a, b| b.score.total_cmp(&a.score));
        let mut seen: Vec<String> = Vec::new();
        scored.retain(|c| {
            let k = c.word.to_lowercase();
            if seen.contains(&k) {
                false
            } else {
                seen.push(k);
                true
            }
        });
        scored.truncate(self.params.max_results);
        scored
    }
}

fn context_bonus(lang: &Language, prev: Option<&str>, word: &str) -> f32 {
    let Some(prev) = prev else { return 0.0 };
    let mut b = 0.0;
    if let Some(d) = &lang.dict {
        if let (Some((p, _)), Some((w, _))) = (d.lookup_any_case(prev), d.lookup(word)) {
            if let Some(f) = d.bigram(p, w) {
                b += 0.5 + f as f32 / 255.0 * 2.0;
            }
        }
    }
    let c = lang.user.bigram(prev, word);
    if c > 0 {
        b += 0.6 + (c as f32).ln() * 0.6;
    }
    b
}

/// Synthesise a plausible human gesture for `word` (tests and benchmarks):
/// a smoothed polyline through the key centres with seeded noise.
pub fn synth_gesture(word: &str, keys: &KeyMap, noise: f32, seed: u64) -> Vec<Pt> {
    let mut rng = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut rnd = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        ((rng >> 11) as f64 / (1u64 << 53) as f64) as f32 * 2.0 - 1.0
    };
    let Some(letters) = word_keys(word, keys) else { return Vec::new() };
    let pts: Vec<Pt> = letters.iter().map(|p| Pt { x: p.x + rnd() * noise, y: p.y + rnd() * noise * 0.8 }).collect();
    let mut out = Vec::new();
    if pts.len() == 1 {
        return vec![pts[0], Pt { x: pts[0].x + 0.05, y: pts[0].y }];
    }
    // Corner cutting: humans do not hit key centres exactly.
    for w in pts.windows(2) {
        let steps = ((w[0].d(w[1]) / 0.15).ceil() as usize).max(2);
        for s in 0..steps {
            let t = s as f32 / steps as f32;
            let sag = (t * std::f32::consts::PI).sin() * noise * 0.5 * rnd();
            out.push(Pt { x: w[0].x + (w[1].x - w[0].x) * t + sag, y: w[0].y + (w[1].y - w[0].y) * t + sag });
        }
    }
    out.push(*pts.last().unwrap());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use glissando_lang::dict::DictBuilder;
    use glissando_lang::user::UserModel;

    pub fn qwerty() -> KeyMap {
        let rows = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
        let offs = [0.0, 0.5, 1.5];
        let mut keys = Vec::new();
        for (r, row) in rows.iter().enumerate() {
            for (i, c) in row.chars().enumerate() {
                keys.push((c, offs[r] + i as f32 + 0.5, r as f32 * 1.4 + 0.7));
            }
        }
        KeyMap::new(keys)
    }

    fn lang(words: &[(&str, u8)]) -> Language {
        let mut b = DictBuilder::default();
        for (w, f) in words {
            b.add_word(w, *f);
        }
        Language::new("en", Some(Dict::from_bytes(b.build()).unwrap()), UserModel::in_memory())
    }

    #[test]
    fn resample_is_equidistant() {
        let r = resample(&[Pt { x: 0.0, y: 0.0 }, Pt { x: 10.0, y: 0.0 }], 11);
        assert_eq!(r.len(), 11);
        assert!((r[5].x - 5.0).abs() < 1e-3);
    }

    #[test]
    fn decodes_clean_gestures() {
        let keys = qwerty();
        let words = [
            ("hello", 200),
            ("help", 190),
            ("world", 190),
            ("word", 200),
            ("the", 230),
            ("they", 210),
            ("this", 220),
            ("keyboard", 150),
            ("good", 200),
            ("food", 170),
            ("quick", 160),
            ("brown", 150),
        ];
        let l = vec![lang(&words)];
        let dec = ShapeDecoder::default();
        for (w, _) in words {
            for seed in 0..3 {
                let g = synth_gesture(w, &keys, 0.18, seed);
                let r = dec.decode(&g, &keys, &l, &[(0, 0.0)], None);
                assert!(!r.is_empty(), "{w}: no result");
                let top3: Vec<&str> = r.iter().take(3).map(|c| c.word.as_str()).collect();
                assert!(top3.contains(&w), "{w} (seed {seed}) not in {top3:?}");
            }
        }
    }

    #[test]
    fn cyrillic_and_folded_letters() {
        let rows = ["йцукенгшщзх", "фывапролджэ", "ячсмитьбю"];
        let offs = [0.0, 0.0, 1.0];
        let mut k = Vec::new();
        for (r, row) in rows.iter().enumerate() {
            for (i, c) in row.chars().enumerate() {
                k.push((c, offs[r] + i as f32 + 0.5, r as f32 * 1.4 + 0.7));
            }
        }
        let keys = KeyMap::new(k);
        let l = vec![lang(&[("привет", 200), ("ещё", 200), ("подъезд", 150), ("пока", 190)])];
        let dec = ShapeDecoder::default();
        for w in ["привет", "ещё", "подъезд"] {
            let g = synth_gesture(w, &keys, 0.12, 7);
            let r = dec.decode(&g, &keys, &l, &[(0, 0.0)], None);
            assert_eq!(r.first().map(|c| c.word.as_str()), Some(w));
        }
    }
}
