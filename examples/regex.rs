use regex::{Captures, Regex};
use std::{collections::HashSet, error::Error, fs, path::PathBuf};

struct IncludeInfo {
    dir: PathBuf,
    included: HashSet<String>,
    depth: u32,
}

impl IncludeInfo {
    pub fn new(entryfile: &str) -> Self {
        let mut dir = PathBuf::from(entryfile);
        dir.pop();
        Self {
            dir,
            included: HashSet::new(),
            depth: 0,
        }
    }
}

fn handle_library(capture: &Captures, libs: &[&str]) {
    let library = &capture[1];
    if !libs.contains(&library) {
        panic!("Unknown library '{}' in macro {}", library, &capture[0]);
    }
    println!("Adding library: {}", library)
}

fn handle_file_import(capture: &Captures, info: &mut IncludeInfo, output: &mut String) {
    info.depth += 1;
    let filename = &capture[1]
        .strip_prefix(r#"""#)
        .unwrap()
        .strip_suffix(r#"""#)
        .unwrap();
    let mut path = PathBuf::from(&info.dir);
    path.push(filename);
    println!(
        "Pasting contents from filename: {} {}",
        path.to_str().unwrap(),
        &capture[0]
    );

    if !info.included.insert(capture[1].to_string()) {
        let contents = fs::read_to_string(&path).unwrap();
        *output = output.replacen(&capture[0], contents.as_str(), 1);
    } else {
        *output = output.replacen(&capture[0], "", 1);
    };
}

fn handle_replace(capture: &Captures, output: &mut String) {
    let c = capture;
    println!("Replacing {}: {} => {}", &c[0], &c[1], &c[2]);
    *output = output.replace(&c[1], &c[2]);
}

fn preprocess() -> Result<(), Box<dyn Error>> {
    let entryfile = "imp/unit.imp";

    let contents = fs::read_to_string(entryfile).unwrap();
    let mut output = contents.clone();

    let libs = vec!["stdin"];
    let mut include_tracker = IncludeInfo::new(entryfile);

    let uselib_pattern = Regex::new(r#"#\(import\s+(?P<library>[^"]\S+[^"])\s*\)"#)?;
    let include_pattern = Regex::new(r#"#\(import\s+(?P<filename>"\S+")\s*\)"#)?;
    let var_replace_pattern = Regex::new(r#"#\(replace\s+('\S+)\s+(\S+)\)"#)?;
    let fn_replace_pattern = Regex::new(r#"#\(replace\s+(\('\S+)\s+(\S+\))\s+(.*)\)"#)?;

    output = uselib_pattern.replace_all(&output, "").to_string();
    output = var_replace_pattern.replace_all(&output, "").to_string();
    output = fn_replace_pattern.replace_all(&output, "").to_string();

    uselib_pattern
        .captures_iter(&contents)
        .for_each(|e| handle_library(&e, &libs));

    include_pattern
        .captures_iter(&contents)
        .for_each(|e| handle_file_import(&e, &mut include_tracker, &mut output));

    var_replace_pattern
        .captures_iter(&contents)
        .for_each(|e| handle_replace(&e, &mut output));

    for c in fn_replace_pattern.captures_iter(&contents) {
        println!(
            "Captured from {}: {} ({}) => {}",
            &c[0], &c[1], &c[2], &c[3]
        );
    }

    println!("FINAL RESULT\n{}", output);

    Ok(())
}

fn main() {
    preprocess().unwrap();
}
