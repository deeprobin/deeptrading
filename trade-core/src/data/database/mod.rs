pub mod in_memory;
pub mod influx;

use crate::models::candle::Candle;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::ops::Range;

#[async_trait]
pub trait StockDataCache {
    async fn write_candles(
        &mut self,
        symbol: String,
        candles: Vec<Candle>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_candles(
        &mut self,
        symbol: String,
        range: Range<DateTime<Utc>>,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error>>;
    async fn get_first_candle(
        &mut self,
        symbol: String,
    ) -> Result<Option<Candle>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_last_candle(
        &mut self,
        symbol: String,
    ) -> Result<Option<Candle>, Box<dyn std::error::Error + Send + Sync>>;
}
