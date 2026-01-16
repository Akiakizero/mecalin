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
            "es" => include_str!("../data/keyboard_layouts/es.json"),
            "it" => include_str!("../data/keyboard_layouts/it.json"),
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
        drawing_area.set_size_request(1000, 350);

        let current_key = Rc::new(RefCell::new(None));
        let visible_keys = Rc::new(RefCell::new(None));
        let current_key_sequence = Rc::new(RefCell::new(Vec::new()));
        let sequence_index = Rc::new(RefCell::new(0));
        let current_key_clone = current_key.clone();
        let visible_keys_clone = visible_keys.clone();
        let layout_clone = layout.clone();

        drawing_area.set_draw_func(move |widget, cr, width, height| {
            Self::draw_keyboard(
                widget,
                cr,
                width,
                height,
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

    fn draw_keyboard(
        widget: &gtk::DrawingArea,
        cr: &gtk::cairo::Context,
        width: i32,
        _height: i32,
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

        // Helper function to get color from CSS class
        #[allow(deprecated)]
        let get_color = |class_name: &str| -> (f64, f64, f64) {
            widget.add_css_class(class_name);
            let style_context = widget.style_context();
            let color = style_context.color();
            widget.remove_css_class(class_name);
            (
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
            )
        };

        // Reserve space for left modifiers (widest is 2.0 * key_width for shift)
        let left_margin = key_width * 2.5;
        // Reserve space for right modifiers
        let right_margin = key_width * 2.0;

        let max_keys_in_row = layout_borrowed
            .keys
            .iter()
            .map(|row| row.len())
            .max()
            .unwrap_or(12);
        let keyboard_width = max_keys_in_row as f64 * (key_width + key_spacing) - key_spacing;
        let total_width = left_margin + keyboard_width + right_margin;
        let start_x = (width as f64 - total_width) / 2.0 + left_margin;
        let start_y = 20.0;

        let current = current_key.borrow();

        for (row_idx, row) in layout_borrowed.keys.iter().enumerate() {
            let row_offset = match row_idx {
                1 => key_width * 0.5,
                2 => key_width * 0.75,
                3 => key_width * 1.25,
                _ => 0.0,
            };

            for (key_idx, key_info) in row.iter().enumerate() {
                let key_char = key_info.base.chars().next().unwrap_or(' ');
                let x = start_x + row_offset + key_idx as f64 * (key_width + key_spacing);
                let y = start_y + row_idx as f64 * (key_height + row_spacing);

                let is_current = current.is_some_and(|c| {
                    if c == ' ' {
                        // Space character should only match space, not other keys
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
                });

                let (r, g, b) = if is_current {
                    get_color("keyboard-key-current")
                } else {
                    get_color("keyboard-key")
                };
                cr.set_source_rgb(r, g, b);

                cr.rectangle(x, y, key_width, key_height);
                cr.fill().unwrap();

                let (br, bg, bb) = get_color("keyboard-border");
                cr.set_source_rgb(br, bg, bb);
                cr.set_line_width(1.0);
                cr.rectangle(x, y, key_width, key_height);
                cr.stroke().unwrap();

                let should_show_text = visible_keys_borrowed.as_ref().is_none_or(|visible| {
                    visible.contains(&key_char.to_lowercase().next().unwrap())
                });

                if should_show_text {
                    cr.set_source_rgb(0.0, 0.0, 0.0);
                    cr.select_font_face(
                        "Sans",
                        gtk::cairo::FontSlant::Normal,
                        gtk::cairo::FontWeight::Normal,
                    );

                    // Draw base character (bottom left)
                    let base_text = if key_info.base.chars().next().unwrap().is_alphabetic() {
                        key_info.base.to_uppercase()
                    } else {
                        key_info.base.clone()
                    };

                    // Use larger font for alphabetic keys (show only uppercase, centered)
                    let is_alphabetic = key_info.base.chars().next().unwrap().is_alphabetic();

                    if is_alphabetic {
                        cr.set_font_size(18.0);
                        let text_extents = cr.text_extents(&base_text).unwrap();
                        let text_x = x + (key_width - text_extents.width()) / 2.0;
                        let text_y = y + (key_height + text_extents.height()) / 2.0;
                        cr.move_to(text_x, text_y);
                        cr.show_text(&base_text).unwrap();
                    } else {
                        cr.set_font_size(20.0);
                        cr.move_to(x + 5.0, y + key_height - 5.0);
                        cr.show_text(&base_text).unwrap();

                        // Draw shift character (top left)
                        if let Some(shift_text) = &key_info.shift {
                            cr.move_to(x + 5.0, y + 15.0);
                            cr.show_text(shift_text).unwrap();
                        }

                        // Draw altgr character (bottom right)
                        if let Some(altgr_text) = &key_info.altgr {
                            if !altgr_text.is_empty() {
                                let text_extents = cr.text_extents(altgr_text).unwrap();
                                cr.move_to(
                                    x + key_width - text_extents.width() - 5.0,
                                    y + key_height - 5.0,
                                );
                                cr.show_text(altgr_text).unwrap();
                            }
                        }
                    }
                }
            }
        }

        // Space bar
        let space_x = start_x + key_width * 2.0;
        let space_y = start_y + 4.0 * (key_height + row_spacing);
        let space_width = key_width * 6.0;

        let is_space_current = current.is_some_and(|c| c == ' ');

        let (r, g, b) = if is_space_current {
            get_color("keyboard-key-current")
        } else {
            get_color("keyboard-key")
        };
        cr.set_source_rgb(r, g, b);

        cr.rectangle(space_x, space_y, space_width, key_height);
        cr.fill().unwrap();

        let (br, bg, bb) = get_color("keyboard-border");
        cr.set_source_rgb(br, bg, bb);
        cr.set_line_width(1.0);
        cr.rectangle(space_x, space_y, space_width, key_height);
        cr.stroke().unwrap();

        let should_show_space_text = visible_keys_borrowed
            .as_ref()
            .is_none_or(|visible| visible.contains(&' '));

        if should_show_space_text {
            let space_label = layout_borrowed.space.label.as_deref().unwrap_or("SPACE");
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(11.0);
            let text_extents = cr.text_extents(space_label).unwrap();
            cr.move_to(
                space_x + (space_width - text_extents.width()) / 2.0,
                space_y + key_height / 2.0 + 5.0,
            );
            cr.show_text(space_label).unwrap();
        }

        // Draw modifier keys
        // Calculate row end positions accounting for offsets
        let row_offsets = [
            0.0,
            key_width * 0.5,
            key_width * 0.75,
            key_width * 1.25,
            0.0,
        ];

        // Tab (left of QWERTY row)
        if let Some(tab) = layout_borrowed.modifiers.get("tab") {
            let tab_width = key_width * 1.5;
            let tab_x = start_x + row_offsets[1] - tab_width - key_spacing;
            let tab_y = start_y + (key_height + row_spacing);

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(tab_x, tab_y, tab_width, key_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(tab_x, tab_y, tab_width, key_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(11.0);
            cr.move_to(tab_x + 10.0, tab_y + key_height / 2.0 + 5.0);
            cr.show_text(&tab.label).unwrap();
        }

        // Caps Lock (left of home row)
        if let Some(caps) = layout_borrowed.modifiers.get("caps_lock") {
            let caps_width = key_width * 1.75;
            let caps_x = start_x + row_offsets[2] - caps_width - key_spacing;
            let caps_y = start_y + 2.0 * (key_height + row_spacing);

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(caps_x, caps_y, caps_width, key_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(caps_x, caps_y, caps_width, key_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(10.0);
            cr.move_to(caps_x + 8.0, caps_y + key_height / 2.0 + 5.0);
            cr.show_text(&caps.label).unwrap();
        }

        // Left Shift (left of bottom letter row, before < key)
        if let Some(shift_l) = layout_borrowed.modifiers.get("shift_left") {
            let shift_width = key_width * 2.25;
            let shift_x = start_x + row_offsets[3] - shift_width - key_spacing;
            let shift_y = start_y + 3.0 * (key_height + row_spacing);

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(shift_x, shift_y, shift_width, key_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(shift_x, shift_y, shift_width, key_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(11.0);
            cr.move_to(shift_x + 10.0, shift_y + key_height / 2.0 + 5.0);
            cr.show_text(&shift_l.label).unwrap();
        }

        // Left Ctrl (bottom left, aligned with left shift)
        if let Some(ctrl_l) = layout_borrowed.modifiers.get("ctrl_left") {
            let ctrl_width = key_width * 1.5;
            let ctrl_x = start_x + row_offsets[3] - key_width * 2.25 - key_spacing;
            let ctrl_y = start_y + 4.0 * (key_height + row_spacing);

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(ctrl_x, ctrl_y, ctrl_width, key_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(ctrl_x, ctrl_y, ctrl_width, key_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(11.0);
            cr.move_to(ctrl_x + 10.0, ctrl_y + key_height / 2.0 + 5.0);
            cr.show_text(&ctrl_l.label).unwrap();
        }

        // Left Alt (bottom, after left ctrl)
        if let Some(alt_l) = layout_borrowed.modifiers.get("alt_left") {
            let ctrl_width = key_width * 1.5;
            let alt_width = key_width * 1.3;
            let alt_x = start_x + row_offsets[3] - key_width * 2.25 - key_spacing
                + ctrl_width
                + key_spacing;
            let alt_y = start_y + 4.0 * (key_height + row_spacing);

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(alt_x, alt_y, alt_width, key_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(alt_x, alt_y, alt_width, key_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(11.0);
            cr.move_to(alt_x + 10.0, alt_y + key_height / 2.0 + 5.0);
            cr.show_text(&alt_l.label).unwrap();
        }

        // Backspace (right of number row)
        if let Some(backspace) = layout_borrowed.modifiers.get("backspace") {
            let row_0_keys = layout_borrowed.keys.first().map(|r| r.len()).unwrap_or(12);
            let bs_width = key_width * 2.0;
            let bs_x = start_x + row_offsets[0] + row_0_keys as f64 * (key_width + key_spacing);
            let bs_y = start_y;

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(bs_x, bs_y, bs_width, key_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(bs_x, bs_y, bs_width, key_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(11.0);
            cr.move_to(bs_x + 10.0, bs_y + key_height / 2.0 + 5.0);
            cr.show_text(&backspace.label).unwrap();
        }

        // Enter (right of home row, spans 2 rows)
        if let Some(enter) = layout_borrowed.modifiers.get("enter") {
            let row_2_keys = layout_borrowed.keys.get(2).map(|r| r.len()).unwrap_or(12);
            let enter_width = key_width * 2.1;
            let enter_x = start_x + row_offsets[2] + row_2_keys as f64 * (key_width + key_spacing);
            let enter_y = start_y + (key_height + row_spacing);
            let enter_height = key_height * 2.0 + row_spacing;

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(enter_x, enter_y, enter_width, enter_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(enter_x, enter_y, enter_width, enter_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(11.0);
            cr.move_to(enter_x + 15.0, enter_y + enter_height / 2.0 + 5.0);
            cr.show_text(&enter.label).unwrap();
        }

        // Right Shift (right of bottom letter row)
        if let Some(shift_r) = layout_borrowed.modifiers.get("shift_right") {
            let row_3_keys = layout_borrowed.keys.get(3).map(|r| r.len()).unwrap_or(10);
            let shift_width = key_width * 2.75;
            let shift_x = start_x + row_offsets[3] + row_3_keys as f64 * (key_width + key_spacing);
            let shift_y = start_y + 3.0 * (key_height + row_spacing);

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(shift_x, shift_y, shift_width, key_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(shift_x, shift_y, shift_width, key_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(11.0);
            cr.move_to(shift_x + 20.0, shift_y + key_height / 2.0 + 5.0);
            cr.show_text(&shift_r.label).unwrap();
        }

        // Right Ctrl (bottom right, right edge aligned with right shift)
        if let Some(ctrl_r) = layout_borrowed.modifiers.get("ctrl_right") {
            let row_3_keys = layout_borrowed.keys.get(3).map(|r| r.len()).unwrap_or(10);
            let shift_width = key_width * 2.75;
            let ctrl_width = key_width * 1.5;
            let shift_end = start_x
                + row_offsets[3]
                + row_3_keys as f64 * (key_width + key_spacing)
                + shift_width;
            let ctrl_x = shift_end - ctrl_width;
            let ctrl_y = start_y + 4.0 * (key_height + row_spacing);

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(ctrl_x, ctrl_y, ctrl_width, key_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(ctrl_x, ctrl_y, ctrl_width, key_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(11.0);
            cr.move_to(ctrl_x + 10.0, ctrl_y + key_height / 2.0 + 5.0);
            cr.show_text(&ctrl_r.label).unwrap();
        }

        // Right Alt (bottom, right after space bar)
        if let Some(alt_r) = layout_borrowed.modifiers.get("alt_right") {
            let alt_width = key_width * 1.3;
            let alt_x = space_x + space_width + key_spacing;
            let alt_y = start_y + 4.0 * (key_height + row_spacing);

            let (r, g, b) = get_color("keyboard-modifier");
            cr.set_source_rgb(r, g, b);
            cr.rectangle(alt_x, alt_y, alt_width, key_height);
            cr.fill().unwrap();
            let (br, bg, bb) = get_color("keyboard-border");
            cr.set_source_rgb(br, bg, bb);
            cr.set_line_width(1.0);
            cr.rectangle(alt_x, alt_y, alt_width, key_height);
            cr.stroke().unwrap();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(10.0);
            cr.move_to(alt_x + 5.0, alt_y + key_height / 2.0 + 5.0);
            cr.show_text(&alt_r.label).unwrap();
        }
    }
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self::new()
    }
}
