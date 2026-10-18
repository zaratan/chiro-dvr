use std::fs::File;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let big: u64 = args[2].parse().unwrap();
    let mut file = File::create(path).unwrap();
    let entries = [("092/extraits/piste_01.mp4", 1_500_000), ("092/params.json", 4_000), ("092/big.bin", big), ("resultats.html", 300_000)];
    let (sent, largest) = zipstream::archive(&entries, 65_536, |bytes| file.write_all(bytes).unwrap()).unwrap();
    println!("sent {sent} largest write {largest}");
}
