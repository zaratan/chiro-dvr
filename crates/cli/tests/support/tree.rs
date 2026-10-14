use std::fs;
use std::path::Path;

#[cfg(test)]
pub(crate) fn tree(folder: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for entry in fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        if entry.file_type().unwrap().is_dir() {
            names.extend(tree(&entry.path()).into_iter().map(|inner| format!("{name}/{inner}")));
            names.push(format!("{name}/"));
        } else {
            names.push(name);
        }
    }
    names.sort();
    names
}
