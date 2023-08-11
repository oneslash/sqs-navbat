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
    
    // Cleanup empty attributes
    attrs.retain(|attr| attr.name != "");

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    /// test populate_attributes
    #[test]
    fn test_populate_attributes() {
        let mut extra = HashMap::new();
        extra.insert("Attribute.1.Name".to_string(), "DelaySeconds".to_string());
        extra.insert("Attribute.1.Value".to_string(), "10".to_string());
        extra.insert("Attribute.2.Name".to_string(), "MaximumMessageSize".to_string());
        extra.insert("Attribute.2.Value".to_string(), "262144".to_string());

        let attrs = super::populate_attributes(extra);
        assert!(attrs.is_some());
        let attrs = attrs.unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].name, "DelaySeconds");
        assert_eq!(attrs[0].value, "10");
        assert_eq!(attrs[1].name, "MaximumMessageSize");
        assert_eq!(attrs[1].value, "262144");
    }
}

