use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use libadwaita as adw;
use libadwaita::subclass::prelude::*;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/typing_row.ui")]
    pub struct TypingRow {
        #[template_child]
        pub target_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub text_input: TemplateChild<gtk::Text>,
        #[template_child]
        pub repetition_label: TemplateChild<gtk::Label>,
        pub cursor_position: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TypingRow {
        const NAME: &'static str = "MecalinTypingRow";
        type Type = super::TypingRow;
        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TypingRow {}
    impl WidgetImpl for TypingRow {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);
            self.draw_cursor(snapshot);
        }

        fn grab_focus(&self) -> bool {
            self.text_input.grab_focus()
        }
    }
    impl ListBoxRowImpl for TypingRow {}
    impl PreferencesRowImpl for TypingRow {}
}

impl imp::TypingRow {
    fn char_pos_to_byte_index(text: &str, char_pos: usize) -> usize {
        let mut char_count = 0;
        for (byte_idx, _) in text.char_indices() {
            if char_count >= char_pos {
                return byte_idx;
            }
            char_count += 1;
        }
        text.len()
    }

    fn draw_cursor(&self, snapshot: &gtk::Snapshot) {
        let cursor_pos = self.cursor_position.get();
        let label = self.target_label.get();

        let text = label.text();
        if text.is_empty() {
            return;
        }

        let layout = label.layout();

        let index = Self::char_pos_to_byte_index(&text, cursor_pos as usize);
        let (rect, _) = layout.cursor_pos(index as i32);

        // Get the label's position relative to the TypingRow widget
        let typing_row = self.obj();
        let point = label
            .compute_point(
                typing_row.upcast_ref::<gtk::Widget>(),
                &gtk::graphene::Point::new(0.0, 0.0),
            )
            .unwrap_or_else(|| gtk::graphene::Point::new(0.0, 0.0));

        let x = point.x() + (rect.x() / pango::SCALE) as f32;
        let y = point.y() + (rect.y() / pango::SCALE) as f32;
        let height = (rect.height() / pango::SCALE) as f32;

        #[allow(deprecated)]
        let style_ctx = label.style_context();
        #[allow(deprecated)]
        let color = style_ctx.color();

        let cursor_rect = gtk::graphene::Rect::new(x, y, 2.0, height);
        snapshot.append_color(&color, &cursor_rect);
    }
}

glib::wrapper! {
    pub struct TypingRow(ObjectSubclass<imp::TypingRow>)
        @extends adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl TypingRow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_target_text(&self, text: &str) {
        let imp = self.imp();
        imp.target_label.set_text(text);
        imp.cursor_position.set(0);
        self.queue_draw();
    }

    pub fn set_cursor_position(&self, position: i32) {
        let imp = self.imp();
        imp.cursor_position.set(position);
        self.queue_draw();
    }

    pub fn text_input(&self) -> gtk::Text {
        self.imp().text_input.get()
    }

    pub fn buffer(&self) -> gtk::EntryBuffer {
        self.imp().text_input.buffer()
    }

    pub fn set_repetition_text(&self, text: &str) {
        self.imp().repetition_label.set_text(text);
    }

    pub fn show_error(&self) {
        self.add_css_class("typing-error");

        glib::timeout_add_local_once(std::time::Duration::from_millis(400), {
            let typing_row = self.clone();
            move || {
                typing_row.remove_css_class("typing-error");
            }
        });
    }
}

impl Default for TypingRow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_pos_to_byte_index_ascii() {
        let text = "hello";
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 0), 0);
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 2), 2);
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 5), 5);
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 10), 5); // Beyond end
    }

    #[test]
    fn test_char_pos_to_byte_index_multibyte() {
        let text = "añ";
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 0), 0); // 'a' at byte 0
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 1), 1); // 'ñ' at byte 1
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 2), 3); // End (ñ is 2 bytes)
    }

    #[test]
    fn test_char_pos_to_byte_index_mixed() {
        let text = "hola ñ mundo";
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 0), 0); // 'h'
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 5), 5); // 'ñ'
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 6), 7); // ' ' after ñ
    }

    #[test]
    fn test_char_pos_to_byte_index_empty() {
        let text = "";
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 0), 0);
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 5), 0);
    }
}
