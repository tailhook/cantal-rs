use serde_json::Value;

use name::{Name, Serializer};


impl Name for Value {
    fn get_key(&self, key: &str) -> Option<&str> {
        self.as_object()
        .and_then(|x| x.get(key))
        .and_then(|x| x.as_str())
    }
    fn serialize(&self, s: &mut Serializer) {
        if let Some(obj) = self.as_object() {
            for (k, v) in obj {
                if let Some(vs) = v.as_str() {
                    s.add_pair(k, vs);
                }
            }
        }
    }
}
