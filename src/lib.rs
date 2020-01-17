pub mod errors;
use errors::FastaError;

use std::fmt;
use std::io;
use std::io::BufRead;
use std::io::Lines;
use std::iter::Peekable;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
/// fasta::Record represents a single FASTA record
pub struct Record {
    pub id: String,
    pub description: String,
    pub sequence: String,
}

impl Record {
    /// Returns a new fasta::Record with appropriate default fields
    fn new() -> Record {
        Record {
            ..Default::default()
        }
    }

    fn set_header(&mut self, s: &str) {
        let mut parts = if s.starts_with('>') {
            s[1..].splitn(2, char::is_whitespace)
        } else {
            s.splitn(2, char::is_whitespace)
        };
        self.id = parts.next().unwrap_or("").to_owned();
        self.description = parts.next().unwrap_or("").to_owned();
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            ">{} {}\n{}...",
            self.id,
            self.description,
            self.sequence.get(0..40).unwrap_or(&self.sequence)
        )
    }
}

/// fasta::Fasta represents a collection of FASTA records
#[derive(Debug, Default)]
pub struct Fasta(Vec<Record>);

impl Fasta {
    pub fn new() -> Fasta {
        Fasta {
            ..Default::default()
        }
    }
}

impl Deref for Fasta {
    type Target = Vec<Record>;
    fn deref(&self) -> &Vec<Record> {
        &self.0
    }
}

impl DerefMut for Fasta {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Fasta {
    /// Fasta::parse parses lines from a type implementing BufRead,
    /// returning a Fasta object containing all the records
    /// contained in the BufRead.
    ///
    /// If you actually need all the records than Fasta::Parse
    /// is faster than iterating on FastaIter, but FastaIter allows
    /// you to filter the records lazily or to take a limited number
    /// of records, or any of the other "magic" that object implementing
    /// Iterator can do.
    pub fn parse(r: impl BufRead) -> Result<Fasta, FastaError> {
        let mut records: Fasta = Fasta::new();
        let mut current_rec = Record::new();
        let mut active_record = false;

        for line in r.lines().filter_map(Result::ok) {
            match line.trim().chars().next() {
                None | Some(';') => continue, // empty line or comment
                Some('>') => {
                    if active_record {
                        records.push(current_rec)
                    }
                    current_rec = Record::new();
                    current_rec.set_header(&line);
                    active_record = true;
                }
                Some(_) => {
                    if active_record {
                        current_rec.sequence.push_str(&line);
                    }
                }
            }
        }
        // deal with final record at EOF
        if active_record {
            records.push(current_rec);
        }
        Ok(records)
    }
}

fn set_header(rec: &mut Record, s: &str) {
    let mut parts = s[1..].splitn(2, char::is_whitespace);
    rec.id = parts.next().unwrap_or("").to_owned();
    rec.description = parts.next().unwrap_or("").to_owned();
}

fn get_record_factory(rdr: impl BufRead) -> impl FnMut() -> Option<Record> {
    let mut lineiter = rdr.lines().filter_map(Result::ok).peekable();
    move || {
        let mut rec: Record = Record::new();
        let mut active_record = false;
        loop {
            match lineiter.peek() {
                // match on first non whitespace character
                Some(s) => match s.trim().chars().next() {
                    Some('>') if active_record => return Some(rec),
                    Some('>') => {
                        rec = Record::new();
                        let header = lineiter.next().unwrap().trim().to_owned();
                        set_header(&mut rec, &header);
                        active_record = true;
                        continue;
                    }
                    Some(_) if active_record => {
                        rec.sequence
                            .push_str(&lineiter.next().unwrap().trim().to_owned());
                        continue;
                    }
                    _ => {
                        lineiter.next();
                        continue;
                    }
                },
                None if active_record => return Some(rec),
                None => return None,
            }
        }
    }
}

// fn get_record_factory2(rdr: impl BufRead) -> impl FnMut() -> Option<Record> {
//     let mut lineiter = rdr.lines().filter_map(Result::ok);
//     let mut stack: Vec<Record> = Vec::new();
//     move || {
//         let mut rec: Record = Record::new();
//         let mut active_record = false;
//         if !stack.is_empty() {
//             rec = stack.pop().unwrap();
//             active_record = true;
//         }
//         loop {
//             let line = lineiter.next();
//             match line {
//                 // match on first non whitespace character
//                 Some(l) => match l.trim().chars().next() {
//                     Some('>') if active_record => {
//                         let tmprec = rec;
//                         rec = Record::new();
//                         let header = l.trim().to_owned();
//                         set_header(&mut rec, &header);
//                         stack.push(rec);
//                         return Some(tmprec);
//                     }
//                     Some('>') => {
//                         rec = Record::new();
//                         let header = l.trim().to_owned();
//                         set_header(&mut rec, &header);
//                         active_record = true;
//                         continue;
//                     }
//                     Some(_) if active_record => {
//                         rec.sequence.push_str(&l.trim().to_owned());
//                         continue;
//                     }
//                     _ => continue,
//                 },
//                 None if active_record => return Some(rec),
//                 None => return None,
//             }
//         }
//     }
// }

/// PeekableLines is an iterator like object over the lines of any type
/// implementing the BufRead trait.
///
/// PeekableLines implements two public functions
/// 1. peekline -- returns the next line w/out advancing the iterator
/// 2. advanceline -- advances the iterator
///
pub struct PeekableLines<B: BufRead> {
    iter: Peekable<Lines<B>>,
}

impl<B: BufRead> PeekableLines<B> {
    pub fn peekline(&mut self) -> Option<&'_ Result<String, io::Error>> {
        self.iter.peek()
    }

