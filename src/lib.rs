//! # warp-range
//! 
//! A Rust library for creating a warp filter for serving file content with range like mp3 audio or mp4 video.
//! This warp filter can be used in a HTTP server based on warp. 
//! 
//! The content is served like streaming. If you view a movie served by this filter, you can seek through it even if the file is not completely downloaded.
//!
//! Here is an easy example to add range to an existing warp server:
//! ``` 
//! use hyper::{Body, Response};
//! use warp::{Filter, Reply, fs::{File, dir}};
//! use warp_range::{filter_range, get_range};
//! 
//! #[tokio::main]
//! async fn main() {
//!     let test_video = "/home/uwe/Videos/Drive.mkv";
//!     
//!     let port = 9860;
//!     println!("Running test server on http://localhost:{}", port);
//! 
//!     let route_get_range = 
//!         warp::path("getvideo")
//!         .and(warp::path::end())
//!         .and(filter_range())
//!         .and_then(move |range_header| get_range(range_header, test_video, "video/mp4"))
//! 
//!     let route_static = dir(".");
//!     
//!     let routes = route_get_range
//!         .or(route_static);
//! 
//!     warp::serve(routes)
//!         .run(([127, 0, 0, 1], port))
//!         .await;    
//! }
//! ``` 

use async_stream::stream;
use hyper::StatusCode;
use std::{
    cmp::min, io::SeekFrom, num::ParseIntError
};
use tokio::io::{
    AsyncReadExt, AsyncSeekExt
};
use warp::{
    Filter, Rejection, http::HeaderValue, hyper::HeaderMap
};

/// This function filters and extracts the "Range"-Header
pub fn filter_range() -> impl Filter<Extract = (Option<String>,), Error = Rejection> + Copy {
    warp::header::optional::<String>("Range")
}

/// This function retrives the range of bytes requested by the web client
pub async fn get_range(range_header: Option<String>, file: &str, content_type: &str) -> Result<impl warp::Reply, Rejection> {
    internal_get_range(range_header, file, content_type, None).await.map_err(|e| {
        println!("Error in get_range: {}", e.message);
        warp::reject()
    })
}

/// This function retrives the range of bytes requested by the web client. You can define a callback function for logging purpose or media access control
pub async fn get_range_with_cb(range_header: Option<String>, file: &str, content_type: &str, progress: fn(size: u64)) -> Result<impl warp::Reply, Rejection> {
    internal_get_range(range_header, file, content_type, Some(progress)).await.map_err(|e| {
        println!("Error in get_range: {}", e.message);
        warp::reject()
    })
}

fn get_range_params(range: &Option<String>, size: u64)->Result<(u64, u64), Error> {
    match range {
        Some(range) => {
            let range: Vec<String> = range
                .replace("bytes=", "")
                .split("-")
                .filter_map(|n| if n.len() > 0 {Some(n.to_string())} else {None})
                .collect();
            let start = if range.len() > 0 { 
                range[0].parse::<u64>()? 
            } else { 
                0 
            };
            let end = if range.len() > 1 {
                range[1].parse::<u64>()?
            } else {
                size-1 
            };
            Ok((start, end))
        },
        None => Ok((0, size-1))
    }
}

#[derive(Debug)]
struct Error {
    message: String
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error { message: err.to_string() }
    }
}
impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error { message: err.to_string() }
    }
}

async fn internal_get_range(range_header: Option<String>, file: &str, content_type: &str, cb: Option<fn(u64)>) -> Result<impl warp::Reply, Error> {
    let mut file = tokio::fs::File::open(file).await?;
    let metadata = file.metadata().await?;
    let size = metadata.len();
    let (start_range, end_range) = get_range_params(&range_header, size)?;
    let byte_count = end_range - start_range + 1;
    file.seek(SeekFrom::Start(start_range)).await?;

    let stream = stream! {
        let bufsize = 16384;
        let cycles = byte_count / bufsize as u64 + 1;
        let mut sent_bytes: u64 = 0;
        for _ in 0..cycles {
            let mut buffer: Vec<u8> = vec![0; min(byte_count - sent_bytes, bufsize) as usize];
            let bytes_read = file.read_exact(&mut buffer).await.unwrap();
            sent_bytes += bytes_read as u64;
            if let Some(cb) = cb { 
                cb(sent_bytes);
            } 
            yield Ok(buffer) as Result<Vec<u8>, hyper::Error>;
        }
    };
    let body = hyper::Body::wrap_stream(stream);
    let mut response = warp::reply::Response::new(body);
    
    let headers = response.headers_mut();
    let mut header_map = HeaderMap::new();
    header_map.insert("Content-Type", HeaderValue::from_str(content_type).unwrap());
    header_map.insert("Accept-Ranges", HeaderValue::from_str("bytes").unwrap());
    header_map.insert("Content-Range", HeaderValue::from_str(&format!("bytes {}-{}/{}", start_range, end_range, size)).unwrap());
    header_map.insert("Content-Length", HeaderValue::from(byte_count));
    headers.extend(header_map);

    if range_header.is_some() {
        *response.status_mut() = StatusCode::PARTIAL_CONTENT;
    }
    Ok (response)
}
