use std::cell::OnceCell;

use async_std::fs::File;
use async_std::io::WriteExt;
use time::OffsetDateTime;
use xactor::{Actor, Context, Handler};

use crate::actors::Report;
use crate::ToRfc2822;

#[derive(Default)]
pub struct StockDataReporter {
    file: OnceCell<File>,
}

#[async_trait::async_trait]
impl Actor for StockDataReporter {
    async fn started(&mut self, ctx: &mut Context<Self>) -> xactor::Result<()> {
        println!("timestamp, period start,symbol,price,change %,min,max,30d avg");

        let file = File::create(format!("target/{}.csv", OffsetDateTime::now_utc().to_rfc2822())).await?;
        let _ = self.file.set(file);
        let file = self.file.get_mut().unwrap();
        file.write_all(b"period start,symbol,price,change %,min,max,30d avg\n").await?;

        ctx.subscribe::<Report>().await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<Report> for StockDataReporter {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Report) {
        let entry = format!(
            "{},{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
            msg.timestamp,
            msg.period_start,
            msg.symbol,
            msg.last_price,
            msg.pct_change * 100.0,
            msg.period_min,
            msg.period_max,
            msg.sma
        );

        println!("{}", entry);

        let file = self.file.get_mut().unwrap();
        let _ = file.write(entry.as_bytes()).await;
        let _ = file.write(b"\n").await;
        let _ = file.flush().await;
    }
}