use serde::ser::{Serialize, Serializer, SerializeSeq, SerializeMap, Error};

use collection::{Collection, Visitor};
use name::{Name, NameVisitor};
use value::Value;


pub struct Json<T: Collection>(pub T);

struct JsonVisitor<'a, Ok, E, S>(&'a mut S, &'a mut Option<E>)
    where E: Error + 'a,
          S: SerializeSeq<Ok=Ok, Error=E> + 'a;

struct JsonNameVisitor<'a, Ok, E, S>(&'a mut S, &'a mut Option<E>)
    where E: Error + 'a,
          S: SerializeMap<Ok=Ok, Error=E> + 'a;

pub struct JsonName<'a>(pub &'a Name);
struct JsonValue<'a>(&'a Value);

impl<'a, Ok, E, S> NameVisitor for JsonNameVisitor<'a, Ok, E, S>
    where E: Error,
          S: SerializeMap<Ok=Ok, Error=E> + 'a,
{
    fn visit_pair(&mut self, key: &str, value: &str) {
        match self.0.serialize_entry(&key, &value) {
            Ok(()) => {}
            Err(e) => {
                if self.1.is_none() {
                    // only keep track first error
                    *self.1 = Some(e);
                }
            }
        }
    }
}

impl<'a> Serialize for JsonName<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut map = serializer.serialize_map(None)?;
        let mut err = None;
        self.0.visit(&mut JsonNameVisitor(&mut map, &mut err));
        if let Some(err) = err {
            return Err(err);
        }
        map.end()
    }
}

impl<'a> Serialize for JsonValue<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(self.0.raw_type().as_json_str())?;
        seq.serialize_element(&self.0.as_json())?;
        seq.end()
    }
}


impl<'a, Ok, E, S> Visitor for JsonVisitor<'a, Ok, E, S>
    where E: Error,
          S: SerializeSeq<Ok=Ok, Error=E> + 'a,
{
    fn metric(&mut self, name: &Name, value: &Value)
    {
        match self.0.serialize_element(&(JsonName(name), JsonValue(value))) {
            Ok(()) => {}
            Err(e) => {
                if self.1.is_none() {
                    // only keep track first error
                    *self.1 = Some(e);
                }
            }
        }
    }
}

impl<T: Collection> Serialize for Json<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(None)?;
        let mut err = None;
        self.0.visit(&mut JsonVisitor(&mut seq, &mut err));
        if let Some(err) = err {
            return Err(err);
        }
        seq.end()
    }
}
/*
impl<'a> Json<'a> {
    pub fn new<T: Collection + ?Sized>(coll: &T) -> Json {
        Json(coll as &Collection)
    }
}
*/
