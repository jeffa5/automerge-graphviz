use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::Context;
use automerge_graphviz::graph_deps;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// File to read automerge backend from.
    input: PathBuf,

    /// File to write dot graph to.
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::from_args();

    let mut backend_bytes = Vec::new();
    File::open(opts.input)
        .context("Failed reading input file")?
        .read_to_end(&mut backend_bytes)
        .context("Failed reading all of the input")?;

    let backend =
        automerge::Backend::load(backend_bytes).context("Failed loading the backend from bytes")?;

    let mut v = Vec::new();
    let changes = backend.get_changes(&[]);
    graph_deps(&changes, &mut v);

    let mut out = File::create(opts.output).context("Failed creating output file")?;
    out.write_all(&v)
        .context("Failed writing graph to output file")?;

    Ok(())
}
