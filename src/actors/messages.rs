use xactor::message;
use time::OffsetDateTime;

#[message]
#[derive(Clone)]
pub struct FetchAll;

#[message]
#[derive(Clone)]
pub struct DataRequest {
    pub(crate) symbol: String,
}

#[message]
pub struct Data {
    pub(crate) symbol: String,
    pub(crate) closes: Vec<f64>,
}

impl Data {
    pub fn new(symbol: String, closes: Vec<f64>) -> Self {
        Self {
            symbol,
            closes,
        }
    }
}

#[message]
#[derive(Clone)]
pub struct Report {
    pub(crate) timestamp: OffsetDateTime,
    pub(crate) symbol: String,
    pub(crate) last_price: f64,
    pub(crate) period_min: f64,
    pub(crate) period_max: f64,
    pub(crate) pct_change: f64,
    pub(crate) sma: f64,
}
