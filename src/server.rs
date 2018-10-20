use std::result::Result;

use quicli::prelude::Error;

use clap_port_flag::Port;
use futures::prelude::*;
use hyper::{self, service::service_fn, Body, Response, Server, StatusCode};
use mime_guess;
use std::sync::Arc;
use tokio;

use Site;

pub fn serve(site: Site, port: &Port) -> Result<(), Error> {
    let site = Arc::new(site);

    let listener = port.bind()?;

    let handle = tokio::reactor::Handle::current();
    let listener = tokio::net::TcpListener::from_std(listener, &handle)?;
    let addr = listener.local_addr()?;

    let service = move || {
        let site = site.clone();
        service_fn(move |req| {
            let path = &req.uri().path()[1..];
            let page = site.get(path);
            if let Some(&page) = page {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(hyper::header::CONTENT_ENCODING, "gzip")
                    .header(hyper::header::CONTENT_DISPOSITION, "inline")
                    .header(
                        hyper::header::CONTENT_TYPE,
                        mime_guess::guess_mime_type_opt(path)
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "text/html".to_string()),
                    ).body(Body::from(page))
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not found"))
            }
        })
    };
    let server = Server::builder(listener.incoming())
        .serve(service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Server listening on {}", addr);
    tokio::run(server);

    Ok(())
}
