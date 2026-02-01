use gtk::gdk;
use gtk::glib;
use gtk::graphene;
use gtk::pango;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use i18n_format::i18n_fmt;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct ScrollingText {
    text: String,
    x: f64,
}

mod lane_widget {
    use super::*;

    pub struct LaneWidget {
        pub(super) lane_index: RefCell<usize>,
        pub(super) current_lane: RefCell<Rc<RefCell<usize>>>,
        pub(super) texts: RefCell<Rc<RefCell<Vec<Vec<ScrollingText>>>>>,
        pub(super) bg_color: RefCell<gdk::RGBA>,
        pub(super) current_color: RefCell<gdk::RGBA>,
        pub(super) text_color: RefCell<gdk::RGBA>,
    }

    impl Default for LaneWidget {
        fn default() -> Self {
            Self {
                lane_index: RefCell::new(0),
                current_lane: RefCell::new(Rc::new(RefCell::new(0))),
                texts: RefCell::new(Rc::new(RefCell::new(Vec::new()))),
                bg_color: RefCell::new(gdk::RGBA::BLACK),
                current_color: RefCell::new(gdk::RGBA::BLACK),
                text_color: RefCell::new(gdk::RGBA::WHITE),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LaneWidget {
        const NAME: &'static str = "LaneWidget";
        type Type = super::LaneWidget;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for LaneWidget {}

    impl WidgetImpl for LaneWidget {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;

            if width <= 0.0 || height <= 0.0 {
                return;
            }

            let current = *self.current_lane.borrow().borrow();
            let lane_index = *self.lane_index.borrow();
            let bg_color = if current == lane_index {
                &*self.current_color.borrow()
            } else {
                &*self.bg_color.borrow()
            };

            // Background
            let bounds = graphene::Rect::new(0.0, 0.0, width, height);
            snapshot.append_color(bg_color, &bounds);

            // Draw texts
            let pango_context = widget.pango_context();
            let layout = pango::Layout::new(&pango_context);
            let font_desc = pango::FontDescription::from_string("Sans 20");
            layout.set_font_description(Some(&font_desc));

            let text_color = self.text_color.borrow();

            if let Ok(all_texts) = self.texts.borrow().try_borrow() {
                for text in &all_texts[lane_index] {
                    if text.x < width as f64 && text.x > -200.0 {
                        layout.set_text(&text.text);
                        snapshot.save();
                        snapshot.translate(&graphene::Point::new(text.x as f32, 30.0));
                        snapshot.append_layout(&layout, &text_color);
                        snapshot.restore();
                    }
                }
            }
        }

        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Vertical => (100, 100, -1, -1),
                _ => (0, 0, -1, -1),
            }
        }
    }
}

glib::wrapper! {
    pub struct LaneWidget(ObjectSubclass<lane_widget::LaneWidget>)
        @extends gtk::Widget;
}

impl LaneWidget {
    pub(crate) fn new(
        lane_index: usize,
        current_lane: Rc<RefCell<usize>>,
        texts: Rc<RefCell<Vec<Vec<ScrollingText>>>>,
    ) -> Self {
        let widget: Self = glib::Object::new();
        let imp = widget.imp();
        *imp.lane_index.borrow_mut() = lane_index;
        *imp.current_lane.borrow_mut() = current_lane;
        *imp.texts.borrow_mut() = texts;
        widget.set_vexpand(true);
        widget.set_hexpand(true);
        widget
    }

    pub fn set_colors(&self, bg: gdk::RGBA, current: gdk::RGBA, text: gdk::RGBA) {
        let imp = self.imp();
        *imp.bg_color.borrow_mut() = bg;
        *imp.current_color.borrow_mut() = current;
        *imp.text_color.borrow_mut() = text;
    }
}

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/scrolling_lanes_game.ui")]
    pub struct ScrollingLanesGame {
        #[template_child]
        pub game_area: TemplateChild<gtk::Box>,
        #[template_child]
        pub score_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub level_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub results_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub results_score_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub results_level_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub restart_button: TemplateChild<gtk::Button>,

        pub lanes: Rc<RefCell<Vec<LaneWidget>>>,
        pub(crate) lane_texts: Rc<RefCell<Vec<Vec<ScrollingText>>>>,
        pub current_lane: Rc<RefCell<usize>>,
        pub score: RefCell<u32>,
        pub difficulty: RefCell<u32>,
        pub speed: RefCell<f64>,
        pub game_over: RefCell<bool>,
        pub game_loop_running: RefCell<bool>,
        pub lanes_container: RefCell<Option<gtk::Box>>,
        pub word_list: RefCell<Vec<String>>,
    }

