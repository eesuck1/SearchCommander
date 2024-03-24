use std::collections::HashMap;
use std::io::Result;
use std::path::PathBuf;

use crate::files_cache::CacheMap;

pub struct Search
{
    cache: CacheMap
}

impl Search
{
    pub fn new(cache_path: &str, files_cache_path: &str) -> Result<Self>
    {
        Ok(Search { cache: CacheMap::new(cache_path, files_cache_path).expect("Failed to create CacheMap") })
    }
    
    pub fn default() -> Result<Self>
    {
        const CACHE_ITSELF_PATH: &str = "Cache\\cache.bin";
        const FILES_CACHE_FOLDER: &str = "Cache\\FilesCache";
        
        Self::new(CACHE_ITSELF_PATH, FILES_CACHE_FOLDER)
    }

    pub fn search(&mut self, root: PathBuf, prompt: &str) -> Result<Vec<PathBuf>>
    {
        let files = self.cache.get_files(root).expect("Failed to create Files structure");
        let mut scores: HashMap<PathBuf, f32> = HashMap::new();

        prompt.split_whitespace()
            .for_each(|word|
                {
                    let lower = &word.to_lowercase();
                    let in_dictionary = files.count_in_dictionary(lower);

                    files.count_in_files(lower).unwrap()
                        .into_iter()
                        .for_each(|(path, count)|
                            {
                                let score = *count as f32 / *in_dictionary as f32;

                                *scores.entry(path.clone()).or_insert(score) += score;
                            });
                });

        let mut vector: Vec<_> = scores.into_iter().collect::<Vec<_>>();

        vector.sort_by(|a, b| b.1.total_cmp(&a.1));

        Ok(vector.into_iter()
            .take(10)
            .map(|(path, _)| path)
            .collect())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_search() -> Result<()>
    {
        let root = PathBuf::from("Test\\");
        let prompt = "programming";

        let mut search = Search::default().expect("Failed to create Search instance");

        search.search(root, prompt)?
            .into_iter()
            .for_each(|path| println!("{}", path.display()));

        Ok(())
    }
}