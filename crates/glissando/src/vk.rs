//! XKB keymap for the virtual keyboard. Navigation/editing keys use their
//! real evdev codes; arbitrary characters (fallback typing when no input
//! method client is focused) get dynamically assigned codes, wtype style.

use crate::keyboard::NavKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VkKey {
    BackSpace,
    Return,
    Tab,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Delete,
    Ctrl,
    Shift,
}

impl From<NavKey> for VkKey {
    fn from(n: NavKey) -> VkKey {
        match n {
            NavKey::Left => VkKey::Left,
            NavKey::Right => VkKey::Right,
            NavKey::Up => VkKey::Up,
            NavKey::Down => VkKey::Down,
            NavKey::Home => VkKey::Home,
            NavKey::End => VkKey::End,
            NavKey::Tab => VkKey::Tab,
            NavKey::Escape => VkKey::Escape,
        }
    }
}

/// (key, evdev code, xkb key name, keysym)
const FIXED: &[(VkKey, u32, &str, &str)] = &[
    (VkKey::Escape, 1, "ESC", "Escape"),
    (VkKey::BackSpace, 14, "BKSP", "BackSpace"),
    (VkKey::Tab, 15, "TAB", "Tab"),
    (VkKey::Return, 28, "RTRN", "Return"),
    (VkKey::Ctrl, 29, "LCTL", "Control_L"),
    (VkKey::Shift, 42, "LFSH", "Shift_L"),
    (VkKey::Home, 102, "HOME", "Home"),
    (VkKey::Up, 103, "UP", "Up"),
    (VkKey::Left, 105, "LEFT", "Left"),
    (VkKey::Right, 106, "RGHT", "Right"),
    (VkKey::End, 107, "END", "End"),
    (VkKey::Down, 108, "DOWN", "Down"),
    (VkKey::Delete, 111, "DELE", "Delete"),
];

/// Modifier masks in the order of the "complete" types/compat.
pub const MOD_SHIFT: u32 = 1;
pub const MOD_CTRL: u32 = 4;

/// First evdev code used for dynamically mapped characters.
const DYN_BASE: u32 = 184;
const DYN_SLOTS: usize = 60;

#[derive(Debug, Default)]
pub struct Keymap {
    dynamic: Vec<char>,
    /// Bumped when the keymap text changes and must be re-uploaded.
    pub generation: u64,
}

impl Keymap {
    pub fn code(&self, k: VkKey) -> u32 {
        FIXED.iter().find(|f| f.0 == k).map(|f| f.1).unwrap()
    }

    /// Code for a character, adding it to the keymap if needed. Returns
    /// (evdev code, keymap changed).
    pub fn char_code(&mut self, c: char) -> (u32, bool) {
        if let Some(i) = self.dynamic.iter().position(|&d| d == c) {
            return (DYN_BASE + i as u32, false);
        }
        if self.dynamic.len() >= DYN_SLOTS {
            self.dynamic.remove(0);
        }
        self.dynamic.push(c);
        self.generation += 1;
        let i = self.dynamic.len() - 1;
        (DYN_BASE + i as u32, true)
    }

    pub fn text(&self) -> String {
        let mut codes = String::new();
        let mut syms = String::new();
        for (_, code, name, sym) in FIXED {
            codes.push_str(&format!("    <{name}> = {};\n", code + 8));
            syms.push_str(&format!("    key <{name}> {{ [ {sym} ] }};\n"));
        }
        for (i, c) in self.dynamic.iter().enumerate() {
            let code = DYN_BASE + i as u32 + 8;
            codes.push_str(&format!("    <I{code}> = {code};\n"));
            syms.push_str(&format!("    key <I{code}> {{ [ U{:04X} ] }};\n", *c as u32));
        }
        format!(
            "xkb_keymap {{\n  xkb_keycodes \"glissando\" {{\n    minimum = 8;\n    maximum = 255;\n{codes}  }};\n  \
             xkb_types \"glissando\" {{ include \"complete\" }};\n  \
             xkb_compatibility \"glissando\" {{ include \"complete\" }};\n  \
             xkb_symbols \"glissando\" {{\n{syms}    modifier_map Control {{ <LCTL> }};\n    modifier_map Shift {{ <LFSH> }};\n  }};\n}};\n"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keymap_text_contains_dynamic_chars() {
        let mut k = Keymap::default();
        let (c1, changed) = k.char_code('ж');
        assert!(changed);
        let (c2, changed) = k.char_code('ж');
        assert!(!changed);
        assert_eq!(c1, c2);
        let t = k.text();
        assert!(t.contains("U0436"));
        assert!(t.contains("<BKSP> = 22;"));
        assert_eq!(k.code(VkKey::Return), 28);
    }
}
