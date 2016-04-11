#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate url;
extern crate serde;
extern crate serde_json as json;

use hyper::{Client, header};

use serde::ser::{self, Serialize};

#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    Json(json::Error)
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error::Hyper(err)
    }
}
impl From<json::Error> for Error {
    fn from(err: json::Error) -> Error {
        Error::Json(err)
    }
}

#[derive(Copy, Clone)]
pub enum Channel {
    Stable,
    Nightly,
    Beta,
}

impl Serialize for Channel {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: ser::Serializer {
        match *self {
            Channel::Stable => "stable",
            Channel::Nightly => "nightly",
            Channel::Beta => "beta",
        }.serialize(serializer)
    }
}

#[derive(Copy, Clone)]
pub enum Optimize {
    O0,
    O1,
    O2,
    O3,
}

impl Serialize for Optimize {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: ser::Serializer {
        match *self {
            Optimize::O0 => "0",
            Optimize::O1 => "1",
            Optimize::O2 => "2",
            Optimize::O3 => "3",
        }.serialize(serializer)
    }
}


fn serialize_backtrace<S>(act: &bool, serializer: &mut S) -> Result<(), S::Error> where S: ser::Serializer {
    if *act {
        "1"
    } else {
        "0"
    }.serialize(serializer)
}

#[derive(Deserialize)]
pub struct Response {
    #[serde(default)]
    #[serde(rename="program")]
    pub output: Option<String>,
    #[serde(default)]
    pub rustc: String,
    #[serde(default)]
    #[serde(rename="error")]
    pub playpen_error: Option<String>,
}

#[derive(Serialize)]
pub struct Request<'a> {
    pub code: &'a str,
    pub version: Channel,
    pub optimize: Optimize,
    #[serde(serialize_with="serialize_backtrace")]
    pub backtrace: bool,
    pub color: bool,
    pub test: bool,
    separate_output: bool,
}

impl<'a> Default for Request<'a> {
    fn default() -> Request<'a> {
        Request {
            code: "",
            version: Channel::Nightly,
            optimize: Optimize::O2,
            backtrace: true,
            color: false,
            test: false,
            separate_output: true,
        }
    }
}

impl<'a> From<&'a str> for Request<'a> {
    fn from(s: &str) -> Request {
        Request {
            code: s,
            .. Default::default()
        }
    }
}

pub fn eval<'a, R: Into<Request<'a>>>(req: R) -> Result<Response, Error> {
    let client = Client::new();

    let req = req.into();

    Ok(try!(json::from_reader(try!(client.post("https://play.rust-lang.org/evaluate.json")
        .header(header::Connection::keep_alive())
        .header(header::ContentType::json())
        .body(&try!(json::to_string(&req))).send()))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_succ() {
        assert_eq!(&eval("fn main() { println!(\"yoyoyo\") }").unwrap().output.unwrap(), "yoyoyo\n");
    }
}
