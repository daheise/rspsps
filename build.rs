fn main(){
    println!("cargo:rustc-link-lib=static=spsps");
    println!("cargo:rustc-link-search=native=./spsps/build");
    println!("cargo:libdir=./spsps/build");
}
