use crate::model::Site;
use crate::renderer::Renderer;
use fs_extra::dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use structopt::StructOpt;

mod markdown;
mod model;
mod renderer;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "static-wiki",
    about = "Generate static html files and search index for a wiki."
)]
struct Opt {
    /// Input folder
    #[structopt(parse(from_os_str), short)]
    input: PathBuf,

    /// Output folder
    #[structopt(parse(from_os_str), short)]
    output: PathBuf,

    /// Template folder
    #[structopt(parse(from_os_str), short)]
    template: PathBuf,

    /// Static folder
    #[structopt(parse(from_os_str), short, long = "static")]
    static_path: PathBuf,
}

fn main() {
    let opt: Opt = Opt::from_args();
    let template_path = opt
        .template
        .to_str()
        .unwrap()
        .trim_end_matches('/')
        .to_string()
        + "/*";
    let renderer = Renderer::load_from_path(&template_path);
    let site = Site::load_from_path(opt.input);
    renderer.render_to(site, &opt.output);
    copy_static(opt.static_path, opt.output);
}

fn copy_static(static_path: impl AsRef<Path>, output_base_path: impl AsRef<Path>) {
    let options = dir::CopyOptions {
        overwrite: true,
        copy_inside: true,
        ..Default::default()
    };
    dir::copy(
        &static_path,
        &output_base_path.as_ref().join("static"),
        &options,
    )
    .unwrap();
    fs::copy(
        &output_base_path
            .as_ref()
            .join("static")
            .join("manifest.json"),
        &output_base_path.as_ref().join("manifest.json"),
    )
    .unwrap();
    fs::remove_file(
        &output_base_path
            .as_ref()
            .join("static")
            .join("manifest.json"),
    )
    .unwrap();
    let pattern = output_base_path
        .as_ref()
        .join("static")
        .to_str()
        .unwrap()
        .trim_end_matches('/')
        .to_string()
        + "/*.ts";
    let subprocesses = glob::glob(&pattern)
        .expect("Failed to read glob pattern")
        .map(|entry| match entry {
            Ok(path) => {
                let command = Command::new("tsc")
                    .arg("--target")
                    .arg("es5")
                    .arg(&path.file_name().unwrap().to_str().unwrap())
                    .current_dir(output_base_path.as_ref().join("static"))
                    .spawn()
                    .expect("failed to execute process");
                (command, path)
            }
            Err(e) => {
                panic!("{:?}", e)
            }
        })
        .collect::<Vec<_>>();
    for (mut subprocess, path) in subprocesses {
        subprocess.wait().unwrap();
        fs::remove_file(path).unwrap();
    }
}
