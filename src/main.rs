use clap::Parser;
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
    let to = OffsetDateTime::now_utc();

    let max = MaxPrice{};
    let min = MinPrice{};
    let diff = PriceDifference{};
    let window = WindowedSMA{ window_size: 30};

    // a simple way to output a CSV header
    println!("period start,symbol,price,change %,min,max,30d avg");
    for symbol in opts.symbols.split(',') {
        let closes = fetch_closing_data(&symbol, from, to).await?;
        if !closes.is_empty() {
            // min/max of the period. unwrap() because those are Option types
            let period_max: f64 = max.calculate(&closes).await.unwrap();
            let period_min: f64 = min.calculate(&closes).await.unwrap();
            let last_price = *closes.last().unwrap_or(&0.0);
            let (_, pct_change) = diff.calculate(&closes).await.unwrap_or((0.0, 0.0));
            let sma = window.calculate(&closes).await.unwrap_or(vec![]);

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
    Ok(())
}
