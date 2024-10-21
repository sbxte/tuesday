use std::io::{Read, Write};

use anyhow::{bail, Result};
use serde::Serialize;
use yaml_rust2::{YamlEmitter, YamlLoader};

use super::utils;

pub fn save_config<T>(name: &str, config: T) -> Result<()>
where
    T: Serialize,
{
    // Read from existing configs
    let save = utils::get_save_default(".tueconf");
    if save.is_none() {
        bail!("Config file not found!");
    }
    let mut save = save.unwrap();
    let mut content = String::new();
    save.read_to_string(&mut content)?;

    let mut yaml = YamlLoader::load_from_str(&content)?;
    let yaml = &mut yaml[0];

    // Merge new config into configs
    let selected_yaml = &mut yaml[name];
    let new_yaml = YamlLoader::load_from_str(&serde_yaml_ng::to_string(&config)?)?
        .first()
        .unwrap()
        .clone(); // Lord forgive me for what I have done here
    *selected_yaml = new_yaml;

    // Write it back into configs
    save.set_len(0)?;
    let mut s = String::new();
    YamlEmitter::new(&mut s).dump(yaml).unwrap();
    save.write_all(s.as_bytes())?;

    Ok(())
}
