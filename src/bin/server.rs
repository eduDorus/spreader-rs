pub mod orderbook {
    #![allow(non_snake_case)]
    tonic::include_proto!("orderbook");
}

use anyhow::Result;
use keyrock::setup_tracing_subscriber;
use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use orderbook::{Empty, Level, Summary};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tracing::error;

#[derive(Debug, Default)]
pub struct OrderbookAggregatorService {}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorService {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (tx, rx) = mpsc::channel(1);

        let asks = vec![
            Level {
                exchange: "binance".to_string(),
                price: 100.50,
                amount: 100.0,
            },
            Level {
                exchange: "bitfinex".to_string(),
                price: 101.0,
                amount: 10.0,
            },
        ];

        let bids = vec![
            Level {
                exchange: "binance".to_string(),
                price: 99.0,
                amount: 100.0,
            },
            Level {
                exchange: "bitfinex".to_string(),
                price: 99.4,
                amount: 100.0,
            },
        ];

        let summary = Summary {
            spread: asks[0].price - bids[0].price,
            bids: bids,
            asks: asks,
        };

        tokio::spawn(async move {
            while !tx.is_closed() {
                if let Err(e) = tx.send(Ok(summary.clone())).await {
                    error!("Error sending summary: {}", e);
                    break;
                }
                // Sleep for 10 milliseconds
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing_subscriber()?;

    let addr = "[::1]:50051".parse()?;
    let svc_orderbook = OrderbookAggregatorServer::new(OrderbookAggregatorService::default());

    Server::builder()
        .add_service(svc_orderbook)
        .serve(addr)
        .await?;
    Ok(())
}
