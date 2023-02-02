use macrox::Builder;

#[derive(Default, Debug, PartialEq, Builder)]
struct Config {
    timeout: u32,
    retry: bool,
    list: Option<Vec<i32>>,
}

fn main() {
    let cfg = Config::builder()
        .with_timeout(1000u32)
        .with_retry(true)
        .with_list(vec![100])
        .build()
        .unwrap();

    assert_eq!(
        Config {
            timeout: 1000,
            retry: true,
            list: Some(vec![100])
        },
        cfg
    );
}
