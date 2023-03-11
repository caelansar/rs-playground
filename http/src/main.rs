mod axum_style_params;
mod error;
mod session;

use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use session::get_session_id;
use std::{borrow::Cow, collections::HashMap, net::SocketAddr};

#[tokio::main]
async fn main() {
    // initialize tracing

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .route("/users", get(get_users));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), error::AppError> {
    if payload.username.len() < 4 {
        return Err(AppError::new("create user")
            .with_status(StatusCode::BAD_REQUEST)
            .with_details(Value::String("username too short".to_string())));
    }
    // insert your application logic here
    let user = User {
        id: 1,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    Ok((StatusCode::CREATED, Json(user)))
}

async fn get_users(
    Query(params): Query<HashMap<Cow<'_, str>, Cow<'_, str>>>,
    headers: HeaderMap,
) -> Json<Vec<User>> {
    let headers: HashMap<Cow<str>, Cow<str>> = headers
        .iter()
        .map(|x| {
            (
                Cow::Owned(x.0.to_string()),
                Cow::Borrowed(x.1.to_str().unwrap()),
            )
        })
        .collect();

    let session_id = get_session_id(&headers, &params);
    println!("session_id: {:?}", session_id);

    return Json(vec![
        User {
            id: 1,
            username: "a".to_string(),
        },
        User {
            id: 2,
            username: "b".to_string(),
        },
    ]);
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, Router};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_request() {
        let app = Router::new()
            .route("/users", post(create_user))
            .route("/users", get(get_users));

        let app1 = app.clone();

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let (parts, body) = resp.into_parts();
        let body = hyper::body::to_bytes(body).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(
            body,
            json!([
               {
                "id": 1,
                "username": "a"
               },
               {
                "id": 2,
                "username": "b"
               }
            ])
        );

        let resp = app1
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({"username": "a"})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let (parts, body) = resp.into_parts();
        let body = hyper::body::to_bytes(body).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(
            body,
            json!({
                "error":"create user",
                "error_details":"username too short"
            })
        );
    }
}
