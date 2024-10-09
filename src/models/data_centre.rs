use serde::{Serialize, Deserialize};
use crate::crdt::{List, CmRDT};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(unused)]
pub enum GPUBusType {
    PCIe,
    NVLink,
    ROCm,
    InfinityFabric,
}

///
/// GPU data structure
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(unused)]
pub struct GPU {
    pub name: String,
    pub clock_ghz: i32,
    pub ram_gb: i32,
    pub bus_type: GPUBusType,
}


impl GPU {
    /// GPU instance
    ///
    /// * `n` - a gpu name.
    /// * `c` - clock rate in GHz.
    /// * `r` - ram size in GB.
    /// * `b` - bus type
    pub fn new(n: String, c: i32, r: i32, b: GPUBusType) -> GPU {

        GPU {
            name: n,
            clock_ghz: c,
            ram_gb: r,
            bus_type: b,
        }
    }
}

///
/// Network connections
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(unused)]
pub struct InterConnect {
    pub name: String,
    pub speed_gb: i32,
    pub low_latency: bool,
}

impl InterConnect {
    pub fn clone(&self) -> InterConnect {
        InterConnect {
            name: self.name.clone(),
            speed_gb: self.speed_gb,
            low_latency: self.low_latency,
        }
    }
}

impl InterConnect {
    /// Interconnect or NIC instance
    ///
    /// * `n` - an interconnect name.
    /// * `s` - speed in Gigabit per second (Gbps).
    /// * `l` - low latency flag.
    pub fn new(n: String, s: i32, l: bool) -> InterConnect {
        InterConnect {
            name: n,
            speed_gb: s,
            low_latency: l,
        }
    }
}

///
/// Disk Types
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(unused)]
pub enum DiskType {
    Spinning,
    SolidState,
    NVMe
}

impl DiskType {
    fn from_str(s: &str) -> DiskType {
        match s {
            "spinning" => DiskType::Spinning,
            "solid_state" => DiskType::SolidState,
            "nvme" => DiskType::NVMe,
            _ => DiskType::Spinning,
        }
    }
}

///
/// Disks
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(unused)]
pub struct Disk {
    pub name: String,
    pub disk_type: DiskType,
    pub capacity_tb: i32,
    pub write_speed_mb: Option<i32>,
    pub read_speed_mb: Option<i32>,
    pub iops: Option<i32>,
}

impl Disk {
    /// Disk instance
    ///
    /// * `n` - a disk name.
    /// * `t` - type of disk.
    /// * `c` - capacity in TerraByte (TB).
    /// * `w` - optional write speed in MegaByte per second (MBps).
    /// * `r` - optional read speed in MegaByte per second (MBps).
    /// * `i` - optional IOPS.
    pub fn new(n: String, t: DiskType, c: i32, w: Option<i32>, r: Option<i32>, i: Option<i32>) -> Disk {
        Disk {
            name: n,
            disk_type: t,
            capacity_tb: c,
            write_speed_mb: w,
            read_speed_mb: r,
            iops: i,
        }
    }
}

///
/// Main data structure for storing and retrieving compute resources
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(unused)]
pub struct Compute {
    pub name: String,
    pub cores: i32,
    pub core_ghz: i32,
    pub ram_gb: i32,
    pub disks: Vec<Disk>,
    pub links: Vec<InterConnect>,
    pub gpus: Vec<GPU>,
}

impl Compute {
    /// Compute instance
    ///
    /// * `n` - server name.
    /// * `c` - number of cores.
    /// * `g` - core Gigaherz (GHz).
    /// * `r` - amount of RAM in GigaBytes (GB).
    pub fn new(n: String, c: i32, g: i32, r: i32) -> Compute {
        Compute {
            name: n,
            cores: c,
            core_ghz: g,
            ram_gb: r,
            disks: Vec::new(),
            links: Vec::new(),
            gpus: Vec::new(),
        }
    }

    pub(crate) fn add_disk(&mut self, d: Disk) {
        self.disks.push(d);
    }

    pub fn add_link(&mut self, l: InterConnect) {
        self.links.push(l);
    }

    pub fn add_gpu(&mut self, g: GPU) {
        self.gpus.push(g);
    }
}

