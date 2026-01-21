use glib::Unichar;
use gtk::prelude::*;
use gtk::DrawingArea;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub base: String,
    #[serde(default)]
    pub label: Option<String>,
    pub shift: Option<String>,
    pub altgr: Option<String>,
    pub finger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierKey {
    pub label: String,
    pub finger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
    pub name: String,
    pub keys: Vec<Vec<KeyInfo>>,
    pub space: KeyInfo,
    #[serde(default)]
    pub modifiers: HashMap<String, ModifierKey>,
}

impl KeyboardLayout {
    pub fn load_from_json(layout_code: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json_data = match layout_code {
            "us" => include_str!("../data/keyboard_layouts/us.json"),
            "es" | "gl" => include_str!("../data/keyboard_layouts/es.json"),
            "it" => include_str!("../data/keyboard_layouts/it.json"),
            "pl" => include_str!("../data/keyboard_layouts/pl.json"),
            _ => return Err(format!("Unsupported layout: {}", layout_code).into()),
        };
        Ok(serde_json::from_str(json_data)?)
    }
}

impl Default for KeyboardLayout {
    fn default() -> Self {
        Self::load_from_json("us").unwrap_or_else(|_| Self {
            name: "US".to_string(),
            keys: vec![vec![
                KeyInfo {
                    base: "q".to_string(),
                    label: None,
                    shift: Some("Q".to_string()),
                    altgr: None,
                    finger: "left_pinky".to_string(),
                },
                KeyInfo {
                    base: "w".to_string(),
                    label: None,
                    shift: Some("W".to_string()),
                    altgr: None,
                    finger: "left_ring".to_string(),
                },
                KeyInfo {
                    base: "e".to_string(),
                    label: None,
                    shift: Some("E".to_string()),
                    altgr: None,
                    finger: "left_middle".to_string(),
                },
            ]],
            space: KeyInfo {
                base: " ".to_string(),
                label: Some("SPACE".to_string()),
                shift: None,
                altgr: None,
                finger: "both_thumbs".to_string(),
            },
            modifiers: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_json_us() {
        let layout = KeyboardLayout::load_from_json("us").unwrap();
        assert_eq!(layout.name, "US QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_es() {
        let layout = KeyboardLayout::load_from_json("es").unwrap();
        assert_eq!(layout.name, "Spanish QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_it() {
        let layout = KeyboardLayout::load_from_json("it").unwrap();
        assert_eq!(layout.name, "Italian QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_pl() {
        let layout = KeyboardLayout::load_from_json("pl").unwrap();
        assert_eq!(layout.name, "Polish QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_invalid() {
        let result = KeyboardLayout::load_from_json("invalid");
        assert!(result.is_err());
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardWidget {
    drawing_area: DrawingArea,
    current_key: Rc<RefCell<Option<char>>>,
    visible_keys: Rc<RefCell<Option<std::collections::HashSet<char>>>>,
    current_key_sequence: Rc<RefCell<Vec<char>>>,
    sequence_index: Rc<RefCell<usize>>,
}

impl KeyboardWidget {
    pub fn new() -> Self {
        let layout_code = crate::utils::language_from_locale();
        let layout = Rc::new(RefCell::new(
            KeyboardLayout::load_from_json(layout_code).unwrap_or_default(),
        ));
        let drawing_area = DrawingArea::new();

        // Calculate keyboard dimensions
        let key_width = 50.0;
        let key_height = 50.0;
        let key_spacing = 5.0;
        let row_spacing = 5.0;

        // Calculate width for each row and find the maximum
        let layout_borrowed = layout.borrow();

        // Row 0: number keys + backspace
        let row0_keys = layout_borrowed.keys.first().map(|r| r.len()).unwrap_or(12);
        let row0_width = row0_keys as f64 * key_width
            + (row0_keys - 1) as f64 * key_spacing
            + key_spacing
            + key_width * 2.0;

        // Row 1: Tab + QWERTY + Enter
        let row1_keys = layout_borrowed.keys.get(1).map(|r| r.len()).unwrap_or(12);
        let row1_width = key_width * 1.5
            + key_spacing
            + row1_keys as f64 * key_width
            + (row1_keys - 1) as f64 * key_spacing
            + key_spacing
            + key_width * 1.75;

        // Row 2: Caps + home row + Enter - overlap
        // The last key in row2 overlaps with Enter position, so we subtract two spacings
        let row2_keys = layout_borrowed.keys.get(2).map(|r| r.len()).unwrap_or(12);
        let row2_width = key_width * 1.75
            + key_spacing
            + row2_keys as f64 * key_width
            + row2_keys as f64 * key_spacing
            + key_width * 1.75
            - key_spacing * 2.0;

        // Row 3: Left Shift + bottom row + Right Shift
        let row3_keys = layout_borrowed.keys.get(3).map(|r| r.len()).unwrap_or(11);
        let row3_width = key_width * 1.25
            + key_spacing
            + row3_keys as f64 * key_width
            + (row3_keys - 1) as f64 * key_spacing
            + key_spacing
            + key_width * 3.0;

        // Row 4: Ctrl + Alt + Space + Alt + Ctrl
        let row4_width = key_width * 1.5
            + key_spacing
            + key_width * 1.3
            + key_spacing
            + key_width * 6.0
            + key_spacing
            + key_width * 1.3
            + key_spacing
            + key_width * 1.5;

        let keyboard_width = row0_width
            .max(row1_width)
            .max(row2_width)
            .max(row3_width)
            .max(row4_width) as i32;
        let keyboard_height = (5.0 * key_height + 4.0 * row_spacing) as i32;
        drop(layout_borrowed);

        drawing_area.set_size_request(keyboard_width, keyboard_height);

        let current_key = Rc::new(RefCell::new(None));
        let visible_keys = Rc::new(RefCell::new(None));
        let current_key_sequence = Rc::new(RefCell::new(Vec::new()));
        let sequence_index = Rc::new(RefCell::new(0));
        let current_key_clone = current_key.clone();
        let visible_keys_clone = visible_keys.clone();
        let layout_clone = layout.clone();

        drawing_area.set_draw_func(move |widget, cr, _width, _height| {
            Self::draw_keyboard(
                widget,
                cr,
                &current_key_clone,
                &layout_clone,
                &visible_keys_clone,
            );
        });

        Self {
            drawing_area,
            current_key,
            visible_keys,
            current_key_sequence,
            sequence_index,
        }
    }

    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }

    pub fn set_current_key(&self, key: Option<char>) {
        // Check if this is a composed character that needs decomposition
        if let Some(ch) = key {
            if let glib::CharacterDecomposition::Pair(dead_key, base_char) = ch.decompose() {
                // This is a composed character - set up sequence
                *self.current_key_sequence.borrow_mut() = vec![dead_key, base_char];
                *self.sequence_index.borrow_mut() = 0;
                *self.current_key.borrow_mut() = Some(dead_key);
            } else {
                // Regular character - no sequence
                *self.current_key_sequence.borrow_mut() = Vec::new();
                *self.sequence_index.borrow_mut() = 0;
                *self.current_key.borrow_mut() = key;
            }
        } else {
            // No key - clear everything
            *self.current_key_sequence.borrow_mut() = Vec::new();
            *self.sequence_index.borrow_mut() = 0;
            *self.current_key.borrow_mut() = None;
        }

        self.drawing_area.queue_draw();
    }

    pub fn advance_sequence(&self) {
        let sequence = self.current_key_sequence.borrow();
        let mut index = self.sequence_index.borrow_mut();

        if !sequence.is_empty() && *index < sequence.len() - 1 {
            *index += 1;
            *self.current_key.borrow_mut() = Some(sequence[*index]);
            self.drawing_area.queue_draw();
        }
    }

    pub fn set_visible_keys(&self, keys: Option<HashSet<char>>) {
        *self.visible_keys.borrow_mut() = keys;
        self.drawing_area.queue_draw();
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_single_key(
        cr: &gtk::cairo::Context,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        key_info: Option<&KeyInfo>,
        label: Option<&str>,
        is_current: bool,
        should_show_text: bool,
        key_color: (f64, f64, f64),
        key_current_color: (f64, f64, f64),
        key_text_color: (f64, f64, f64),
        key_current_text_color: (f64, f64, f64),
        key_border_color: (f64, f64, f64),
    ) {
        let (r, g, b) = if is_current {
            key_current_color
        } else {
            key_color
        };
        cr.set_source_rgb(r, g, b);
        cr.rectangle(x, y, width, height);
        cr.fill().unwrap();

        cr.set_source_rgb(key_border_color.0, key_border_color.1, key_border_color.2);
        cr.set_line_width(1.0);
        cr.rectangle(x, y, width, height);
        cr.stroke().unwrap();

        if should_show_text {
            let (r, g, b) = if is_current {
                key_current_text_color
            } else {
                key_text_color
            };
            cr.set_source_rgb(r, g, b);
            cr.select_font_face(
                "Sans",
                gtk::cairo::FontSlant::Normal,
                gtk::cairo::FontWeight::Normal,
            );

            if let Some(label_text) = label {
                cr.set_font_size(11.0);
                let text_extents = cr.text_extents(label_text).unwrap();
                cr.move_to(
                    x + (width - text_extents.width()) / 2.0,
                    y + height / 2.0 + 5.0,
                );
                cr.show_text(label_text).unwrap();
            } else if let Some(key) = key_info {
                let base_text = if key.base.chars().next().unwrap().is_alphabetic() {
                    key.base.to_uppercase()
                } else {
                    key.base.clone()
                };
                let is_alphabetic = key.base.chars().next().unwrap().is_alphabetic();

                if is_alphabetic {
                    cr.set_font_size(18.0);
                    let text_extents = cr.text_extents(&base_text).unwrap();
                    cr.move_to(
                        x + (width - text_extents.width()) / 2.0,
                        y + (height + text_extents.height()) / 2.0,
                    );
                    cr.show_text(&base_text).unwrap();
                } else {
                    cr.set_font_size(20.0);
                    cr.move_to(x + 5.0, y + height - 5.0);
                    cr.show_text(&base_text).unwrap();

                    if let Some(shift_text) = &key.shift {
                        cr.move_to(x + 5.0, y + 15.0);
                        cr.show_text(shift_text).unwrap();
                    }

                    if let Some(altgr_text) = &key.altgr {
                        if !altgr_text.is_empty() {
                            let text_extents = cr.text_extents(altgr_text).unwrap();
                            cr.move_to(x + width - text_extents.width() - 5.0, y + height - 5.0);
                            cr.show_text(altgr_text).unwrap();
                        }
                    }
                }
            }
        }
    }

    fn draw_keyboard(
        widget: &gtk::DrawingArea,
        cr: &gtk::cairo::Context,
        current_key: &Rc<RefCell<Option<char>>>,
        layout: &Rc<RefCell<KeyboardLayout>>,
        visible_keys: &Rc<RefCell<Option<HashSet<char>>>>,
    ) {
        let layout_borrowed = layout.borrow();
        let visible_keys_borrowed = visible_keys.borrow();

        let key_width = 50.0;
        let key_height = 50.0;
        let key_spacing = 5.0;
        let row_spacing = 5.0;

        let get_color = |class_name: &str| -> (f64, f64, f64) {
            widget.add_css_class(class_name);
            let color = widget.color();
            widget.remove_css_class(class_name);
            (
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
            )
        };

        let modifier_color = get_color("keyboard-modifier");
        let modifier_text_color = get_color("keyboard-modifier-text");
        let key_text_color = get_color("keyboard-key-text");
        let key_color = get_color("keyboard-key");
        let key_current_text_color = get_color("keyboard-key-current-text");
        let key_current_color = get_color("keyboard-key-current");
        let key_border_color = get_color("keyboard-border");

        let current = current_key.borrow();

        let is_key_current = |key_info: &KeyInfo| -> bool {
            let key_char = key_info.base.chars().next().unwrap_or(' ');
            current.is_some_and(|c| {
                if c == ' ' {
                    false
                } else {
                    let c_lower = c.to_lowercase().next().unwrap();
                    let base_lower = key_char.to_lowercase().next().unwrap();
                    c_lower == base_lower
                        || key_info
                            .shift
                            .as_ref()
                            .is_some_and(|s| s.chars().next().unwrap_or(' ') == c)
                        || key_info
                            .altgr
                            .as_ref()
                            .is_some_and(|a| a.chars().next().unwrap_or(' ') == c)
                }
            })
        };

        let should_show_key = |key_char: char| -> bool {
            visible_keys_borrowed
                .as_ref()
                .is_none_or(|visible| visible.contains(&key_char.to_lowercase().next().unwrap()))
        };

        // Row 0: Number row + Backspace
        let mut x = 0.0;
        let y = 0.0;
        if let Some(row) = layout_borrowed.keys.first() {
            for key_info in row {
                let key_char = key_info.base.chars().next().unwrap_or(' ');
                Self::draw_single_key(
                    cr,
                    x,
                    y,
                    key_width,
                    key_height,
                    Some(key_info),
                    None,
                    is_key_current(key_info),
                    should_show_key(key_char),
                    key_color,
                    key_current_color,
                    key_text_color,
                    key_current_text_color,
                    key_border_color,
                );
                x += key_width + key_spacing;
            }
            if let Some(backspace) = layout_borrowed.modifiers.get("backspace") {
                Self::draw_single_key(
                    cr,
                    x,
                    y,
                    key_width * 2.0,
                    key_height,
                    None,
                    Some(&backspace.label),
                    false,
                    true,
                    modifier_color,
                    key_current_color,
                    modifier_text_color,
                    key_current_text_color,
                    key_border_color,
                );
            }
        }

        // Row 1: Tab + QWERTY row + Enter (spans to row 2)
        x = 0.0;
        let y1 = y + key_height + row_spacing;
        if let Some(tab) = layout_borrowed.modifiers.get("tab") {
            Self::draw_single_key(
                cr,
                x,
                y1,
                key_width * 1.5,
                key_height,
                None,
                Some(&tab.label),
                false,
                true,
                modifier_color,
                key_current_color,
                modifier_text_color,
                key_current_text_color,
                key_border_color,
            );
            x += key_width * 1.5 + key_spacing;
        }
        if let Some(row) = layout_borrowed.keys.get(1) {
            for key_info in row {
                let key_char = key_info.base.chars().next().unwrap_or(' ');
                Self::draw_single_key(
                    cr,
                    x,
                    y1,
                    key_width,
                    key_height,
                    Some(key_info),
                    None,
                    is_key_current(key_info),
                    should_show_key(key_char),
                    key_color,
                    key_current_color,
                    key_text_color,
                    key_current_text_color,
                    key_border_color,
                );
                x += key_width + key_spacing;
            }
        }
        if let Some(enter) = layout_borrowed.modifiers.get("enter") {
            let enter_height = key_height * 2.0 + row_spacing;
            Self::draw_single_key(
                cr,
                x,
                y1,
                key_width * 1.75,
                enter_height,
                None,
                Some(&enter.label),
                false,
                true,
                modifier_color,
                key_current_color,
                modifier_text_color,
                key_current_text_color,
                key_border_color,
            );
        }

        // Row 2: Caps Lock + Home row (Enter already drawn)
        x = 0.0;
        let y2 = y1 + key_height + row_spacing;
        if let Some(caps) = layout_borrowed.modifiers.get("caps_lock") {
            Self::draw_single_key(
                cr,
                x,
                y2,
                key_width * 1.75,
                key_height,
                None,
                Some(&caps.label),
                false,
                true,
                modifier_color,
                key_current_color,
                modifier_text_color,
                key_current_text_color,
                key_border_color,
            );
            x += key_width * 1.75 + key_spacing;
        }
        if let Some(row) = layout_borrowed.keys.get(2) {
            for key_info in row {
                let key_char = key_info.base.chars().next().unwrap_or(' ');
                Self::draw_single_key(
                    cr,
                    x,
                    y2,
                    key_width,
                    key_height,
                    Some(key_info),
                    None,
                    is_key_current(key_info),
                    should_show_key(key_char),
                    key_color,
                    key_current_color,
                    key_text_color,
                    key_current_text_color,
                    key_border_color,
                );
                x += key_width + key_spacing;
            }
        }

        // Row 3: Left Shift + Bottom row + Right Shift
        x = 0.0;
        let y3 = y2 + key_height + row_spacing;
        if let Some(shift_l) = layout_borrowed.modifiers.get("shift_left") {
            Self::draw_single_key(
                cr,
                x,
                y3,
                key_width * 1.25,
                key_height,
                None,
                Some(&shift_l.label),
                false,
                true,
                modifier_color,
                key_current_color,
                modifier_text_color,
                key_current_text_color,
                key_border_color,
            );
            x += key_width * 1.25 + key_spacing;
        }
        if let Some(row) = layout_borrowed.keys.get(3) {
            for key_info in row {
                let key_char = key_info.base.chars().next().unwrap_or(' ');
                Self::draw_single_key(
                    cr,
                    x,
                    y3,
                    key_width,
                    key_height,
                    Some(key_info),
                    None,
                    is_key_current(key_info),
                    should_show_key(key_char),
                    key_color,
                    key_current_color,
                    key_text_color,
                    key_current_text_color,
                    key_border_color,
                );
                x += key_width + key_spacing;
            }
        }
        if let Some(shift_r) = layout_borrowed.modifiers.get("shift_right") {
            Self::draw_single_key(
                cr,
                x,
                y3,
                key_width * 3.0,
                key_height,
                None,
                Some(&shift_r.label),
                false,
                true,
                modifier_color,
                key_current_color,
                modifier_text_color,
                key_current_text_color,
                key_border_color,
            );
        }

        // Row 4: Ctrl + Alt + Space + Alt + Ctrl
        x = 0.0;
        let y4 = y3 + key_height + row_spacing;
        if let Some(ctrl_l) = layout_borrowed.modifiers.get("ctrl_left") {
            Self::draw_single_key(
                cr,
                x,
                y4,
                key_width * 1.5,
                key_height,
                None,
                Some(&ctrl_l.label),
                false,
                true,
                modifier_color,
                key_current_color,
                modifier_text_color,
                key_current_text_color,
                key_border_color,
            );
            x += key_width * 1.5 + key_spacing;
        }
        if let Some(alt_l) = layout_borrowed.modifiers.get("alt_left") {
            Self::draw_single_key(
                cr,
                x,
                y4,
                key_width * 1.3,
                key_height,
                None,
                Some(&alt_l.label),
                false,
                true,
                modifier_color,
                key_current_color,
                modifier_text_color,
                key_current_text_color,
                key_border_color,
            );
            x += key_width * 1.3 + key_spacing;
        }
        let is_space_current = current.is_some_and(|c| c == ' ');
        let space_label = layout_borrowed.space.label.as_deref().unwrap_or("SPACE");
        Self::draw_single_key(
            cr,
            x,
            y4,
            key_width * 6.0,
            key_height,
            None,
            Some(space_label),
            is_space_current,
            should_show_key(' '),
            key_color,
            key_current_color,
            key_text_color,
            key_current_text_color,
            key_border_color,
        );
        x += key_width * 6.0 + key_spacing;
        if let Some(alt_r) = layout_borrowed.modifiers.get("alt_right") {
            Self::draw_single_key(
                cr,
                x,
                y4,
                key_width * 1.3,
                key_height,
                None,
                Some(&alt_r.label),
                false,
                true,
                modifier_color,
                key_current_color,
                modifier_text_color,
                key_current_text_color,
                key_border_color,
            );
            x += key_width * 1.3 + key_spacing;
        }
        if let Some(ctrl_r) = layout_borrowed.modifiers.get("ctrl_right") {
            Self::draw_single_key(
                cr,
                x,
                y4,
                key_width * 1.5,
                key_height,
                None,
                Some(&ctrl_r.label),
                false,
                true,
                modifier_color,
                key_current_color,
                modifier_text_color,
                key_current_text_color,
                key_border_color,
            );
        }
    }
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self::new()
    }
}
