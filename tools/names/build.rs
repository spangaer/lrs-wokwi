use std::path::Path;

fn main() {
    let file_path = "src/other_names.rs";

    // Check if the file exists.
    if Path::new(file_path).exists() {
        println!("cargo:rustc-cfg=feature=\"other_names\"");
    }
}
