pub fn partition_tests(vec: &[String], m: usize) -> Vec<Vec<String>> {
    let n = vec.len();
    if m == 0 {
        return Vec::new();
    }
    if n == 0 {
        return vec![];
    }
    let mut partitions = vec![vec![]; m];

    for (i, item) in vec.iter().enumerate() {
        partitions[i % m].push(item.clone());
    }

    partitions.into_iter().filter(|p| !p.is_empty()).collect()
}
