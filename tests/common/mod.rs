use libtheia::models::data_centre::{Compute, DataCentre, Disk, DiskType, GPUBusType, InterConnect, Storage, GPU};

extern crate redis;
use redis::Commands;

use testcontainers::{core::{IntoContainerPort, WaitFor}, ContainerAsync, GenericImage};
use testcontainers::runners::AsyncRunner;
use libtheia::models::RedisInstance;

//
// pub async fn setup() -> Result<(ContainerAsync<GenericImage>, RedisInstance), String> {
//     /// setup test container
//     let redis_port = 6379;
//     let redis_host = "localhost".to_string();
//     let redis = GenericImage::new("redis", "7.2.4")
//         .with_exposed_port(redis_port.tcp())
//         .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
//         .start()
//         .await
//         .expect("Redis started");
//     /// setup defaults in Redis
//     let compute_interconnect = InterConnect::new("compute_interconnect".to_string(), 100, true);
//     let gpu = GPU::new("gpu1".to_string(), 4, 16, GPUBusType::PCIe);
//     let disk = Disk::new("disk1".to_string(), DiskType::SolidState, 100, None, None, None);
//     let storage = Storage::new("storage1".to_string(), Some(vec![disk]), 100);
//     let mut compute1 = Compute::new("compute1".to_string(), 4, 3, 16);
//     compute1.add_link(compute_interconnect.clone());
//     let mut compute2 = Compute::new("compute2".to_string(), 4, 3, 16);
//     compute2.add_gpu(gpu);
//     compute2.add_link(compute_interconnect.clone());
//     let mut compute3 = Compute::new("compute3".to_string(), 4, 3, 16);
//     compute3.add_link(compute_interconnect.clone());
//     let mut dc1 = DataCentre::new("dc1".to_string());
//     dc1.add_compute(compute1);
//     dc1.add_compute(compute2);
//     dc1.add_storage(storage);
//     dc1.add_interconnect(InterConnect::new("dc1_interconnect".to_string(), 10, true));
//     let mut dc2 = DataCentre::new("dc2".to_string());
//     dc2.add_compute(compute3);
//     dc2.add_interconnect(InterConnect::new("dc2_interconnect".to_string(), 1, true));
//     /// write defaults to Redis
//     let connection_string = format!("redis://{}:{}/", redis_host, redis_port);
//     let mut rdc = redis::Client::open(connection_string).unwrap().get_connection();
//     match rdc {
//         Ok(c) => {
//             let mut c = c;
//             let _: () = c.set(dc1.name.to_string(), serde_json::to_string(&dc1).unwrap()).unwrap();
//             let _: () = c.set(dc2.name.to_string(), serde_json::to_string(&dc2).unwrap()).unwrap();
//         }
//         Err(e) => {
//             return Err(format!("Error connecting to Redis: {:?}", e));
//         }
//     }
//     /// return the connection details
//     Ok((redis, RedisInstance::new(redis_host, redis_port)))
// }
//
// pub async fn teardown(redis: ContainerAsync<GenericImage>) {
//     redis.stop().await.expect("Redis stopped");
// }
