use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use i18n_format::i18n_fmt;
use std::cell::{Cell, RefCell};

use crate::course::Lesson;
use crate::hand_widget::HandWidget;
use crate::keyboard_widget::KeyboardWidget;
use crate::typing_row::TypingRow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/nacho/mecalin/ui/lesson_view.ui")]
    #[properties(wrapper_type = super::LessonView)]
    pub struct LessonView {
        #[template_child]
        pub lesson_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub step_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub continue_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub text_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub typing_row: TemplateChild<TypingRow>,
        #[template_child]
        pub keyboard_container: TemplateChild<gtk::Box>,

        pub hand_widget: RefCell<Option<HandWidget>>,
        pub keyboard_widget: RefCell<Option<KeyboardWidget>>,
        #[property(get, set, nullable)]
        pub current_lesson: RefCell<Option<glib::BoxedAnyObject>>,
        #[property(get, set)]
        pub current_step_index: Cell<u32>,
        pub current_repetition: Cell<u32>,
        pub course: RefCell<Option<crate::course::Course>>,
        pub has_mistake: Cell<bool>,
        pub composition_in_progress: Cell<bool>,
        pub pending_dead_key: RefCell<Option<char>>,
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
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.setup_keyboard();
            self.setup_signals();
            self.setup_settings_signals();
            self.obj().load_course_and_lesson();
        }
    }

    impl WidgetImpl for LessonView {
        fn grab_focus(&self) -> bool {
            self.typing_row.text_input().grab_focus()
        }
    }

    impl BoxImpl for LessonView {}
}

impl imp::LessonView {
    fn setup_keyboard(&self) {
        let hand = HandWidget::new();
        hand.set_halign(gtk::Align::Center);
        hand.set_margin_bottom(24);
        self.keyboard_container.append(&hand);
        self.hand_widget.replace(Some(hand));

        let keyboard = KeyboardWidget::new();
        self.keyboard_container.append(&keyboard);
        self.keyboard_widget.replace(Some(keyboard));
    }

