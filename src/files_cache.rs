use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Result};
use std::path::PathBuf;

use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::files::Files;


#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMap
{
    caches: HashMap<PathBuf, PathBuf>,
    path: String,
    files_cache: String,
}

impl Drop for CacheMap
{
    fn drop(&mut self)
    {
        self.to_binary(&self.path).expect("Failed to serialize CacheMap")
    }
}

impl CacheMap
{
    pub fn new(path: &str, files_cache: &str) -> Result<Self>
    {
        match File::open(path)
        {
            Ok(mut file) =>
                {
                    let mut buffer = Vec::new();

                    file.read_to_end(&mut buffer).expect("Failed to read a fs::File");
                    let mut deserializing = Deserializer::new(&*buffer);

                    Ok(Deserialize::deserialize(&mut deserializing).expect("Error while deserializing CacheMap structure"))
                }
            Err(error) =>
                {
                    println!("{error} occurred while deserializing - `{}`\nEmpty CacheMap structure returned at this path", path);

                    Ok(Self::build(path, files_cache))
                }
        }
    }

    pub fn build(path: &str, files_cache: &str) -> Self
    {
        File::create(path).expect("Failed to create a CacheMap file");

        CacheMap { caches: HashMap::new(), path: path.to_owned(), files_cache: files_cache.to_owned() }
    }

    pub fn to_binary(&self, path: &str) -> Result<()>
    {
        let file = File::create(path).expect("Failed to create a fs::File}");
        let mut serializer = Serializer::new(file);

        self.serialize(&mut serializer).expect("Failed to serialize CacheMap");

        Ok(())
    }
    pub fn add_files(&mut self, root: PathBuf, files: &Files) -> Result<()>
    {
        match self.caches.get(&root)
        {
            Some(_) => Ok(()),
            None =>
                {
                    let path = format!("{}\\{}.{}",
                                       self.files_cache,
                                       self
                                           .get_cache_index()
                                           .expect("Failed to get Cache Files index"),
                                       "bin");

                    files.to_binary(&path).expect("Failed to cache Files");
                    self.caches.insert(root.clone(), PathBuf::from(path));

                    Ok(())
                }
        }
    }

    pub fn get_files(&mut self, root: PathBuf) -> Result<Files>
    {
        match self.caches.get(&root)
        {
            Some(path) => Files::from_binary(path.to_str().unwrap()),
            None =>
                {
                    let files = Files::new(root.to_str().unwrap()).expect("Failed to create Files structure");
                    self.add_files(root.clone(), &files)
                        .expect("Failed to Files structure");

                    Ok(files.clone())
                }
        }
    }
    fn get_cache_index(&self) -> Result<usize>
    {
        let mut counter: usize = 0;

        WalkDir::new(self.files_cache.clone())
            .into_iter()
            .for_each(|entry|
                {
                    if entry.unwrap().path().is_file()
                    {
                        counter += 1;
                    }
                });

        Ok(counter)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_cache() -> Result<()>
    {
        const CACHE_ITSELF_PATH: &str = "Cache\\cache.bin";
        const FILES_CACHE_FOLDER: &str = "Cache\\FilesCache";

        let mut cache = CacheMap::new(CACHE_ITSELF_PATH, FILES_CACHE_FOLDER).expect("Failed to create CacheMap");

        let root = "Test\\Math";
        let files = Files::new(root)?;

        cache.add_files(PathBuf::from(root), &files)
    }
}