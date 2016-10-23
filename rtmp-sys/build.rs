extern crate make_cmd;

fn main() {
  let output = make_cmd::make().current_dir("rtmpdump/librtmp").output();

  match output {
    Ok(output) => {
      if !output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("can't build librtmp : {}", String::from_utf8_lossy(&output.stderr))
      }
      print!("{}", String::from_utf8_lossy(&output.stdout))
    },
    Err(err) => {
      panic!("{}", err)
    }
  }
  return
}
