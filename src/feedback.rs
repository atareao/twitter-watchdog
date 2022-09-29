use serde::{Serialize, Deserialize};
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct Feedback{
    pub category: String,
    pub reference: String,
    pub content: String,
    pub username: String,
    pub nickname: String,
    pub applied: i64,
    pub source: String,
}

impl Feedback{
    pub fn new(category: &str, reference: &str, content: &str, username: &str,
               nickname: &str, applied: i64, source: &str)->Self{
        Feedback {
            category: category.to_string(),
            reference: reference.to_string(),
            content: content.to_string(),
            username: username.to_string(),
            nickname: nickname.to_string(),
            applied,
            source: source.to_string(), 
        }

    }
    pub async fn post(&self, url: &str, token: &str){
        println!("{}", url);
        match Client::new()
            .post(url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", &token))
            .json(self)
            .send()
            .await{
                Ok(response) => {
                    println!("Mensaje envíado: {}", response.status().to_string());
                },
                Err(error) => {
                    println!("No he podido enviar el mensaje: {}",error.to_string());
                },
            };
    }
}
