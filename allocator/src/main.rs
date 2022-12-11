use std::borrow::Cow;

mod alloc;

fn process_string(s: &str) -> String {
    match s.find(" ") {
        Some(i) => uppercase_first(&s[0..i]),
        None => uppercase_first(s),
    }
}

fn process_cow(s: &str) -> Cow<str> {
    match s.find(" ") {
        Some(i) => Cow::Owned(uppercase_first(&s[0..i])),
        None => {
            if s.starts_with(|x: char| x.is_uppercase()) {
                Cow::Borrowed(s)
            } else {
                Cow::Owned(uppercase_first(s))
            }
        }
    }
}

fn uppercase_first(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
}

fn main() {
    let s = "Fsdfdswerweerwewerfsdfsadfsdfad";
    let _ = process_string(s);
    // let _ = process_cow(s);
}
