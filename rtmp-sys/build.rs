extern crate cmake;

fn main() {
    let dst = cmake::Config::new("librtmp").build_target("rtmp").build();

    println!("cargo:rustc-link-search=native={}\\build\\Release", dst.display());
    println!("cargo:rustc-link-lib=static=rtmp");
}
