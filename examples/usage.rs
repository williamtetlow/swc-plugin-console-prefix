use anyhow::Context;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use swc::{self, config::Options};
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, SourceMap,
};

fn get_options(wasm_bin_path: &PathBuf) -> Options {
    let swcrc = format!(
        r#"{{
    "jsc": {{
        "experimental": {{
            "plugins": [
                ["{}", {{ "ignore": ["info"] }}]
            ]
        }}
    }}
}}"#,
        wasm_bin_path.to_string_lossy()
    );

    serde_json::from_str(&swcrc)
        .context("failed to parse .swrc")
        .unwrap()
}

fn get_abs_path_to_wasm_bin() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/wasm32-wasi/debug/swc_plugin_console_prefix.wasm")
}

fn main() {
    let wasm_bin_path = get_abs_path_to_wasm_bin();

    if !wasm_bin_path.exists() {
        panic!("wasm bindary does not exist - you need to run cargo build before running example")
    }

    let opts = get_options(&wasm_bin_path);
    let code = r#"console.log("hello world")"#;

    let cm = Arc::<SourceMap>::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let c = swc::Compiler::new(cm.clone());
    let fm = cm.new_source_file(FileName::Custom("usage.js".into()), code.into());

    let out = c
        .process_js_file(fm, &handler, &opts)
        .context("failed to process js file")
        .unwrap();

    println!("{}", out.code);
}
