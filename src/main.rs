mod application;
mod course;
mod falling_keys_game;
mod keyboard_widget;
mod lesson_view;
mod main_action_list;
mod scrolling_lanes_game;
mod target_text_view;
mod text_view;

mod config {
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}
mod utils;
mod window;

use anyhow::Result;
use gettextrs::{bind_textdomain_codeset, bindtextdomain, setlocale, textdomain, LocaleCategory};
use gio::prelude::*;
use std::path::PathBuf;

use application::MecalinApplication;

fn run_application() -> Result<()> {
    setlocale(LocaleCategory::LcAll, "");

    let localedir = PathBuf::from(config::DATADIR).join("locale");
    bindtextdomain(config::PACKAGE, localedir)?;
    bind_textdomain_codeset(config::PACKAGE, "UTF-8")?;
    textdomain(config::PACKAGE)?;

    gio::resources_register_include!("resources.gresource")?;

    let app = MecalinApplication::new();
    app.run();

    Ok(())
}

fn main() {
    if let Err(e) = run_application() {
        eprintln!("Application initialization failed: {e}");
    }
}
