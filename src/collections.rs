use collection::{Collection, Visitor};
use name::Name;
use value::Value;


impl<'a, T: Name> Collection for &'a [(T, &'a Value)] {
    fn visit(&self, visitor: &mut Visitor) {
        for &(ref k, v) in self.iter() {
            visitor.metric(k, v);
        }
    }
}
