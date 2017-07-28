/// A structure used to serialize `Name` objects to submit to agent
pub trait NameVisitor {
    /// Report a keyword name and value used to demark a metric
    fn visit_pair(&mut self, key: &str, value: &str);
}


/// Name of the metric
///
/// You may use `HashMap<String, String>` or `BTreeMap<String, String>` for
/// the name, but it might be more efficient to have a structure as a name
/// and used static strings as key names instead.
pub trait Name {
    /// Get item by key
    fn get(&self, key: &str) -> Option<&str>;
    /// Visit all keys in metric
    fn visit(&self, s: &mut NameVisitor);
}

