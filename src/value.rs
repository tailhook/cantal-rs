use std::fmt::Display;

use serde_json;


pub trait Value: Display {
    fn type_name(&self) -> &str;
    fn as_json(&self) -> serde_json::Value;
}
