use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use tower_sessions::Session;

#[derive(Deserialize, Serialize, Debug)]
pub struct CounterForm {
    pub action: CounterAction,
}

impl fmt::Display for CounterAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CounterAction::Increment => write!(f, "Increment"),
            CounterAction::Decrement => write!(f, "Decrement"),
        }
    }
}

impl FromStr for CounterAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Increment" => Ok(CounterAction::Increment),
            "Decrement" => Ok(CounterAction::Decrement),
            _ => Err(()),
        }
    }
}
// Define the CounterAction enum
#[derive(Deserialize, Serialize, Debug)]
pub enum CounterAction {
    Increment,
    Decrement,
}

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct Counter {
    pub value: i32,
}

impl Counter {
    pub const KEY: &'static str = "counter";
}

#[async_trait]
impl<S> FromRequestParts<S> for Counter
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;

        let counter: Counter = session
            .get(Self::KEY)
            .expect("Could not deserialize.")
            .unwrap_or_default();

        session
            .insert(Self::KEY, &counter)
            .expect("Could not serialize.");

        Ok(counter)
    }
}
