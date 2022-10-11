#[derive(Default, Debug, PartialEq)]
pub struct Config {
    timeout: u32,
    retry: bool,
}

macro_rules! new_config {
    () => {{
        let cfg = Config::default();
        cfg
    }};
    ( $($x:expr),*) => {{
        let mut cfg = Config::default();
        $($x(&mut cfg);)*
        cfg
    }};
}

#[cfg(test)]
mod tests {
    use crate::Config;

    #[test]
    fn macro_should_work() {
        let cfg = new_config!();
        assert_eq!(
            Config {
                timeout: 0,
                retry: false
            },
            cfg
        );

        let cfg = new_config!(with_timeout(1000), with_retry(true));
        assert_eq!(
            Config {
                timeout: 1000,
                retry: true,
            },
            cfg
        );
    }

    fn with_timeout(timeout: u32) -> impl FnOnce(&mut Config) {
        let f = move |cfg: &mut Config| cfg.timeout = timeout;
        f
    }
    fn with_retry(retry: bool) -> impl FnOnce(&mut Config) {
        let f = move |cfg: &mut Config| cfg.retry = retry;
        f
    }
}
