use warp::Filter;

#[tokio::main]
async fn main() {
    let telemetry = warp::post()
        .and(warp::path("telemetry"))
        .and(warp::body::bytes())
        .map(|body: bytes::Bytes| {
            println!("Received telemetry ({} bytes):", body.len());
            if let Ok(s) = std::str::from_utf8(&body) {
                println!("{}", s);
            } else {
                println!("<binary payload>");
            }
            warp::reply::with_status("ok", warp::http::StatusCode::OK)
        });

    println!("Demo receiver listening on http://127.0.0.1:3030/telemetry");
    warp::serve(telemetry).run(([127, 0, 0, 1], 3030)).await;
}
