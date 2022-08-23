pub mod rest;

use http_body::combinators::BoxBody;
use hyper::{body::Bytes, Error};
use tower::util::BoxCloneService;

pub type RouteService<B> = BoxCloneService<
    http::Request<B>,
    http::Response<BoxBody<Bytes, Error>>,
    Box<dyn std::error::Error + Send + Sync>,
>;
