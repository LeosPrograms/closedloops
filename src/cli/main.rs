#![warn(clippy::all, clippy::pedantic)]

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
use csv::{Reader as CsvReader, Writer as CsvWriter};
use log::LevelFilter;
use mtcs::{check, run, ObligationNetwork, SetoffNotice};
use simplelog::{Config as SimpleLoggerConfig, SimpleLogger};

/// Tool for running Multilateral Trade Credit Set-off (MTCS) on an obligation network
#[derive(Parser, Debug)]
#[command(version, long_about = None)]
struct Args {
    /// Path to input CSV file with obligations (fields - `id` (optional), `debtor`, `creditor`, `amount`)
    #[arg(short, long)]
    input_file: PathBuf,

    /// Path to output CSV file
    #[arg(short, long)]
    output_file: PathBuf,

    /// Log level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

// Read the obligations from CSV file
fn read_obligations_csv(reader: impl Read, _has_headers: bool) -> ObligationNetwork {
    let mut rdr = CsvReader::from_reader(reader);
    let rows: Result<Vec<_>, _> = rdr.deserialize().collect();
    let rows = rows.unwrap();
    ObligationNetwork { rows }
}

// Write the clearing results to CSV file
fn write_csv(res: Vec<SetoffNotice>, writer: impl Write) -> Result<(), Box<dyn Error>> {
    let mut wtr = CsvWriter::from_writer(writer);
    for setoff in res {
        wtr.serialize(setoff)?;
    }
    wtr.flush()?;
    Ok(())
}

fn log_level_from_u8(level: u8) -> LevelFilter {
    match level {
        0 => LevelFilter::Off,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        3.. => LevelFilter::Trace,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI args
    let args = Args::parse();

    // Initialize the logger
    let log_level = log_level_from_u8(args.verbose);
    SimpleLogger::init(log_level, SimpleLoggerConfig::default()).unwrap();

    // Read the obligations from the input CSV file
    let input_file = File::open(args.input_file)?;
    let on = read_obligations_csv(&input_file, true);

    // Run the MTCS algorithm
    let res = run(on);
    check(&res);

    // Write the result to the output CSV file
    let output_file = File::create(args.output_file)?;
    write_csv(res, &output_file)
}
