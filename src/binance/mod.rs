use serde::{Deserialize, Serialize};
use serde_this_or_that::as_f64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscribe {
    pub method: String,
    pub params: Vec<String>,
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct DiffDepthUpdate {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialDepthUpdate {
    pub last_update_id: i64,
    pub bids: Vec<DepthUpdateValues>,
    pub asks: Vec<DepthUpdateValues>,
}

#[derive(Debug, Deserialize)]
// #[serde(expecting = "expecting [<key>, <price>, <quantity>] array")]
pub struct DepthUpdateValues {
    #[serde(deserialize_with = "as_f64")]
    pub price: f64,
    #[serde(deserialize_with = "as_f64")]
    pub quantity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_subscribe() {
        let json = r#"{"method":"SUBSCRIBE","params":["btcusdt@depth10@100ms"],"id":1}"#;
        let subscribe = serde_json::from_str::<Subscribe>(json).unwrap();
        assert_eq!(subscribe.method, "SUBSCRIBE");
        assert_eq!(subscribe.params.len(), 1);
        assert_eq!(subscribe.params[0], "btcusdt@depth10@100ms");
        assert_eq!(subscribe.id, 1);
    }

    #[test]
    #[traced_test]
    fn test_depth_10() {
        let json = r#"{"lastUpdateId":38453964643,"bids":[["25930.04000000","5.67067000"],["25930.00000000","0.22058000"],["25929.91000000","0.00040000"],["25929.78000000","0.00170000"],["25929.73000000","0.24032000"],["25929.30000000","0.26292000"],["25929.14000000","0.59662000"],["25929.10000000","0.55323000"],["25928.81000000","0.20000000"],["25928.42000000","0.00049000"]],"asks":[["25930.05000000","16.77157000"],["25930.27000000","0.45711000"],["25930.28000000","0.28990000"],["25930.29000000","0.20000000"],["25930.32000000","1.90000000"],["25930.39000000","0.08805000"],["25930.65000000","0.07102000"],["25930.73000000","0.38316000"],["25930.75000000","0.20006000"],["25930.83000000","0.06000000"]]}"#;
        let depth_update = serde_json::from_str::<PartialDepthUpdate>(json).unwrap();
        assert_eq!(depth_update.last_update_id, 38453964643);
        assert_eq!(depth_update.bids.len(), 10);
        assert_eq!(depth_update.asks.len(), 10);
    }

    #[test]
    #[traced_test]
    fn test_diff_depth() {
        let json = r#"{"e":"depthUpdate","E":1692790478292,"s":"BTCUSDT","U":38454044678,"u":38454044684,"b":[["25951.30000000","0.00286000"],["25950.52000000","0.00000000"],["25949.10000000","0.00000000"],["25949.06000000","0.38556000"]],"a":[["25952.60000000","16.73558000"],["25955.05000000","0.38547000"],["25955.22000000","0.00000000"]]}"#;
        let depth_update = serde_json::from_str::<DiffDepthUpdate>(json).unwrap();
        assert_eq!(depth_update.event_type, "depthUpdate");
        assert_eq!(depth_update.timestamp, 1692790478292);
        assert_eq!(depth_update.instrument, "BTCUSDT");
        assert_eq!(depth_update.first_update_id, 38454044678);
        assert_eq!(depth_update.final_update_id, 38454044684);
        assert_eq!(depth_update.bids.len(), 4);
        assert_eq!(depth_update.asks.len(), 3);
    }
}
