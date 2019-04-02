use rustyline::Editor;

pub struct Reader {
    rl: Editor<()>,
}

impl Reader {
    pub fn new(history_file: &str) -> Reader {
        let mut result = Reader {
            rl: Editor::<()>::new(),
        };
        let _ = result.rl.load_history(history_file);
        result
    }

    pub fn readline(&mut self, prompt: &str) -> rustyline::Result<String> {
        let input = self.rl.readline(prompt)?;
        self.rl.add_history_entry(input.as_ref());
        self.rl.save_history("mal_history.txt")?;
        Ok(input)
    }
}
