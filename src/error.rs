use std::ffi::NulError;
use std::fmt::{self, Display, Formatter};
use std::{error, io};

use axum::body::HttpBody;
use axum::response::IntoResponse;
use gdal::errors::GdalError;
use hyper::{Body, Response, StatusCode};
use tokio::task::JoinError;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Nul(NulError),
    Gdal(GdalError),
    Hyper(hyper::Error),
    Join(JoinError),
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

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Error::Io(v)
    }
}

impl From<hyper::Error> for Error {
    fn from(v: hyper::Error) -> Self {
        Error::Hyper(v)
    }
}

impl From<JoinError> for Error {
    fn from(v: JoinError) -> Self {
        Error::Join(v)
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
            Error::Hyper(e) => e.fmt(f),
            Error::Join(e) => e.fmt(f),
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
            Error::Hyper(e) => Some(e),
            Error::Join(e) => Some(e),
            Error::OutsideBounds => None,
            Error::Infallible(e) => Some(e),
        }
    }
}

impl IntoResponse for Error {
    type Body = hyper::Body;
    type BodyError = <Self::Body as HttpBody>::Error;

    fn into_response(self) -> Response<Body> {
        match self {
            Error::OutsideBounds => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap(),
            _ => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
        }
    }
}
