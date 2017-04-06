#[derive(Debug)]
pub enum Error {
  Io(::std::io::Error),
  Other(String),
}

impl From<::std::io::Error> for Error {
  fn from(err: ::std::io::Error) -> Error {
    Error::Io(err)
  }
}

impl From<String> for Error {
  fn from(err: String) -> Error {
    Error::Other(err)
  }
}

impl<'a> From<&'a str> for Error {
  fn from(err: &'a str) -> Error {
    Error::Other(err.to_owned())
    // Alias: Error::from(err.to_owned())
    // Alias: err.to_owned().into()
  }
}

pub type Result<T> = ::std::result::Result<T, Error>;