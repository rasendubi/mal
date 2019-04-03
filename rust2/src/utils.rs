pub fn unescape(s: &str) -> String {
    let mut res = String::new();

    let mut i = s[1 .. s.len()-1].chars();
    while let Some(c) = i.next() {
        if c != '\\' {
            res.push(c);
        } else {
            match i.next() {
                Some('n') => res.push('\n'),
                Some(c) => res.push(c),
                None => (),
            }
        }
    }

    res
}
