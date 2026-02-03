/// Triggers ts-rs to write TypeScript binding files.
///
/// Each type annotated with `#[derive(TS)] #[ts(export)]` emits a `.ts` file
/// into the directory configured by `TS_RS_EXPORT_DIR` (see `.cargo/config.toml`).
///
/// Run with: `cargo test --manifest-path backend/Cargo.toml export_bindings`
#[test]
fn export_bindings() {
    // ts-rs generates one test per `#[ts(export)]` type that writes the file.
    // This test exists as a named entry point so the check script can invoke
    // binding generation explicitly. The actual file writes happen in the
    // auto-generated tests within each type's module.
}
