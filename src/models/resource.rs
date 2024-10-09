use crate::models::data_centre::{Compute, Storage, InterConnect};
use chrono::NaiveDate;

///
/// Claim resources from a logical infrastructure
///
pub struct Claim {
    pub compute: Vec<Compute>,
    pub storage: Vec<Storage>,
    pub network: InterConnect,
    pub from: NaiveDate,
    pub till: Option<NaiveDate>,
}

///
/// Possible claim response types
///
pub enum ResponseType {
    Success,
    ResourceFailure,
    PlannerFailure,
}

///
/// Response to a claim request
///
pub struct ClaimResponse {
    pub result: ResponseType,
    pub message: String,
}
