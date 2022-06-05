use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, InfluxDbWriteable)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub time: DateTime<Utc>,
}
