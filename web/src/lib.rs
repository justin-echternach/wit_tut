
#[allow(warnings)]
mod bindings;

use std::io::Read;
use serde::de::DeserializeOwned;
use serde::*;
use url::Url;

use bindings::wasi::http::types::*;
use bindings::exports::tut::web::iweb::Guest;
use bindings::wasi::io::streams::StreamError;

#[derive(serde::Deserialize, Debug)]
struct DogResponse {
    message: String,
    status: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HttpBinResponse {
    args: serde_json::Value, // Using serde_json::Value for empty objects
    data: String,
    files: serde_json::Value,
    form: serde_json::Value,
    headers: HttpBinHeaders,
    json: serde_json::Value,
    origin: String,
    url: String,    
}

#[derive(Serialize, Deserialize, Debug)]
struct HttpBinHeaders {
    #[serde(rename = "Content-Length")]
    content_length: String,
    #[serde(rename = "Host")]
    host: String,
    #[serde(rename = "X-Amzn-Trace-Id")]
    x_amzn_trace_id: String,
}

// #[derive(Serialize, Deserialize, Debug)]
// struct TestData {
//     test_key: String,
// }

struct Component;

impl Guest for Component {
    fn make_get_request(url: String) -> String {                       
        let response = make_http_request(url.as_str(), &Method::Get, None, None).expect("Failed to make HTTP request");
        response        
    }

    fn make_post_request(url: String, body: String) -> String {
        let response = make_json_http_request(url.as_str(), &Method::Post, Some(body.as_str()), None).expect("Failed to make HTTP request");
        response
    }
}

bindings::export!(Component with_types_in bindings);

fn make_json_http_request(url: &str, method: &Method, json_body: Option<&str>, headers: Option<Fields>) -> Result<String, String> {
    let byte_body = json_body.map(|s| s.as_bytes());
    println!("json body length {}", byte_body.unwrap().len());
    make_http_request(url, method, byte_body, headers)
}

fn make_http_request(url: &str, method: &Method, body: Option<&[u8]>, headers: Option<Fields>) -> Result<String, String> {
    let headers = headers.unwrap_or_else(Fields::new);
    let req = bindings::wasi::http::outgoing_handler::OutgoingRequest::new(headers);
    let (scheme, domain, path_and_query) = parse_url(url).unwrap();
    let scheme = parse_scheme(&scheme);
    req.set_scheme(Some(&scheme)).unwrap();
    req.set_authority(Some(domain.as_str())).unwrap(); // Modify to extract authority from URL
    req.set_path_with_query(Some(path_and_query.as_str())).unwrap();
    //req.set_method(&bindings::wasi::http::types::Method::Other(method.to_string())).unwrap();
    req.set_method(&method).unwrap();
    println!("Request Method {:?}", req.method());
    if let Some(b) = body {
        let req_body = req.body().expect("failed getting outgoing body");
        write_outgoing_body(req_body, b).expect("failed writing outgoing body");
    }

    match bindings::wasi::http::outgoing_handler::handle(req, None) {
        Ok(resp) => {
            resp.subscribe().block();
            //println!("Response received ...{:?}", resp);
            let response: IncomingResponse = resp.get().unwrap().unwrap().unwrap();
            if response.status() == 200 {
                let response_body = response.consume().unwrap();
                let mut buf = vec![];
                let mut stream = response_body.stream().unwrap();
                InputStreamReader::from(&mut stream).read_to_end(&mut buf).unwrap();
                Ok(String::from_utf8(buf).unwrap())
            } else {
                Err(format!("HTTP request failed with status code {}", response.status()))
            }
        }
        Err(e) => Err(format!("Error during HTTP request: {}", e)),
    }
}

fn parse_url(url: &str) -> Result<(String, String, String), String> {
    let parsed_url = Url::parse(url).map_err(|e| e.to_string())?;
    let scheme = parsed_url.scheme().to_string();
    let domain = parsed_url.host_str().ok_or("Domain is missing in URL")?.to_string();
    let path_and_query = parsed_url.path().to_string() + &parsed_url.query().unwrap_or("").to_string();
    Ok((scheme, domain, path_and_query))
}

fn parse_scheme(scheme: &str) -> Scheme {
    match scheme {
        "http" => Scheme::Http,
        "https" => Scheme::Https,
        _ => Scheme::Other(scheme.to_string()),
    }
}

fn parse_method(method: &str) -> Method {
    match method.to_uppercase().as_str() {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "DELETE" => Method::Delete,
        "PATCH" => Method::Patch,
        "HEAD" => Method::Head,
        "OPTIONS" => Method::Options,
        "CONNECT" => Method::Connect,
        "TRACE" => Method::Trace,
        _ => Method::Other(method.to_string()),
    }
}

fn write_outgoing_body(body: crate::bindings::wasi::http::types::OutgoingBody, body_bytes: &[u8]) -> Result<(), StreamError> {
    let body_stream = body.write().expect("outgoing stream error");

    let pollable = body_stream.subscribe();        
    
    // while !body_bytes.is_empty() {
    //     // Wait for the stream to become writable
    //     pollable.block();
    //     let n = body_stream.check_write().unwrap();
    //     let len = (n as usize).min(body_bytes.len());
    //     let (chunk, rest) = body_bytes.split_at_mut(len);
    //     body_stream.write(chunk).unwrap();
    //     body_bytes = rest;
    // }
    
    let mut offset = 0;
    while offset < body_bytes.len() {
        // Wait for the stream to become writable
        pollable.block();
        let n = body_stream.check_write().unwrap();
        let len = (n as usize).min(body_bytes.len() - offset);
        let chunk = &body_bytes[offset..offset + len];
        body_stream.write(chunk).unwrap();
        offset += len;
    }

    body_stream.flush().expect("failed flushing response");
    pollable.block();
    let _ = body_stream.check_write();
    drop(pollable); // this is important or Rust will panic
    drop(body_stream);
    OutgoingBody::finish(body, None).unwrap();

    Ok(())
}

// Generic function to parse JSON response into a struct
fn parse_json_response<T: DeserializeOwned>(json_data: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str::<T>(json_data)
}

pub struct InputStreamReader<'a> {
    stream: &'a mut crate::bindings::wasi::io::streams::InputStream,
}

impl<'a> From<&'a mut crate::bindings::wasi::io::streams::InputStream> for InputStreamReader<'a> {
    fn from(stream: &'a mut crate::bindings::wasi::io::streams::InputStream) -> Self {
        Self { stream }
    }
}

impl Read for InputStreamReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use crate::bindings::wasi::io::streams::StreamError;
        use std::io;

        let n = buf
            .len()
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        match self.stream.blocking_read(n) {
            Ok(chunk) => {
                let n = chunk.len();
                if n > buf.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "more bytes read than requested",
                    ));
                }
                buf[..n].copy_from_slice(&chunk);
                Ok(n)
            }
            Err(StreamError::Closed) => Ok(0),
            Err(StreamError::LastOperationFailed(e)) => {
                Err(io::Error::new(io::ErrorKind::Other, e.to_debug_string()))
            }
        }
    }
}