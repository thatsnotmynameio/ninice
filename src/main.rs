//! Binary entry point. Currently a domain stub; real entry point
//! (CLI/HTTP/worker) is decided in a later iteration.

fn main() {
    println!("ninice {}", env!("CARGO_PKG_VERSION"));
}
