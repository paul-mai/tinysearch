use bincode::{self, Error as BincodeError};
use cuckoofilter::{self, CuckooFilter, ExportedCuckooFilter};
use serde_derive::{Deserialize, Serialize};
use std::convert::From;

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub type Filters = HashMap<PathBuf, CuckooFilter<DefaultHasher>>;
type ExportedFilters = HashMap<PathBuf, ExportedCuckooFilter>;

pub struct Storage {
    pub filters: Filters,
}

impl From<Filters> for Storage {
    fn from(filters: Filters) -> Self {
        Storage { filters }
    }
}

impl Storage {
    pub fn to_bytes(&self) -> Result<Vec<u8>, BincodeError> {
        let encoded: Vec<u8> = bincode::serialize(&self.dehydrate())?;
        Ok(encoded)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BincodeError> {
        let decoded: ExportedFilters = bincode::deserialize(bytes)?;
        Ok(Storage {
            filters: Storage::hydrate(decoded),
        })
    }

    fn dehydrate(&self) -> ExportedFilters {
        self.filters
            .iter()
            .map(|(key, filter)| (key.clone(), filter.export()))
            .collect()
    }

    fn hydrate(exportedFilters: ExportedFilters) -> Filters {
        exportedFilters
            .into_iter()
            .map(|(key, exported)| (key.clone(), CuckooFilter::<DefaultHasher>::from(exported)))
            .collect()
    }
    //     let decoded: ExportedCuckooFilter = deserialize(&raw[..]).unwrap();
    // let recovered_filter = CuckooFilter::<DefaultHasher>::from(decoded);
}
