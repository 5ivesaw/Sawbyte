include!("../packaging/windows_resource.rs");

fn main() {
    #[cfg(windows)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        embed_icon(&root);
    }
}
