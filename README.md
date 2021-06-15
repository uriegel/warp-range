# warp-range
A Rust library for creating a warp filter for serving file content with range like mp3 audio or mp4 video.
This warp filter can be used in a HTTP server based on warp. 

The content is served like streaming. If you view a movie served by this filter, you can seek through it even if the file is not completely downloaded.
