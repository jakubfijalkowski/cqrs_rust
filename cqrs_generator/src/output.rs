use std::path::{Path, PathBuf};

use prost::Message;

use crate::{contracts::Export, hierarchy::Hierarchy, stmt_builder::StmtBuilder};

pub fn generate(input: impl AsRef<Path>) {
    let mut out_dir: PathBuf = std::env::var("OUT_DIR")
        .expect("this should be run in Cargo")
        .try_into()
        .unwrap();
    let input = std::fs::read(input).expect("cannot read the input file");
    let parsed_export = Export::decode(&input[..])
        .expect("cannot decode the Export, the file is malformed, probably");

    let mut filename = parsed_export.project_name.to_lowercase();
    filename.push_str(".rs");

    out_dir.push(filename);

    write_to(parsed_export, out_dir);
}

pub fn generate_to(input: impl AsRef<Path>, output: impl AsRef<Path>) {
    let input = std::fs::read(input).expect("cannot read the input file");
    let parsed_export = Export::decode(&input[..])
        .expect("cannot decode the Export, the file is malformed, probably");
    write_to(parsed_export, output);
}

pub fn write_to(input: Export, output: impl AsRef<Path>) {
    let hierarchy: Hierarchy = input.into();
    let builder: StmtBuilder = hierarchy.into();
    let contents = builder.build();

    std::fs::write(output, contents).expect("cannot write the output file");
}
