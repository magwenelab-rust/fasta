pub mod errors;

use std::fmt;
use std::io;
use std::io::BufRead;
use std::io::Lines;
use std::io::Write;
use std::iter::Peekable;

/*----------------------------------------------------------------------------*/

fn wrap_string(s: &str, w: usize) -> String {
    let mut result = String::new();

    let mut ctr = 0;
    for i in (0..(s.len() - w)).step_by(w) {
        result.push_str(&s[i..(i + w)]);
        result.push('\n');
        ctr = i;
    }
    result.push_str(&s[(ctr + w)..]);

    result
}

#[derive(Debug, Default)]
/// fasta::Record represents a single FASTA record
pub struct Record {
    pub id: String,
    pub description: String,
    pub sequence: String,
}

impl Record {
    /// Returns a new fasta::Record with appropriate default fields
    pub fn new() -> Record {
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

    /// Generate a String representation of a fasta::Record
    pub fn as_string(&self) -> String {
        let wrappedseq = wrap_string(&self.sequence, 80);
        let result = format!(">{} {}\n{}\n", self.id, self.description, wrappedseq);
        result
    }

    /// Write a fasta::Record to an object implementing Write
    pub fn write(&mut self, w: &mut impl Write) -> std::io::Result<()> {
        w.write_all(self.as_string().as_bytes())
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

/// PeekableLines is an iterator like object over the lines of any type
/// implementing the BufRead trait.
///
/// PeekableLines implements two public functions
/// 1. peekline -- returns the next line w/out advancing the iterator
/// 2. advanceline -- advances the iterator
///
struct PeekableLines<B: BufRead> {
    iter: Peekable<Lines<B>>,
}

impl<B: BufRead> PeekableLines<B> {
    /// Peek at the next line in the buffer, w/out advancing the iterator
    pub fn peekline(&mut self) -> Option<&'_ Result<String, io::Error>> {
        self.iter.peek()
    }

    /// Return the next line in the buffer, advancing the iterator
    pub fn advanceline(&mut self) -> Option<Result<String, io::Error>> {
        self.iter.next()
    }
}

impl<B: BufRead> From<B> for PeekableLines<B> {
    /// Convert an object implement BufRead to a PeekableLines
    fn from(buf: B) -> PeekableLines<B> {
        PeekableLines {
            iter: buf.lines().peekable(),
        }
    }
}

/// FastaBuffer is the public interface for working
/// with FASTA records in an iterator like manner
pub struct FastaBuffer<B: BufRead>(PeekableLines<B>);

impl<B: BufRead> FastaBuffer<B> {
    /// Create a FastaBuffer from instance that implements BufRead
    pub fn from(b: B) -> FastaBuffer<B> {
        FastaBuffer(PeekableLines::from(b))
    }
}

/// An iterator that returns FASTA records from a FastaBuffer
impl<B: BufRead> Iterator for FastaBuffer<B> {
    type Item = Result<Record, io::Error>;

    /// Return the next FASTA record
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

#[cfg(test)]
mod tests {

    #[test]
    fn wrap_str_test() {
        let s = "hello world how are you today?";
        let ws = super::wrap_string(&s, 14);
        println!("{}", ws);
    }
}
