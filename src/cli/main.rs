#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::error::Error;

use csv::Writer;
use mtcs::{max_flow_network_simplex, ObligationNetwork};

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
    wtr.write_record(["id", "amount"])?;
    for obligation in res {
        let id = obligation.0;
        let amount = obligation.1;
        wtr.write_record([&id.to_string(), &amount.to_string()])?;
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
