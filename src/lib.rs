use std::{borrow::Cow, io::Write};

use automerge::Change;
use automerge_protocol::ChangeHash;

type Nd = ChangeHash;
type Ed = (ChangeHash, ChangeHash);

struct Edges(Vec<Ed>);

impl<'a> dot::Labeller<'a, Nd, Ed> for Edges {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("automerge").unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        let hex = hex::encode(n.0);
        dot::Id::new(format!("N{}", hex)).unwrap()
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Edges {
    fn nodes(&self) -> dot::Nodes<'a, Nd> {
        // (assumes that |N| \approxeq |E|)
        let &Edges(ref v) = self;
        let mut nodes = Vec::with_capacity(v.len());
        for &(s, t) in v {
            nodes.push(s);
            nodes.push(t);
        }
        nodes.sort();
        nodes.dedup();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        let &Edges(ref edges) = self;
        Cow::Borrowed(&edges[..])
    }

    fn source(&self, e: &Ed) -> Nd {
        e.0
    }

    fn target(&self, e: &Ed) -> Nd {
        e.1
    }
}

pub fn graph_deps<W: Write>(changes: &[&Change], output: &mut W) {
    let mut edges = Vec::new();
    for change in changes {
        for dep in change.deps.iter().cloned() {
            edges.push((change.hash, dep));
        }
    }
    // for each change add it as a node (with hash)
    //
    // add edges to deps
    dot::render(&Edges(edges), output).unwrap()
}

#[cfg(test)]
mod tests {
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
        graph_deps(&[], &mut v);
        let s = String::from_utf8(v).unwrap();
        assert_eq!(
            &s,
            "digraph automerge {
}
"
        );
    }
}
