use collection::{Collection, Visitor};
use name::Name;
use value::Value;


impl<'a, T: Name> Collection for [(T, &'a Value)] {
    fn visit<'x>(&'x self, visitor: &mut Visitor<'x>) {
        for &(ref k, v) in self.iter() {
            visitor.metric(k, v);
        }
    }
}
