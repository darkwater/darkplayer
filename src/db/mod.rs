pub mod indexing;

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Database {
    pub index: HashMap<PathBuf, Ed2kHash>,

    pub fid_by_ed2k: BTreeMap<Ed2kHash, Fid>,
    pub files: BTreeMap<Fid, Cached<ranidb::File>>,
    pub episodes: BTreeMap<Eid, Cached<ranidb::Episode>>,
    pub animes: BTreeMap<Aid, Cached<ranidb::Anime>>,
    pub group: BTreeMap<Gid, Cached<ranidb::Group>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Ed2kHash(pub String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Fid(pub i64);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Eid(pub i64);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Aid(pub i64);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Gid(pub i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cached<T: CacheTarget> {
    pub value: T,
    pub last_update: DateTime<Utc>,
}

pub trait CacheTarget {
    fn ttl(&self) -> chrono::Duration;
}

impl CacheTarget for ranidb::File {
    fn ttl(&self) -> chrono::Duration {
        if self.aired_date.signed_duration_since(Utc::now()) < chrono::Duration::days(7) {
            chrono::Duration::days(1)
        } else {
            chrono::Duration::days(30)
        }
    }
}

impl CacheTarget for ranidb::Episode {
    fn ttl(&self) -> chrono::Duration {
        if self.aired.signed_duration_since(Utc::now()) < chrono::Duration::days(7) {
            chrono::Duration::days(1)
        } else {
            chrono::Duration::days(30)
        }
    }
}

impl CacheTarget for ranidb::Anime {
    fn ttl(&self) -> chrono::Duration {
        if self
            .end_date
            .is_none_or(|d| d.signed_duration_since(Utc::now()) < chrono::Duration::days(14))
        {
            chrono::Duration::days(1)
        } else if self
            .end_date
            .is_some_and(|d| d.signed_duration_since(Utc::now()) < chrono::Duration::days(365))
        {
            chrono::Duration::days(7)
        } else {
            chrono::Duration::days(30)
        }
    }
}

impl CacheTarget for ranidb::Group {
    fn ttl(&self) -> chrono::Duration {
        chrono::Duration::days(30)
    }
}
