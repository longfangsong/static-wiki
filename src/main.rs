mod markdown;

use crate::markdown::Markdown;
use fs_extra::dir;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

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
}

fn main() {
    let opt = Opt::from_args();
    fs::remove_dir_all(&opt.output).unwrap_or(());
    fs::create_dir_all(&opt.output).unwrap();
    let mut search_indexes = vec![];
    for entry in fs::read_dir(opt.input).unwrap().into_iter() {
        let entry = entry.unwrap();
        if entry.file_name() == ".DS_Store" {
            // fuck you, macOS!
            continue;
        }
        if entry.metadata().unwrap().is_file() && entry.path().extension().unwrap() == "md" {
            let mut destination_filename = entry.path();
            destination_filename.set_extension("htmlpart");
            let destination_filename = destination_filename.file_name().unwrap();
            let destination_path = opt.output.join(&destination_filename);
            let input_file = fs::File::open(entry.path()).unwrap();
            let mut output_file = fs::File::create(destination_path).unwrap();
            let markdown = Markdown::from_file(
                input_file,
                entry.path().file_stem().unwrap().to_str().unwrap(),
            )
            .unwrap();
            search_indexes.push(markdown.search_index());
            output_file.write_all(markdown.html().as_bytes()).unwrap();
        } else if entry.metadata().unwrap().is_file() {
            let destination_path = opt.output.join(entry.file_name());
            fs::copy(entry.path(), destination_path).unwrap();
        } else {
            let destination_path = opt.output.join(entry.file_name());
            let config = dir::CopyOptions {
                overwrite: true,
                copy_inside: true,
                ..Default::default()
            };
            dir::copy(entry.path(), destination_path, &config).unwrap();
        }
    }
    let index_file = fs::File::create(opt.output.join("index.json")).unwrap();
    serde_json::to_writer(index_file, &search_indexes).unwrap();
}
