use std::fmt;
use std::io::{self, Write};

use collection::{Collection, Visitor};

use name::{Name, NameVisitor};
use value::Value;


struct PrintVisitor<'a, W: Write + ?Sized + 'a>(&'a mut W);

struct NameFormatter<'a, N: Name + ?Sized + 'a>(&'a N);
struct FmtSerializer<'a: 'b, 'b>(fmt::DebugMap<'b, 'a>);

impl<'a, 'b> NameVisitor for FmtSerializer<'a, 'b> {
    fn visit_pair(&mut self, key: &str, value: &str) {
        self.0.entry(&key, &value);
    }
}

impl<'a, N: Name + ?Sized + 'a> fmt::Display for NameFormatter<'a, N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = FmtSerializer(f.debug_map());
        self.0.visit(&mut s);
        s.0.finish()
    }
}


impl<'a, W: Write + 'a> Visitor for PrintVisitor<'a, W> {
    fn metric(&mut self, name: &Name, value: &Value)
    {
        println!("{} {}", NameFormatter(name), value);
    }
}

pub fn print<C: Collection, W: Write>(col: C, mut out: W) -> io::Result<()> {
    col.visit(&mut PrintVisitor(&mut out));
    Ok(())
}
