use super::StockDataCache;
use crate::models::candle::Candle;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::ops::Range;

pub struct InMemoryStockDataCache {
    candles: HashMap<String, Vec<Candle>>,
}

impl InMemoryStockDataCache {
    pub fn new() -> Self {
        InMemoryStockDataCache {
            candles: HashMap::new(),
        }
    }
}

#[async_trait]
impl StockDataCache for InMemoryStockDataCache {
    async fn write_candles(
        &mut self,
        symbol: String,
        candles: Vec<Candle>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(candle_vec) = self.candles.get_mut(&symbol) {
            for candle in candles {
                if !candle_vec.contains(&candle) {
                    candle_vec.push(candle);
                }
            }
        } else {
            self.candles.insert(symbol, candles);
        }
        Ok(())
    }
    async fn get_candles(
        &mut self,
        symbol: String,
        range: Range<DateTime<Utc>>,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
        if let Some(candle_vec) = self.candles.get(&symbol) {
            let mut result_vec: Vec<_> = candle_vec
                .iter()
                .filter(|&item| range.contains(&item.time))
                .cloned()
                .collect();
            result_vec.sort_by(|a, b| a.time.cmp(&b.time));
            Ok(result_vec)
        } else {
            Ok(vec![])
        }
    }
    async fn get_first_candle(
        &mut self,
        symbol: String,
    ) -> Result<Option<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(candle_vec) = self.candles.get(&symbol) {
            let mut result_vec: Vec<_> = candle_vec.iter().collect();
            result_vec.sort_by(|a, b| a.time.cmp(&b.time));
            Ok(result_vec.first().cloned().cloned())
        } else {
            Ok(None)
        }
    }
    async fn get_last_candle(
        &mut self,
        symbol: String,
    ) -> Result<Option<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(candle_vec) = self.candles.get(&symbol) {
            let mut result_vec: Vec<_> = candle_vec.iter().collect();
            result_vec.sort_by(|a, b| a.time.cmp(&b.time));
            Ok(result_vec.last().cloned().cloned())
        } else {
            Ok(None)
        }
    }
}
