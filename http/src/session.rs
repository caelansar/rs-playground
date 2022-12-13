use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashMap;

pub(crate) fn get_session_id(
    headers: &HashMap<Cow<str>, Cow<str>>,
    query: &HashMap<Cow<str>, Cow<str>>,
) -> Option<String> {
    let options = &[
        (headers, "session-id"),
        (headers, "session_id"),
        (query, "session_id"),
    ];
    get_first_match(options)
}

fn get_first_match<'a, const N: usize>(
    options: &[(&'a HashMap<Cow<str>, Cow<str>>, &'a str); N],
) -> Option<String> {
    options
        .iter()
        .fold_while(None, |acc, (h, s)| match h.get(*s) {
            Some(v) => Done(Some(v.to_string())),
            None => Continue(acc),
        })
        .into_inner()
}
