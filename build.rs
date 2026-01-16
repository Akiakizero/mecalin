use glib_build_tools::compile_resources;
use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;

fn generate_config() {
    let template = fs::read_to_string("src/config.rs.in").expect("Failed to read src/config.rs.in");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("config.rs");

    let re = Regex::new(r"@(\w+)@").unwrap();
    let content = re
        .replace_all(&template, |captures: &regex::Captures| {
            env::var(&captures[1]).unwrap_or_default()
        })
        .into_owned();

    fs::write(&dest_path, content).unwrap();
    println!("cargo:rerun-if-changed=src/config.rs.in");
}

fn main() {
    generate_config();

    compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "resources.gresource",
    );
}
