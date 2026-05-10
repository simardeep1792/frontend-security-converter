use static_files::resource_dir;

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=static");
    println!("cargo:rerun-if-changed=build.rs");
    resource_dir("./static").build()
}
