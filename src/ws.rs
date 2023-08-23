use std::time::Duration;

use anyhow::Result;
use flume::{Receiver, Sender};
use futures::{future, pin_mut, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};

const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);

pub async fn start_websocket(url: String, rx: Receiver<Message>, tx: Sender<Message>) {
    let mut reconnect_delay = INITIAL_RECONNECT_DELAY;
    loop {
        let rx = rx.clone();
        let tx = tx.clone();
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                info!("Sucessfully connected to {}!", &url);
                // Reset the reconnect delay after a successful connection
                reconnect_delay = INITIAL_RECONNECT_DELAY;

                // Split the stream into a sender and receiver
                let (ws_sink, ws_stream) = ws_stream.split();

                let forward_to_ws = rx.into_stream().map(Ok).forward(ws_sink);

                let forward_from_ws = {
                    ws_stream.for_each(|message| async {
                        // Check if the message is Ok
                        match message {
                            Ok(message) => {
                                if let Err(e) = tx.send_async(message).await {
                                    error!("Error sending message to channel: {}", e);
                                    return;
                                }
                            }
                            Err(e) => {
                                error!("Error receiving message from ws: {}", e);
                                return;
                            }
                        };
                    })
                };

                pin_mut!(forward_to_ws, forward_from_ws);
                future::select(forward_from_ws, forward_to_ws).await;

                // Connection was closed or lost, time to reconnect
                println!("Connection lost, reconnecting...");
            }
            Err(e) => {
                error!(
                    "Failed to connect: {}. Retrying in {:?}...",
                    e, reconnect_delay
                );
            }
        }

        // Apply the backoff strategy
        tokio::time::sleep(reconnect_delay).await;
        if reconnect_delay < MAX_RECONNECT_DELAY {
            reconnect_delay = std::cmp::min(reconnect_delay * 2, MAX_RECONNECT_DELAY);
        }
    }
}

pub async fn websocket(url: String) -> Result<(Sender<Message>, Receiver<Message>)> {
    let (tx_to_ws, rx_to_ws) = flume::unbounded();
    let (tx_from_ws, rx_from_ws) = flume::unbounded();
    tokio::spawn(start_websocket(url, rx_to_ws, tx_from_ws));
    Ok((tx_to_ws, rx_from_ws))
}
