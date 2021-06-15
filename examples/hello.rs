use chrono::Utc;
use warp::{
    Filter, Reply, fs::{
        File, dir
    }, http::HeaderValue, hyper::{
        Body, HeaderMap, Response
    }
};

#[tokio::main]
async fn main() {
    let port = 9860;
    println!("Running test server on http://localhost:{}", port);

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

    let route_static = dir(".")
        .map(add_headers);

    
    warp::serve(route_static)
        .run(([127, 0, 0, 1], port))
        .await;        
}