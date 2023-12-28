use async_std::task;
use clap::Parser;
use tide::{Body, Request, Response, StatusCode};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use xactor::{Addr, Supervisor};

use manning_lp_async_rust_project_1_m1::actors::{BufferSink, StockFetcher, TailRequest};
use manning_lp_async_rust_project_1_m1::actors::looper::Looper;
use manning_lp_async_rust_project_1_m1::actors::reporter::StockDataReporter;

#[derive(Parser, Debug)]
#[clap()]
struct Opts {
    #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
    symbols: String,
    #[clap(short, long)]
    from: String,
}

#[xactor::main]
async fn main() -> xactor::Result<()> {
    let opts = Opts::parse();
    let from: OffsetDateTime = OffsetDateTime::parse(&opts.from, &Rfc3339).expect("Couldn't parse 'from' date");
    let symbols = opts.symbols.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>();

    let sink = Supervisor::start(|| BufferSink::new(16)).await?;
    let _reporter = Supervisor::start(|| StockDataReporter::default()).await?;
    let _fetcher = Supervisor::start(move || {
        StockFetcher::new(symbols.clone(), from)
    }).await?;
    let next = Supervisor::start(|| Looper).await?;


    let mut app = tide::with_state(sink.clone());
    let _endpoint = task::spawn(async move  {
        app.at("/tail/:n").get(tail);
        let _ = app.listen("127.0.0.1:8080").await;
    });

    next.wait_for_stop().await;
    Ok(())
}

async fn tail(req: Request<Addr<BufferSink>>) -> tide::Result {
    let n = req.param("n")?.parse()?;

    let storage = req.state();
    let result = storage.call(TailRequest(n)).await?;

    let mut response = Response::new(StatusCode::Ok);
    response.set_body(Body::from_json(&result)?);
    Ok(response)
}