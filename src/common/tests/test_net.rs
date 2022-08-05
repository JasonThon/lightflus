use common::net::local_ip;

#[test]
fn test_local_ip() {
    let option = local_ip();
    assert!(option.is_some());
    println!("{}", option.unwrap())
}