///
/// Storage, where capacity is either given, or the sum of all disks
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(unused)]
pub struct Storage {
    pub name: String,
    pub disks: Option<Vec<Disk>>,
    pub capacity_gb: i32,
    pub links: Vec<InterConnect>,
}

impl Storage {
    /// Storage instance
    ///
    /// * `n` - storage name.
    /// * `d` - optional list of disks.
    pub fn new(n: String, d: Option<Vec<Disk>>, c: i32) -> Storage {
        Storage {
            name: n,
            disks: d,
            capacity_gb: c,
            links: Vec::new(),
        }
    }

    pub(crate) fn add_link(&mut self, l: InterConnect) {
        self.links.push(l);
    }
}

///
/// A full DC
///
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(unused)]
pub struct DataCentre {
    pub name: String,
    pub compute: List<Compute, u64>,
    pub storage: List<Storage, u64>,
    pub interconnects: List<InterConnect, u64>,
}

impl PartialEq for DataCentre {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl DataCentre {
    /// DataCentre instance
    ///
    /// * `n` - storage name.
    pub fn new(n: String) -> DataCentre {
        DataCentre{
            name: n,
            compute: List::new(),
            storage: List::new(),
            interconnects: List::new(),
        }
    }

    pub fn add_compute(&mut self, c: Compute) {
        let r = self.compute.len() + 1;
        self.compute.apply(self.compute.append(c, r as u64));
    }

    pub fn add_storage(&mut self, s: Storage) {
        let r = self.compute.len() + 1;
        self.storage.apply(self.storage.append(s, r as u64));
    }

    pub fn add_interconnect(&mut self, i: InterConnect) {
        let r = self.compute.len() + 1;
        self.interconnects.apply(self.interconnects.append(i, r as u64));
    }


    /// Get a compute resource by name
    ///
    /// ```rust
    /// use libtheia::models::data_centre::DataCentre;
    /// use libtheia::models::data_centre::Compute;
    ///
    /// let mut dc = DataCentre::new("test".to_string());
    /// let mut c = Compute::new("test".to_string(), 1, 1, 1);
    /// dc.add_compute(c.clone());
    /// let fc = dc.get_compute("test").unwrap();
    ///
    /// assert_eq!(c, fc.clone());
    /// ```
    pub fn get_compute(&self, name: &str) -> Option<&Compute> {
        for c in self.compute.iter() {
            if c.name == name {
                return Some(c);
            }
        }
        None
    }

    pub fn get_storage(&self, name: &str) -> Option<&Storage> {
        for s in self.storage.iter() {
            if s.name == name {
                return Some(s);
            }
        }
        None
    }

    pub fn get_interconnect(&self, name: &str) -> Option<&InterConnect> {
        for i in self.interconnects.iter() {
            if i.name == name {
                return Some(i);
            }
        }
        None
    }

    pub fn remove_compute(&mut self, name: &str) {
        fn find_index(list: &List<Compute, u64>, name: &str) -> Option<usize> {
            for (i, c) in list.iter().enumerate() {
                if c.name == name {
                    Some(i);
                }
            }
            None
        }
        if let Some(i) = find_index(&self.compute, name) {
            self.compute.apply(self.compute.delete_index(i, i as u64).unwrap());
        }
    }

    pub fn remove_storage(&mut self, name: &str) {
        fn find_index(list: &List<Storage, u64>, name: &str) -> Option<usize> {
            for (i, s) in list.iter().enumerate() {
                if s.name == name {
                    Some(i);
                }
            }
            None
        }
        if let Some(i) = find_index(&self.storage, name) {
            self.storage.apply(self.storage.delete_index(i, i as u64).unwrap());
        }
    }

    pub fn remove_interconnect(&mut self, name: &str) {
        fn find_index(list: &List<InterConnect, u64>, name: &str) -> Option<usize> {
            for (i, s) in list.iter().enumerate() {
                if s.name == name {
                    Some(i);
                }
            }
            None
        }
        if let Some(i) = find_index(&self.interconnects, name) {
            self.interconnects.apply(self.interconnects.delete_index(i, i as u64).unwrap());
        }
    }
}