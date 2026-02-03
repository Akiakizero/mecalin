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
                gtk::Orientation::Horizontal => (240, 240, -1, -1),
                gtk::Orientation::Vertical => (125, 125, -1, -1),
                _ => (0, 0, -1, -1),
            }
        }
    }

    impl HandWidget {
        fn get_finger_css_class(finger: &str) -> String {
            format!("finger-{}", finger.replace('_', "-"))
        }

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

            let settings = gtk::gio::Settings::new("io.github.nacho.mecalin");
            let use_finger_colors = settings.boolean("use-finger-colors");
            let default_color = get_color("hand-finger-default");
            let default_border = get_color("hand-finger-border");

            // Finger layout: (name, x, y, width, height)
            let fingers = [
                ("left_pinky", 9.0, 34.0, 17.0, 43.0),
                ("left_ring", 30.0, 17.0, 17.0, 51.0),
                ("left_middle", 51.0, 9.0, 17.0, 60.0),
                ("left_index", 72.0, 17.0, 17.0, 51.0),
                ("right_index", 166.0, 17.0, 17.0, 51.0),
                ("right_middle", 187.0, 9.0, 17.0, 60.0),
                ("right_ring", 208.0, 17.0, 17.0, 51.0),
                ("right_pinky", 230.0, 34.0, 17.0, 43.0),
            ];

            let thumbs = [
                ("left_thumb", 92.0, 68.0, 30.0, 24.0),
                ("right_thumb", 133.0, 68.0, 30.0, 24.0),
            ];

            // Draw left palm
            let palm_color = get_color("hand-palm");
            let palm_border_color = get_color("hand-palm-border");
            let left_palm_rect = graphene::Rect::new(9.0, 64.0, 89.0, 68.0);
            let left_palm_rounded = gsk::RoundedRect::new(
                left_palm_rect,
                graphene::Size::new(17.0, 17.0),
                graphene::Size::new(17.0, 17.0),
                graphene::Size::new(17.0, 17.0),
                graphene::Size::new(17.0, 17.0),
            );
            snapshot.push_rounded_clip(&left_palm_rounded);
            snapshot.append_color(&palm_color, &left_palm_rect);
            let border_width = [1.0, 1.0, 1.0, 1.0];
            let border_color = [
                palm_border_color,
                palm_border_color,
                palm_border_color,
                palm_border_color,
            ];
            snapshot.append_border(&left_palm_rounded, &border_width, &border_color);
            snapshot.pop();

            // Draw right palm
            let right_palm_rect = graphene::Rect::new(157.0, 64.0, 89.0, 68.0);
            let right_palm_rounded = gsk::RoundedRect::new(
                right_palm_rect,
                graphene::Size::new(17.0, 17.0),
                graphene::Size::new(17.0, 17.0),
                graphene::Size::new(17.0, 17.0),
                graphene::Size::new(17.0, 17.0),
            );
            snapshot.push_rounded_clip(&right_palm_rounded);
            snapshot.append_color(&palm_color, &right_palm_rect);
            snapshot.append_border(&right_palm_rounded, &border_width, &border_color);
            snapshot.pop();

            // Draw fingers
            for (finger_name, x, y, w, h) in &fingers {
                let is_current = current.as_ref().is_some_and(|f| f == finger_name);
                let (fill_color, border_color) = if is_current {
                    let c = get_color("hand-finger-current");
                    (c, c)
                } else if use_finger_colors {
                    let c = get_color(&Self::get_finger_css_class(finger_name));
                    (default_color, c)
                } else {
                    (default_color, default_border)
                };
                Self::draw_finger(
                    snapshot,
                    *x,
                    *y,
                    *w,
                    *h,
                    &fill_color,
                    &border_color,
                    is_current,
                );
            }

            // Draw thumbs
            for (i, (thumb_name, x, y, w, h)) in thumbs.iter().enumerate() {
                let is_current = current
                    .as_ref()
                    .is_some_and(|f| f == "both_thumbs" || f == thumb_name);
                let (fill_color, border_color) = if is_current {
                    let c = get_color("hand-finger-current");
                    (c, c)
                } else if use_finger_colors {
                    let c = get_color(&Self::get_finger_css_class(thumb_name));
                    (default_color, c)
                } else {
                    (default_color, default_border)
                };
                let angle = if i == 0 { -35.0 } else { 35.0 };
                Self::draw_thumb(
                    snapshot,
                    *x,
                    *y,
                    *w,
                    *h,
                    &fill_color,
                    &border_color,
                    is_current,
                    angle,
                );
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn draw_finger(
            snapshot: &gtk::Snapshot,
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            color: &gdk::RGBA,
            border_color: &gdk::RGBA,
            is_active: bool,
        ) {
            let rect = graphene::Rect::new(x, y, w, h);
            let rounded = gsk::RoundedRect::new(
                rect,
                graphene::Size::new(9.0, 9.0), // top-left (tip)
                graphene::Size::new(9.0, 9.0), // top-right (tip)
                graphene::Size::new(4.0, 4.0), // bottom-right (base)
                graphene::Size::new(4.0, 4.0), // bottom-left (base)
            );

            if is_active {
                snapshot.push_rounded_clip(&rounded);
                snapshot.append_color(color, &rect);
                snapshot.pop();
            } else {
                snapshot.push_rounded_clip(&rounded);
                snapshot.append_color(color, &rect);
                snapshot.pop();

                let border_width = [1.0, 1.0, 1.0, 1.0];
                let border_colors = [*border_color, *border_color, *border_color, *border_color];
                snapshot.append_border(&rounded, &border_width, &border_colors);
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn draw_thumb(
            snapshot: &gtk::Snapshot,
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            color: &gdk::RGBA,
            border_color: &gdk::RGBA,
            is_active: bool,
            angle: f32,
        ) {
            let center_x = x + w / 2.0;
            let center_y = y + h / 2.0;

            snapshot.save();
            snapshot.translate(&graphene::Point::new(center_x, center_y));
            snapshot.rotate(angle);
            snapshot.translate(&graphene::Point::new(-center_x, -center_y));

            let rect = graphene::Rect::new(x, y, w, h);
            // For thumbs, the "outer" end should be more rounded
            // Left thumb: outer is right side, right thumb: outer is left side
            let rounded = if angle < 0.0 {
                // Left thumb - more rounded on right (outer) side
                gsk::RoundedRect::new(
                    rect,
                    graphene::Size::new(7.0, 7.0),   // top-left (inner)
                    graphene::Size::new(12.0, 12.0), // top-right (outer tip)
                    graphene::Size::new(12.0, 12.0), // bottom-right (outer base)
                    graphene::Size::new(7.0, 7.0),   // bottom-left (inner)
                )
            } else {
                // Right thumb - more rounded on left (outer) side
                gsk::RoundedRect::new(
                    rect,
                    graphene::Size::new(12.0, 12.0), // top-left (outer tip)
                    graphene::Size::new(7.0, 7.0),   // top-right (inner)
                    graphene::Size::new(7.0, 7.0),   // bottom-right (inner)
                    graphene::Size::new(12.0, 12.0), // bottom-left (outer base)
                )
            };

            if is_active {
                snapshot.push_rounded_clip(&rounded);
                snapshot.append_color(color, &rect);
                snapshot.pop();
            } else {
                snapshot.push_rounded_clip(&rounded);
                snapshot.append_color(color, &rect);
                snapshot.pop();

                let border_width = [1.0, 1.0, 1.0, 1.0];
                let border_colors = [*border_color, *border_color, *border_color, *border_color];
                snapshot.append_border(&rounded, &border_width, &border_colors);
            }

            snapshot.restore();
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
