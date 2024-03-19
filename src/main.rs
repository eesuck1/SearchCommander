use std::io;
use open::that;

mod files;
mod counts;
mod search;


fn main() -> io::Result<()>
{
    println!("Введіть корінь директорії: ");

    let stdin = io::stdin();

    let mut root = String::new();
    stdin.read_line(&mut root).expect("Помилка при читанні кореня!");

    let counts = counts::Count::new(&root.trim());
    let mut line = String::new();

    println!("Введіть ключові слова: (S - для виходу)");
    stdin.read_line(&mut line).expect("Помилка читання ключового слова!");

    while line.trim() != "S"
    {
        let search_result = search::search(&line, &counts)?;

        search_result
            .iter()
            .enumerate()
            .for_each(|(index, &(ref path, _))| println!("[{}] {}", index + 1, path.display()));

        println!("Введіть, який файл відкрити: (S - для виходу)");

        let mut response = String::new();
        stdin.read_line(&mut response).expect("Помилка при читанні індекса");

        if response.trim() != "S"
        {
            let index: usize = response.trim().parse().expect("Помилка конвертування індексу!");
            that(search_result[index - 1].0.clone()).expect("Помилка при відкриванні файлу!");
        }

        println!("Введіть ключові слова: ");
        stdin.read_line(&mut line).expect("Помилка читання ключового слова!");
    }

    Ok(())
}
