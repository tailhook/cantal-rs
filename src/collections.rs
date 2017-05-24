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

impl<'a, T: Name> Collection for Vec<(T, &'a Value)> {
    fn visit<'x>(&'x self, visitor: &mut Visitor<'x>) {
        for &(ref k, v) in self.iter() {
            visitor.metric(k, v);
        }
    }
}

impl<'a, T: Collection> Collection for Vec<T> {
    fn visit<'x>(&'x self, visitor: &mut Visitor<'x>) {
        for sub in self.iter() {
            sub.visit(visitor);
        }
    }
}

impl<'a, T: Collection> Collection for &'a [T] {
    fn visit<'x>(&'x self, visitor: &mut Visitor<'x>) {
        for sub in self.iter() {
            sub.visit(visitor);
        }
    }
}

impl<'a, T: Collection+?Sized> Collection for Box<T> {
    fn visit<'x>(&'x self, visitor: &mut Visitor<'x>) {
        (**self).visit(visitor);
    }
}
