//! Collection utilities

use std::collections::HashMap;

/// Merge two HashMaps (b overwrites a)
pub fn merge_maps<K: Eq + std::hash::Hash + Clone, V: Clone>(a: &mut HashMap<K, V>, b: HashMap<K, V>) {
    for (k, v) in b {
        a.insert(k, v);
    }
}

/// Get or insert with default
pub fn get_or_insert<K: Eq + std::hash::Hash + Clone, V: Clone>(
    map: &mut HashMap<K, V>,
    key: K,
    default: V,
) -> V {
    map.entry(key).or_insert(default).clone()
}

/// Filter map by predicate
pub fn filter_map<K: Eq + std::hash::Hash + Clone, V: Clone>(
    map: &HashMap<K, V>,
    predicate: impl Fn(&K, &V) -> bool,
) -> HashMap<K, V> {
    map.iter()
        .filter(|(k, v)| predicate(k, v))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Map values
pub fn map_values<K: Eq + std::hash::Hash + Clone, V: Clone, U>(
    map: &HashMap<K, V>,
    f: impl Fn(&V) -> U,
) -> HashMap<K, U> {
    map.iter().map(|(k, v)| (k.clone(), f(v))).collect()
}

/// Group by key
pub fn group_by<T, K: Eq + std::hash::Hash + Clone>(
    items: Vec<T>,
    key_fn: impl Fn(&T) -> K,
) -> HashMap<K, Vec<T>> {
    let mut map = HashMap::new();
    for item in items {
        let key = key_fn(&item);
        map.entry(key).or_default().push(item);
    }
    map
}

/// Partition vec by predicate
pub fn partition<T>(items: Vec<T>, predicate: impl Fn(&T) -> bool) -> (Vec<T>, Vec<T>) {
    let mut pass = Vec::new();
    let mut fail = Vec::new();
    for item in items {
        if predicate(&item) {
            pass.push(item);
        } else {
            fail.push(item);
        }
    }
    (pass, fail)
}

/// Unique elements (preserves order)
pub fn unique<T: Eq + std::hash::Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

/// Chunk vec into chunks of size n
pub fn chunks<T>(items: Vec<T>, chunk_size: usize) -> Vec<Vec<T>> {
    items.chunks(chunk_size).map(|c| c.to_vec()).collect()
}

/// Flatten vec of vecs
pub fn flatten<T>(items: Vec<Vec<T>>) -> Vec<T> {
    items.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_merge_maps() {
        let mut a = HashMap::new();
        a.insert("a", 1);
        let mut b = HashMap::new();
        b.insert("b", 2);
        b.insert("a", 3);
        merge_maps(&mut a, b);
        assert_eq!(a.get("a"), Some(&3));
        assert_eq!(a.get("b"), Some(&2));
    }

    #[test]
    fn test_unique() {
        assert_eq!(unique(vec![1, 2, 2, 3, 3, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn test_chunks() {
        assert_eq!(chunks(vec![1, 2, 3, 4, 5], 2), vec![vec![1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn test_partition() {
        let (even, odd) = partition(vec![1, 2, 3, 4, 5], |x| x % 2 == 0);
        assert_eq!(even, vec![2, 4]);
        assert_eq!(odd, vec![1, 3, 5]);
    }
}
