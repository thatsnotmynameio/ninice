//! ninice — TODO crate description.

fn main() {
    println!("{}", greeting());
}

fn greeting() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_returns_hello_world() {
        assert_eq!(greeting(), "Hello, world!");
    }
}
