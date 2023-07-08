use serde_json::Value;
use reqwest::{Client, Response, StatusCode};
use reqwest::header::{HeaderMap, HeaderValue, HeaderName};
use std::str::FromStr;

#[derive(Debug)]
pub struct Zinc{
    url: String,
    token: String,
}

#[derive(Debug)]
pub enum CustomError{
    ReqwestError(reqwest::Error),
    MessageError(String)
}


impl Zinc{
    pub fn new(base_url: &str, indice: &str, token: &str) -> Self{
        Self {
            url: format!("https://{}/api/default/{}/_json", base_url, indice),
            token: token.to_string(),
        }
    }

    pub async fn publish(&self, body: &Value) -> Result<Response, CustomError>{
        self.post(&self.url, body).await
    }

    async fn post(&self, url: &str, body: &Value)->Result<Response, CustomError>{
        println!("URL: {}", url);
        let mut header_map = HeaderMap::new();
        header_map.insert(HeaderName::from_str("Content-type").unwrap(),
                          HeaderValue::from_str("application/json").unwrap());
        header_map.insert(HeaderName::from_str("Accept").unwrap(),
                          HeaderValue::from_str("application/json").unwrap());
        header_map.insert(HeaderName::from_str("Authorization").unwrap(),
                          HeaderValue::from_str(&format!("Basic {}", self.token)).unwrap());
        let client = Client::builder()
            .default_headers(header_map)
            .build()
            .unwrap();
        let content = serde_json::to_string(body).unwrap();
        match client.post(url).body(content).send().await{
            Ok(res) => {
                if res.status() == StatusCode::OK{
                    Ok(res)
                }else{
                    let msg = "Esto es un error".to_string();
                    Err(CustomError::MessageError(msg))
                }

            },
            Err(e) => {
                Err(CustomError::ReqwestError(e))
            },
        }
    }
}
#[cfg(test)]
mod tests{
    use crate::zinc::Zinc;
    use serde_json::json;
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn publish_in_zinc() {
        dotenv().ok();
        let base_url = std::env::var("ZINC_BASE_URL").expect("ZINC_BASE_URL not set");
        let indice = std::env::var("ZINC_INDICE").expect("ZINC_INDICE not set");
        let token = std::env::var("ZINC_TOKEN").expect("TOKEN not set");
        println!("{}", &token);
        let zinc = Zinc::new(&base_url, &indice, &token);
        let data = json!([{
            "test": "test"
        }]);
        let res = zinc.publish(&data).await;
        assert!(res.is_ok())
    }
}

