use std::future::Future;

#[allow(dead_code)]
fn return_impl_future(_s: &str) -> impl Future<Output = ()> {
    async {}
}

#[allow(dead_code)]
fn return_impl_future_with_lifetime(s: &str) -> impl Future<Output = ()> + '_ {
    async move {
        println!("{}", s);
    }
}

#[allow(dead_code)]
async fn async_fn(_s: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_param_lifetime() {
        let future;

        {
            let s = String::from("abc");
            future = return_impl_future(&s);
            // error[E0597]: `s` does not live long enough
            // future = return_impl_future_with_lifetime(&s);
            // error[E0597]: `s` does not live long enough
            // future = async_fn(&s);
        }

        _ = future;
    }
}
