use genmagics::write_magics_to_file;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../genmagics/src/lib.rs");
    println!("cargo::rerun-if-changed=../genmagics/src/main.rs");

    write_magics_to_file(std::env::var("OUT_DIR").unwrap() + "/magics.rs")
}
