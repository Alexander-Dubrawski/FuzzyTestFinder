use crate::cache::types::CacheEntry;
mod python;

pub trait Parser {
    fn parse_test(&self) -> CacheEntry;
    fn update_tests(&self, cache_entry: &mut CacheEntry) -> bool;
}

pub trait Test {
    fn runtime_argument(self) -> String;
    fn name(&self) -> String;
}

pub trait Tests {
    fn to_json(&self) -> String;
    fn tests(self) -> Vec<impl Test>;
}
