use gettextrs::{bind_textdomain_codeset, bindtextdomain, setlocale, textdomain, LocaleCategory};
use gio::prelude::*;
use glib::warn;
use std::path::PathBuf;

const G_LOG_DOMAIN: &str = "Mecalin";

mod application;
mod config;
mod course;
mod falling_keys_game;
mod keyboard_widget;
mod lesson_view;
mod main_action_list;
mod scrolling_lanes_game;
mod target_text_view;
mod text_view;
mod utils;
mod window;

use application::MecalinApplication;

fn main() {
    setlocale(LocaleCategory::LcAll, "");

    let localedir = PathBuf::from(config::DATADIR).join("locale");
    if let Err(e) = bindtextdomain(config::PACKAGE, localedir) {
        warn!("Failed to bind text domain: {}", e);
    }

    if let Err(e) = bind_textdomain_codeset(config::PACKAGE, "UTF-8") {
        warn!("Failed to set text domain codeset: {}", e);
    }

    if let Err(e) = textdomain(config::PACKAGE) {
        warn!("Failed to set text domain: {}", e);
    }

    if let Err(e) = gio::resources_register_include!("resources.gresource") {
        warn!("Failed to register resources: {}", e);
    }

    let app = MecalinApplication::new();
    app.run();
}
