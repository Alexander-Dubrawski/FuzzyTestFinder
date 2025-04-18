use crate::cache::types::CacheEntry;

pub trait Parser {
    fn parse_test(&self) -> CacheEntry;
    fn update_tests(&self, cache_entry: &mut CacheEntry) -> bool;
}
