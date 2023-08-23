use anyhow::Result;
use spreader_rs::{binance, bitstamp, logging, ws::websocket};
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    logging::setup_tracing_subscriber()?;

    info!("🚀 Starting system");

    tokio::spawn(async {
        let (tx_binance, rx_biance) = websocket("wss://stream.binance.com:9443/ws".to_string())
            .await
            .expect("Failed to connect to binance");
        let sub_msg = binance::Subscribe {
            method: "SUBSCRIBE".to_string(),
            params: vec!["btcusdt@depth10@100ms".to_string()],
            id: 1,
        };
        tx_binance
            .send(Message::Text(serde_json::to_string(&sub_msg).unwrap()))
            .unwrap();
        while let Ok(msg) = rx_biance.recv_async().await {
            if let Ok(depth_update) =
                serde_json::from_str::<binance::PartialDepthUpdate>(&msg.to_string())
            {
                info!("BINANCE: {:?}", depth_update);
            } else {
                warn!(
                    "BINANCE Received message that was no depth update: {:?}",
                    msg
                );
            }
        }
    });
    tokio::spawn(async {
        let (tx_bitstamp, rx_bitstamp) = websocket("wss://ws.bitstamp.net".to_string())
            .await
            .expect("Failed to connect to bitstamp");
        let sub_msg = bitstamp::Subscribe {
            event: "bts:subscribe".to_string(),
            data: bitstamp::SubscribeData {
                channel: "order_book_btcusd".to_string(),
            },
        };
        tx_bitstamp
            .send(Message::Text(serde_json::to_string(&sub_msg).unwrap()))
            .unwrap();
        while let Ok(msg) = rx_bitstamp.recv_async().await {
            if let Ok(depth_update) =
                serde_json::from_str::<bitstamp::PartialDepthUpdate>(&msg.to_string())
            {
                info!("BITSTAMP: {:?}", depth_update);
            } else {
                warn!(
                    "BITSTAMP Received message that was no depth update: {:?}",
                    msg
                );
            }
        }
    });

    // Stop on ctrl C
    tokio::signal::ctrl_c().await?;
    Ok(())
}
