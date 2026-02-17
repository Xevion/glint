use glint::models::WorkItem;
use glint::models::agent::*;
use schemars::schema_for;
use std::fs;
use std::path::PathBuf;

fn schema_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../schemas")
}

fn write_schema<T: schemars::JsonSchema>(name: &str) {
    let schema = schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    fs::write(schema_dir().join(format!("{name}.json")), json).unwrap();
}

#[test]
fn export_all_schemas() {
    let dir = schema_dir();
    fs::create_dir_all(&dir).unwrap();

    write_schema::<PrepareUploadRequest>("PrepareUploadRequest");
    write_schema::<PrepareUploadResponse>("PrepareUploadResponse");
    write_schema::<CaptureRecord>("CaptureRecord");
    write_schema::<OrchestrationManifest>("OrchestrationManifest");
    write_schema::<OrchestrationInfo>("OrchestrationInfo");
    write_schema::<OrchestrationStatus>("OrchestrationStatus");
    write_schema::<CaptureSessionData>("CaptureSessionData");
    write_schema::<MinecraftInfo>("MinecraftInfo");
    write_schema::<CaptureEntry>("CaptureEntry");
    write_schema::<ShaderMetadata>("ShaderMetadata");
    write_schema::<Resolution>("Resolution");
    write_schema::<Position>("Position");
    write_schema::<Camera>("Camera");
    write_schema::<WorkItem>("WorkItem");
}
