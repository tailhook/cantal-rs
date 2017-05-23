use serde_json::Value;

use name::{Name, NameVisitor};


impl Name for Value {
    fn get(&self, key: &str) -> Option<&str> {
        self.as_object()
        .and_then(|x| x.get(key))
        .and_then(|x| x.as_str())
    }
    fn visit(&self, s: &mut NameVisitor) {
        if let Some(obj) = self.as_object() {
            for (k, v) in obj {
                if let Some(vs) = v.as_str() {
                    s.visit_pair(k, vs);
                }
            }
        }
    }
}
