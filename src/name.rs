/// A structure used to serialize `Name` objects to submit to agent
pub trait NameVisitor {
    fn visit_pair(&mut self, key: &str, value: &str);
}


/// Name of the metric
///
/// You may use `HashMap<String, String>` or `BTreeMap<String, String>` for
/// the name, but it might be more efficient to have a structure as a name
/// and used static strings as key names instead.
pub trait Name {
    fn get(&self, key: &str) -> Option<&str>;
    fn visit(&self, s: &mut NameVisitor);
}

