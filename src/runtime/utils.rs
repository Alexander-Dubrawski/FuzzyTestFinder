use crate::utils::process::OutputFormatter;

use super::engine::TestItem;

pub fn partition_tests<F: OutputFormatter + Clone + Sync + Send>(
    vec: Vec<TestItem<F>>,
    m: usize,
) -> Vec<Vec<TestItem<F>>> {
    let n = vec.len();
    if m == 0 {
        return Vec::new();
    }
    if n == 0 {
        return vec![];
    }
    let mut partitions = vec![vec![]; m];

    for (i, item) in vec.into_iter().enumerate() {
        partitions[i % m].push(item);
    }

    partitions.into_iter().filter(|p| !p.is_empty()).collect()
}
