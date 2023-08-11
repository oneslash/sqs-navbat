use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ParamValues {
    pub name: String,
    pub value: String,
}

pub fn populate_attributes(extra: HashMap<String, String>) -> Option<Vec<ParamValues>> {
    // Let's live dangerously and unwrap this
    let re = Regex::new(r"\.(\d+)\.(.+)$").unwrap();
    let mut attrs: Vec<ParamValues> = Vec::new();
    for _i in 0..extra.len() {
        attrs.push(ParamValues {
            name: "".to_string(),
            value: "".to_string(),
        });
    }

    for (key, value) in extra.iter() {
        if let Some(caps) = re.captures(key) {
            let index = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
            let attr_name = caps.get(2).unwrap().as_str().to_string();

            match attr_name.as_str() {
                "Name" => attrs[index - 1].name = value.to_string(),
                "Value" => attrs[index - 1].value = value.to_string(),
                _ => (),
            }
        }
    }

    Some(attrs)
}

pub fn get_attrbutes_hashmap(attributes: Option<Vec<ParamValues>>) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Some(attrs) = attributes {
        for attr in attrs {
            map.insert(attr.name, attr.value);
        }
    }

    return map;
}
