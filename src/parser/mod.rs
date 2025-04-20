pub mod python;

pub trait Test {
    fn runtime_argument(self) -> String;
    fn search_item_name(&self) -> String;
}

pub trait Tests {
    fn to_json(&self) -> String;
    fn tests(self) -> Vec<impl Test>;
    fn update(&mut self, only_check_for_update: bool) -> bool;
}
