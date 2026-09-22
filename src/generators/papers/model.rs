use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    pub source_windows: BTreeSet<u64>,
}

pub fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

pub fn fingerprint(text: &str) -> u64 {
    text.bytes().fold(5381_u64, |hash, byte| {
        hash.wrapping_mul(33).wrapping_add(byte as u64)
    })
}