    fn setup_signals(&self) {
        // Setup continue button for introduction steps
        let lesson_view_weak = self.obj().downgrade();
        self.continue_button.connect_clicked(move |_| {
            if let Some(lesson_view) = lesson_view_weak.upgrade() {
                lesson_view.advance_to_next_step();
            }
        });

        // Track composition state for dead keys
        let lesson_view_weak = self.obj().downgrade();
        self.typing_row
            .text_input()
            .connect_preedit_changed(move |_, preedit| {
                if let Some(lesson_view) = lesson_view_weak.upgrade() {
                    let imp = lesson_view.imp();
                    let is_composing = !preedit.is_empty();
                    imp.composition_in_progress.set(is_composing);

                    // If composition started, store the dead key
                    if is_composing && preedit.len() == 1 {
                        if let Some(dead_key) = preedit.chars().next() {
                            *imp.pending_dead_key.borrow_mut() = Some(dead_key);

                            // Advance keyboard sequence to show next character
                            if let Some(keyboard) = imp.keyboard_widget.borrow().as_ref() {
                                keyboard.advance_sequence();
                            }
                        }
                    } else if !is_composing {
                        // Composition ended, clear pending dead key
                        *imp.pending_dead_key.borrow_mut() = None;
                    }
                }
            });

        // Prevent cursor movement - always keep cursor at the end
        let text_input = self.typing_row.text_input();
        let text_input_clone = text_input.clone();
        text_input.connect_move_cursor(move |_, _, _, _| {
            glib::idle_add_local_once({
                let text = text_input_clone.clone();
                move || {
                    let buffer = text.buffer();
                    let text_len = buffer.text().len() as u16;
                    text.set_position(text_len as i32);
                }
            });
        });

        // Also reset cursor position on any notify::cursor-position
        let text_input_clone2 = text_input.clone();
        text_input.connect_notify_local(Some("cursor-position"), move |_, _| {
            let buffer = text_input_clone2.buffer();
            let text_len = buffer.text().len() as u16;
            let current_pos = text_input_clone2.position();
            if current_pos != text_len as i32 {
                text_input_clone2.set_position(text_len as i32);
            }
        });

        let keyboard_widget = self.keyboard_widget.borrow();
        let hand_widget = self.hand_widget.borrow();
        if let Some(keyboard) = keyboard_widget.as_ref() {
            let keyboard_clone = keyboard.clone();
            let hand_clone = hand_widget.as_ref().cloned();
            let typing_row = self.typing_row.clone();
            let lesson_view_clone = self.obj().downgrade();

            let buffer = self.typing_row.buffer();
            buffer.connect_notify_local(Some("text"), move |buffer, _| {
                let typed_text = buffer.text();
                let target_text = typing_row.imp().target_label.text();

                let typed_str = typed_text.as_str();
                let target_str = target_text.as_str();

                // Check if the new text would match target text
                if !target_str.starts_with(typed_str) && !typed_str.is_empty() {
                    // Show error animation
                    typing_row.show_error();

                    // Find the last space position or go to beginning
                    let last_space_pos = typed_str.rfind(' ').map(|pos| pos + 1).unwrap_or(0);

                    // Mark as mistake if:
                    // - Not at the beginning (last_space_pos > 0), OR
                    // - At the beginning but on a repetition after the first (current_repetition > 0)
                    if let Some(lesson_view) = lesson_view_clone.upgrade() {
                        let imp = lesson_view.imp();
                        if last_space_pos > 0 || imp.current_repetition.get() > 0 {
                            imp.has_mistake.set(true);
                        }
                    }

                    // Reset to last space position
                    let corrected_text = &typed_str[..last_space_pos];

                    glib::idle_add_local_once({
                        let buffer = buffer.clone();
                        let typing_row = typing_row.clone();
                        let corrected_text = corrected_text.to_string();
                        let corrected_len = corrected_text.len();
                        move || {
                            buffer.delete_text(0, None);
                            buffer.insert_text(0, &corrected_text);
                            // Set cursor to end of corrected text
                            typing_row.text_input().set_position(corrected_len as i32);
                        }
                    });
                    return;
                }

                let cursor_pos = typed_str.chars().count() as i32;
                typing_row.set_cursor_position(cursor_pos);

                // Check if step is completed
                if typed_str == target_str && !target_str.is_empty() {
                    // Step completed - check if we need more repetitions
                    glib::idle_add_local_once({
                        let lesson_view = lesson_view_clone.clone();
                        move || {
                            if let Some(lesson_view) = lesson_view.upgrade() {
                                lesson_view.handle_step_completion();
                            }
                        }
                    });
                    return;
                }

                // Update keyboard highlighting for next character
                let next_char = target_str.chars().nth(cursor_pos as usize);
                keyboard_clone.set_current_key(next_char);

                // Update hand widget to highlight the finger for next character
                if let Some(hand) = &hand_clone {
                    let finger = next_char.and_then(|ch| keyboard_clone.get_finger_for_char(ch));
                    hand.set_current_finger(finger);
                }
            });
        }
    }

