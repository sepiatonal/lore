use goblin;
use stillwind;

pub fn main() {
    // goblin::api::test_engine();
    let src = std::fs::read_to_string("test.sw").unwrap();
    println!("{:?}", stillwind::parse(&src));
}