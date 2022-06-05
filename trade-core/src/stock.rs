use crate::models::candle::Candle;
use chrono::{DateTime, Utc};
use futures::stream::Stream;

pub trait Stock {
    type Error;
    type Stream: Stream<Item = Result<Candle, Self::Error>>;
    fn get_candles_before(&mut self, before: DateTime<Utc>) -> Self::Stream;
    fn get_candles_between(&mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self::Stream;

    fn stream_candles(&mut self) -> Self::Stream;
}

#[cfg(test)]
mod test_infrastructure {
    use chrono::{DateTime, Utc};
    use futures::stream::Stream;
    use std::task::Poll;

    use crate::models::candle::Candle;

    use super::Stock;
    pub struct TestStock {
        pub candles: Vec<Candle>,
    }

    impl Stock for TestStock {
        type Error = ();
        type Stream = TestStream;
        fn get_candles_before(&mut self, before: DateTime<Utc>) -> TestStream {
            self.candles
                .iter()
                .take_while(|c| c.time > before)
                .copied()
                .collect::<Vec<_>>()
                .into()
        }

        fn get_candles_between(&mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> TestStream {
            self.candles
                .iter()
                .take_while(|c| c.time >= start && c.time <= end)
                .copied()
                .collect::<Vec<_>>()
                .into()
        }

        fn stream_candles(&mut self) -> TestStream {
            unimplemented!()
        }
    }
    pub struct TestStream {
        iter: Box<dyn Iterator<Item = Candle>>,
    }

    #[cfg(test)]
    impl Stream for TestStream {
        type Item = Result<Candle, ()>;
        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            match self.iter.next() {
                Some(candle) => Poll::Ready(Some(Ok(candle))),
                None => Poll::Ready(None),
            }
        }
    }

    #[cfg(test)]
    impl From<Vec<Candle>> for TestStream {
        fn from(vec: Vec<Candle>) -> Self {
            TestStream {
                iter: Box::new(vec.into_iter()),
            }
        }
    }
}