    fn setup_settings_signals(&self) {
        let obj = self.obj();
        obj.connect_notify_local(Some("current-step-index"), |lesson_view, _| {
            let settings = gio::Settings::new("io.github.nacho.mecalin");
            settings
                .set_uint("current-step", lesson_view.current_step_index() + 1)
                .unwrap();
        });
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

    fn load_course_and_lesson(&self) {
        let language = crate::utils::language_from_locale();
        let course = crate::course::Course::new_with_language(language).unwrap_or_default();

        let settings = gio::Settings::new("io.github.nacho.mecalin");
        let current_lesson = settings.uint("current-lesson");
        let current_step = settings.uint("current-step");

        if let Some(lesson) = course.get_lesson(current_lesson) {
            self.set_course(course.clone());
            self.set_lesson(lesson);

            // Load the saved step
            if current_step > 0 {
                let step_index = current_step - 1;
                self.load_step(step_index);
            }
        }
    }

    fn set_lesson(&self, lesson: &Lesson) {
        self.set_current_lesson(Some(glib::BoxedAnyObject::new(lesson.clone())));

        // Save current lesson to settings
        let settings = gio::Settings::new("io.github.nacho.mecalin");
        settings.set_uint("current-lesson", lesson.id).unwrap();

        let imp = self.imp();
        imp.lesson_description.set_text(&lesson.description);

        // Reset step index and repetition count
        self.set_current_step_index(0);
        imp.current_repetition.set(0);

        if lesson.introduction {
            // Introduction lesson - show description and continue button, hide everything else
            imp.step_description.set_visible(false);
            imp.continue_button.set_visible(true);
            imp.text_container.set_visible(false);
        } else {
            // Regular lesson - handle first step
            // Set the first step's text as target text
            if let Some(first_step) = lesson.steps.first() {
                if first_step.introduction {
                    imp.step_description.set_visible(true);
                    imp.step_description.set_text(
                        first_step
                            .description
                            .as_deref()
                            .unwrap_or(&first_step.text),
                    );
                    imp.continue_button.set_visible(true);
                    imp.text_container.set_visible(false);
                } else {
                    imp.step_description.set_visible(false);
                    imp.continue_button.set_visible(false);
                    imp.text_container.set_visible(true);
                    imp.typing_row.set_target_text(&first_step.text);

                    // Show step description if available
                    if let Some(description) = &first_step.description {
                        imp.step_description.set_visible(true);
                        imp.step_description.set_text(description);
                    }

                    self.update_repetition_label();

                    // Focus the text view for immediate typing
                    imp.typing_row.text_input().grab_focus();
                }

                // Extract unique characters from the lesson text for keyboard display
                let mut target_keys = std::collections::HashSet::new();
                for ch in first_step.text.chars() {
                    if !ch.is_control() {
                        target_keys.insert(ch.to_lowercase().next().unwrap_or(ch));
                    }
                }

                let keyboard_widget = imp.keyboard_widget.borrow();
                if let Some(keyboard) = keyboard_widget.as_ref() {
                    keyboard.set_visible_keys(Some(target_keys));
                }
            }
        }

        imp.typing_row.buffer().delete_text(0, None);
        imp.has_mistake.set(false);
    }

    fn load_step(&self, step_index: u32) {
        self.set_current_step_index(step_index);

        let imp = self.imp();
        // Reset repetition count for new step
        imp.current_repetition.set(0);
        imp.has_mistake.set(false);

        let current_lesson_boxed = imp.current_lesson.borrow();
        if let Some(boxed) = current_lesson_boxed.as_ref() {
            if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                if let Some(step) = lesson.steps.get(step_index as usize) {
                    if step.introduction {
                        // Introduction step - show description and continue button, hide text views
                        imp.step_description.set_visible(true);
                        imp.step_description
                            .set_text(step.description.as_deref().unwrap_or(&step.text));
                        imp.continue_button.set_visible(true);
                        imp.text_container.set_visible(false);
                    } else {
                        // Regular step - show description if available, show text views
                        if let Some(description) = &step.description {
                            imp.step_description.set_visible(true);
                            imp.step_description.set_text(description);
                        } else {
                            imp.step_description.set_visible(false);
                        }
                        imp.continue_button.set_visible(false);
                        imp.text_container.set_visible(true);
                        imp.typing_row.set_target_text(&step.text);
                        imp.typing_row.buffer().delete_text(0, None);
                        self.update_repetition_label();

                        // Focus the text view for immediate typing
                        imp.typing_row.text_input().grab_focus();
                    }

                    // Update keyboard for this step
                    let mut target_keys = std::collections::HashSet::new();
                    for ch in step.text.chars() {
                        if !ch.is_control() {
                            target_keys.insert(ch.to_lowercase().next().unwrap_or(ch));
                        }
                    }

                    let keyboard_widget = imp.keyboard_widget.borrow();
                    let hand_widget = imp.hand_widget.borrow();
                    if let Some(keyboard) = keyboard_widget.as_ref() {
                        keyboard.set_visible_keys(Some(target_keys));

                        // Set initial key/finger highlight
                        let first_char = step.text.chars().next();
                        keyboard.set_current_key(first_char);

                        if let Some(hand) = hand_widget.as_ref() {
                            let finger = first_char.and_then(|ch| keyboard.get_finger_for_char(ch));
                            hand.set_current_finger(finger);
                        }
                    }
                }
            }
        }
    }

    fn set_course(&self, course: crate::course::Course) {
        let imp = self.imp();
        *imp.course.borrow_mut() = Some(course);
    }

    fn reset_repetition_count(&self) {
        let imp = self.imp();
        imp.current_repetition.set(0);
        self.update_repetition_label();
    }

