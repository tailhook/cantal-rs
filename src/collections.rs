use collection::{Collection, Visitor};
use name::Name;
use value::Value;


impl<'a, T: Name> Collection for &'a [(T, &'a Value)] {
    fn visit<V: Visitor>(&self, visitor: &mut V) {
        for &(ref k, ref v) in self.iter() {
            //visit
        }
    }
}
