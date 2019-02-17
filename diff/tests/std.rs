#[macro_use]
mod common;

debug_equivalence! {
    vec => vec![true; 10];
    btreeset => {
        let mut set = std::collections::BTreeSet::new();
        set.insert(12u32);
        set.insert(1234);
        set
    };
    btreemap => {
        let mut map = std::collections::BTreeMap::new();
        map.insert(12u32, "hello");
        map.insert(1234, "there");
        map
    };
    hashset => {
        let mut set = std::collections::HashSet::new();
        set.insert(12u32);
        set.insert(1234);
        set
    };
    hashmap => {
        let mut map = std::collections::HashMap::new();
        map.insert(12u32, "hello");
        map.insert(1234, "there");
        map
    };
}
