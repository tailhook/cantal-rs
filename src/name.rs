/// A structure used to serialize `Name` objects to submit to agent
pub struct Serializer<'a> {
}

impl<'a> Serializer<'a> {
    pub fn add_pair(&mut self, key: &str, value: &str) {
        unimplemented!();
    }
}


/// Name of the metric
///
/// You may use `HashMap<String, String>` or `BTreeMap<String, String>` for
/// the name, but it might be more efficient to have a structure as a name
/// and used static strings as key names instead.
pub trait Name {
    fn get_key(&self, key: &str) -> Option<&str>;
    fn serialize(&self, s: &mut Serializer);
}

