use cache::Error as CacheError;
use hyper::Error as HyperError;
use lavalink::Error as LavalinkError;
use lavalink_http_server_requester::Error as LavalinkHttpServerRequesterError;
use lavalink_queue_requester::Error as LavalinkQueueRequesterError;
use native_tls::Error as NativeTlsError;
use redis_async::error::Error as RedisError;
use serde_json::Error as SerdeJsonError;
use serenity::{
    prelude::HttpError as SerenityHttpError,
    Error as SerenityError,
};
use std::{
    cell::BorrowMutError,
    error::Error as StdError,
    fmt::{Display, Error as FmtError, Formatter, Result as FmtResult},
    io::Error as IoError,
    net::AddrParseError,
    num::ParseIntError,
    option::NoneError,
    result::Result as StdResult,
};
use tokio::timer::Error as TokioTimerError;
use toml::de::Error as TomlDeError;
use tungstenite::error::Error as TungsteniteError;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    AddrParse(AddrParseError),
    BorrowMut(BorrowMutError),
    Cache(CacheError),
    Fmt(FmtError),
    Hyper(HyperError),
    Io(IoError),
    Lavalink(LavalinkError),
    LavalinkHttpServerRequester(LavalinkHttpServerRequesterError),
    LavalinkQueueRequester(LavalinkQueueRequesterError),
    NativeTls(NativeTlsError),
    None(NoneError),
    ParseInt(ParseIntError),
    Redis(RedisError),
    SerdeJson(SerdeJsonError),
    Serenity(SerenityError),
    SerenityHttp(SerenityHttpError),
    Timer(TokioTimerError),
    TomlDe(TomlDeError),
    Tungstenite(TungsteniteError),
}

impl From<AddrParseError> for Error {
    fn from(e: AddrParseError) -> Self {
        Error::AddrParse(e)
    }
}

impl From<BorrowMutError> for Error {
    fn from(e: BorrowMutError) -> Self {
        Error::BorrowMut(e)
    }
}

impl From<CacheError> for Error {
    fn from(e: CacheError) -> Self {
        Error::Cache(e)
    }
}

impl From<FmtError> for Error {
    fn from(e: FmtError) -> Self {
        Error::Fmt(e)
    }
}

impl From<HyperError> for Error {
    fn from(e: HyperError) -> Self {
        Error::Hyper(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}

impl From<LavalinkError> for Error {
    fn from(e: LavalinkError) -> Self {
        Error::Lavalink(e)
    }
}

impl From<LavalinkHttpServerRequesterError> for Error {
    fn from(e: LavalinkHttpServerRequesterError) -> Self {
        Error::LavalinkHttpServerRequester(e)
    }
}

impl From<LavalinkQueueRequesterError> for Error {
    fn from(e: LavalinkQueueRequesterError) -> Self {
        Error::LavalinkQueueRequester(e)
    }
}

impl From<NativeTlsError> for Error {
    fn from(e: NativeTlsError) -> Self {
        Error::NativeTls(e)
    }
}

impl From<NoneError> for Error {
    fn from(e: NoneError) -> Self {
        Error::None(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

impl From<RedisError> for Error {
    fn from(e: RedisError) -> Self {
        Error::Redis(e)
    }
}

impl From<SerdeJsonError> for Error {
    fn from(e: SerdeJsonError) -> Self {
        Error::SerdeJson(e)
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

impl From<TokioTimerError> for Error {
    fn from(e: TokioTimerError) -> Self {
        Error::Timer(e)
    }
}

impl From<TomlDeError> for Error {
    fn from(e: TomlDeError) -> Self {
        Error::TomlDe(e)
    }
}

impl From<TungsteniteError> for Error {
    fn from(e: TungsteniteError) -> Self {
        Error::Tungstenite(e)
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
            Error::AddrParse(ref e) => e.description(),
            Error::BorrowMut(ref e) => e.description(),
            Error::Cache(ref e) => e.description(),
            Error::Fmt(ref e) => e.description(),
            Error::Hyper(ref e) => e.description(),
            Error::Io(ref e) => e.description(),
            Error::Lavalink(ref e) => e.description(),
            Error::LavalinkHttpServerRequester(ref e) => e.description(),
            Error::LavalinkQueueRequester(ref e) => e.description(),
            Error::NativeTls(ref e) => e.description(),
            Error::None(_) => "Option value not present",
            Error::ParseInt(ref e) => e.description(),
            Error::Redis(ref e) => e.description(),
            Error::SerdeJson(ref e) => e.description(),
            Error::Serenity(ref e) => e.description(),
            Error::SerenityHttp(ref e) => e.description(),
            Error::Timer(ref e) => e.description(),
            Error::TomlDe(ref e) => e.description(),
            Error::Tungstenite(ref e) => e.description(),
        }
    }
}
