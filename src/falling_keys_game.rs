use gtk::gdk;
use gtk::glib;
use gtk::graphene;
use gtk::pango;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use i18n_format::i18n_fmt;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

const KEYS: &[char] = &[
    'a', 's', 'd', 'f', 'j', 'k', 'l', 'q', 'w', 'e', 'r', 'u', 'i', 'o', 'p',
];

#[derive(Clone)]
pub(crate) struct FallingKey {
    key: char,
    x: f64,
    y: f64,
}

mod falling_keys_widget {
    use super::*;

    pub struct FallingKeysWidget {
        pub(super) falling_keys: RefCell<Rc<RefCell<Vec<FallingKey>>>>,
        pub(super) text_color: RefCell<gdk::RGBA>,
    }

    impl Default for FallingKeysWidget {
        fn default() -> Self {
            Self {
                falling_keys: RefCell::new(Rc::new(RefCell::new(Vec::new()))),
                text_color: RefCell::new(gdk::RGBA::BLACK),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FallingKeysWidget {
        const NAME: &'static str = "FallingKeysWidget";
        type Type = super::FallingKeysWidget;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for FallingKeysWidget {}

    impl WidgetImpl for FallingKeysWidget {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let pango_context = widget.pango_context();
            let layout = pango::Layout::new(&pango_context);
            let font_desc = pango::FontDescription::from_string("Sans 24");
            layout.set_font_description(Some(&font_desc));

            let text_color = self.text_color.borrow();

            for key in self.falling_keys.borrow().borrow().iter() {
                layout.set_text(&key.key.to_string());
                snapshot.save();
                snapshot.translate(&graphene::Point::new(key.x as f32, key.y as f32));
                snapshot.append_layout(&layout, &text_color);
                snapshot.restore();
            }
        }
    }
}

glib::wrapper! {
    pub struct FallingKeysWidget(ObjectSubclass<falling_keys_widget::FallingKeysWidget>)
        @extends gtk::Widget;
}

impl FallingKeysWidget {
    pub(crate) fn new(falling_keys: Rc<RefCell<Vec<FallingKey>>>) -> Self {
        let widget: Self = glib::Object::new();
        widget.imp().falling_keys.replace(falling_keys.clone());
        widget.set_vexpand(true);
        widget.set_hexpand(true);
        widget
    }

    pub fn set_text_color(&self, color: gdk::RGBA) {
        *self.imp().text_color.borrow_mut() = color;
    }
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/falling_keys_game.ui")]
    pub struct FallingKeysGame {
        #[template_child]
        pub game_area: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub score_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub difficulty_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub results_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub results_score_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub results_level_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub restart_button: TemplateChild<gtk::Button>,

        pub falling_keys_widget: RefCell<Option<FallingKeysWidget>>,
        pub keyboard_widget: RefCell<Option<crate::keyboard_widget::KeyboardWidget>>,
        pub(crate) falling_keys: Rc<RefCell<Vec<FallingKey>>>,
        pub score: RefCell<u32>,
        pub difficulty: RefCell<u32>,
        pub speed: RefCell<f64>,
        pub game_over: RefCell<bool>,
        pub game_loop_running: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FallingKeysGame {
        const NAME: &'static str = "FallingKeysGame";
        type Type = super::FallingKeysGame;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FallingKeysGame {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_game();
        }
    }
    impl WidgetImpl for FallingKeysGame {}
    impl BoxImpl for FallingKeysGame {}
}

glib::wrapper! {
    pub struct FallingKeysGame(ObjectSubclass<imp::FallingKeysGame>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl FallingKeysGame {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup_game(&self) {
        let imp = self.imp();

        // Create keyboard widget
        let keyboard = crate::keyboard_widget::KeyboardWidget::new();
        imp.game_area.add_overlay(&keyboard);
        keyboard.set_halign(gtk::Align::Center);
        keyboard.set_valign(gtk::Align::End);
        keyboard.set_margin_bottom(20);
        imp.keyboard_widget.replace(Some(keyboard));

        // Query text color from CSS
        let temp_widget = gtk::Label::new(None);
        temp_widget.add_css_class("falling-key-text");
        let text_color = temp_widget.color();

        // Create falling keys widget
        let keys_widget = FallingKeysWidget::new(imp.falling_keys.clone());
        keys_widget.set_text_color(text_color);
        keys_widget.set_can_focus(true);
        keys_widget.set_focusable(true);

        imp.game_area.add_overlay(&keys_widget);
        imp.falling_keys_widget.replace(Some(keys_widget.clone()));

        // Setup keyboard input
        let key_controller = gtk::EventControllerKey::new();
        let obj = self.downgrade();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if let Some(obj) = obj.upgrade() {
                if let Some(c) = key.to_unicode() {
                    obj.handle_key_press(c.to_ascii_lowercase());
                }
            }
            glib::Propagation::Stop
        });
        keys_widget.add_controller(key_controller);

        keys_widget.grab_focus();

        // Setup restart button
        let obj = self.downgrade();
        imp.restart_button.connect_clicked(move |_| {
            if let Some(obj) = obj.upgrade() {
                obj.restart_game();
            }
        });

        // Start game loop
        self.start_game_loop();
    }

    fn start_game_loop(&self) {
        let imp = self.imp();

        // Don't start if already running
        if *imp.game_loop_running.borrow() {
            return;
        }
        *imp.game_loop_running.borrow_mut() = true;

        let obj = self.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            if let Some(obj) = obj.upgrade() {
                if *obj.imp().game_over.borrow() {
                    *obj.imp().game_loop_running.borrow_mut() = false;
                    return glib::ControlFlow::Break;
                }
                obj.update_game();
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        // Spawn new keys
        let obj = self.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
            if let Some(obj) = obj.upgrade() {
                if *obj.imp().game_over.borrow() {
                    return glib::ControlFlow::Break;
                }
                obj.spawn_key();
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    fn spawn_key(&self) {
        let imp = self.imp();
        let mut rng = rand::thread_rng();

        if let Some(drawing_area) = imp.falling_keys_widget.borrow().as_ref() {
            let width = drawing_area.width() as f64;
            if width > 100.0 {
                let key = KEYS[rng.gen_range(0..KEYS.len())];

                imp.falling_keys.borrow_mut().push(FallingKey {
                    key,
                    x: rng.gen_range(50.0..width - 50.0),
                    y: 0.0,
                });
            }
        }
    }

    fn update_game(&self) {
        let imp = self.imp();
        let speed = *imp.speed.borrow();

        if let Some(drawing_area) = imp.falling_keys_widget.borrow().as_ref() {
            let height = drawing_area.height() as f64;
            let mut keys = imp.falling_keys.borrow_mut();

            // Update positions
            for key in keys.iter_mut() {
                key.y += speed;
            }

            // Check for game over - key reached bottom of view
            if keys.iter().any(|k| k.y > height) {
                *imp.game_over.borrow_mut() = true;
                self.show_game_over();
            }

            drawing_area.queue_draw();
        }
    }

    fn handle_key_press(&self, key: char) {
        let imp = self.imp();

        // Highlight key on keyboard
        if let Some(keyboard) = imp.keyboard_widget.borrow().as_ref() {
            keyboard.set_current_key(Some(key));

            let keyboard_clone = keyboard.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                keyboard_clone.set_current_key(None);
            });
        }

        let mut keys = imp.falling_keys.borrow_mut();

        if let Some(pos) = keys.iter().position(|k| k.key == key) {
            keys.remove(pos);

            let mut score = imp.score.borrow_mut();
            *score += 1;
            let score_text = i18n_fmt! { i18n_fmt("Score: {}", *score) };
            imp.score_label.set_text(&score_text);

            // Increase difficulty every 10 points
            if (*score).is_multiple_of(10) {
                let mut difficulty = imp.difficulty.borrow_mut();
                *difficulty += 1;
                let level_text = i18n_fmt! { i18n_fmt("Level: {}", *difficulty) };
                imp.difficulty_label.set_text(&level_text);

                let mut speed = imp.speed.borrow_mut();
                *speed += 0.5;
            }

            if let Some(drawing_area) = imp.falling_keys_widget.borrow().as_ref() {
                drawing_area.queue_draw();
            }
        } else {
            // Wrong key pressed - decrease score
            let mut score = imp.score.borrow_mut();
            if *score > 0 {
                *score -= 1;
                let score_text = i18n_fmt! { i18n_fmt("Score: {}", *score) };
                imp.score_label.set_text(&score_text);
            }
        }
    }

    fn show_game_over(&self) {
        let imp = self.imp();
        *imp.game_over.borrow_mut() = true;

        if let Some(child) = imp.game_area.child() {
            child.set_visible(false);
        }
        if let Some(keyboard) = imp.keyboard_widget.borrow().as_ref() {
            keyboard.set_visible(false);
        }

        imp.results_score_label
            .set_text(&imp.score.borrow().to_string());
        imp.results_level_label
            .set_text(&imp.difficulty.borrow().to_string());
        imp.results_box.set_visible(true);
    }

    fn restart_game(&self) {
        let imp = self.imp();

        imp.results_box.set_visible(false);

        if let Some(child) = imp.game_area.child() {
            child.set_visible(true);
        }
        if let Some(drawing_area) = imp.falling_keys_widget.borrow().as_ref() {
            drawing_area.set_visible(true);
            drawing_area.grab_focus();
        }
        if let Some(keyboard) = imp.keyboard_widget.borrow().as_ref() {
            keyboard.set_visible(true);
        }

        self.reset();
    }

    pub fn reset(&self) {
        let imp = self.imp();
        imp.falling_keys.borrow_mut().clear();
        *imp.score.borrow_mut() = 0;
        *imp.difficulty.borrow_mut() = 1;
        *imp.speed.borrow_mut() = 2.0;
        *imp.game_over.borrow_mut() = false;

        imp.score_label
            .set_text(&i18n_fmt! { i18n_fmt("Score: {}", 0) });
        imp.difficulty_label
            .set_text(&i18n_fmt! { i18n_fmt("Level: {}", 1) });

        if let Some(drawing_area) = imp.falling_keys_widget.borrow().as_ref() {
            drawing_area.grab_focus();
            drawing_area.queue_draw();
        }

        self.start_game_loop();
    }
}