    impl Default for ScrollingLanesGame {
        fn default() -> Self {
            use std::str::FromStr;

            let lang_code = crate::utils::language_from_locale();
            let language = crate::text_generation::Language::from_str(lang_code)
                .unwrap_or(crate::text_generation::Language::English);
            let text = crate::text_generation::simple(language);
            let word_list: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();

            Self {
                game_area: Default::default(),
                score_label: Default::default(),
                level_label: Default::default(),
                results_box: Default::default(),
                results_score_label: Default::default(),
                results_level_label: Default::default(),
                restart_button: Default::default(),
                lanes: Default::default(),
                lane_texts: Default::default(),
                current_lane: Default::default(),
                score: Default::default(),
                difficulty: Default::default(),
                speed: Default::default(),
                game_over: Default::default(),
                game_loop_running: Default::default(),
                lanes_container: Default::default(),
                word_list: RefCell::new(word_list),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScrollingLanesGame {
        const NAME: &'static str = "ScrollingLanesGame";
        type Type = super::ScrollingLanesGame;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScrollingLanesGame {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_game();
        }
    }
    impl WidgetImpl for ScrollingLanesGame {}
    impl BoxImpl for ScrollingLanesGame {}
}

glib::wrapper! {
    pub struct ScrollingLanesGame(ObjectSubclass<imp::ScrollingLanesGame>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ScrollingLanesGame {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup_game(&self) {
        let imp = self.imp();

        // Create 4 lanes
        let lanes_container = gtk::Box::new(gtk::Orientation::Vertical, 2);
        lanes_container.set_vexpand(true);
        lanes_container.set_hexpand(true);

        // Query colors
        let temp_widget = gtk::Label::new(None);
        temp_widget.add_css_class("lane-background");
        let bg_color = temp_widget.color();
        temp_widget.remove_css_class("lane-background");

        temp_widget.add_css_class("lane-current");
        let current_color = temp_widget.color();
        temp_widget.remove_css_class("lane-current");

        temp_widget.add_css_class("lane-text");
        let text_color = temp_widget.color();

        let mut lanes = Vec::new();
        let lane_texts = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        imp.lane_texts.replace(lane_texts);

        for i in 0..4 {
            let lane = LaneWidget::new(i, imp.current_lane.clone(), imp.lane_texts.clone());
            lane.set_colors(bg_color, current_color, text_color);
            lanes_container.append(&lane);
            lanes.push(lane);
        }

        imp.game_area.append(&lanes_container);
        imp.lanes.replace(lanes);
        imp.lanes_container.replace(Some(lanes_container));

        // Setup restart button
        let obj = self.downgrade();
        imp.restart_button.connect_clicked(move |_| {
            if let Some(obj) = obj.upgrade() {
                obj.restart_game();
            }
        });

        // Setup keyboard input
        let key_controller = gtk::EventControllerKey::new();
        let obj = self.downgrade();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if let Some(obj) = obj.upgrade() {
                obj.handle_key_press(key);
            }
            glib::Propagation::Stop
        });
        self.add_controller(key_controller);

        self.set_can_focus(true);
        self.set_focusable(true);

        // Grab focus after widget is realized
        let obj = self.downgrade();
        self.connect_realize(move |_| {
            if let Some(obj) = obj.upgrade() {
                obj.grab_focus();
            }
        });

        // Start game loop
        self.start_game_loop();
    }

    fn start_game_loop(&self) {
        let imp = self.imp();

        if *imp.game_loop_running.borrow() {
            return;
        }
        *imp.game_loop_running.borrow_mut() = true;

        // Update game
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

        // Spawn new texts
        let obj = self.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
            if let Some(obj) = obj.upgrade() {
                if *obj.imp().game_over.borrow() {
                    return glib::ControlFlow::Break;
                }
                obj.spawn_text();
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    fn spawn_text(&self) {
        let imp = self.imp();
        let mut rng = rand::thread_rng();

        let lane_index = rng.gen_range(0..4);
        let word = imp
            .word_list
            .borrow()
            .choose(&mut rng)
            .expect("word list contains at least 1 word")
            .clone();

        if let Some(lane) = imp.lanes.borrow().get(lane_index) {
            let width = lane.width() as f64;
            imp.lane_texts.borrow_mut()[lane_index].push(ScrollingText {
                text: word,
                x: width,
            });
        }
    }

    fn update_game(&self) {
        let imp = self.imp();
        let speed = *imp.speed.borrow();

        let mut texts = imp.lane_texts.borrow_mut();
        let lanes = imp.lanes.borrow();

        for (lane_index, lane_texts) in texts.iter_mut().enumerate() {
            for text in lane_texts.iter_mut() {
                text.x -= speed;
            }

            // Check for game over
            if lane_texts.iter().any(|t| t.x < -200.0) {
                *imp.game_over.borrow_mut() = true;
                drop(texts);
                drop(lanes);
                self.show_game_over();
                return;
            }

            // Queue redraw
            if let Some(lane) = lanes.get(lane_index) {
                lane.queue_draw();
            }
        }
    }

    fn handle_key_press(&self, key: gtk::gdk::Key) {
        let imp = self.imp();
        let key_name = key.name();

        if key_name.as_deref() == Some("Up") {
            {
                let mut current = imp.current_lane.borrow_mut();
                if *current > 0 {
                    *current -= 1;
                }
            }
            // Redraw all lanes
            for lane in imp.lanes.borrow().iter() {
                lane.queue_draw();
            }
        } else if key_name.as_deref() == Some("Down") {
            {
                let mut current = imp.current_lane.borrow_mut();
                if *current < 3 {
                    *current += 1;
                }
            }
            // Redraw all lanes
            for lane in imp.lanes.borrow().iter() {
                lane.queue_draw();
            }
        } else if let Some(c) = key.to_unicode() {
            // Type to clear text in current lane
            self.handle_typing(c);
            // Redraw all lanes
            for lane in imp.lanes.borrow().iter() {
                lane.queue_draw();
            }
        }
    }

    fn handle_typing(&self, c: char) {
        let imp = self.imp();
        let current_lane = *imp.current_lane.borrow();

        let (found, score_changed) = {
            let mut texts = imp.lane_texts.borrow_mut();

            if let Some(lane_texts) = texts.get_mut(current_lane) {
                // Find leftmost text that starts with this character
                if let Some(pos) = lane_texts.iter().position(|t| t.text.starts_with(c)) {
                    // Remove first character from the text
                    let text = &mut lane_texts[pos].text;
                    text.remove(0);

                    // If text is now empty, remove it completely
                    if text.is_empty() {
                        lane_texts.remove(pos);
                    }

                    (true, true)
                } else {
                    (false, true)
                }
            } else {
                (false, false)
            }
        };

        if score_changed {
            let mut score = imp.score.borrow_mut();
            if found {
                *score += 1;
                let score_text = i18n_fmt! { i18n_fmt("Score: {}", *score) };
                imp.score_label.set_text(&score_text);

                if (*score).is_multiple_of(10) {
                    let mut difficulty = imp.difficulty.borrow_mut();
                    *difficulty += 1;
                    let level_text = i18n_fmt! { i18n_fmt("Level: {}", *difficulty) };
                    imp.level_label.set_text(&level_text);

                    let mut speed = imp.speed.borrow_mut();
                    *speed += 0.5;
                }
            } else if *score > 0 {
                *score -= 1;
                let score_text = i18n_fmt! { i18n_fmt("Score: {}", *score) };
                imp.score_label.set_text(&score_text);
            }
        }
    }

    fn show_game_over(&self) {
        let imp = self.imp();
        *imp.game_over.borrow_mut() = true;

        if let Some(lanes) = imp.lanes_container.borrow().as_ref() {
            lanes.set_visible(false);
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

        if let Some(lanes) = imp.lanes_container.borrow().as_ref() {
            lanes.set_visible(true);
        }

        self.grab_focus();
        self.reset();
    }

    pub fn reset(&self) {
        let imp = self.imp();
        imp.lane_texts
            .borrow_mut()
            .iter_mut()
            .for_each(|v| v.clear());
        *imp.current_lane.borrow_mut() = 0;
        *imp.score.borrow_mut() = 0;
        *imp.difficulty.borrow_mut() = 1;
        *imp.speed.borrow_mut() = 2.0;
        *imp.game_over.borrow_mut() = false;

        imp.score_label
            .set_text(&i18n_fmt! { i18n_fmt("Score: {}", 0) });
        imp.level_label
            .set_text(&i18n_fmt! { i18n_fmt("Level: {}", 1) });

        for lane in imp.lanes.borrow().iter() {
            lane.queue_draw();
        }

        // Ensure focus
        glib::idle_add_local_once({
            let obj = self.clone();
            move || {
                obj.grab_focus();
            }
        });

        self.start_game_loop();
    }
}
