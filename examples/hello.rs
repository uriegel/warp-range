use std::cmp::min;

use chrono::Utc;
use hyper::{Error, StatusCode};
use tokio::io::AsyncReadExt;
use async_stream::stream;
use warp::{Filter, Rejection, Reply, fs::{
        File, dir
    }, http::HeaderValue, hyper::{
        Body, HeaderMap, Response
    }, reply::WithStatus};

pub fn filter_range() -> impl Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::header::<String>("Range")
}

pub fn with_partial_content_status<T: Reply>(reply: T) -> WithStatus<T> {
    warp::reply::with_status(reply, StatusCode::PARTIAL_CONTENT) 
}

fn add_headers(reply: File)->Response<Body> {
    let mut res = reply.into_response();
    let headers = res.headers_mut();
    let header_map = create_headers();
    headers.extend(header_map);
    res
}

fn create_headers() -> HeaderMap {
    let mut header_map = HeaderMap::new();
    let now = Utc::now();
    let now_str = now.format("%a, %d %h %Y %T GMT").to_string();
    header_map.insert("Expires", HeaderValue::from_str(now_str.as_str()).unwrap());
    header_map.insert("Server", HeaderValue::from_str("warp-range").unwrap());
    header_map
}

// fn get_range_params(range: &str)->Result<(u64, u64), Error> {
//     let range: Vec<String> = range
//         .replace("bytes=", "")
//         .split("-")
//         .filter_map(|n| if n.len() > 0 {Some(n.to_string())} else {None})
//         .collect();
//     let start = if let start = range[0] {start} else {0}

//     }
    
//     let end = if range.len() > 0 {range[1]}  (size-1).to_string()
// }

pub async fn get_range(range_header: String, file: &str) -> Result<impl warp::Reply, warp::Rejection> {
    match tokio::fs::File::open(file).await {
        Ok(mut file ) => {
            if let Ok(metadata) = file.metadata().await {
                let size = metadata.len();

        

                let stream = stream! {
                    let bufsize = 16384;
                    let cycles = size / bufsize as u64 + 1;
                    let mut sent_bytes: u64 = 0;
                    for _ in 0..cycles {
                        let mut buffer: Vec<u8> = vec![0; min(size - sent_bytes, bufsize) as usize];
                        match file.read_exact(&mut buffer).await {
                            Ok(res) => {
                                sent_bytes += res as u64;
                                println!("Video stream: {}, {}, {}", res, buffer.len(), sent_bytes)
                            },
                            Err(error) => println!("Could not get video stream: {:?}", error),
                        }
                        yield Ok(buffer) as Result<Vec<u8>, Error>;
                    }
                };
                let body = hyper::Body::wrap_stream(stream);
                let mut response = warp::reply::Response::new(body);
                
                let headers = response.headers_mut();
                let mut header_map = create_headers();
                header_map.insert("Content-Type", HeaderValue::from_str("video/mp4").unwrap());
                header_map.insert("Accept-Ranges", HeaderValue::from_str("bytes").unwrap());
                header_map.insert("Content-Range", HeaderValue::from_str(&format!("bytes {}-{}/{}", 0, size - 1, size)).unwrap());
                header_map.insert("Content-Length", HeaderValue::from(size));
                headers.extend(header_map);
                Ok (response)
            } else {
                println!("Could not get video stream");
                Err(warp::reject())
            }
        },
        Err(err) => {
            println!("Could not get pdf: {}", err);
            Err(warp::reject())
        }
    }
}

#[tokio::main]
async fn main() {
    let test_video = "/home/uwe/Videos/Drive.mkv";
    //let test_video = "/home/uwe/Videos/essen.mp4";
    
    let port = 9860;
    println!("Running test server on http://localhost:{}", port);

    let route_get_range = 
        warp::path("getvideo")
        .and(warp::path::end())
        
        .and(filter_range())
        .and_then(move |range_header| get_range(range_header, test_video))
        .map(with_partial_content_status);

    let route_static = dir(".")
        .map(add_headers);
    
    let routes = route_get_range
        .or(route_static);

    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;        
}