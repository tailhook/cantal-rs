/// A structure used to serialize `Name` objects to submit to agent
pub struct Serializer {
}


/// Name of the metric
///
/// You may use `HashMap<String, String>` or `BTreeMap<String, String>` for
/// the name, but it might be more efficient to have a structure as a name
/// and used static strings as key names instead.
pub trait Name {
    fn get_key(&self, key: &str) -> &str;
    fn serialize(&self, s: &mut Serializer);
}

