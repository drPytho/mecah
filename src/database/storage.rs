use std::collections::HashMap;
use std::fmt;

/// This enum is just a easy to read description
/// of possible errors ocurred when consulting
/// the database
pub enum StorageError {
    KeyNotFound,
    InternalError,
}

impl fmt::Debug for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match *self {
            StorageError::KeyNotFound => "Key not found",
            StorageError::InternalError => "Internal error",
        };

        write!(f, "{}", message)
    }
}

/// This is the main data structure we will be using
///
/// It consist on a HashMap which has a String as key and
/// a StorageValue (enum) as a value, which can be a plain
/// value (also String) or a Set (HashSet)
pub struct Storage {
    data: HashMap<String, String>,
}

// Let's implement Storage by adding all Redis
// commands we want to support
impl Storage {
    /// Creates a new storage instance
    ///
    /// This is a static method (not attached to an instance)
    /// used to create an instance of the structure. It's idiomatic
    /// to call these kind of methods as "new" in Rust
    ///
    /// # Examples
    ///
    /// ```
    /// use mecah::database::storage::Storage;
    ///
    /// let storage = Storage::new();
    /// ```
    pub fn new() -> Storage {
        Storage {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: String) -> Result<&String, StorageError> {
        if !self.data.contains_key(&key) {
            return Err(StorageError::KeyNotFound);
        }

        let data = self.data.get(&key).unwrap();

        Ok(&data)
    }

    /// Sets a new plain key-value into the database
    ///
    ///
    /// As we are going to modify the HashMap (inserting)
    /// we need to get a reference to self '&mut self'
    ///
    pub fn set(&mut self, key: String, value: String) -> Result<bool, StorageError> {
        self.data.insert(key, value);
        Ok(true)
    }

    pub fn exists(&mut self, key: String) -> Result<bool, StorageError> {
        Ok(self.data.contains_key(&key))
    }

    pub fn del(&mut self, key: String) -> Result<bool, StorageError> {
        if !self.data.contains_key(&key) {
            return Err(StorageError::KeyNotFound);
        }

        self.data.remove(&key);

        Ok(true)
    }

    pub fn count_keys(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::Storage;

    #[test]
    fn test_database_set_get() {
        let mut storage = Storage::new();

        assert_eq!(
            storage.set("test".to_string(), "test".to_string()).unwrap(),
            true
        );
        assert_eq!(
            storage.get("test".to_string()).unwrap().to_string(),
            "test".to_string()
        );
    }

    #[test]
    fn test_get_with_no_data() {
        let storage = Storage::new();

        assert_eq!(storage.get("test".to_string()).is_err(), true);
    }

    #[test]
    fn test_exists() {
        let mut storage = Storage::new();

        assert_eq!(
            storage.set("test".to_string(), "test".to_string()).unwrap(),
            true
        );
        assert_eq!(storage.exists("test".to_string()).unwrap(), true);
    }

    #[test]
    fn test_set_overwrite_key() {
        let mut storage = Storage::new();

        assert_eq!(
            storage.set("test".to_string(), "test".to_string()).unwrap(),
            true
        );
        assert_eq!(
            storage.get("test".to_string()).unwrap().to_string(),
            "test".to_string()
        );
        assert_eq!(
            storage
                .set("test".to_string(), "test2".to_string())
                .unwrap(),
            true
        );
        assert_eq!(
            storage.get("test".to_string()).unwrap().to_string(),
            "test2".to_string()
        );
        assert_eq!(storage.count_keys(), 1);
    }

    #[test]
    fn test_delete_key() {
        let mut storage = Storage::new();

        assert_eq!(
            storage.set("test".to_string(), "test".to_string()).unwrap(),
            true
        );
        assert_eq!(
            storage.get("test".to_string()).unwrap().to_string(),
            "test".to_string()
        );
        assert_eq!(storage.del("test".to_string()).unwrap(), true);
        assert_eq!(storage.get("test".to_string()).is_err(), true);
        assert_eq!(storage.count_keys(), 0);
    }

    #[test]
    fn test_count_keys() {
        let mut storage = Storage::new();

        assert_eq!(
            storage.set("test".to_string(), "test".to_string()).unwrap(),
            true
        );
        assert_eq!(
            storage
                .set("test2".to_string(), "test2".to_string())
                .unwrap(),
            true
        );

        assert_eq!(storage.count_keys(), 2);
    }
}
