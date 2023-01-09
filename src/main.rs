#![warn(clippy::all, clippy::pedantic)]
use csv::Writer;
use mcmf::{Capacity, Cost, GraphBuilder, Vertex};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;

//
// Define the Obligation network
//
#[derive(Clone, Debug, Deserialize)]
struct Obligation {
    id: i32,
    debtor: i32,
    creditor: i32,
    amount: i32,
}

#[derive(Clone, Debug, Default)]
struct ObligationNetwork {
    pub rows: Vec<Obligation>,
}
// Function to read the obligatinos from CSV file
fn read_obligations_csv(filepath: &str, _has_headers: bool) -> ObligationNetwork {
    let file = std::fs::File::open(filepath).unwrap();
    let mut rdr = csv::Reader::from_reader(file);
    let rows: Result<Vec<_>, _> = rdr.deserialize().collect();
    let rows = rows.unwrap();
    ObligationNetwork { rows }
}

// Function to write the clearing results
fn write_csv(res: Vec<(i32, i32)>, filepath: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(filepath)?;
    wtr.write_record(&["id", "amount"])?;
    for obligation in res {
        let id = obligation.0;
        let amount = obligation.1;
        wtr.write_record(&[&id.to_string(), &amount.to_string()])?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() {
    // get the filename to process
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);    // Test output
    let mut inputfile = "./data/".to_string();
    inputfile += &args[1];
    let mut outputfile = "./result/".to_string();
    outputfile += &args[1];

    // Read the obligatins CSV file from CosmWasm CoFi Clearing MVP
    let on = read_obligations_csv(&inputfile, true);

    let res = max_flow_network_simplex(on);

    let _res_w = write_csv(res, &outputfile);
}

fn max_flow_network_simplex(on: ObligationNetwork) -> Vec<(i32, i32)> {
    // Calculate the net_position "b" vector as a hashmap
    //          liabilities
    //          and a graph "g"
    // Prepare the clearing as a hashmap
    let mut net_position: HashMap<i32, i32> = HashMap::new();
    let mut liabilities: HashMap<(i32, i32), i32> = HashMap::new();
    let mut td: i64 = 0;
    let mut g = GraphBuilder::new();
    // let mut clearing : HashMap<i32, (i32, i32, i32)>= HashMap::new();
    let mut clearing = Vec::new();
    for o in on.rows {
        g.add_edge(o.debtor, o.creditor, Capacity(o.amount), Cost(1));
        let balance = net_position.entry(o.debtor).or_insert(0);
        *balance -= o.amount;
        let balance = net_position.entry(o.creditor).or_insert(0);
        *balance += o.amount;
        let liability = liabilities.entry((o.debtor, o.creditor)).or_insert(0);
        *liability += o.amount;
        td += o.amount as i64;
        clearing.push((o.id, o.debtor, o.creditor, o.amount));
        // println!("{:?}", o.id);
    }
    // for liability in &clearing {
    //     println!("{:?}", liability);  // Test output
    // }

    // Add source and sink flows based on values of "b" vector
    for (&firm, balance) in &net_position {
        match balance {
            x if x < &0 => g.add_edge(Vertex::Source, firm, Capacity(-balance), Cost(0)),
            x if x > &0 => g.add_edge(firm, Vertex::Sink, Capacity(*balance), Cost(0)),
            &_ => continue,
        };
    }

    // Get the minimum cost maximum flow paths and calculate "nid"
    let (remained, paths) = g.mcmf();
    let nid: i32 = net_position
        .into_values()
        .filter(|balance| balance > &0)
        .sum();

    // substract minimum cost maximum flow from the liabilities to get the clearing solution
    let mut tc: i64 = td;
    for path in paths {
        // print!("{:?} Flow trough: ", path.flows[0].amount);   // Test output
        let _result = path
            .vertices()
            .windows(2)
            .filter(|w| (w[0].as_option() != None) & (w[1].as_option() != None))
            .inspect(|w| {
                // print!("{} --> {} : ", w[0].as_option().unwrap(), w[1].as_option().unwrap());  // Test output
                liabilities
                    .entry((w[0].as_option().unwrap(), w[1].as_option().unwrap()))
                    .and_modify(|e| *e -= (path.flows[0].amount) as i32);
                tc -= path.flows[0].amount as i64;
            })
            .collect::<Vec<_>>();
        // println!();  // Test output
    }
    // for r in &liabilities {
    //     println!("{:?}", r);    // Test output
    // }

    // Print key results and check for correct sums
    println!("----------------------------------");
    println!("            NID = {:?}", nid);
    println!("     Total debt = {:?}", td);
    println!("Total remainder = {:?}", remained);
    println!("  Total cleared = {:?}", tc);
    // assert_eq!(td, remained + tc);

    // Assign cleared amounts to individual obligations
    let mut res: Vec<(i32, i32)> = Vec::new();
    for o in clearing {
        // println!("{:?} {:?}", o.0, o.3);     // Test output
        match liabilities.get(&(o.1, o.2)).unwrap() {
            0 => continue,
            x if x < &o.3 => {
                res.push((o.0, *liabilities.get(&(o.1, o.2)).unwrap()));
                liabilities.entry((o.1, o.2)).and_modify(|e| *e = 0);
            }
            _ => {
                liabilities.entry((o.1, o.2)).and_modify(|e| *e -= o.3);
                res.push((o.0, o.3));
            }
        }
    }
    res
}
