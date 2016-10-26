extern crate cmake;

fn main() {
    let dst = cmake::Config::new("librtmp").build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=foo");
}
