//! Library containing the entrypoint for all logic.
//!
//! The executable is a wrapper around the library functions.
//!
#[macro_use]
extern crate lazy_static;

mod settings;
use settings::Settings;

pub mod models;
use models::dc::DataCentre;

use redis;
use redis::Commands;

lazy_static! {
    pub static ref CONFIG: Settings = match Settings::new() {
        Ok(cfg) => cfg,
        Err(error) => panic!("failure in loading settings {:?}", error),
    };
}

/// Simple function to add two ints
///
/// # Arguments
///
/// * `a` - left integer to add to right
/// * `b` - right integer to add to left
///
/// # Examples
///
/// add_two(1, 2)
///
/// ```
/// let r = libtheia::add_two(3, 4);
/// assert_eq!(r, 7);
/// ```
pub fn add_two(a: i32, b: i32) -> i32 {
    a + b
}

/// Insert a DataCentre into the Redis instance
///
/// * `con` - Redis connection
/// * `dc` - DataCentre to insert
pub fn insert_data_centre(con: &mut redis::Connection, dc: DataCentre) -> Result<(), String> {
    if con.exists(dc.name.as_str()).unwrap() {
        let m = format!("DataCentre {} already exists", dc.name);
        Err(m)
    } else {
        let r = con.set(dc.name.as_str(), serde_json::to_string(&dc).unwrap());
        Ok(r.unwrap())
    }
}
