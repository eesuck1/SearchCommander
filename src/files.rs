use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{Read, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use lopdf::Document;
use rayon::prelude::*;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct File
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

    fn get_count(&self, word: &str) -> &usize
    {
        self.counts.get(word).unwrap_or(&0)
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

        let content: Vec<String> = fs::read_to_string(&self.path).expect("Failed to read file")
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Files
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
                        match File::new(path)
                        {
                            Ok(file) => result.files.push(file),
                            Err(error) => println!("{} occurred while opening - `{}`\nFiles skipped", error, path.display()),
                        }
                    }
                });

        result.dictionary = result.files
            .iter()
            .flat_map(|file| file.counts.iter())
            .map(|(&ref word, &counts)| (word.clone(), counts))
            .collect();

        Ok(result)
    }

    pub fn count_in_dictionary(&self, word: &str) -> &usize
    {
        self.dictionary.get(word).unwrap_or(&0)
    }

    pub fn count_in_files(&self, prompt: &str) -> Result<Vec<(&PathBuf, &usize)>>
    {
        Ok(self.files
            .par_iter()
            .map(|file| (&file.path, file.get_count(prompt)))
            .collect())
    }


    pub fn to_binary(&self, filename: &str) -> Result<()>
    {
        let file = fs::File::create(filename).expect("Failed to create a fs::File}");
        let mut serializer = Serializer::new(file);

        self.serialize(&mut serializer).expect("Failed to serialize Files");

        Ok(())
    }

    pub fn from_binary(filename: &str) -> Result<Files>
    {
        match fs::File::open(filename)
        {
            Ok(mut file) =>
                {
                    let mut buffer = Vec::new();

                    file.read_to_end(&mut buffer).expect("Failed to read a fs::File");
                    let mut deserializing = Deserializer::new(&*buffer);

                    Ok(Deserialize::deserialize(&mut deserializing).expect("Error while deserializing Files structure"))
                }
            Err(error) =>
                {
                    println!("{error} occurred while deserializing - `{}`\nEmpty Files structure returned", filename);

                    Files::build()
                }
        }

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
    #[test]
    fn test_files_count() -> Result<()>
    {
        let root = "Test\\";
        let files = Files::new(root)?;

        let word = "tensor";
        let count = files.count_in_dictionary(word);

        println!("{count}");

        Ok(())
    }

    #[test]
    fn test_serialization() -> Result<()>
    {
        let root = "Test\\";
        let cache = "Cache\\file_cache.bin";
        let files = Files::new(root)?;

        println!("{files}");

        files.to_binary(cache)?;

        println!("{}", Files::from_binary(cache)?);

        Ok(())
    }
}