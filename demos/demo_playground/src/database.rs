use std::collections::HashMap;

pub struct Database {
    data: HashMap<String, String>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn delete(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    pub fn keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }
}
