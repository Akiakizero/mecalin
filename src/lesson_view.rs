use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gettextrs::gettext;

use crate::keyboard_widget::KeyboardWidget;
use crate::course::Lesson;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/lesson_view.ui")]
    pub struct LessonView {
        #[template_child]
        pub lesson_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub lesson_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub text_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub keyboard_container: TemplateChild<gtk::Box>,
        
        pub keyboard_widget: std::cell::RefCell<Option<KeyboardWidget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LessonView {
        const NAME: &'static str = "LessonView";
        type Type = super::LessonView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LessonView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_keyboard();
            self.setup_signals();
        }
    }
    impl WidgetImpl for LessonView {}
    impl BoxImpl for LessonView {}
}

impl imp::LessonView {
    fn setup_keyboard(&self) {
        let keyboard = KeyboardWidget::new();
        self.keyboard_container.append(keyboard.widget());
        *self.keyboard_widget.borrow_mut() = Some(keyboard);
    }

    fn setup_signals(&self) {
        let keyboard_widget = self.keyboard_widget.borrow();
        if let Some(keyboard) = keyboard_widget.as_ref() {
            let keyboard_clone = keyboard.clone();
            self.text_entry.connect_changed(move |entry| {
                let text = entry.text();
                if let Some(last_char) = text.chars().last() {
                    keyboard_clone.set_current_key(Some(last_char));
                } else {
                    keyboard_clone.set_current_key(None);
                }
            });
        }
    }
}

glib::wrapper! {
    pub struct LessonView(ObjectSubclass<imp::LessonView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl LessonView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_lesson(&self, lesson: &Lesson) {
        let imp = self.imp();
        let title = format!("{} {}", gettext("Lesson"), lesson.id);
        imp.lesson_title.set_text(&title);
        imp.lesson_description.set_text(&lesson.description);
        imp.text_entry.set_text("");
    }
}
