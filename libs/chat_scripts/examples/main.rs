use std::path::PathBuf;

use chat_scripts::ChatScripts;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_module_path =
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "\\examples\\main.rs"))
            .with_extension("js");
    let mut scripts = ChatScripts::new(|| {}).unwrap();
    scripts.load_module(main_module_path)?;

    scripts.run_js().await?;

    Ok(())
}
