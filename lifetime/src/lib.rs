pub fn strtok<'a>(s: &mut &'a str, delimiter: char) -> &'a str {
    if let Some(i) = s.find(delimiter) {
        let prefix = &s[..i];
        let suffix = &s[(i + delimiter.len_utf8())..];
        *s = suffix;
        prefix
    } else {
        let prefix = *s;
        *s = "";
        prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strtok_should_work() {
        let s = "hello world".to_owned();
        let mut s1 = s.as_str();
        let prefix = strtok(&mut s1, ' ');
        println!("prefix: {}, s1: {}, s: {}", prefix, s1, s);
    }
}
