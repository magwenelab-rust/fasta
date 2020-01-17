use std::cmp;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

use flate2::read::GzDecoder;

use fasta;
use fasta::{Fasta, PeekableLines, Record};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        panic!("Must specify file argument")
    }

    let file = File::open(&args[1]).unwrap();
    let file2 = File::open(&args[1]).unwrap();

    let extension = Path::new(&args[1]).extension().unwrap().to_str().unwrap();
    let rdr: Box<dyn Read> = match extension {
        "gz" => Box::new(GzDecoder::new(file)),
        _ => Box::new(file),
    };

    let rdr2: Box<dyn Read> = match extension {
        "gz" => Box::new(GzDecoder::new(file2)),
        _ => Box::new(file2),
    };

    let fastabuf = fasta::FastaBuffer::from(BufReader::new(rdr));
    let mut recs: Vec<Record> = Vec::new();

    let mut timer = Instant::now();
    for rec in fastabuf.filter_map(Result::ok) {
        recs.push(rec);
    }
    // while let Some(rec) = fasta::next_record(&mut linebuf) {
    //     recs.push(rec);
    // }
    let mut duration = timer.elapsed();

    println!(
        "Time elapsed to parse records via FastaIterator: {:?}",
        duration
    );
    println!("Number of records: {}", recs.len());

    timer = Instant::now();
    let recs2 = Fasta::parse(BufReader::new(rdr2)).unwrap();
    duration = timer.elapsed();

    println!(
        "Time elapsed to parse records via Fasta::parse: {:?}",
        duration
    );
    println!("Number of records: {}", recs2.len());

    let mut totbases = 0;
    for r in &recs {
        totbases += r.sequence.len();
    }

    println!("Total size of sequence data: {}", totbases);

    println!("\nFirst 5 records, next_record:");
    for r in &recs[..cmp::min(5, recs.len())] {
        println!("{}", r);
    }
    println!();
    println!("\nFirst 5 records, Fasta:");
    for r in &recs2[..cmp::min(5, recs2.len())] {
        println!("{}", r);
    }

    Ok(())
}
