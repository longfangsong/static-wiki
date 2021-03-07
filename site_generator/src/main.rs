use crate::model::Site;
// use crate::renderer::Renderer;
use crate::renderer::Renderer;
use fs_extra::dir;
use std::fs;
use std::process::Command;

mod markdown;
mod model;
mod renderer;

fn main() {
    let renderer = Renderer::load_from_path("example/templates/*");
    let site = Site::load_from_path("./example/input/");
    renderer.render_to(site, "./example/output/");
    copy_static();
}

fn copy_static() {
    let options = dir::CopyOptions {
        overwrite: true,
        copy_inside: true,
        ..Default::default()
    };
    dir::copy("./example/static", "./example/output/static", &options).unwrap();
    Command::new("tsc")
        .arg("table.ts")
        .current_dir("./example/output/static")
        .output()
        .expect("failed to execute process");
    for entry in glob::glob("./example/output/static/*.ts").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => fs::remove_file(path).unwrap(),
            Err(e) => println!("{:?}", e),
        }
    }
}
