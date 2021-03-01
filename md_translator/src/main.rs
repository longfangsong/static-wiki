use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use structopt::StructOpt;

use crate::markdown::{ContentFile, IndexFile, Markdown, SearchIndex};

mod markdown;

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

fn translate_dir<T1, T2>(dir: PathBuf, base_path: T1, output_base_path: T2) -> Vec<SearchIndex>
where
    T1: AsRef<Path>,
    T2: AsRef<Path>,
{
    let mut search_indexes = Vec::new();
    for entry in fs::read_dir(dir).unwrap().into_iter() {
        let entry = entry.unwrap();
        let path = entry.path().clone();
        if path.file_name().unwrap() == ".DS_Store" {
            // fuck you, macOS!
            continue;
        } else if entry.metadata().unwrap().is_file() && path.extension().unwrap() == "md" {
            let relative_path = path.strip_prefix(&base_path).unwrap();
            let mut destination_path = output_base_path.as_ref().join(relative_path);
            fs::create_dir_all(destination_path.parent().unwrap()).unwrap();
            destination_path.set_extension("htmlpart");
            let input_file = fs::File::open(&path).unwrap();
            let mut output_file = fs::File::create(destination_path).unwrap();
            if entry.file_name() == "index.md" {
                let markdown =
                    IndexFile::from_file(input_file, relative_path.to_str().unwrap()).unwrap();
                output_file.write_all(markdown.html().as_bytes()).unwrap();
            } else {
                let markdown =
                    ContentFile::from_file(input_file, relative_path.to_str().unwrap()).unwrap();
                output_file.write_all(markdown.html().as_bytes()).unwrap();
                search_indexes.push(markdown.search_index());
            }
        } else if entry.metadata().unwrap().is_file() {
            let relative_path = path.strip_prefix(&base_path).unwrap();
            let destination_path = output_base_path.as_ref().join(relative_path);
            fs::create_dir_all(destination_path.parent().unwrap()).unwrap();
            fs::copy(path, destination_path).unwrap();
        } else if entry.metadata().unwrap().is_dir() {
            let mut sub_entry_result =
                translate_dir(path.clone(), base_path.as_ref(), output_base_path.as_ref());
            search_indexes.append(&mut sub_entry_result);
        }
    }
    search_indexes
}

fn main() {
    let opt = Opt::from_args();
    let output = opt.output.clone();
    let search_indexes = translate_dir(opt.input.clone(), opt.input.clone(), output);
    let index_file = fs::File::create(opt.output.join("index.json")).unwrap();
    serde_json::to_writer(index_file, &search_indexes).unwrap();
}