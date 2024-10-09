pub mod data_centre;
pub(crate) mod resource;
pub mod infrastructure;


pub struct RedisInstance {
    pub host: String,
    pub port: u16
}
impl RedisInstance {
    pub fn new(host: String, port: u16) -> Self {
        RedisInstance {
            host,
            port
        }
    }
}
