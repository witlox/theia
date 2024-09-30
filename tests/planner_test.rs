use redis::{Commands, RedisError};
use libtheia::models::dc::{Compute, DataCentre, InterConnect};
use crate::common::{setup, teardown};

mod common;


#[tokio::test]
async fn dc_simple_claim_works() {
    /// setup
    let s = setup().await.unwrap();
    let redis = s.0;
    let redis_instance = s.1;
    /// test
    let dc_name = "DC";
    let dc = DataCentre::new(dc_name.to_string());
    let connection_string = format!("redis://{}:{}/", redis_instance.host, redis_instance.port);
    let mut rdc = redis::Client::open(connection_string).unwrap().get_connection().unwrap();
    let write: Result<String, RedisError> = rdc.set(dc.name.to_string(), serde_json::to_string(&dc).unwrap());
    /// teardown
    let _ = teardown(redis);
    match write {
        Err(e) => {
            panic!("Error writing to Redis: {:?}", e);
        }
        _ => {}
    }
}
