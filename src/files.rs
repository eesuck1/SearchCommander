use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use lopdf::Document;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct Files
{
    pub paths: Vec<PathBuf>,
    pub files: HashMap<PathBuf, Vec<String>>,
}

impl Files
{
    pub fn new(root: &str) -> Self
    {
        let paths: Vec<PathBuf> = WalkDir::new(root)
            .into_iter()
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.path().to_owned())
            .collect();

        let mut result = Files { paths, files: HashMap::new() };

        let content = result.read_files().unwrap();
        result.files = result.split_files(content).unwrap();

        result
    }

    pub fn print_files(&self) -> io::Result<()>
    {
        let _ = self.paths
            .iter()
            .for_each(|entry| println!("{}", entry.display()));

        Ok(())
    }

    pub fn read_txt_files(&self) -> io::Result<HashMap<PathBuf, String>>
    {
        let result: HashMap<PathBuf, String> = self.paths
            .par_iter()
            .filter(|path| path.extension().unwrap_or_default() == "txt")
            .map(|path| -> (PathBuf, String)
                {
                    let content= fs::read_to_string(path).unwrap();

                    (path.to_owned(), content)
                }).collect();

        Ok(result)
    }

    pub fn read_pdf_files(&self) -> io::Result<HashMap<PathBuf, String>>
    {
        let result: HashMap<PathBuf, String> = self.paths
            .par_iter()
            .filter(|path| path.extension().unwrap_or_default() == "pdf")
            .map(|path| -> (PathBuf, String)
                {
                    let document = Document::load(path).unwrap();

                    let content = document
                        .get_pages()
                        .iter()
                        .map(|(&number, _)|
                            {
                                let number = number;
                                return document.extract_text(&[number]).unwrap_or_default();
                            }).collect();

                    (path.to_owned(), content)
                }).collect();

        Ok(result)
    }

    pub fn read_files(&self) -> io::Result<HashMap<PathBuf, String>>
    {
        let mut files = self.read_txt_files()?;
        files.extend(self.read_pdf_files()?);

        Ok(files)
    }

    pub fn split_files(&self, files: HashMap<PathBuf, String>) -> io::Result<HashMap<PathBuf, Vec<String>>>
    {
        let result: HashMap<PathBuf, Vec<String>>  = files
            .into_par_iter()
            .map(|(path, content)|
                (path.clone(),
                 content.split_whitespace()
                     .map(|item|
                         item.chars()
                             .filter(|symbol| symbol.is_alphabetic())
                             .flat_map(|symbol| symbol.to_lowercase())
                             .collect())
                     .collect()))
            .collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_print() -> io::Result<()>
    {
        let root = "Test\\";
        let files = Files::new(root);

        files.print_files()?;

        Ok(())
    }
    #[test]
    fn read_txt_files() -> io::Result<()>
    {
        let root = "Test\\";
        let files = Files::new(root);

        files.read_txt_files()?
            .iter()
            .par_bridge()
            .for_each(|(path, content)| println!("{}: {}", path.display(), content));

        Ok(())
    }
    #[test]
    fn read_pdf_files() -> io::Result<()>
    {
        let root = "Test\\";
        let files = Files::new(root);

        files.read_pdf_files()?
            .iter()
            .par_bridge()
            .for_each(|(path, content)| println!("{}: {}", path.display(), content));


        Ok(())
    }

    #[test]
    fn test_files() -> io::Result<()>
    {
        let root = "Test\\";
        let files = Files::new(root);

        files.files
            .iter()
            .par_bridge()
            .for_each(|(path, content)|
                println!("{}: {}", path.display(), content.join(" ")));

        Ok(())
    }
}
