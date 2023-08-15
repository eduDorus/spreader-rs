use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use anyhow::Result;
use futures::{
    future::ready,
    stream::{self, StreamExt},
    SinkExt, Stream,
};
use keyrock::setup_tracing_subscriber;
use serde::Deserialize;
use serde_this_or_that::as_f64;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tracing::{error, info};

type WSStream = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing_subscriber()?;

    // let url = "wss://stream.binance.com:9443/ws/btcusdt@depth@100ms";
    // tokio::spawn(start_websocket(url));

    let url = "wss://ws.bitstamp.net";
    tokio::spawn(start_websocket(url));

    // Stop on ctrl C
    info!("🚀 Starting system");
    tokio::signal::ctrl_c().await?;
    Ok(())
}

async fn start_websocket(url: &str) -> Result<()> {
    let mut reconnect_delay = INITIAL_RECONNECT_DELAY;

    loop {
        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                info!("Connected!");
                // Reset the reconnect delay after a successful connection
                reconnect_delay = INITIAL_RECONNECT_DELAY;
                // Handle the connection (this will return if the connection is lost)
                handle_connection(ws_stream).await?;
            }
            Err(e) => {
                error!(
                    "Failed to connect: {}. Retrying in {:?}...",
                    e, reconnect_delay
                );
            }
        }

        sleep(reconnect_delay).await;

        // Apply the backoff strategy
        if reconnect_delay < MAX_RECONNECT_DELAY {
            reconnect_delay = std::cmp::min(reconnect_delay * 2, MAX_RECONNECT_DELAY);
        }
    }
}

async fn handle_connection(mut ws_stream: WSStream) -> Result<()> {
    let subscribe = r#"{
        "event": "bts:subscribe",
        "data": {
            "channel": "diff_order_book_btcusd"
        }
    }"#;
    ws_stream.send(Message::Text(subscribe.into())).await?;
    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(Message::Text(text)) => parse_message(text).await,
            Ok(Message::Binary(bin)) => info!("Received some bytes: {:?}", bin),
            Ok(Message::Ping(_)) => {
                info!("Received a ping");
                ws_stream.send(Message::Pong(vec![])).await?;
            }
            Err(e) => error!("Error reading message: {}", e),
            _ => {}
        }
    }
    Ok(())
}

async fn parse_message(message: String) {
    let message = serde_json::from_str::<BitstampDepthUpdate>(&message);
    match message {
        Ok(message) => {
            info!("Received a message: {:?}", message);
        }
        Err(e) => error!("Error parsing message: {}", e),
    }
}

#[derive(Deserialize, Debug)]
pub struct BinanceDepthUpdate {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub timestamp: i64,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "U")]
    pub first_update_id: i64,
    #[serde(rename = "u")]
    pub final_update_id: i64,
    #[serde(rename = "b")]
    pub bids: Vec<DepthUpdateValues>,
    #[serde(rename = "a")]
    pub asks: Vec<DepthUpdateValues>,
}

// Implement display for BinanceDepthUpdate

#[derive(Debug, Clone, PartialEq, Deserialize)]
// #[serde(expecting = "expecting [<key>, <price>, <quantity>] array")]
pub struct DepthUpdateValues {
    #[serde(deserialize_with = "as_f64")]
    pub price: f64,
    #[serde(deserialize_with = "as_f64")]
    pub quantity: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct BitstampDepthUpdate {
    pub data: BitstampDepthUpdateData,
    pub channel: String,
    pub event: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct BitstampDepthUpdateData {
    pub timestamp: String,
    pub microtimestamp: String,
    pub bids: Vec<DepthUpdateValues>,
    pub asks: Vec<DepthUpdateValues>,
}
async fn handle_streams() {
    let my_stream = MyStream { counter: 100 };

    let sum = my_stream
        .filter(|x| ready(x % 2 == 0))
        .map(|x| x * 2)
        .fold(0, |acc, x| ready(acc + x))
        .await;

    println!("sum = {}", sum);

    let stream1 = stream::iter(vec![1, 3, 5, 7, 9]);
    let stream2 = stream::iter(vec![0, 2, 4, 6, 8]);
    let stream3 = stream::iter(vec![0, 2, 4, 6, 8]);

    // Merge the three streams into one stream
    let mut merged_stream = stream::select_all(vec![stream1, stream2, stream3]);

    while let Some(value) = merged_stream.next().await {
        println!("{}", value);
    }
}

/// A stream that counts down till 0
struct MyStream {
    counter: u64,
}

impl Stream for MyStream {
    type Item = u64;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<u64>> {
        if self.counter == 0 {
            Poll::Ready(None)
        } else {
            self.counter -= 1;
            Poll::Ready(Some(self.counter))
        }
    }
}
