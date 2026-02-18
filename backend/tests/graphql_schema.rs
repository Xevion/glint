use std::path::Path;

#[test]
fn export_graphql_sdl() {
    let schema = glint::graphql::build_schema_for_sdl();
    let sdl = schema.sdl();
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../frontend/src/lib/graphql/schema.graphql");

    if std::env::var("GRAPHQL_SCHEMA_CHECK").is_ok() {
        // Check mode (used by CI): verify the exported schema matches the committed file
        let existing = std::fs::read_to_string(&path)
            .expect("schema.graphql not found — run `just bindings` to regenerate");
        assert_eq!(
            existing, sdl,
            "GraphQL schema is stale. Run `just bindings` to regenerate."
        );
    } else {
        // Write mode (default): overwrite the file
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create graphql directory");
        }
        std::fs::write(&path, &sdl).expect("Failed to write schema.graphql");
    }
}
