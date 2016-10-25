extern crate gcc;

const SOURCES: &'static [&'static str] = &[
    "rtmpdump/librtmp/rtmp.c",
    "rtmpdump/librtmp/log.c",
    "rtmpdump/librtmp/amf.c",
    "rtmpdump/librtmp/hashswf.c",
    "rtmpdump/librtmp/parseurl.c",
];

fn main() {
  let mut config = gcc::Config::new();
  for source in SOURCES {
      config.file(source);
  }
  config.define("NO_CRYPTO", None);
  config.compile("librtmp.a");
}
