use std::collections::HashMap;
use std::io::Result;
use std::path::PathBuf;

use crate::files::Files;

pub struct Search;

impl Search
{
    pub fn new() -> Self
    {
        Search {}
    }

    pub fn search(&self, files: &Files, prompt: &str) -> Result<Vec<PathBuf>>
    {
        let mut scores: HashMap<PathBuf, f32> = HashMap::new();

        prompt.split_whitespace()
            .for_each(|word|
                {
                    let in_dictionary = files.count_in_dictionary(word);

                    files.count_in_files(word).unwrap()
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
        let root = "Test\\";
        let files = Files::new(root)?;
        let prompt = "what is a tensor";

        let search = Search::new();

        search.search(&files, prompt)?
            .into_iter()
            .for_each(|path| println!("{}", path.display()));

        Ok(())
    }
}