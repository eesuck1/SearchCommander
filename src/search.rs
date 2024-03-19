use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use rayon::prelude::*;

use crate::counts;

pub fn search(line: &str, counts: &counts::Count) -> io::Result<Vec<(PathBuf, f32)>>
{
    let scores: Mutex<HashMap<PathBuf, f32>> = Mutex::new(HashMap::new());
    let words: Vec<&str> = line.split_whitespace().collect();

    counts.in_file_counts
        .par_iter()
        .for_each(|(path, words_count)|
            {
                let mut scores_pointer = scores.lock().unwrap();
                let _ = *scores_pointer.entry(path.to_owned()).or_insert(
                            words.iter()
                                .map(|word|
                                    {
                                        match words_count.get(word.to_owned()) {
                                            None => 0.0,
                                            Some(value) =>
                                                {
                                                    match counts.overall_counts.get(word.to_owned()) {
                                                        None => 0.0,
                                                        Some(overall) => *value as f32 / *overall as f32,
                                                    }
                                                }
                                        }
                                    }).sum());
            });

    let binding = scores.lock().unwrap();
    let mut sorted: Vec<(&PathBuf, &f32)> = binding
        .par_iter()
        .collect();
    sorted.sort_by(|first, second| second.1.total_cmp(first.1));

    let top_ten: Vec<(PathBuf, f32)> = sorted.par_iter()
        .take(10)
        .cloned().map(|(path, score)| (path.to_owned(), score.to_owned())).collect();

    Ok(top_ten)
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_search() -> io::Result<()>
    {
        let root = "Test\\";
        let counts = counts::Count::new(root);

        let top_ten = search("the category", &counts)?;
        top_ten.iter().for_each(|(path, score)| println!("{}: {}", path.display(), score));

        Ok(())
    }
}