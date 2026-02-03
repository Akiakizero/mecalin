use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use i18n_format::i18n_fmt;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::course::Course;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/preferences_view.ui")]
    pub struct PreferencesView {
        #[template_child]
        pub show_hand_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub show_keyboard_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub use_finger_colors_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub lesson_combo: TemplateChild<adw::ComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesView {
        const NAME: &'static str = "MecalinPreferencesView";
        type Type = super::PreferencesView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_settings();
        }
    }

    impl WidgetImpl for PreferencesView {}
    impl BoxImpl for PreferencesView {}

    impl PreferencesView {
        fn setup_settings(&self) {
            let settings = gio::Settings::new("io.github.nacho.mecalin");

            // Bind switches to settings
            settings
                .bind("show-hand-widget", &*self.show_hand_switch, "active")
                .build();
            settings
                .bind(
                    "show-keyboard-widget",
                    &*self.show_keyboard_switch,
                    "active",
                )
                .build();
            settings
                .bind(
                    "use-finger-colors",
                    &*self.use_finger_colors_switch,
                    "active",
                )
                .build();

            // Load course and setup lesson combo
            let layout_code = crate::utils::language_from_locale();
            if let Ok(course) = Course::new_with_language(layout_code) {
                let lesson_names: Vec<String> = course
                    .get_lessons()
                    .iter()
                    .enumerate()
                    .map(|(i, lesson)| i18n_fmt!(i18n_fmt("Lesson {}: {}", i, &lesson.title)))
                    .collect();
                let lesson_strs: Vec<&str> = lesson_names.iter().map(|s| s.as_str()).collect();
                let lesson_model = gtk::StringList::new(&lesson_strs);
                self.lesson_combo.set_model(Some(&lesson_model));
                self.lesson_combo
                    .set_selected(settings.uint("current-lesson"));

                self.lesson_combo.connect_selected_notify(move |combo| {
                    let settings = gio::Settings::new("io.github.nacho.mecalin");
                    settings.set_uint("current-lesson", combo.selected()).ok();
                    // Reset step to 0 when lesson changes
                    settings.set_uint("current-step", 0).ok();
                });
            }
        }
    }
}

glib::wrapper! {
    pub struct PreferencesView(ObjectSubclass<imp::PreferencesView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PreferencesView {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for PreferencesView {
    fn default() -> Self {
        Self::new()
    }
}
