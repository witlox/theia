use serde::{Deserialize, Serialize};
use crate::crdt::{CmRDT, List};
use crate::models::data_centre::DataCentre;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(unused)]
pub struct LogicalInfrastructure {
    pub data_centres: List<DataCentre, u64>,
}

impl LogicalInfrastructure {
    pub fn new() -> LogicalInfrastructure {
        LogicalInfrastructure {
            data_centres: List::new(),
        }
    }

    pub fn add_data_centre(&mut self, dc: DataCentre) {
        let index = self.data_centres.len() + 1;
        self.data_centres.apply(self.data_centres.append(dc, index as u64));
    }

    /// Get a data centre by name
    ///
    /// ```rust
    /// use libtheia::models::infrastructure::LogicalInfrastructure;
    /// use libtheia::models::data_centre::DataCentre;
    ///
    /// let mut infra = LogicalInfrastructure::new();
    /// let mut dc = DataCentre::new("test".to_string());
    /// infra.add_data_centre(dc.clone());
    /// let fdc = infra.get_data_centre("test").unwrap();
    ///
    /// assert_eq!(dc, fdc.clone());
    pub fn get_data_centre(&self, name: &str) -> Option<&DataCentre> {
        for dc in self.data_centres.iter() {
            if dc.name == name {
                return Some(dc);
            }
        }
        None
    }

    pub fn remove_data_centre(&mut self, name: &str) {
        fn find_index(list: &List<DataCentre, u64>, name: &str) -> Option<usize> {
            for (i, s) in list.iter().enumerate() {
                if s.name == name {
                    Some(i);
                }
            }
            None
        }
        if let Some(i) = find_index(&self.data_centres, name) {
            self.data_centres.apply(self.data_centres.delete_index(i, i as u64).unwrap());
        }
    }
}
