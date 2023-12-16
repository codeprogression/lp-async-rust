use chrono::prelude::*;
use clap::Parser;

mod utils;
mod lib;

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

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let from: DateTime<Utc> = opts.from.parse().expect("Couldn't parse 'from' date");
    let to = Utc::now();

    // a simple way to output a CSV header
    println!("period start,symbol,price,change %,min,max,30d avg");
    for symbol in opts.symbols.split(',') {
        let closes = utils::fetch_closing_data(&symbol, &from, &to)?;
        if !closes.is_empty() {
                // min/max of the period. unwrap() because those are Option types
                let period_max: f64 = utils::max(&closes).unwrap();
                let period_min: f64 = utils::min(&closes).unwrap();
                let last_price = *closes.last().unwrap_or(&0.0);
                let (_, pct_change) = utils::price_diff(&closes).unwrap_or((0.0, 0.0));
                let sma = utils::n_window_sma(30, &closes).unwrap_or_default();

            // a simple way to output CSV data
            println!(
                "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                from.to_rfc3339(),
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
