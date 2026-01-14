use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::course::Course;
use crate::lesson_view::LessonView;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/study_room.ui")]
    pub struct StudyRoom {
        #[template_child]
        pub lesson_view_widget: TemplateChild<LessonView>,

        pub course: std::cell::RefCell<Option<Course>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StudyRoom {
        const NAME: &'static str = "StudyRoom";
        type Type = super::StudyRoom;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StudyRoom {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_room();
        }
    }

    impl WidgetImpl for StudyRoom {}
    impl BoxImpl for StudyRoom {}
}

impl imp::StudyRoom {
    fn setup_room(&self) {
        let language = crate::utils::language_from_locale();
        let course = Course::new_with_language(language).unwrap_or_default();

        let settings = gio::Settings::new("io.github.nacho.mecalin");
        let current_lesson = settings.uint("current-lesson");

        if let Some(lesson) = course.get_lesson(current_lesson) {
            self.lesson_view_widget.set_course(course.clone());
            self.lesson_view_widget.set_lesson(lesson);
        }

        self.course.replace(Some(course));
    }
}

glib::wrapper! {
    pub struct StudyRoom(ObjectSubclass<imp::StudyRoom>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl StudyRoom {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn can_go_back(&self) -> bool {
        false
    }

    pub fn lesson_view(&self) -> &LessonView {
        &self.imp().lesson_view_widget
    }
}
