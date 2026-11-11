//! Fetching todos over HTTP. Like the model, nothing here depends on egui: it is plain
//! async code that can be tested against a local server without opening a window.

use std::fmt;
use std::time::Duration;

use serde::Deserialize;

use crate::model::NewTodo;

/// A public demo API that returns todo-shaped JSON. `/todos` on its own returns 200
/// items, so the default asks for the first five.
pub const DEFAULT_URL: &str = "https://jsonplaceholder.typicode.com/todos?_limit=5";

/// How long one request may take from start to finish, body included.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// One item exactly as the server sends it. Keeping the wire format in its own type
/// means a change to the API touches this file and nothing else. Fields the app has
/// no use for (`id`, `userId`) are simply not listed; serde skips them.
#[derive(Deserialize)]
struct RemoteTodo {
    title: String,
    completed: bool,
}

impl From<RemoteTodo> for NewTodo {
    fn from(remote: RemoteTodo) -> Self {
        Self {
            title: remote.title,
            done: remote.completed,
        }
    }
}

/// Why an import failed, in terms the user can act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportError {
    InvalidUrl,
    TimedOut,
    /// The server could not be reached at all.
    Unreachable,
    /// The server answered, but not with a success status.
    Status(u16),
    /// The body was not the JSON we expected.
    UnexpectedBody,
}

impl fmt::Display for ImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrl => write!(formatter, "That is not a valid http(s) URL."),
            Self::TimedOut => write!(formatter, "The server took too long to answer."),
            Self::Unreachable => write!(formatter, "Could not reach the server."),
            Self::Status(code) => write!(formatter, "The server answered with HTTP {code}."),
            Self::UnexpectedBody => write!(formatter, "The response was not a list of todos."),
        }
    }
}

impl From<reqwest::Error> for ImportError {
    fn from(error: reqwest::Error) -> Self {
        // reqwest reports every failure through one error type; these predicates sort
        // it into the cases the user would want told apart.
        if error.is_timeout() {
            Self::TimedOut
        } else if error.is_builder() {
            Self::InvalidUrl
        } else if error.is_decode() {
            Self::UnexpectedBody
        } else {
            Self::Unreachable
        }
    }
}

/// Builds the HTTP client. A `Client` holds a connection pool, so the app creates one
/// and reuses it for every request instead of building one per call.
pub fn build_client(timeout: Duration) -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        // Without a timeout a stalled server would leave the request waiting forever.
        .timeout(timeout)
        .user_agent("egui-todo-tutorial")
        .build()
}

/// Downloads a JSON array of todos from `url`.
pub async fn fetch_todos(client: &reqwest::Client, url: &str) -> Result<Vec<NewTodo>, ImportError> {
    let response = client.get(url.trim()).send().await?;

    // A 404 or 500 is a *successful* request as far as the transport is concerned, so
    // it has to be checked for explicitly.
    let status = response.status();
    if !status.is_success() {
        return Err(ImportError::Status(status.as_u16()));
    }

    let remote_todos: Vec<RemoteTodo> = response.json().await?;
    Ok(remote_todos.into_iter().map(NewTodo::from).collect())
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    /// Starts a local server that answers `GET /todos` with `response`.
    async fn server_answering(response: ResponseTemplate) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/todos"))
            .respond_with(response)
            .mount(&server)
            .await;
        server
    }

    fn todos_url(server: &MockServer) -> String {
        format!("{}/todos", server.uri())
    }

    #[tokio::test]
    async fn a_json_array_becomes_new_todos() {
        let body = r#"[
            {"userId": 1, "id": 1, "title": "Buy milk", "completed": false},
            {"userId": 1, "id": 2, "title": "Call the bank", "completed": true}
        ]"#;
        let server =
            server_answering(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
                .await;
        let client = build_client(REQUEST_TIMEOUT).unwrap();

        let todos = fetch_todos(&client, &todos_url(&server)).await.unwrap();

        assert_eq!(
            todos,
            [
                NewTodo {
                    title: "Buy milk".to_owned(),
                    done: false
                },
                NewTodo {
                    title: "Call the bank".to_owned(),
                    done: true
                },
            ]
        );
    }

    #[tokio::test]
    async fn an_error_status_is_reported_with_its_code() {
        let server = server_answering(ResponseTemplate::new(500)).await;
        let client = build_client(REQUEST_TIMEOUT).unwrap();

        let error = fetch_todos(&client, &todos_url(&server)).await.unwrap_err();

        assert_eq!(error, ImportError::Status(500));
    }

    #[tokio::test]
    async fn a_body_that_is_not_a_todo_list_is_rejected() {
        let server = server_answering(
            ResponseTemplate::new(200).set_body_raw(r#"{"message": "hello"}"#, "application/json"),
        )
        .await;
        let client = build_client(REQUEST_TIMEOUT).unwrap();

        let error = fetch_todos(&client, &todos_url(&server)).await.unwrap_err();

        assert_eq!(error, ImportError::UnexpectedBody);
    }

    #[tokio::test]
    async fn a_slow_server_times_out() {
        let server = server_answering(
            ResponseTemplate::new(200)
                .set_body_raw("[]", "application/json")
                .set_delay(Duration::from_secs(5)),
        )
        .await;
        let impatient_client = build_client(Duration::from_millis(200)).unwrap();

        let error = fetch_todos(&impatient_client, &todos_url(&server))
            .await
            .unwrap_err();

        assert_eq!(error, ImportError::TimedOut);
    }

    #[tokio::test]
    async fn a_malformed_url_is_rejected_before_any_request_is_made() {
        let client = build_client(REQUEST_TIMEOUT).unwrap();

        let error = fetch_todos(&client, "not a url").await.unwrap_err();

        assert_eq!(error, ImportError::InvalidUrl);
    }

    #[tokio::test]
    async fn a_server_that_is_not_listening_is_unreachable() {
        // Port 9 ("discard") on localhost has nothing listening on a normal machine.
        let client = build_client(REQUEST_TIMEOUT).unwrap();

        let error = fetch_todos(&client, "http://127.0.0.1:9/todos")
            .await
            .unwrap_err();

        assert_eq!(error, ImportError::Unreachable);
    }
}
