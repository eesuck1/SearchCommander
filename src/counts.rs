use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Mutex, Arc};

use crate::files::Files;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

const COUNTS_JSON: &str = "JSON\\counts.json";
const ROOT_JSON: &str = "JSON\\root.json";

#[derive(Debug, Serialize, Deserialize)]
struct Root
{
    root: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Count
{
    root: String,
    pub files: Files,
    pub overall_counts: HashMap<String, usize>,
    pub in_file_counts: HashMap<PathBuf, HashMap<String, usize>>,
}

impl Drop for Count
{
    fn drop(&mut self)
    {
        let mut counts_root_file = File::open(ROOT_JSON).unwrap();
        let mut counts_root_string = String::new();

        counts_root_file.read_to_string(&mut counts_root_string).unwrap();
        let mut counts_root: Root = serde_json::from_str(&counts_root_string).unwrap();

        if counts_root.root != self.root
        {
            let mut json_string = serde_json::to_string(&self).unwrap();
            std::fs::write(COUNTS_JSON, json_string).expect("Failed to Write a JSON");

            counts_root.root = self.root.clone();

            json_string = serde_json::to_string(&counts_root).unwrap();
            std::fs::write(ROOT_JSON, json_string).expect("Failed to Write a JSON");
        }
    }
}

impl Count
{
    pub fn new(root: &str) -> Count
    {
        let mut counts_root_file = File::open(ROOT_JSON).unwrap();
        let mut counts_root_string = String::new();

        counts_root_file.read_to_string(&mut counts_root_string).unwrap();
        let counts_root: Root = serde_json::from_str(&counts_root_string).unwrap();

        if counts_root.root != root
        {
            let mut counts = Count { root: root.to_owned(), files: Files::new(root), overall_counts: HashMap::new(), in_file_counts: HashMap::new() };

            counts.overall_counts = counts.count_overall().unwrap();
            counts.in_file_counts = counts.in_files_count().unwrap();

            return counts
        }

        let mut counts = File::open(COUNTS_JSON).unwrap();
        let mut counts_string = String::new();

        counts.read_to_string(&mut counts_string).unwrap();
        let counts: Count = serde_json::from_str(&counts_string).unwrap();

        counts
    }

    fn count_overall(&self) -> io::Result<HashMap<String, usize>>
    {
        let overall_count = Mutex::new(HashMap::new());

        self.files.files
            .par_iter()
            .for_each(|(_, content)|
            {
            content
                .par_iter()
                .for_each(|word|
                {
                    let mut count_map = overall_count.lock().unwrap();
                    *count_map.entry(word.to_owned()).or_insert(0) += 1;
                });
            });

        let result = overall_count.lock().unwrap().clone();

        Ok(result)
    }

    pub fn in_files_count(&self) -> io::Result<HashMap<PathBuf, HashMap<String, usize>>>
    {
        let in_files_count = Arc::new(Mutex::new(HashMap::new()));

        self.files.files
            .par_iter()
            .for_each(|(path, content)|
            {
                let mut in_file_count = HashMap::new();

                content
                    .iter()
                    .for_each(|word|
                    {
                        *in_file_count.entry(word.to_owned()).or_insert(0) += 1;
                    });

                in_files_count.lock().unwrap().insert(path.to_owned(), in_file_count);
            });

        let result = Arc::try_unwrap(in_files_count)
            .unwrap()
            .into_inner()
            .unwrap();

        Ok(result)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_overall_counts() -> io::Result<()>
    {
        let root = "Test\\";
        let counts = Count::new(root);

        counts.overall_counts.
            par_iter()
            .for_each(|(word, count)| println!("{}: {}", word, count));

        Ok(())
    }

    #[test]
    fn test_in_files_count() -> io::Result<()>
    {
        let root = "Test\\";
        let counts = Count::new(root);

        counts.in_file_counts
            .iter()
            .for_each(|(path, content)|
                {
                    println!("{}:", path.display());
                    content.
                        par_iter()
                        .for_each(|(word, count)| println!("{}: {}", word, count))
                });

        Ok(())
    }
}