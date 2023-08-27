use serde_json::{json, Value};
use reqwest::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, HeaderName};
use reqwest::Error;
use std::str::FromStr;
use urlencoding::encode;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Matrix<'a>{
    base_url: &'a str,
    token: &'a str,
}

impl<'a> Matrix<'a>{
    pub fn new(base_url: &'a str, token: &'a str) -> Self{
        Self {
            base_url,
            token,
        }
    }

    pub async fn post_message(&self, room_id: &str, message: &str) -> Result<Response, Error>{
        let room = encode(room_id);
        let now = SystemTime::now();
        let ts = now.duration_since(UNIX_EPOCH).expect("Time went backwrds").as_secs();
        let url = format!(
            "https://{}/_matrix/client/v3/rooms/{}:{}/send/m.room.message/{}",
            self.base_url,
            room,
            self.base_url,
            ts
        );
        let body = json!({
            "msgtype": "m.text",
            "body": message,
        });
        let mut header_map = HeaderMap::new();
        header_map.insert(HeaderName::from_str("Content-type").unwrap(),
                          HeaderValue::from_str("application/json").unwrap());
        header_map.append(HeaderName::from_str("Authorization").unwrap(),
                          HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap());
        Self::put(&url, header_map, &body).await
    }

    async fn post(&self, url: &str, body: Option<Value>)->Result<Response, Error>{
        println!("URL: {}", url);
        let mut header_map = HeaderMap::new();
        header_map.insert(HeaderName::from_str("Content-type").unwrap(),
                          HeaderValue::from_str("application/json").unwrap());
        header_map.insert(HeaderName::from_str("Authorization").unwrap(),
                          HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap());
        let client = Client::builder()
            .default_headers(header_map)
            .build()
            .unwrap();
        match body{
            Some(value) => {
                let content = serde_json::to_string(&value).unwrap();
                let res = client.post(url).body(content).send().await?;
                Ok(res)
            },
            None => {
                let res = client.post(url).send().await?;
                Ok(res)
            },
        }
    }
    async fn put(url: &str, header_map: HeaderMap, body: &Value) -> Result<Response, reqwest::Error>{
        let client = Client::builder()
            .default_headers(header_map)
            .build()
            .unwrap();
        let content = serde_json::to_string(body).unwrap();
        client.put(url).body(content).send().await
    }
}
#[cfg(test)]
mod tests{
    use crate::matrix::Matrix;
    use dotenv::dotenv;

    #[tokio::test]
    async fn post_message() {
        dotenv().ok();
        let base_url = std::env::var("MATRIX_BASE_URL").expect("BASE_URL not set");
        let token = std::env::var("MATRIX_ACCESS_TOKEN").expect("TOKEN not set");
        let room_id = std::env::var("MATRIX_ROOM_ID").expect("ROOM_ID not set");
        println!("{}", &token);
        let matrix_client = Matrix::new(&base_url, &token);
        let res = matrix_client.post_message(&room_id, "Esto es una prueba").await;
        println!("{:?}", res);
    }
}

