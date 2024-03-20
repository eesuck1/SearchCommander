use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{PathBuf, Path};
use std::io::{Result};
use std::fs::{read_to_string};
use std::sync::{Mutex, Arc};

use serde::{Deserialize, Serialize};
use rayon::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
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
        file.count().expect("Error while counting");

        Ok(file)
    }

    fn build(path: &Path) -> Result<Self>
    {
        Ok(File { path: path.to_path_buf(), counts: HashMap::new() })
    }

    fn count(&mut self) -> Result<()>
    {
        let counts = Arc::new(Mutex::new(HashMap::new()));

        self.read().expect("Error while reading file (in count function)")
            .par_iter()
            .for_each(|word|
                {
                    let mut counts = counts.lock().unwrap();

                    *counts.entry(word.to_string()).or_insert(0) += 1;
                });

        self.counts = counts.lock().expect("Error while locking `counts`").clone();

        Ok(())
    }
    fn read(&self) -> Result<Vec<String>>
    {
        let content: Vec<String> = read_to_string(&self.path).expect("Failed to read file")
            .split_whitespace()
            .map(String::from)
            .collect();

        Ok(content)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_print() -> Result<()>
    {
        let file_path = "Test\\test.txt";
        let path = Path::new(&file_path);
        let file = File::new(&path)?;

        println!("{}", file);

        Ok(())
    }
}