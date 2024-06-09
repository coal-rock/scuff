use crate::{makefile::TargetData, parser::Stmt, project::Project};
use md5::{Digest, Md5};
use std::{fs::File, io::Write, path::PathBuf};
use zip::{write::SimpleFileOptions, ZipWriter};

// TODO: error handling, please!

pub fn package_project(
    project: &Project,
    targets: Vec<(TargetData, Vec<Stmt>)>,
    output_path: PathBuf,
) {
    let file = File::create(output_path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let json = serde_json::to_string(project).unwrap();

    zip.start_file("project.json", options).unwrap();
    zip.write_all(&json.into_bytes()).unwrap();

    let mut written_filenames: Vec<String> = vec![];

    for target in targets {
        for costume in target.0.costumes {
            let extension = costume.path.extension().unwrap().to_str().unwrap();

            let mut hasher = Md5::new();
            hasher.update(&costume.content);
            let hash = format!("{:x}", hasher.finalize());

            let filename = format!("{}.{}", hash, extension);

            if written_filenames.contains(&filename) {
                continue;
            }

            written_filenames.push(filename.clone());
            zip.start_file(filename, options).unwrap();
            zip.write_all(&costume.content).unwrap();
        }

        // TODO: sound!!!
    }

    zip.finish().unwrap();
}
