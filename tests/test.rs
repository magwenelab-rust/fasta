use fasta;

#[test]
fn new_record() {
    let r = fasta::Record::new();
    assert_eq!(r.id.as_str(), "");
    assert_eq!(r.description.as_str(), "");
}
