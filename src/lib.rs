use std::{borrow::Cow, io::Write};

use automerge::Change;
use automerge_protocol::ChangeHash;

type Nd = ChangeHash;
type Ed = (ChangeHash, ChangeHash);

struct Edges {
    edges: Vec<Ed>,
    hash_length: usize,
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Edges {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("automerge").unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        let hex: String = hex::encode(n.0).chars().take(self.hash_length).collect();
        dot::Id::new(format!("N{}", hex)).unwrap()
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Edges {
    fn nodes(&self) -> dot::Nodes<'a, Nd> {
        // (assumes that |N| \approxeq |E|)
        let &Edges { ref edges, .. } = self;
        let mut nodes = Vec::with_capacity(edges.len());
        for &(s, t) in edges {
            nodes.push(s);
            nodes.push(t);
        }
        nodes.sort();
        nodes.dedup();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        let &Edges { ref edges, .. } = self;
        Cow::Borrowed(&edges[..])
    }

    fn source(&self, e: &Ed) -> Nd {
        e.0
    }

    fn target(&self, e: &Ed) -> Nd {
        e.1
    }
}

pub fn graph_deps<W: Write>(changes: &[&Change], output: &mut W, hash_length: usize) {
    let mut edges = Vec::new();
    for change in changes {
        for dep in change.deps.iter().cloned() {
            edges.push((change.hash, dep));
        }
    }
    // for each change add it as a node (with hash)
    //
    // add edges to deps
    dot::render(&Edges { edges, hash_length }, output).unwrap()
}

#[cfg(test)]
mod tests {
    use automerge::{LocalChange, Path, Primitive, Value};

    use super::*;

    #[derive(PartialEq, Eq)]
    #[doc(hidden)]
    pub struct PrettyString<'a>(pub &'a str);

    /// Make diff to display string as multi-line string
    impl<'a> std::fmt::Debug for PrettyString<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str(self.0)
        }
    }

    macro_rules! assert_eq {
        ($left:expr, $right:expr) => {
            pretty_assertions::assert_eq!(PrettyString($left), PrettyString($right));
        };
    }

    #[test]
    fn empty() {
        let mut v = Vec::new();
        graph_deps(&[], &mut v, 7);
        let s = String::from_utf8(v).unwrap();
        assert_eq!(
            &s,
            "digraph automerge {
}
"
        );
    }

    #[test]
    fn minimal() {
        let mut f = automerge::Frontend::new();
        let mut b = automerge::Backend::init();

        let change = f
            .change::<_, _, std::convert::Infallible>(None, |d| {
                d.add_change(LocalChange::set(
                    Path::root().key("a"),
                    Value::Primitive(Primitive::Int(1)),
                ))
                .unwrap();
                Ok(())
            })
            .unwrap()
            .1
            .unwrap();
        b.apply_local_change(change).unwrap();
        let change = f
            .change::<_, _, std::convert::Infallible>(None, |d| {
                d.add_change(LocalChange::set(
                    Path::root().key("b"),
                    Value::Primitive(Primitive::Int(1)),
                ))
                .unwrap();
                Ok(())
            })
            .unwrap()
            .1
            .unwrap();
        b.apply_local_change(change).unwrap();

        let mut v = Vec::new();
        graph_deps(&b.get_changes(&[]), &mut v, 7);
        String::from_utf8(v).unwrap();
    }
}
