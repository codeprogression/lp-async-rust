use std::time::Duration;

use async_std::prelude::*;
use async_std::task;
use clap::Parser;
use futures::future::join_all;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use manning_lp_async_rust_project_1_m1::*;

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

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let from: OffsetDateTime = OffsetDateTime::parse(&opts.from, &Rfc3339).expect("Couldn't parse 'from' date");
    let symbols = opts.symbols.split(',').collect::<Vec<&str>>();
    let mut interval = async_std::stream::interval(Duration::from_secs(5));

    loop {
        process(symbols.clone(), from).await?;
        let _ = interval.next().await;
    }
}

async fn process(symbols: Vec<&str>, from: OffsetDateTime) -> std::io::Result<()> {
    println!("period start,symbol,price,change %,min,max,30d avg");
    let tasks: Vec<_> = symbols.iter().map(|symbol| {
        task::spawn(get_stock_info(symbol.to_string(), from, OffsetDateTime::now_utc()))
    }).collect();
    let _ = join_all(tasks).await;
    println!();
    Ok(())
}

async fn get_stock_info(symbol: String, from: OffsetDateTime, to: OffsetDateTime) {
    let closes = fetch_closing_data(&symbol, from, to).await.unwrap();
    if !closes.is_empty() {
        // min/max of the period. unwrap() because those are Option types
        let period_max: f64 = MaxPrice.calculate(&closes).await.unwrap();
        let period_min: f64 = MinPrice.calculate(&closes).await.unwrap();
        let last_price = *closes.last().unwrap_or(&0.0);
        let (_, pct_change) = PriceDifference.calculate(&closes).await.unwrap_or((0.0, 0.0));
        let sma = WindowedSMA { window_size: 30 }.calculate(&closes).await.unwrap_or(vec![]);

        // a simple way to output CSV data
        println!(
            "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
            from,
            symbol,
            last_price,
            pct_change * 100.0,
            period_min,
            period_max,
            sma.last().unwrap_or(&0.0)
        );
    }
}
