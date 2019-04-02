use rustyline::Editor;

pub fn read(prompt: &str) -> rustyline::Result<String> {
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history("mal_history.txt");

    let input = rl.readline(prompt)?;
    rl.add_history_entry(input.as_ref());
    rl.save_history("mal_history.txt")?;
    Ok(input)
}
