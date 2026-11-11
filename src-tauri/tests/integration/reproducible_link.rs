#[test]
fn the_msvc_link_is_reproducible() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/build.rs");
    let source = std::fs::read_to_string(path).expect("build.rs");
    assert!(
        source.contains("cargo:rustc-link-arg-bins=/Brepro"),
        "sans /Brepro l'editeur de liens MSVC ecrit un horodatage et un GUID de debogage a chaque build"
    );
}
