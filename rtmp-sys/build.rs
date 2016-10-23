extern crate make_cmd;

use std::env;

fn main() {
  let mut command = make_cmd::make();
  command.current_dir("rtmpdump/librtmp");
  let status = command.status().unwrap();

  if !status.success() {
    panic!("Can't build librtmp!");
  }

  println!("cargo:rustc-link-search=native={}/rtmpdump/librtmp", env::var("CARGO_MANIFEST_DIR").unwrap());
  println!("cargo:rustc-link-lib=static=rtmp");
}
