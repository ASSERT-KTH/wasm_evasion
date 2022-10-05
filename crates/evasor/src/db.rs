use core::slice;
use std::{
    borrow::Borrow,
    cell::RefCell,
    fmt::Debug,
    fs,
    io::{BufReader, Read},
};

use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};

use crate::errors::{AResult, CliError};

#[derive(Debug, Clone)]
pub struct DB<'a> {
    pub(crate) f: &'a str,
    pub(crate) config: sled::Config,
    pub(crate) db: Option<sled::Db>,
}

impl<'a> DB<'a> {
    pub fn new(f: &'a str, cache_size: u64) -> AResult<Self> {
        let config = sled::Config::default()
            .path(f.to_owned())
            .cache_capacity(cache_size);

        Ok(DB {
            f: f.clone(),
            config,
            db: None,
        })
    }

    pub fn open(&mut self) -> AResult<bool> {
        log::debug!("Opening db");
        self.db = Some(self.config.open()?);
        Ok(true)
    }

    pub fn create(&self) -> AResult<bool> {
        log::debug!("Creating db");
        fs::create_dir(self.f);
        Ok(true)
    }

    pub fn save(&self) -> AResult<bool> {
        self.db
            .as_ref()
            .ok_or(CliError::Any("Non existing db".into()))?
            .flush()?;
        Ok(true)
    }

    pub fn get_count(&self) -> AResult<usize> {
        Ok(self
            .db
            .as_ref()
            .ok_or(CliError::Any("Non existing db".into()))?
            .len())
    }

    pub fn set<T>(&self, k: &AsRef<[u8]>, v: T) -> AResult<bool>
    where
        T: Serialize,
    {
        self.db
            .as_ref()
            .ok_or(CliError::Any("Non existing db".into()))?
            .insert(k, serde_json::to_string_pretty(&v)?.as_bytes())?;
        self.db
            .as_ref()
            .ok_or(CliError::Any("Non existing db".into()))?
            .flush()?;
        Ok(true)
    }

    fn get_inner<K>(&self, k: &'a K) -> AResult<Vec<u8>>
    where
        K: AsRef<[u8]> + Debug,
    {
        let kvalue = self
            .db
            .as_ref()
            .ok_or(CliError::Any("Non existing db".into()))?
            .get(k)?;

        match kvalue {
            None => return Err(CliError::KeyNotFound(format!("{:?}", k))),
            Some(v) => return Ok(v.to_vec()),
        }
    }

    pub fn has<K, T>(&self, k: &'a K) -> AResult<bool>
    where
        K: AsRef<[u8]> + Debug,
    {
        let kv = self
            .db
            .as_ref()
            .ok_or(CliError::Any("Non existing db".into()))?
            .contains_key(k)?;
        Ok(kv)
    }

    pub fn get_all<T>(&self) -> AResult<impl Iterator<Item = T>>
    where
        T: DeserializeOwned + 'static,
    {
        let it = self
            .db
            .as_ref()
            .ok_or(CliError::Any("Non existing db".into()))?
            .iter()
            .filter_map(|f| f.ok())
            .map(move |(_, v)| {
                let cp = v.to_vec();
                let item: T = serde_json::from_str(&String::from_utf8(cp).unwrap()).unwrap();
                item
            })
            .into_iter();

        Ok(it)
    }

    pub fn get<K, T>(&self, k: &'a K) -> AResult<T>
    where
        T: DeserializeOwned + 'static,
        K: AsRef<[u8]> + Debug,
    {
        let kv = self.get_inner(&k)?;
        let item = serde_json::from_str(&String::from_utf8(kv)?)?;

        Ok(item)
    }
}
