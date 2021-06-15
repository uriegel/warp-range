use chrono::Utc;
use tokio_util::codec::{BytesCodec, FramedRead};
use warp::{
    Filter, Reply, fs::{
        File, dir
    }, http::HeaderValue, hyper::{
        Body, HeaderMap, Response
    }
};

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

pub async fn get_video() -> Result<impl warp::Reply, warp::Rejection> {


    // TODO: content-length!!!

    match tokio::fs::File::open("/home/uwe/Videos/Drive.mkv").await {
        Ok(file) => {
            let stream = FramedRead::new(file, BytesCodec::new());
            let body = hyper::Body::wrap_stream(stream);
            let mut response = warp::reply::Response::new(body);
            let headers = response.headers_mut();
            let mut header_map = create_headers();
            header_map.insert("Content-Type", HeaderValue::from_str("video/mp4").unwrap());
            headers.extend(header_map);
            Ok (response)
        },
        Err(err) => {
            println!("Could not get pdf: {}", err);
            Err(warp::reject())
        }
    }
}

pub async fn get_range(range: String) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Range: {}", range);
    match tokio::fs::File::open("/home/uwe/Videos/Drive.mkv").await {
        Ok(file) => {
            let stream = FramedRead::new(file, BytesCodec::new());
            let body = hyper::Body::wrap_stream(stream);
            let mut response = warp::reply::Response::new(body);
            let headers = response.headers_mut();
            let mut header_map = create_headers();
            header_map.insert("Content-Type", HeaderValue::from_str("video/mp4").unwrap());
            headers.extend(header_map);
            Ok (response)
        },
        Err(err) => {
            println!("Could not get pdf: {}", err);
            Err(warp::reject())
        }
    }
}

#[tokio::main]
async fn main() {
    let port = 9860;
    println!("Running test server on http://localhost:{}", port);

    let route_get_view = 
        warp::path("getvideo")
        .and(warp::path::end())
        .and_then(get_video);

    let route_get_range = 
        warp::path("getvideo")
        .and(warp::path::end())
        .and(warp::header::<String>("Range"))
        .and_then(get_range);

    let route_static = dir(".")
        .map(add_headers);
    
    let routes = route_get_range
        .or(route_get_view)
        .or(route_static);

    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;        
}