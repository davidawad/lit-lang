//! Prints the schema bundle (`litgraph_schema_bundle::bundle`) as pretty JSON.

fn main() {
    println!("{}", litgraph_schema_bundle::render());
}
