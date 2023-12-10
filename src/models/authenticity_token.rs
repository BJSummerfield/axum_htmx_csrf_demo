pub struct AuthenticityToken {}

impl AuthenticityToken {
    pub fn key() -> String {
        "authenticity_token".to_owned()
    }
}
