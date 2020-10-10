use bincode::Error as BincodeError;
use cuckoofilter::{self, CuckooFilter, ExportedCuckooFilter};
use std::convert::From;

use std::collections::hash_map::DefaultHasher;

pub type PostId = (String, String);
pub type Filters = Vec<(PostId, CuckooFilter<DefaultHasher>)>;
type ExportedFilters = Vec<(PostId, ExportedCuckooFilter)>;

pub struct Storage {
    pub filters: Filters,
}

impl From<Filters> for Storage {
    fn from(filters: Filters) -> Self {
        Storage { filters }
    }
}

pub trait Score {
    fn score<A: AsRef<str>, I: IntoIterator<Item = A>>(&self, terms: I) -> u32;
}

// the score is the number of terms from the query that are contained in the
// current filter
impl Score for CuckooFilter<DefaultHasher> {
    fn score<A: AsRef<str>, I: IntoIterator<Item = A>>(&self, terms: I) -> u32 {
        terms
            .into_iter()
            .filter(|term| self.contains(term.as_ref()))
            .count() as u32
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

    fn hydrate(exported_filters: ExportedFilters) -> Filters {
        exported_filters
            .into_iter()
            .map(|(key, exported)| (key, CuckooFilter::<DefaultHasher>::from(exported)))
            .collect()
    }
}
