use std::ffi::NulError;
use std::fmt::{self, Display, Formatter};
use std::{error, io};

use actix_web::error::BlockingError;
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use gdal::errors::GdalError;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Nul(NulError),
    Gdal(GdalError),
    Blocking(Box<dyn error::Error + Send + Sync>),
    OutsideBounds,
    Infallible(std::convert::Infallible),
}

impl From<NulError> for Error {
    fn from(v: NulError) -> Self {
        Error::Nul(v)
    }
}

impl From<GdalError> for Error {
    fn from(v: GdalError) -> Self {
        Error::Gdal(v)
    }
}

impl<T: fmt::Debug + Send + Sync + 'static> From<BlockingError<T>> for Error
where
    Error: From<T>,
{
    fn from(v: BlockingError<T>) -> Self {
        Error::Blocking(Box::new(v))
    }
}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Error::Io(v)
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(v: std::convert::Infallible) -> Self {
        Error::Infallible(v)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::Nul(e) => e.fmt(f),
            Error::Gdal(e) => e.fmt(f),
            Error::Blocking(e) => e.fmt(f),
            Error::OutsideBounds => f.write_str("tile is outside image bounds"),
            Error::Infallible(e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Nul(e) => Some(e),
            Error::Gdal(e) => Some(e),
            Error::Blocking(e) => Some(e.as_ref()),
            Error::OutsideBounds => None,
            Error::Infallible(e) => Some(e),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::OutsideBounds => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
