#![warn(clippy::all, clippy::pedantic)]
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
use csv::{Reader as CsvReader, Writer as CsvWriter};
use mtcs::{max_flow_network_simplex, ObligationNetwork};

#[derive(Parser, Debug)]
#[command(version, long_about = None)]
struct Args {
    /// Path to input file
    #[arg(short, long)]
    input_file: PathBuf,

    /// Path to output file
    #[arg(short, long)]
    output_file: PathBuf,
}

// Read the obligations from CSV file
fn read_obligations_csv(reader: impl Read, _has_headers: bool) -> ObligationNetwork {
    let mut rdr = CsvReader::from_reader(reader);
    let rows: Result<Vec<_>, _> = rdr.deserialize().collect();
    let rows = rows.unwrap();
    ObligationNetwork { rows }
}

// Write the clearing results to CSV file
fn write_csv(res: Vec<(i32, i32)>, writer: impl Write) -> Result<(), Box<dyn Error>> {
    let mut wtr = CsvWriter::from_writer(writer);
    wtr.write_record(["id", "amount"])?;
    for obligation in res {
        let id = obligation.0;
        let amount = obligation.1;
        wtr.write_record([&id.to_string(), &amount.to_string()])?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let input_file = File::open(args.input_file)?;

    // Read the obligations CSV file
    let on = read_obligations_csv(&input_file, true);
    let res = max_flow_network_simplex(on);

    let output_file = File::open(args.output_file)?;
    write_csv(res, &output_file)
}
