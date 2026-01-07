use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/main_menu.ui")]
    pub struct MainMenu {
        #[template_child]
        pub action_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainMenu {
        const NAME: &'static str = "MainMenu";
        type Type = super::MainMenu;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MainMenu {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_actions();
        }
    }
    impl WidgetImpl for MainMenu {}
    impl BoxImpl for MainMenu {}
}

impl imp::MainMenu {
    fn setup_actions(&self) {
        let actions = [
            ("Study room", "Learn typing fundamentals"),
            ("Student control", "Manage student progress"),
            ("Skill game", "Practice with games"),
            ("Videos", "Watch typing tutorials"),
            ("About", "Application information"),
        ];

        for (title, subtitle) in actions {
            let row = gtk::ListBoxRow::new();
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            hbox.set_margin_top(12);
            hbox.set_margin_bottom(12);
            hbox.set_margin_start(12);
            hbox.set_margin_end(12);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
            let title_label = gtk::Label::new(Some(title));
            title_label.set_halign(gtk::Align::Start);
            title_label.add_css_class("heading");
            
            let subtitle_label = gtk::Label::new(Some(subtitle));
            subtitle_label.set_halign(gtk::Align::Start);
            subtitle_label.add_css_class("dim-label");

            vbox.append(&title_label);
            vbox.append(&subtitle_label);
            hbox.append(&vbox);
            row.set_child(Some(&hbox));
            
            self.action_list.append(&row);
        }
    }
}

glib::wrapper! {
    pub struct MainMenu(ObjectSubclass<imp::MainMenu>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MainMenu {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
