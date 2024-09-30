mod model {
    mod dc {
        ///
        /// GPU data structure
        ///
        pub struct GPU {
            pub name: String,
            pub clock_ghz: i32,
            pub ram_gb: i32,
            pub bus_speed: i32,
        }

        ///
        /// Network connections
        ///
        pub struct InterConnect {
            pub name: String,
            pub speed_gb: i32,
            pub low_latency: bool,
        }

        ///
        /// Disk Types
        ///
        pub enum DiskType {
            Spinning,
            SolidState,
            NVMe
        }

        ///
        /// Disks
        ///
        pub struct Disk {
            pub name: String,
            pub disk_type: DiskType,
            pub capacity_tb: i32,
            pub write_speed_mb: Option<i32>,
            pub read_speed_mb: Option<i32>,
            pub iops: Option<i32>,
        }

        ///
        /// Main data structure for storing and retrieving compute resources
        ///
        pub struct Compute {
            pub name: String,
            pub cores: i32,
            pub core_ghz: i32,
            pub ram_gb: i32,
            pub disks: Vec<Disk>,
            pub links: Vec<InterConnect>,
            pub gpus: Vec<GPU>,
        }

        ///
        /// Storage, where capacity is either given, or the sum of all disks
        ///
        pub struct Storage {
            pub name: String,
            pub disks: Option<Vec<Disk>>,
            pub capacity_gb: i32,
            pub links: Vec<InterConnect>,
        }

        ///
        /// A full DC
        ///
        pub struct DataCentre {
            pub name: String,
            pub compute: Vec<Compute>,
            pub storage: Vec<Storage>,
            pub interconnects: Vec<InterConnect>,
        }
    }
}