    fn update_repetition_label(&self) {
        let imp = self.imp();
        let current_repetition = imp.current_repetition.get();

        let current_lesson_boxed = imp.current_lesson.borrow();
        if let Some(boxed) = current_lesson_boxed.as_ref() {
            if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                let step_index = self.current_step_index() as usize;
                if let Some(step) = lesson.steps.get(step_index) {
                    let label_text =
                        i18n_fmt! { i18n_fmt("{}/{} Good", current_repetition, step.repetitions) };
                    imp.typing_row.set_repetition_text(&label_text);
                }
            }
        }
    }

    fn handle_step_completion(&self) {
        let imp = self.imp();

        // Check if there was a mistake during this attempt
        if imp.has_mistake.get() {
            // Restart the step - reset repetition count and clear text
            self.reset_repetition_count();
            imp.has_mistake.set(false);
            imp.typing_row.buffer().delete_text(0, None);
            imp.typing_row.text_input().grab_focus();
            return;
        }

        let current_repetition = imp.current_repetition.get() + 1;
        imp.current_repetition.set(current_repetition);

        let current_lesson_boxed = imp.current_lesson.borrow();
        if let Some(boxed) = current_lesson_boxed.as_ref() {
            if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                let step_index = self.current_step_index() as usize;
                if let Some(step) = lesson.steps.get(step_index) {
                    self.update_repetition_label();

                    if current_repetition >= step.repetitions {
                        // Required repetitions completed, advance to next step
                        self.advance_to_next_step();
                    } else {
                        // Need more repetitions, clear text for next attempt
                        imp.typing_row.buffer().delete_text(0, None);

                        // Focus the text view for next repetition
                        imp.typing_row.text_input().grab_focus();
                    }
                }
            }
        }
    }

    fn advance_to_next_step(&self) {
        let imp = self.imp();

        // Check if this is an introduction lesson
        let is_introduction_lesson = {
            let current_lesson_boxed = imp.current_lesson.borrow();
            if let Some(boxed) = current_lesson_boxed.as_ref() {
                if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                    lesson.introduction
                } else {
                    false
                }
            } else {
                false
            }
        };

        if is_introduction_lesson {
            // Introduction lesson completed - try to load next lesson
            let current_lesson_id = {
                let current_lesson_boxed = imp.current_lesson.borrow();
                if let Some(boxed) = current_lesson_boxed.as_ref() {
                    if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                        lesson.id
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            };

            let next_lesson_option = {
                let course = imp.course.borrow();
                course
                    .as_ref()
                    .and_then(|c| c.get_lesson(current_lesson_id + 1).cloned())
            };

            if let Some(next_lesson) = next_lesson_option {
                // Load next lesson
                self.set_lesson(&next_lesson);
            } else {
                // All lessons completed
                imp.lesson_description
                    .set_text(&gettext("Course completed! Congratulations!"));
                imp.step_description.set_visible(false);
                imp.continue_button.set_visible(false);
                imp.text_container.set_visible(false);
            }
            return;
        }

        // Get the current lesson info without borrowing
        let (current_lesson_id, current_step, total_steps) = {
            let current_lesson_boxed = imp.current_lesson.borrow();
            if let Some(boxed) = current_lesson_boxed.as_ref() {
                if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                    (
                        lesson.id,
                        self.current_step_index() as usize,
                        lesson.steps.len(),
                    )
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        let next_step = current_step + 1;

        if next_step < total_steps {
            // Move to next step within current lesson
            self.load_step(next_step as u32);
        } else {
            // Current lesson completed - try to load next lesson
            let next_lesson_option = {
                let course = imp.course.borrow();
                course
                    .as_ref()
                    .and_then(|c| c.get_lesson(current_lesson_id + 1).cloned())
            };

            if let Some(next_lesson) = next_lesson_option {
                // Load next lesson
                self.set_lesson(&next_lesson);
            } else {
                // Check if we have a course to determine the message
                let has_course = imp.course.borrow().is_some();
                if has_course {
                    // All lessons completed
                    imp.typing_row
                        .set_target_text(&gettext("Course completed! Congratulations!"));
                } else {
                    // No course set, just show lesson completion
                    imp.typing_row
                        .set_target_text(&gettext("Lesson completed! Well done!"));
                }
                imp.typing_row.buffer().delete_text(0, None);
            }
        }
    }
}
