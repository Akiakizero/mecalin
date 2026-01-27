use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/main_action_list.ui")]
    pub struct MainActionList {
        #[template_child]
        pub action_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainActionList {
        const NAME: &'static str = "MainActionList";
        type Type = super::MainActionList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_action("action.lessons", None, |obj, _, _| {
                obj.emit_by_name::<()>("lessons-selected", &[]);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MainActionList {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_actions();
            self.setup_signals();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: std::sync::OnceLock<Vec<glib::subclass::Signal>> =
                std::sync::OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    glib::subclass::Signal::builder("lessons-selected").build(),
                    glib::subclass::Signal::builder("game-selected").build(),
                    glib::subclass::Signal::builder("lanes-game-selected").build(),
                    glib::subclass::Signal::builder("about-selected").build(),
                ]
            })
        }
    }
    impl WidgetImpl for MainActionList {}
    impl BoxImpl for MainActionList {}
}

impl imp::MainActionList {
    fn setup_actions(&self) {
        let actions = [
            (&gettext("Lessons"), &gettext("Learn typing fundamentals")),
            (
                &gettext("Falling Keys"),
                &gettext("Practice with a fun game"),
            ),
            (
                &gettext("Scrolling Lanes"),
                &gettext("Type fast in multiple lanes"),
            ),
            (&gettext("About"), &gettext("Application information")),
        ];

        for (title, subtitle) in actions {
            let row = adw::ActionRow::builder()
                .title(title)
                .subtitle(subtitle)
                .activatable(true)
                .build();

            self.action_list.append(&row);
        }
    }

    fn setup_signals(&self) {
        let obj = self.obj().downgrade();
        self.action_list.connect_row_activated(move |_, row| {
            if let Some(obj) = obj.upgrade() {
                match row.index() {
                    0 => obj.emit_by_name::<()>("lessons-selected", &[]),
                    1 => obj.emit_by_name::<()>("game-selected", &[]),
                    2 => obj.emit_by_name::<()>("lanes-game-selected", &[]),
                    3 => obj.emit_by_name::<()>("about-selected", &[]),
                    _ => {}
                }
            }
        });
    }
}

glib::wrapper! {
    pub struct MainActionList(ObjectSubclass<imp::MainActionList>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MainActionList {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
