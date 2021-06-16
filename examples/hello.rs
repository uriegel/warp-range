use chrono::Utc;
use hyper::Error;
use tokio::io::AsyncReadExt;
use async_stream::stream;
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

pub async fn get_range(range: String, file: &str) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Range: {}", range);
    match tokio::fs::File::open(file).await {
        Ok(mut file ) => {
            if let Ok(metadata) = file.metadata().await {
                let size = metadata.len();
                let stream = stream! {
                    let mut buffer: Vec<u8> = vec![0; size as usize];
                    match file.read_exact(&mut buffer).await {
                        Ok(res) => println!("Video stream: {}, {}", res, buffer.len()),
                        Err(error) => println!("Could not get video stream: {:?}", error),
                    }
                    yield Ok(buffer) as Result<Vec<u8>, Error>;
                };
                let body = hyper::Body::wrap_stream(stream);
                let mut response = warp::reply::Response::new(body);
                
                let headers = response.headers_mut();
                let mut header_map = create_headers();
                header_map.insert("Content-Type", HeaderValue::from_str("video/mp4").unwrap());
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
    //let test_video = "/home/uwe/Videos/Drive.mkv";
    let test_video = "/home/uwe/Videos/essen.mp4";
    
    let port = 9860;
    println!("Running test server on http://localhost:{}", port);

    let route_get_range = 
        warp::path("getvideo")
        .and(warp::path::end())
        .and(warp::header::<String>("Range"))
        .and_then(move |r| get_range(r, test_video));

    let route_static = dir(".")
        .map(add_headers);
    
    let routes = route_get_range
        .or(route_static);

    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;        
}