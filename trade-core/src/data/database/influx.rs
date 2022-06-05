use crate::models::candle::Candle;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use influxdb::{Client, ReadQuery};
use std::ops::Range;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::StockDataCache;

pub struct InfluxStockDataCache {
    client: Arc<Mutex<Client>>,
}

impl InfluxStockDataCache {
    pub fn new(client: Arc<Mutex<Client>>) -> Self {
        InfluxStockDataCache { client }
    }
}

#[async_trait]
impl StockDataCache for InfluxStockDataCache {
    async fn write_candles(
        &mut self,
        symbol: String,
        candles: Vec<Candle>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let influx_candles = candles
            .iter()
            .map(|candle| InfluxCandle::from_candle(symbol.clone(), candle))
            .map(|influx_candle| influx_candle.into_query("candle"));

        let client = self.client.lock().await;
        for query in influx_candles {
            client.query(query).await?;
        }
        Ok(())
    }
    async fn get_candles(
        &mut self,
        symbol: String,
        range: Range<DateTime<Utc>>,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
        let client = self.client.lock().await;
        let query = ReadQuery::new(format!(
            "SELECT * FROM {} WHERE time >= {} AND time < {} AND symbol = \"{}\"",
            "candle",
            range.start.to_rfc3339(),
            range.end.to_rfc3339(),
            symbol
        ));
        let mut query_result = client.json_query(query).await?;
        let queries: Vec<Candle> = query_result
            .deserialize_next::<Candle>()?
            .series
            .iter()
            .flat_map(|series| series.values.clone())
            .collect();
        Ok(queries)
    }
    async fn get_first_candle(
        &mut self,
        symbol: String,
    ) -> Result<Option<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.lock().await;
        let query = ReadQuery::new(format!(
            "SELECT * FROM {} WHERE symbol = \"{}\" LIMIT 1",
            "candle", symbol
        ));
        let mut query_result = client.json_query(query).await?;
        let queries: Option<Candle> = query_result
            .deserialize_next::<Candle>()?
            .series
            .iter()
            .flat_map(|series| series.values.clone())
            .next();
        Ok(queries)
    }
    async fn get_last_candle(
        &mut self,
        symbol: String,
    ) -> Result<Option<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.lock().await;
        let query = ReadQuery::new(format!(
            "SELECT * FROM {} WHERE symbol = \"{}\" ORDER BY time DESC LIMIT 1",
            "candle", symbol
        ));
        let mut query_result = client.json_query(query).await?;
        let queries: Option<Candle> = query_result
            .deserialize_next::<Candle>()?
            .series
            .iter()
            .flat_map(|series| series.values.clone())
            .next();
        Ok(queries)
    }
}

#[derive(Debug, Clone, InfluxDbWriteable)]
struct InfluxCandle {
    time: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    #[influxdb(tag)]
    symbol: String,
}

impl InfluxCandle {
    const fn from_candle(symbol: String, candle: &Candle) -> InfluxCandle {
        InfluxCandle {
            time: candle.time,
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
            volume: candle.volume,
            symbol,
        }
    }
}
