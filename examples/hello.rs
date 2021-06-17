use chrono::Utc;
use hyper::{Body, HeaderMap, Response, header::HeaderValue};
use warp::{Filter, Reply, fs::{File, dir}};
use warp_range::{filter_range, get_range, with_partial_content_status};

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

#[tokio::main]
async fn main() {
    let test_video = "/home/uwe/Videos/Drive.mkv";
    
    let port = 9860;
    println!("Running test server on http://localhost:{}", port);

    let route_get_video = 
        warp::path("getvideo")
        .and(warp::path::end())
        .and_then(move || get_range("".to_string(), test_video));

    let route_get_range = 
        warp::path("getvideo")
        .and(warp::path::end())
        
        .and(filter_range())
        .and_then(move |range_header| get_range(range_header, test_video))
        .map(with_partial_content_status);

    let route_static = dir(".")
        .map(add_headers);
    
    let routes = route_get_range
        .or(route_get_video)
        .or(route_static);

    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;        
}