use jsonwebtoken::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

const SECRET: &[u8] = b"secret";

pub fn create_token(uid: &str) -> String {
    let claims = Claims {
        sub: uid.to_string(),
        exp: 2000000000,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET)).unwrap()
}