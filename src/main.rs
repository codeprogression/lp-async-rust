use clap::Parser;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use xactor::Supervisor;

use manning_lp_async_rust_project_1_m1::actors::*;
use manning_lp_async_rust_project_1_m1::actors::looper::Looper;
use manning_lp_async_rust_project_1_m1::actors::reporter::StockDataReporter;

#[derive(Parser, Debug)]
#[clap(
version = "1.0",
author = "Claus Matzinger",
about = "A Manning LiveProject: async Rust"
)]
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

    let _reporter = Supervisor::start(|| StockDataReporter::default()).await?;
    let _fetcher = Supervisor::start(move || {
        StockFetcher::new(symbols.clone(), from)
    }).await?;

    let next = Supervisor::start(|| Looper).await?;
    next.wait_for_stop().await;

    Ok(())
}
