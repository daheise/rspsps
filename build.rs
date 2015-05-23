fn main(){
    println!("cargo:rustc-link-lib=static=spsps");
    println!("cargo:rustc-link-search=native=./spsps/");
    println!("cargo:libdir=./spsps");
}
