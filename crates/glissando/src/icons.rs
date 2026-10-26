//! Vector icons drawn with cairo paths. All artwork is original to this
//! project; nothing is taken from other keyboards.

use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Shift,
    ShiftOn,
    CapsLock,
    Backspace,
    Enter,
    Globe,
    Emoji,
    Settings,
    Mic,
    Clipboard,
    Hide,
    Search,
    OneHanded,
    Menu,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Keyboard,
    Close,
    Pin,
    Trash,
    Recent,
}

/// Draw `icon` centred at (cx, cy) inside a box of `size`.
/// The current source colour is used.
pub fn draw(cr: &cairo::Context, icon: Icon, cx: f64, cy: f64, size: f64) {
    cr.save().ok();
    cr.translate(cx, cy);
    let s = size / 24.0;
    cr.scale(s, s);
    cr.set_line_width(1.9);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.set_line_join(cairo::LineJoin::Round);
    match icon {
        Icon::Shift | Icon::ShiftOn | Icon::CapsLock => {
            let up = if icon == Icon::CapsLock { -2.5 } else { 0.0 };
            cr.move_to(0.0, -9.0 + up);
            cr.line_to(8.5, 0.5 + up);
            cr.line_to(4.0, 0.5 + up);
            cr.line_to(4.0, 7.0 + up);
            cr.line_to(-4.0, 7.0 + up);
            cr.line_to(-4.0, 0.5 + up);
            cr.line_to(-8.5, 0.5 + up);
            cr.close_path();
            if icon == Icon::Shift {
                cr.stroke().ok();
            } else {
                cr.fill_preserve().ok();
                cr.stroke().ok();
            }
            if icon == Icon::CapsLock {
                cr.rectangle(-4.0, 7.5, 8.0, 2.4);
                cr.fill().ok();
            }
        }
        Icon::Backspace => {
            cr.move_to(-10.0, 0.0);
            cr.line_to(-4.5, -7.0);
            cr.line_to(9.5, -7.0);
            cr.curve_to(10.5, -7.0, 11.0, -6.5, 11.0, -5.5);
            cr.line_to(11.0, 5.5);
            cr.curve_to(11.0, 6.5, 10.5, 7.0, 9.5, 7.0);
            cr.line_to(-4.5, 7.0);
            cr.close_path();
            cr.stroke().ok();
            cr.move_to(-0.5, -3.0);
            cr.line_to(5.5, 3.0);
            cr.move_to(5.5, -3.0);
            cr.line_to(-0.5, 3.0);
            cr.stroke().ok();
        }
        Icon::Enter => {
            cr.move_to(8.0, -7.0);
            cr.line_to(8.0, 1.0);
            cr.curve_to(8.0, 2.5, 7.0, 3.0, 6.0, 3.0);
            cr.line_to(-8.0, 3.0);
            cr.stroke().ok();
            cr.move_to(-4.0, -1.5);
            cr.line_to(-8.5, 3.0);
            cr.line_to(-4.0, 7.5);
            cr.stroke().ok();
        }
        Icon::Globe => {
            cr.arc(0.0, 0.0, 9.0, 0.0, 2.0 * PI);
            cr.stroke().ok();
            cr.save().ok();
            cr.scale(0.45, 1.0);
            cr.arc(0.0, 0.0, 9.0, 0.0, 2.0 * PI);
            cr.restore().ok();
            cr.stroke().ok();
            cr.move_to(-9.0, 0.0);
            cr.line_to(9.0, 0.0);
            cr.move_to(-7.6, -4.6);
            cr.line_to(7.6, -4.6);
            cr.move_to(-7.6, 4.6);
            cr.line_to(7.6, 4.6);
            cr.stroke().ok();
        }
        Icon::Emoji => {
            cr.arc(0.0, 0.0, 9.0, 0.0, 2.0 * PI);
            cr.stroke().ok();
            cr.arc(-3.2, -2.5, 1.3, 0.0, 2.0 * PI);
            cr.fill().ok();
            cr.arc(3.2, -2.5, 1.3, 0.0, 2.0 * PI);
            cr.fill().ok();
            cr.arc(0.0, 1.0, 5.0, 0.2 * PI, 0.8 * PI);
            cr.stroke().ok();
        }
        Icon::Settings => {
            let teeth = 8;
            for i in 0..teeth {
                let a = i as f64 * 2.0 * PI / teeth as f64;
                cr.save().ok();
                cr.rotate(a);
                cr.rectangle(-1.8, -10.0, 3.6, 4.0);
                cr.fill().ok();
                cr.restore().ok();
            }
            cr.arc(0.0, 0.0, 6.8, 0.0, 2.0 * PI);
            cr.stroke().ok();
            cr.arc(0.0, 0.0, 2.6, 0.0, 2.0 * PI);
            cr.stroke().ok();
        }
        Icon::Mic => {
            rounded(cr, -3.5, -10.0, 7.0, 12.0, 3.5);
            cr.stroke().ok();
            cr.arc(0.0, -1.0, 7.0, 0.0, PI);
            cr.stroke().ok();
            cr.move_to(0.0, 6.0);
            cr.line_to(0.0, 9.5);
            cr.stroke().ok();
        }
        Icon::Clipboard => {
            rounded(cr, -7.5, -8.0, 15.0, 18.0, 2.0);
            cr.stroke().ok();
            rounded(cr, -3.5, -10.5, 7.0, 4.5, 1.5);
            cr.fill().ok();
            cr.move_to(-3.5, -1.0);
            cr.line_to(3.5, -1.0);
            cr.move_to(-3.5, 3.5);
            cr.line_to(2.0, 3.5);
            cr.stroke().ok();
        }
        Icon::Hide => {
            cr.move_to(-7.0, -3.0);
            cr.line_to(0.0, 4.0);
            cr.line_to(7.0, -3.0);
            cr.stroke().ok();
        }
        Icon::Search => {
            cr.arc(-2.0, -2.0, 6.5, 0.0, 2.0 * PI);
            cr.stroke().ok();
            cr.move_to(2.8, 2.8);
            cr.line_to(8.5, 8.5);
            cr.stroke().ok();
        }
        Icon::OneHanded => {
            rounded(cr, -2.0, -9.0, 11.0, 18.0, 2.0);
            cr.stroke().ok();
            cr.move_to(-5.0, -4.0);
            cr.line_to(-9.0, 0.0);
            cr.line_to(-5.0, 4.0);
            cr.stroke().ok();
        }
        Icon::Menu => {
            for (x, y) in [(-5.5, -5.5), (5.5, -5.5), (-5.5, 5.5), (5.5, 5.5)] {
                cr.arc(x, y, 2.4, 0.0, 2.0 * PI);
                cr.fill().ok();
            }
        }
        Icon::Tab => {
            cr.move_to(-8.0, 0.0);
            cr.line_to(6.0, 0.0);
            cr.move_to(2.0, -4.0);
            cr.line_to(6.0, 0.0);
            cr.line_to(2.0, 4.0);
            cr.move_to(9.0, -6.0);
            cr.line_to(9.0, 6.0);
            cr.stroke().ok();
        }
        Icon::Left | Icon::Right | Icon::Up | Icon::Down => {
            let a = match icon {
                Icon::Left => PI,
                Icon::Up => -PI / 2.0,
                Icon::Down => PI / 2.0,
                _ => 0.0,
            };
            cr.rotate(a);
            cr.move_to(-7.0, 0.0);
            cr.line_to(7.0, 0.0);
            cr.move_to(2.0, -5.0);
            cr.line_to(7.0, 0.0);
            cr.line_to(2.0, 5.0);
            cr.stroke().ok();
        }
        Icon::Keyboard => {
            rounded(cr, -10.0, -6.5, 20.0, 13.0, 2.0);
            cr.stroke().ok();
            for row in 0..2 {
                for col in 0..4 {
                    cr.rectangle(-7.5 + col as f64 * 4.3, -4.0 + row as f64 * 3.6, 2.2, 2.0);
                }
            }
            cr.fill().ok();
            cr.rectangle(-5.0, 3.2, 10.0, 1.6);
            cr.fill().ok();
        }
        Icon::Close => {
            cr.move_to(-6.0, -6.0);
            cr.line_to(6.0, 6.0);
            cr.move_to(6.0, -6.0);
            cr.line_to(-6.0, 6.0);
            cr.stroke().ok();
        }
        Icon::Pin => {
            cr.move_to(-4.0, -9.0);
            cr.line_to(4.0, -9.0);
            cr.move_to(-3.0, -9.0);
            cr.line_to(-3.0, -2.0);
            cr.line_to(-6.0, 2.0);
            cr.line_to(6.0, 2.0);
            cr.line_to(3.0, -2.0);
            cr.line_to(3.0, -9.0);
            cr.move_to(0.0, 2.0);
            cr.line_to(0.0, 9.5);
            cr.stroke().ok();
        }
        Icon::Trash => {
            cr.move_to(-8.0, -6.0);
            cr.line_to(8.0, -6.0);
            cr.move_to(-3.0, -6.0);
            cr.line_to(-2.0, -9.0);
            cr.line_to(2.0, -9.0);
            cr.line_to(3.0, -6.0);
            cr.stroke().ok();
            cr.move_to(-6.0, -6.0);
            cr.line_to(-5.0, 9.0);
            cr.line_to(5.0, 9.0);
            cr.line_to(6.0, -6.0);
            cr.stroke().ok();
        }
        Icon::Recent => {
            cr.arc(0.0, 0.0, 9.0, 0.0, 2.0 * PI);
            cr.stroke().ok();
            cr.move_to(0.0, -5.0);
            cr.line_to(0.0, 0.0);
            cr.line_to(4.0, 3.0);
            cr.stroke().ok();
        }
    }
    cr.restore().ok();
}

pub fn rounded(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0);
    cr.arc(x + r, y + h - r, r, PI / 2.0, PI);
    cr.arc(x + r, y + r, r, PI, 1.5 * PI);
    cr.close_path();
}
