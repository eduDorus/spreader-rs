pub mod orderbook {
    #![allow(non_snake_case)]
    tonic::include_proto!("orderbook");
}

use anyhow::Result;
use keyrock::setup_tracing_subscriber;
use orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;
use tonic::Request;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing_subscriber()?;

    let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await?;

    let mut stream = client
        .book_summary(Request::new(orderbook::Empty {}))
        .await?
        .into_inner();

    let mut counter = 0;
    while let Some(summary) = stream.message().await? {
        info!("Book Update: Spread: {}", summary.spread);
        counter += 1;

        if counter == 100 {
            break;
        }
    }

    Ok(())
}
