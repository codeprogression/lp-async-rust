use std::collections::BTreeSet;
use time::OffsetDateTime;
use std::cell::OnceCell;
use async_std::fs::File;
use xactor::{Actor, Broker, Context, Handler, Service};
use std::io::{Error, ErrorKind};
use crate::actors::{Data, DataRequest, FetchAll, Report};
use crate::{AsyncStockSignal, MaxPrice, MinPrice, PriceDifference, ToRfc2822, WindowedSMA};

pub struct StockFetcher {
    symbols: BTreeSet<String>,
    from: OffsetDateTime,
    file: OnceCell<File>,
}


impl StockFetcher {
    pub fn new(symbols: Vec<String>, from: OffsetDateTime) -> Self {
        use std::iter::FromIterator;
        let set = BTreeSet::from_iter(symbols.into_iter());
        Self {
            symbols: set,
            from,
            file: OnceCell::new(),
        }
    }

    ///
    /// Retrieve data from a data source and extract the closing prices. Errors during download are mapped onto io::Errors as InvalidData.
    ///
    pub async fn fetch_closing_data(symbol: &str, from: OffsetDateTime) -> std::io::Result<Vec<f64>> {
        let provider = yahoo_finance_api::YahooConnector::new();
        let response = provider
            .get_quote_history(symbol, from, OffsetDateTime::now_utc()).await
            .map_err(|_| Error::from(ErrorKind::InvalidData))?;
        let mut quotes = response
            .quotes()
            .map_err(|_| Error::from(ErrorKind::InvalidData))?;
        if !quotes.is_empty() {
            quotes.sort_by_cached_key(|k| k.timestamp);
            Ok(quotes.iter().map(|q| q.adjclose).collect())
        } else {
            Ok(vec![])
        }
    }
}

#[async_trait::async_trait]
impl Actor for StockFetcher {
    async fn started(&mut self, ctx: &mut Context<Self>) -> xactor::Result<()> {
        ctx.subscribe::<FetchAll>().await?;
        ctx.subscribe::<DataRequest>().await?;
        let file = File::create(format!("target/{}.csv", OffsetDateTime::now_utc().to_rfc2822())).await?;
        self.file.set(file).expect("output file could not be set");
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<FetchAll> for StockFetcher {
    async fn handle(&mut self, ctx: &mut Context<Self>, _msg: FetchAll) {
        for symbol in &self.symbols {
            let request = DataRequest {
                symbol: symbol.to_string()
            };

            if let Err(err) =ctx.address().send(request) {
                eprintln!("{}", err);
            }
        }

    }
}


#[async_trait::async_trait]
impl Handler<DataRequest> for StockFetcher {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: DataRequest) {
        if let Ok(closes) = Self::fetch_closing_data(msg.symbol.as_str(), self.from).await {
            let _ = ctx.address().send(Data::new(msg.symbol, closes));
        }
    }
}

#[async_trait::async_trait]
impl Handler<Data> for StockFetcher {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Data) {
        let closes = &msg.closes;
        if !closes.is_empty() {
            let (_, pct_change) = PriceDifference.calculate(&closes).await.unwrap();
            let sma = WindowedSMA { window_size: 30 }.calculate(&closes).await.unwrap();

            let report = Report {
                timestamp: self.from,
                symbol: msg.symbol.clone(),
                last_price: *closes.last().unwrap_or(&0.0),
                period_min: MinPrice.calculate(&closes).await.unwrap(),
                period_max: MaxPrice.calculate(&closes).await.unwrap(),
                pct_change,
                sma: *sma.last().unwrap_or(&0.0),
            };
            let _ = Broker::from_registry().await.unwrap().publish(report);
        }
    }
}
