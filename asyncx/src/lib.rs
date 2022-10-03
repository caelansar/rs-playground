use std::time::Duration;

use tokio::time::sleep;

async fn request() -> String {
    sleep(Duration::from_secs(2)).await;
    "content".to_string()
}

#[cfg(test)]
mod tests {
    use crate::request;

    #[tokio::test]
    async fn read_content_should_works() {
        let mut handles = Vec::new();
        for _ in 0..10 {
            handles.push(tokio::spawn(request()));
        }

        let mut output = Vec::with_capacity(handles.len());
        for handle in handles {
            output.push(handle.await.unwrap());
        }
        for res in output {
            println!("response {}", res);
        }
    }
}
