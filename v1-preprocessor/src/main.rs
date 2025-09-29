use std::fs;
use std::path::Path;
use v1_preprocessor::error::PreprocessorError;
use v1_preprocessor::processing::process_v1_history;
use v1_preprocessor::v1_format::V1History;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(assets_dir)?;
    }

    let v1_history_path = assets_dir.join("v1-history.json");
    let v1_history_content = fs::read_to_string(v1_history_path).map_err(|e| {
        PreprocessorError::InvalidInput(format!("Could not read assets/v1-history.json: {e}"))
    })?;
    let v1_history: V1History = serde_json::from_str(&v1_history_content)?;

    let v2_config = process_v1_history(v1_history)?;

    let v2_config_path = assets_dir.join("v2-configuration.json");
    let v2_config_content = serde_json::to_string_pretty(&v2_config)?;
    fs::write(v2_config_path, v2_config_content)?;

    println!("Successfully generated assets/v2-configuration.json");

    Ok(())
}
