use std::path::Path;
use std::{env, fs};

use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use dojo_test_utils::compiler::build_test_config;
use scarb::ops;

cairo_lang_test_utils::test_file_test!(
    manifest_file,
    "src/manifest_test_crate",
    {
        manifest: "manifest",
    },
    test_manifest_file
);

pub fn test_manifest_file(
    _inputs: &OrderedHashMap<String, String>,
) -> OrderedHashMap<String, String> {
    let config = build_test_config("./src/manifest_test_crate/Scarb.toml").unwrap();
    let ws = ops::read_workspace(config.manifest_path(), &config).unwrap();

    let packages = ws.members().map(|p| p.id).collect();
    ops::compile(packages, &ws).unwrap_or_else(|op| panic!("Error compiling: {op:?}"));

    let target_dir = config.target_dir().path_existent().unwrap();

    let generated_manifest_path =
        Path::new(target_dir).join(config.profile().as_str()).join("manifest.json");

    let generated_file = fs::read_to_string(generated_manifest_path).unwrap();

    OrderedHashMap::from([("expected_manifest_file".into(), generated_file)])
}
