use std::option::NoneError;
use std::io::Error as IoError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use native_tls::Error as NativeTlsError;
use serenity::Error as SerenityError;
use tungstenite::error::Error as TungsteniteError;
use serde_json::error::Error as SerdeJsonError;
use lavalink_futures::Error as LavalinkError;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    None(NoneError),
    NativeTls(NativeTlsError),
    Serenity(SerenityError),
    Tungstenite(TungsteniteError),
    SerdeJson(SerdeJsonError),
    Lavalink(LavalinkError),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}

impl From<NoneError> for Error {
    fn from(e: NoneError) -> Self {
        Error::None(e)
    }
}

impl From<NativeTlsError> for Error {
    fn from(e: NativeTlsError) -> Self {
        Error::NativeTls(e)
    }
}

impl From<SerenityError> for Error {
    fn from(e: SerenityError) -> Self {
        Error::Serenity(e)
    }
}

impl From<TungsteniteError> for Error {
    fn from(e: TungsteniteError) -> Self {
        Error::Tungstenite(e)
    }
}

impl From<SerdeJsonError> for Error {
    fn from(e: SerdeJsonError) -> Self {
        Error::SerdeJson(e)
    }
}

impl From<LavalinkError> for Error {
    fn from(e: LavalinkError) -> Self {
        Error::Lavalink(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::None(_) => "Option value not present",
            Error::NativeTls(ref e) => e.description(),
            Error::Serenity(ref e) => e.description(),
            Error::Tungstenite(ref e) => e.description(),
            Error::SerdeJson(ref e) => e.description(),
            Error::Lavalink(ref e) => e.description(),
        }
    }
}
