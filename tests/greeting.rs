//! Integration tests for [`ninice::greeting`].

#[test]
fn returns_hello_world() {
    assert_eq!(ninice::greeting(), "Hello, world!");
}
