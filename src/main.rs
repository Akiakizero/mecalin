use gtk::prelude::*;
use libadwaita as adw;

fn main() {
    let app = adw::Application::builder()
        .application_id("com.example.mecalin")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Mecalin")
        .default_width(800)
        .default_height(600)
        .build();

    let header_bar = adw::HeaderBar::new();
    
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let label = gtk::Label::new(Some("Welcome to Mecalin"));
    content.append(&label);

    window.set_titlebar(Some(&header_bar));
    window.set_content(Some(&content));
    window.present();
}
