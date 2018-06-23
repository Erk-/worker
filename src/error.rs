use hyper::Error as HyperError;
use lavalink::Error as LavalinkError;
use lavalink_futures::Error as LavalinkFuturesError;
use native_tls::Error as NativeTlsError;
use serde_json::error::Error as SerdeJsonError;
use serenity::prelude::HttpError as SerenityHttpError;
use serenity::Error as SerenityError;
use std::cell::BorrowMutError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::num::ParseIntError;
use std::option::NoneError;
use tungstenite::error::Error as TungsteniteError;

// TODO: can we do this with a macro

#[derive(Debug)]
pub enum Error {
    BorrowMut(BorrowMutError),
    Io(IoError),
    None(NoneError),
    NativeTls(NativeTlsError),
    Serenity(SerenityError),
    SerenityHttp(SerenityHttpError),
    Hyper(HyperError),
    ParseInt(ParseIntError),
    Tungstenite(TungsteniteError),
    SerdeJson(SerdeJsonError),
    Lavalink(LavalinkError),
    LavalinkFutures(LavalinkFuturesError),
}

impl From<BorrowMutError> for Error {
    fn from(e: BorrowMutError) -> Self {
        Error::BorrowMut(e)
    }
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

impl From<SerenityHttpError> for Error {
    fn from(e: SerenityHttpError) -> Self {
        Error::SerenityHttp(e)
    }
}

impl From<HyperError> for Error {
    fn from(e: HyperError) -> Self {
        Error::Hyper(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::ParseInt(e)
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

impl From<LavalinkFuturesError> for Error {
    fn from(e: LavalinkFuturesError) -> Self {
        Error::LavalinkFutures(e)
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
            Error::BorrowMut(ref e) => e.description(),
            Error::Io(ref e) => e.description(),
            Error::None(_) => "Option value not present",
            Error::NativeTls(ref e) => e.description(),
            Error::Serenity(ref e) => e.description(),
            Error::SerenityHttp(ref e) => e.description(),
            Error::Hyper(ref e) => e.description(),
            Error::ParseInt(ref e) => e.description(),
            Error::Tungstenite(ref e) => e.description(),
            Error::SerdeJson(ref e) => e.description(),
            Error::Lavalink(ref e) => e.description(),
            Error::LavalinkFutures(ref e) => e.description(),
        }
    }
}
