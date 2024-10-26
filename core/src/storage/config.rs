use std::collections::HashMap;
use std::fmt::Debug;

pub trait Config: Debug {
    //
    // GETTERS
    //

    fn get_bool(&self, setting: &str) -> Option<bool>;

    fn get_i64(&self, setting: &str) -> Option<i64>;

    fn get_f64(&self, setting: &str) -> Option<f64>;

    fn get_str(&self, setting: &str) -> Option<&str>;

    fn get_vec_str(&self, setting: &str) -> Option<&Vec<&str>>;

    fn get_vec_str_mut(&mut self, setting: &str) -> Option<&mut Vec<&str>>;

    //
    // SETTERS
    //

    /// Returns true if the setting existed and modifying it was successful
    fn set_bool(&mut self, setting: &str, value: bool) -> bool;

    /// Returns true if the setting existed and modifying it was successful
    fn set_i64(&mut self, setting: &str, value: i64) -> bool;

    /// Returns true if the setting existed and modifying it was successful
    fn set_f64(&mut self, setting: &str, value: f64) -> bool;

    /// Returns true if the setting existed and modifying it was successful
    fn set_str(&mut self, setting: &str, value: &str) -> bool;
}

pub struct ConfigStorage {
    data: HashMap<String, Box<dyn Config>>,
}

impl ConfigStorage {
    pub fn get(&self, config_group: &str) -> Option<&dyn Config> {
        self.data.get(config_group).map(|c| c.as_ref())
    }

    pub fn get_mut<'a>(&'a mut self, config_group: &str) -> Option<&'a mut (dyn Config + 'a)> {
        self.data
            .get_mut(config_group)
            .map::<&'a mut dyn Config, _>(|c| c.as_mut())
    }
}
