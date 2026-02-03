use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{graphene, gsk};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct HandWidget {
        pub current_finger: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HandWidget {
        const NAME: &'static str = "MecalinHandWidget";
        type Type = super::HandWidget;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for HandWidget {}

    impl WidgetImpl for HandWidget {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;

            if width <= 0.0 || height <= 0.0 {
                return;
            }

            Self::draw_hand(snapshot, &widget, &self.current_finger);
        }

        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (290, 290, -1, -1),
                gtk::Orientation::Vertical => (140, 140, -1, -1),
                _ => (0, 0, -1, -1),
            }
        }
    }

    impl HandWidget {
        fn draw_hand(
            snapshot: &gtk::Snapshot,
            widget: &super::HandWidget,
            current_finger: &RefCell<Option<String>>,
        ) {
            let current = current_finger.borrow();

            let get_color = |class_name: &str| -> gdk::RGBA {
                widget.add_css_class(class_name);
                let color = widget.color();
                widget.remove_css_class(class_name);
                color
            };

            // Finger layout: (name, x, y, width, height)
            let fingers = [
                ("left_pinky", 10.0, 40.0, 20.0, 50.0),
                ("left_ring", 35.0, 20.0, 20.0, 60.0),
                ("left_middle", 60.0, 10.0, 20.0, 70.0),
                ("left_index", 85.0, 20.0, 20.0, 60.0),
                ("right_index", 195.0, 20.0, 20.0, 60.0),
                ("right_middle", 220.0, 10.0, 20.0, 70.0),
                ("right_ring", 245.0, 20.0, 20.0, 60.0),
                ("right_pinky", 270.0, 40.0, 20.0, 50.0),
            ];

            let thumbs = [
                ("left_thumb", 110.0, 85.0, 30.0, 25.0),
                ("right_thumb", 160.0, 85.0, 30.0, 25.0),
            ];

            // Draw left palm
            let palm_color = get_color("hand-palm");
            let left_palm_rect = graphene::Rect::new(10.0, 70.0, 105.0, 70.0);
            let left_palm_rounded = gsk::RoundedRect::new(
                left_palm_rect,
                graphene::Size::new(20.0, 20.0),
                graphene::Size::new(20.0, 20.0),
                graphene::Size::new(20.0, 20.0),
                graphene::Size::new(20.0, 20.0),
            );
            snapshot.push_rounded_clip(&left_palm_rounded);
            snapshot.append_color(&palm_color, &left_palm_rect);
            snapshot.pop();

            // Draw right palm
            let right_palm_rect = graphene::Rect::new(185.0, 70.0, 105.0, 70.0);
            let right_palm_rounded = gsk::RoundedRect::new(
                right_palm_rect,
                graphene::Size::new(20.0, 20.0),
                graphene::Size::new(20.0, 20.0),
                graphene::Size::new(20.0, 20.0),
                graphene::Size::new(20.0, 20.0),
            );
            snapshot.push_rounded_clip(&right_palm_rounded);
            snapshot.append_color(&palm_color, &right_palm_rect);
            snapshot.pop();

            // Draw fingers
            for (finger_name, x, y, w, h) in &fingers {
                let is_current = current.as_ref().is_some_and(|f| f == finger_name);
                let color = if is_current {
                    get_color("hand-finger-current")
                } else {
                    get_color("hand-finger-default")
                };
                Self::draw_finger(snapshot, *x, *y, *w, *h, &color);
            }

            // Draw thumbs
            for (thumb_name, x, y, w, h) in &thumbs {
                let is_current = current
                    .as_ref()
                    .is_some_and(|f| f == "both_thumbs" || f == thumb_name);
                let color = if is_current {
                    get_color("hand-finger-current")
                } else {
                    get_color("hand-finger-default")
                };
                Self::draw_finger(snapshot, *x, *y, *w, *h, &color);
            }
        }

        fn draw_finger(
            snapshot: &gtk::Snapshot,
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            color: &gdk::RGBA,
        ) {
            let rect = graphene::Rect::new(x, y, w, h);
            let rounded = gsk::RoundedRect::new(
                rect,
                graphene::Size::new(8.0, 8.0),
                graphene::Size::new(8.0, 8.0),
                graphene::Size::new(8.0, 8.0),
                graphene::Size::new(8.0, 8.0),
            );
            snapshot.push_rounded_clip(&rounded);
            snapshot.append_color(color, &rect);
            snapshot.pop();
        }
    }
}

glib::wrapper! {
    pub struct HandWidget(ObjectSubclass<imp::HandWidget>)
        @extends gtk::Widget;
}

impl HandWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_current_finger(&self, finger: Option<String>) {
        *self.imp().current_finger.borrow_mut() = finger;
        self.queue_draw();
    }
}

impl Default for HandWidget {
    fn default() -> Self {
        Self::new()
    }
}
