# fasta
A Rust library for reading and writing FASTA formatted files.


Example usage:

```rust

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use fasta;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        panic!("Must specify file argument")
    }

    let file = File::open(&args[1]).unwrap();

    let fastabuf = fasta::FastaBuffer::from(BufReader::new(rdr));

    let recs: Vec<fasta::Record> = fastabuf.collect();

    for rec in recs {
      println!("{}", rec);
    }
}
```