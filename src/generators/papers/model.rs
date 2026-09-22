use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CATEGORIES: [&str; 8] = [
    "cs", "math", "physics", "stat", "q-bio", "q-fin", "econ", "eess",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct TextModel {
    pub revision: String,
    pub categories: BTreeMap<String, CategoryModel>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CategoryModel {
    pub starts: Vec<String>,
    pub transitions: BTreeMap<String, Vec<(String, u32)>>,
}

pub fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}
