//! Every YAML under the top-level `examples/` folder must stay loadable, so
//! a config change can't silently break the documented examples.

use std::path::{Path, PathBuf};

use ippdme_imposter::ImposterConfig;

fn yaml_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            yaml_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "yaml") {
            out.push(path);
        }
    }
}

#[test]
fn every_example_imposter_file_parses() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut files = Vec::new();
    yaml_files(&root, &mut files);
    assert!(!files.is_empty(), "no example YAML found under {root:?}");

    for file in files {
        // Parses the YAML and builds the stubs; TLS cert files are only read
        // when an imposter binds, so TLS examples load without certs here.
        let config = ImposterConfig::from_yaml_file(&file)
            .unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        config
            .into_stubs()
            .unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    }
}
