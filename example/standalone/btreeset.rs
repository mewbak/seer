use std::collections::BTreeSet;

fn main() {
    use std::io::Read;
    let mut data = [0; 17];
    let mut stdin = ::std::io::stdin();
    stdin.read_exact(&mut data[..]).unwrap();

    let mut heap = BTreeSet::new();
    for &d in data.iter() {
        heap.insert(d);
    }

    let mut floor = 0;
    for &d in heap.iter() {
        if d < floor {
            panic!()
        }
        floor = d;
    }
}
