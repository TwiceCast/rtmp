extern crate rtmp_sys;

fn main() {
  unsafe {
      println!("{:?}", rtmp_sys::rtmp::RTMP_LibVersion())
  }
}
