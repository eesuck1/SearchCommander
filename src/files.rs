use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use lopdf::Document;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File
{
    path: PathBuf,
    counts: HashMap<String, usize>,
}

impl Display for File
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        Ok(write!(f, "Path: {}\n", self.path.display())?)
    }
}

impl File
{
    pub fn new(path: &Path) -> Result<Self>
    {
        let mut file = Self::build(path)?;
        file.count().expect("Error while counting File");

        Ok(file)
    }

    fn build(path: &Path) -> Result<Self>
    {
        Ok(File { path: path.to_owned(), counts: HashMap::new() })
    }

    fn count(&mut self) -> Result<()>
    {
        let counts = Arc::new(DashMap::new());

        self.read().expect("Error while reading file (in count function)")
            .par_iter()
            .for_each(|word|
                {
                    counts.entry(word.to_string()).and_modify(|count| *count += 1).or_insert(1);
                });

        self.counts = Arc::try_unwrap(counts)
            .expect("Failed to unwrap Count Hash Map")
            .into_iter()
            .collect();

        Ok(())
    }
    fn read(&self) -> Result<Vec<String>>
    {
        if self.path.extension().unwrap_or_default() == "pdf"
        {
            let document = Document::load(&self.path).unwrap();

            let content: String = document
                .get_pages()
                .par_iter()
                .map(|(&number, _)|
                    {
                        let number = number;

                        document.extract_text(&[number]).unwrap_or_default()
                    })
                .collect();

            let splitted: Vec<String> = content
                .split_whitespace()
                .filter_map(|word|
                    {
                        match word.chars().all(char::is_alphanumeric)
                        {
                            true => Some(word.to_lowercase()),
                            false => None,
                        }
                    })
                .collect();

            return Ok(splitted);
        }

        let content: Vec<String> = read_to_string(&self.path).expect("Failed to read file")
            .split_whitespace()
            .filter_map(|word|
                {
                    match word.chars().all(char::is_alphanumeric)
                    {
                        true => Some(word.to_lowercase()),
                        false => None,
                    }
                })
            .collect();

        Ok(content)
    }
}

#[derive(Debug)]
struct Files
{
    files: Vec<File>,
    dictionary: HashMap<String, usize>,
}

impl Display for Files
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        self.files
            .iter()
            .for_each(|file| write!(f, "{}", file).expect("Failed to print Files"));


        Ok(())
    }
}

impl Files
{
    pub fn new(root: &str) -> Result<Self>
    {
        let mut result = Self::build()?;

        WalkDir::new(root)
            .into_iter()
            .for_each(|entry|
                {
                    let binding = entry.unwrap();
                    let path = binding.path();

                    if path.is_file()
                    {
                        result.files.push(File::new(path).expect("Failed to read File"))
                    }
                });

        result.dictionary = result.files
            .iter()
            .flat_map(|file| file.counts.iter())
            .map(|(&ref word, &counts)| (word.clone(), counts))
            .collect();

        Ok(result)
    }

    fn build() -> Result<Self>
    {
        Ok(Files { files: Vec::new(), dictionary: HashMap::new() })
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_print() -> Result<()>
    {
        let file_path = "Test\\Category.pdf";
        let path = Path::new(&file_path);
        let file = File::new(&path)?;

        println!("{}", file);
        println!("{:?}", file.counts);

        Ok(())
    }

    #[test]
    fn test_files() -> Result<()>
    {
        let root = "Test\\";
        let files = Files::new(root)?;

        println!("{}", files);
        println!("{:?}", files.dictionary);

        Ok(())
    }
}