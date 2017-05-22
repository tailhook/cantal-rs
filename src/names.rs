use serde_json::Value;

use name::{Name, Serializer};


impl Name for Value {
    fn get_key(&self, key: &str) -> &str {
        unimplemented!();
    }
    fn serialize(&self, s: &mut Serializer) {
        unimplemented!();
    }
}
