use std::{
    collections::{HashMap, HashSet},
    fs,
};

use rustfix::{CodeFix, Filter, get_suggestions_from_json};

fn main() {
    for diagnostics_path in std::env::args().skip(1) {
        let json =
            fs::read_to_string(&diagnostics_path).unwrap_or_else(|e| panic!("failed to read {diagnostics_path}: {e}"));

        let diagnostics: String = json
            .lines()
            .filter(|line| line.contains(r#""$message_type":"diagnostic""#))
            .collect::<Vec<_>>()
            .join("\n");

        if diagnostics.is_empty() {
            continue;
        }

        let suggestions =
            get_suggestions_from_json(&diagnostics, &HashSet::<String>::new(), Filter::MachineApplicableOnly)
                .unwrap_or_else(|e| panic!("failed to parse {diagnostics_path}: {e}"));

        let mut files: HashMap<String, Vec<_>> = HashMap::new();
        for suggestion in suggestions {
            let file = suggestion.solutions[0].replacements[0].snippet.file_name.clone();
            files.entry(file).or_default().push(suggestion);
        }

        for (source_file, suggestions) in &files {
            let source =
                fs::read_to_string(source_file).unwrap_or_else(|e| panic!("failed to read {source_file}: {e}"));
            let mut fix = CodeFix::new(&source);
            for suggestion in suggestions.iter().rev() {
                if let Err(e) = fix.apply(suggestion) {
                    eprintln!("skipping suggestion in {source_file}: {e}");
                }
            }
            fs::write(source_file, fix.finish().expect("failed to apply fixes"))
                .unwrap_or_else(|e| panic!("failed to write {source_file}: {e}"));
        }
    }
}
