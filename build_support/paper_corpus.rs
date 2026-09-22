use std::{collections::BTreeMap, fs, path::Path, process::Command};

#[path = "../src/generators/papers/model.rs"]
mod model;
use model::{CATEGORIES, CategoryModel, TextModel, words};

pub fn compile(out_dir: &Path) {
    for path in [
        "corpus/papers/manifest.json",
        "scripts/prepare-paper-corpus.py",
        "build_support/paper_corpus.rs",
        "src/generators/papers/model.rs",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    let status = Command::new("python3")
        .arg("scripts/prepare-paper-corpus.py")
        .status()
        .expect("Paper corpus preparation requires Python 3 on the build machine");
    assert!(
        status.success(),
        "Paper corpus preparation failed; see extraction/checksum error above"
    );
    let manifest_text =
        fs::read_to_string("corpus/papers/manifest.json").expect("Missing paper manifest");
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest_text).expect("Invalid manifest");
    let mut compiled = TextModel {
        revision: manifest["revision"]
            .as_str()
            .expect("Missing corpus revision")
            .to_owned(),
        categories: BTreeMap::new(),
    };
    assert_eq!(
        compiled.revision, "v1",
        "Update the paper generator for a new corpus revision"
    );
    let entries = manifest["papers"].as_array().expect("Missing papers");
    for category in CATEGORIES {
        let mut model = CategoryModel::default();
        let mut transitions: BTreeMap<String, BTreeMap<String, u32>> = BTreeMap::new();
        let sources: Vec<_> = entries
            .iter()
            .filter(|entry| entry["category"] == category)
            .collect();
        assert!(
            sources.len() >= 2,
            "Need at least two reviewed sources for {category}"
        );
        for source in sources {
            let path = format!(
                "target/sinkland-paper-corpus/{}.txt",
                source["id"].as_str().unwrap()
            );
            println!("cargo:rerun-if-changed={path}");
            let text = fs::read_to_string(path).expect("Prepared source is missing");
            for sentence in text.split(['.', '!', '?']) {
                let tokens = words(sentence);
                if tokens.len() < 5 {
                    continue;
                }
                model.starts.push(tokens[..2].join(" "));
                for window in tokens.windows(3) {
                    *transitions
                        .entry(window[..2].join(" "))
                        .or_default()
                        .entry(window[2].clone())
                        .or_default() += 1;
                }
            }
        }
        model.starts.sort();
        model.starts.dedup();
        model.transitions = transitions
            .into_iter()
            .map(|(key, successors)| (key, successors.into_iter().collect()))
            .collect();
        assert!(
            !model.starts.is_empty() && !model.transitions.is_empty(),
            "Empty model for {category}"
        );
        compiled.categories.insert(category.to_owned(), model);
    }
    fs::write(
        out_dir.join("paper-model.json"),
        serde_json::to_vec(&compiled).unwrap(),
    )
    .unwrap();
}
