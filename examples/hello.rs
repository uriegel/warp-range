use std::{pin::Pin, task::{Context, Poll}};

use chrono::Utc;
use futures_util::Future;
use hyper::Error;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_stream::StreamExt;
use std::time::{Duration, Instant};
use async_stream::{AsyncStream, stream};
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

pub async fn get_video(filename: &str) -> Result<impl warp::Reply, warp::Rejection> {
    // TODO: content-length!!!
    match tokio::fs::File::open(filename).await {
        Ok(file) => {
            match file.metadata().await {
                Ok(metadata) => {
                    println!("Län: {}", metadata.len());



let stream = stream! {
    let mut when = Instant::now();
    for _ in 0..3 {
        //let delay = tokio::timer::Sleep { when };
        let sleep = tokio::time::sleep(Duration::from_millis(10));
        sleep.await;
        yield 2;
        when += Duration::from_millis(10);
    }
};



                    let stream = FramedRead::new(file, BytesCodec::new());
                    let body = hyper::Body::wrap_stream(stream);
                    let mut response = warp::reply::Response::new(body);
                    let headers = response.headers_mut();
                    let mut header_map = create_headers();
                    header_map.insert("Content-Type", HeaderValue::from_str("video/mp4").unwrap());
                    header_map.insert("Content-Length", HeaderValue::from(metadata.len()));
                    headers.extend(header_map);
                    Ok (response)
                },
                Err(err) => {
                    println!("Could not get pdf: {}", err);
                    Err(warp::reject())
                }
            }
        },
        Err(err) => {
            println!("Could not get pdf: {}", err);
            Err(warp::reject())
        }
    }
}

struct VideoStream {
    file: tokio::fs::File,
    first: bool,
    len: u64
}

// impl VideoStream {
//     async fn get(file: tokio::fs::File) -> Option<VideoStream> {
//         if let Ok(metadata) = file.metadata().await {
//             Some(VideoStream{file, first: true, len: metadata.len()})
//         } else {
//             None
//         }
//     }
// }

// impl tokio_stream::Stream for VideoStream {

//     type Item = Result<Vec<u8>, Error>;

//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
//         -> Poll<Option<()>> {
//         if self.rem == 0 {
//             // No more delays
//             return Poll::Ready(None);
//         }

//         match Pin::new(&mut self.delay).poll(cx) {
//             Poll::Ready(_) => {
//                 let when = self.delay.when + Duration::from_millis(10);
//                 self.delay = Delay { when };
//                 self.rem -= 1;
//                 Poll::Ready(Some(Ok("test".as_bytes().to_vec())))
//             }
//             Poll::Pending => Poll::Pending,
//         }
//     }

    // type Item = <Result<Vec<u8>, Error>;

    // async fn next(&mut self) -> Option<Result<Vec<u8>, Error>> {
    //     if self.first {
    //         self.first = false;
    //         let buffer: Vec<u8> = vec![0; self.len as usize];

    //         Some(Ok("test".as_bytes().to_vec()))
    //     } else {
    //         None
    //     }
        
    // }
//}


fn get_body(file: tokio::fs::File)->Body {
    let stream = stream! {
        let mut when = Instant::now();
        for _ in 0..3 {

            println!("Das bin ich");

            //let delay = tokio::timer::Sleep { when };
            let sleep = tokio::time::sleep(Duration::from_millis(1000));
            sleep.await;
            yield Ok(vec![54 as u8,55,56]) as Result<Vec<u8>, Error>;
            when += Duration::from_millis(10);
        }
    };
    //let stream = FramedRead::new(file, BytesCodec::new());
    hyper::Body::wrap_stream(stream)
}

pub async fn get_range(range: String, file: &str) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Range: {}", range);


    
    

    match tokio::fs::File::open(file).await {
        Ok(file) => {
//            match VideoStream::get(file).await {
//                Some(video_stream) => {
                    //let size = video_stream.len;
                    //let stream = futures_util::stream::iter(video_stream);
                    //let stream = video_stream;



                    //let stream = FramedRead::new(file, BytesCodec::new());
                    //let body = hyper::Body::wrap_stream(stream);
                    let body = get_body(file);
                    let mut response = warp::reply::Response::new(body);
                    
                    let headers = response.headers_mut();
                    let mut header_map = create_headers();
                    //header_map.insert("Content-Type", HeaderValue::from_str("video/mp4").unwrap());
                    header_map.insert("Content-Type", HeaderValue::from_str("text/plain").unwrap());
                    header_map.insert("Content-Length", HeaderValue::from(9));
                    headers.extend(header_map);
                    Ok (response)
                // },
                // None => {
                //     println!("Could not get video stream");
                //     Err(warp::reject())
                //}
            //}
        },
        Err(err) => {
            println!("Could not get pdf: {}", err);
            Err(warp::reject())
        }
    }
}

// TODO
// let chunks: Vec<Result<_, std::io::Error>> = vec![
//     Ok("hello"),
//     Ok(" "),
//     Ok("world"),
// ];

// let stream = futures_util::stream::iter(chunks);

// let body = Body::wrap_stream(stream);

#[tokio::main]
async fn main() {
    //let test_video = "/home/uwe/Videos/Drive.mkv";
    let test_video = "/home/uwe/Videos/essen.mp4";
    
    let port = 9860;
    println!("Running test server on http://localhost:{}", port);

    let route_get_view = 
        warp::path("getvideo")
        .and(warp::path::end())
        .and_then(move | | get_video(test_video));

    let route_get_range = 
        warp::path("getvideo")
        .and(warp::path::end())
        .and(warp::header::<String>("Range"))
        .and_then(move |r| get_range(r, test_video));

    let route_static = dir(".")
        .map(add_headers);
    
    let routes = route_get_range
        .or(route_get_view)
        .or(route_static);

    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;        
}