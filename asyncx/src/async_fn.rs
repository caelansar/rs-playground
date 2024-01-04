use futures::Future;

fn f1(s: &str) -> impl Future<Output = ()> {
    async {}
}

async fn f2(s: &str) {}

fn f3<'a>(s: &'a str) -> impl Future<Output = ()> + 'a {
    async {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_f1() {
        let fut;
        {
            let s = String::from("a");
            fut = f1(&s);
        }
        fut.await
    }

    // #[tokio::test]
    // async fn test_f2() {
    //     let fut;
    //     {
    //         let s = String::from("a");
    //         fut = f2(&s); // ❌ borrowed value does not live long enough
    //     }
    //     fut.await
    // }

    // #[tokio::test]
    // async fn test_f3() {
    //     let fut;
    //     {
    //         let s = String::from("a");
    //         fut = f3(&s); // ❌ borrowed value does not live long enough
    //     }
    //     fut.await
    // }
}
