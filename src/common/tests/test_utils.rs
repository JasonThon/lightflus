#[test]
fn test_serde_env() {
    let origin = "{\"name\":\"${your.name}\", \"card\": \"${your.card}\"}";
    std::env::set_var("your.name", "jason");
    std::env::set_var("your.card", "111");
    let target = common::utils::from_str(origin);
    let result = serde_json::from_str::<Name>(target.as_str());
    if result.is_err() {
        print!("{:?}", result.as_ref().unwrap_err())
    }
    assert!(result.is_ok());
    let name = result.unwrap();
    assert_eq!(&name.name, &"jason".to_string());
    assert_eq!(&name.card, &"111".to_string());
}

#[test]
fn test_env_var_get() {
    std::env::set_var("your.name", "jason");
    let result = std::env::var("your.name".to_string());
    assert!(result.is_ok());
    assert_eq!(result.as_ref().unwrap(), &"jason".to_string())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Name {
    name: String,
    card: String,
}
