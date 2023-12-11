use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct RequestCounter {
    pub value: usize,
}

impl RequestCounter {
    pub const KEY: &'static str = "request_counter";
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestCounter
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;

        let mut counter: RequestCounter = session
            .get(Self::KEY)
            .expect("Could not deserialize.")
            .unwrap_or_default();

        counter.value += 1;

        session
            .insert(Self::KEY, &counter)
            .expect("Could not serialize.");

        Ok(counter)
    }
}