    pub fn advanceline(&mut self) -> Option<Result<String, io::Error>> {
        self.iter.next()
    }
}

impl<B: BufRead> From<B> for PeekableLines<B> {
    fn from(buf: B) -> PeekableLines<B> {
        PeekableLines {
            iter: buf.lines().peekable(),
        }
    }
}

pub struct FastaBuffer<B: BufRead>(PeekableLines<B>);

impl<B: BufRead> FastaBuffer<B> {
    pub fn from(b: B) -> FastaBuffer<B> {
        FastaBuffer(PeekableLines::from(b))
    }
}

impl<B: BufRead> Iterator for FastaBuffer<B> {
    type Item = Result<Record, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut active_record = false;
        let mut rec = Record::new();

        while let Some(nextline) = self.0.peekline() {
            let nextline = match nextline {
                Ok(line) => line.trim(),
                Err(e) => {
                    return Some(Err(io::Error::new(
                        e.kind(),
                        "IO error while parsing Fasta records.",
                    )));
                }
            };
            match nextline.chars().next() {
                None | Some(';') => (),
                Some('>') if active_record => {
                    return Some(Ok(rec));
                }
                Some('>') => {
                    active_record = true;
                    rec.set_header(&nextline);
                }
                Some(_) if active_record => rec.sequence.push_str(nextline),
                _ => (),
            }
            self.0.advanceline();
        }
        if active_record {
            Some(Ok(rec))
        } else {
            None
        }
    }
}

pub fn next_record<B: BufRead>(itr: &mut PeekableLines<B>) -> Option<Record> {
    let mut active_record = false;
    let mut rec = Record::new();

    while let Some(nextline) = itr.peekline() {
        let nextline = match nextline {
            Ok(line) => line.trim(),
            Err(_) => {
                return None;
            }
        };
        match nextline.chars().next() {
            None | Some(';') => (),
            Some('>') if active_record => {
                return Some(rec);
            }
            Some('>') => {
                active_record = true;
                set_header(&mut rec, &nextline);
            }
            Some(_) if active_record => rec.sequence.push_str(nextline),
            _ => (),
        }
        itr.advanceline();
    }
    if active_record {
        Some(rec)
    } else {
        None
    }
}
