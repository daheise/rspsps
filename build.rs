fn main(){
    println!("cargo:rustc-link-lib=dylib=spsps");
    println!("cargo:rustc-link-search=native=./spsps/");
    println!("cargo:libdir=./spsps");
}
