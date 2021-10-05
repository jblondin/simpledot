//! Very basic parser into the intermediate representation. Parses TreeDOT file pass in stdin and
//! outputs debug printout of constructed representation.

use simpledot::ir::parse_graph;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        input
    };
    match parse_graph(input.as_str()) {
        Ok(graph) => {
            println!("{:?}", graph);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    Ok(())
}
