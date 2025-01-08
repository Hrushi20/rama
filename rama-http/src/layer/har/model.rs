#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Har {
    log: Log,
}
#[derive(Debug, Serialize, Deserialize)]
struct Log {
    version: String,
    creator: Creator,
    browser: Option<Browser>,
    pages: Option<Vec<Page>>,
    entries: Vec<Entry>,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Creator {
    name: String,
    version: String,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Browser {
    name: String,
    version: String,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Page {
    startedDateTime: String,
    id: String,
    title: String,
    pageTimings: PageTiming,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PageTiming {
    onContentLoad: Option<f64>,
    onLoad: Option<f64>,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    pageRef: Option<String>,
    startedDateTime: String,
    time: f64,
    request: Request,
    response: Response,
    cache: Cache,
    timings: Timing,
    serverIpAddress: Option<String>,
    connection: Option<String>,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    method: String,
    url: String,
    httpVersion: String,
    cookies: Vec<Cookie>,
    headers: Vec<Header>,
    queryString: Vec<QueryString>,
    postData: Option<PostData>,
    headersSize: i64,
    bodySize: i64,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    status: i64,
    statusText: String,
    httpVersion: String,
    cookies: Vec<Cookie>,
    headers: Vec<Header>,
    content: Content,
    redirectURL: String,
    headersSize: i64,
    bodySize: i64,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cookie {
    name: String,
    value: String,
    path: Option<String>,
    domain: Option<String>,
    expires: Option<String>,
    httpOnly: Option<bool>,
    secure: Option<bool>,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Header {
    name: String,
    value: String,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryString {
    name: String,
    value: String,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PostData {
    mimeType: String,
    params: Vec<Param>,
    text: String,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Param {
    name: String,
    value: Option<String>,
    fileName: Option<String>,
    contentType: Option<String>,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    size: Option<i64>,
    compression: Option<i64>,
    mimeType: Option<String>,
    text: Option<String>,
    encoding: Option<String>,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cache {
    beforeRequest: Option<CacheRequest>,
    afterRequest: Option<CacheRequest>,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheRequest {
    expires: Option<String>,
    lastAccess: String,
    eTag: String,
    hitCount: i64,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Timing {
    blocked: Option<f64>,
    dns: Option<i64>,
    connect: Option<i64>,
    send: Option<f64>,
    wait: Option<f64>,
    receive: Option<f64>,
    ssl: Option<i64>,
    comment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use rama_core::error::BoxError;

    #[test]
    fn serialize_deserialize_har_json() -> Result<(), BoxError> {
        let mut file = File::open(Path::new("/Users/pc/Downloads/userinyerface.com.har"))?;
        let mut har_str = String::new();
        file.read_to_string(&mut har_str)?;
        let har: Har = serde_json::from_str(&har_str)?;

        serde_json::to_string(&har)?;
        Ok(())
    }
}
