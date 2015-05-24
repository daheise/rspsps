use std::env;

fn main(){
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let spps_lib_dir = cargo_dir + "/spsps/build";
    println!("cargo:rustc-link-lib=static=spsps");
    println!("cargo:rustc-link-search=native={}", spps_lib_dir);
    println!("cargo:libdir=./spsps/build");
}
