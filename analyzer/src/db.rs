use core::slice;
use std::{
    borrow::Borrow,
    cell::RefCell,
    fmt::Debug,
    io::{BufReader, Read},
};

use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};

use crate::errors::{AResult, CliError};

#[derive(Debug, Clone)]
pub struct DB<'a> {
    pub(crate) f: &'a str,
    pub(crate) db: sled::Db,
}

impl<'a> DB<'a> {
    pub fn new(f: &'a str) -> AResult<Self> {
        Ok(DB {
            f: f.clone(),
            db: sled::open(f.clone())?,
        })
    }

    pub fn save(&self) -> AResult<bool> {
        self.db.flush()?;
        Ok(true)
    }

    pub fn get_count(&self) -> AResult<usize> {
        Ok(self.db.len())
    }

    pub fn set<T>(&self, k: &AsRef<[u8]>, v: T) -> AResult<bool>
    where
        T: Serialize,
    {
        self.db
            .insert(k, serde_json::to_string_pretty(&v)?.as_bytes())?;
        self.db.flush()?;
        Ok(true)
    }

    fn get_inner<K>(&self, k: &'a K) -> AResult<Vec<u8>>
    where
        K: AsRef<[u8]> + Debug,
    {
        let kvalue = self.db.get(k)?;

        match kvalue {
            None => return Err(CliError::KeyNotFound(format!("{:?}", k))),
            Some(v) => return Ok(v.to_vec()),
        }
    }

    pub fn has<K, T>(&self, k: &'a K) -> AResult<bool>
    where
        K: AsRef<[u8]> + Debug,
    {
        let kv = self.db.contains_key(k)?;
        Ok(kv)
    }

    pub fn get_all<T>(&self) -> AResult<impl Iterator<Item = T>>
    where
        T: DeserializeOwned + 'static,
    {
        let it = self
            .db
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

#[cfg(test)]
pub mod tests {
    use super::DB;

    #[test]
    pub fn test_save() {
        let t = DB::new("test_db").unwrap();
        t.set(&"K1", &"K2").unwrap();
    }

    #[test]
    pub fn test_load() {
        let t = DB::new("test_db").unwrap();
        let h: String = t.get(&"K1").unwrap();

        println!("{:?}", h);
    }

    #[test]
    pub fn count() {
        let t = DB::new("test_db").unwrap();

        println!("count {:?}", t.get_count());
    }
}
