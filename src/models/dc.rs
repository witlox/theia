use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct DataCentre {
    pub name: String,
    pub compute: Vec<Compute>,
    pub storage: Vec<Storage>,
    pub interconnects: Vec<InterConnect>,
}

impl DataCentre {
    /// DataCentre instance
    ///
    /// * `n` - storage name.
    pub fn new(n: String) -> DataCentre {
        DataCentre{
            name: n,
            compute: Vec::new(),
            storage: Vec::new(),
            interconnects: Vec::new(),
        }
    }

    pub fn add_compute(&mut self, c: Compute) {
        self.compute.push(c);
    }

    pub fn add_storage(&mut self, s: Storage) {
        self.storage.push(s);
    }

    pub fn add_interconnect(&mut self, i: InterConnect) {
        self.interconnects.push(i);
    }

    pub(crate) fn get_compute(&self, name: &str) -> Option<&Compute> {
        for c in &self.compute {
            if c.name == name {
                return Some(c);
            }
        }
        None
    }

    pub(crate) fn get_storage(&self, name: &str) -> Option<&Storage> {
        for s in &self.storage {
            if s.name == name {
                return Some(s);
            }
        }
        None
    }

    pub(crate) fn get_interconnect(&self, name: &str) -> Option<&InterConnect> {
        for i in &self.interconnects {
            if i.name == name {
                return Some(i);
            }
        }
        None
    }

    pub(crate) fn update_compute(&mut self, c: Compute) {
        for i in 0..self.compute.len() {
            if self.compute[i].name == c.name {
                self.compute[i] = c;
                return;
            }
        }
    }

    pub(crate) fn update_storage(&mut self, s: Storage) {
        for i in 0..self.storage.len() {
            if self.storage[i].name == s.name {
                self.storage[i] = s;
                return;
            }
        }
    }

    pub(crate) fn update_interconnect(&mut self, i: InterConnect) {
        for j in 0..self.interconnects.len() {
            if self.interconnects[j].name == i.name {
                self.interconnects[j] = i;
                return;
            }
        }
    }

    pub(crate) fn remove_compute(&mut self, name: &str) {
        self.compute.retain(|c| c.name != name);
    }

    pub(crate) fn remove_storage(&mut self, name: &str) {
        self.storage.retain(|s| s.name != name);
    }

    pub(crate) fn remove_interconnect(&mut self, name: &str) {
        self.interconnects.retain(|i| i.name != name);
    }
}