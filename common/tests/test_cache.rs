use std::thread;

#[test]
fn test_concurrent_cache() {
    use common::collections::ConcurrentCache;
    use std::default::Default;
    let cache: ConcurrentCache<String, String> = Default::default();

    cache.put("john".to_string(), "home".to_string());
    let old_value = cache.put("john".to_string(), "school".to_string());
    assert!(old_value.is_some());
    assert_eq!(old_value.unwrap(), "home".to_string());
    let new_value = cache.get(&"john".to_string());
    assert!(new_value.is_some());
    assert_eq!(new_value.unwrap(), "school".to_string());
}