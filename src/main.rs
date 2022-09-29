mod twitter;
mod feedback;
mod config;
mod message;

use dotenv::dotenv;
use std::{thread, time, env};
use tokio;
use crate::{twitter::Twitter, config::Config, feedback::Feedback};
use serde_json::{Map, Value};
use crate::message::{check_key, check_comment};

const FILENAME: &str = "lastid.toml";


#[tokio::main]
async fn main() {
    dotenv().expect("Not found environment");
    let mut config = Config::read("lastid.toml").expect("Can not read last id");
    let mut last_id = config.get_last_id().to_string();
    let url = env::var("URL")
        .expect("Not found URL");
    let token = env::var("TOKEN")
        .expect("Not found TOKEN");
    let sleep_time_in_seconds = env::var("SLEEP_TIME")
        .expect("Not found SLEEP_TIME")
        .parse::<u64>()
        .unwrap();
    let consumer_key = env::var("TW_CONSUMER_KEY").expect("Not foun consumer key");
    let consumer_secret = env::var("TW_CONSUMER_SECRET").expect("Not found consumer secret");
    let access_token = env::var("TW_ACCESS_TOKEN").expect("Not found access token");
    let access_token_secret = env::var("TW_ACCESS_TOKEN_SECRET").expect("Not found access token secret");
    let sleep_time = time::Duration::from_secs(sleep_time_in_seconds);
    let twitter = Twitter::new(&consumer_key, &consumer_secret, &access_token, &access_token_secret);
    //twitter.tweet("Hi from rust!!").await;
    let res = &twitter.get_mentions(&last_id).await;
    if res.is_ok(){
        //println!("{}", res.as_ref().unwrap());
        let mut response: Map<String,Value> = serde_json::from_str(res.as_ref().unwrap()).unwrap();
        let mut statuses = response.get_mut("statuses").unwrap().as_array().unwrap().to_owned();
        statuses.reverse();
        for status in statuses {
            //println!("{}", status);
            let text = status.get("full_text").unwrap().as_str().unwrap();
            let id = status.get("id_str").unwrap().as_str().unwrap();
            let created_at = status.get("created_at").unwrap().as_str().unwrap();
            let user = status.get("user").unwrap();
            let name = user.get("name").unwrap().as_str().unwrap();
            let screen_name = user.get("screen_name").unwrap().as_str().unwrap();
            println!("==========");
            println!("Text: {}", text);
            println!("Id: {}", id);
            println!("created_at: {}", created_at);
            println!("Name: {}", name);
            println!("Screen Name: {}", screen_name);

        }
    }
    let feedback = Feedback::new("idea", "0", "nada", "", "", 0, "Twitter");
    feedback.post(&url, &token).await;
    loop {
        thread::sleep(sleep_time);
        match search(&url, &token, &twitter, &last_id).await{
            Some(new_last_id) => {
                config.last_id = new_last_id.to_string();
                config.save(&FILENAME);
                last_id = new_last_id.to_string();
            },
            _ => {},
        }
        println!("Esto es una prueba");
    }
}

async fn search(url: &str, token: &str, twitter: &Twitter, last_id: &str) -> Option<String>{
    let mut new_last_id: String = "".to_string();
    let res = &twitter.get_mentions(&last_id).await;
    if res.is_ok(){
        let mut response: Map<String,Value> = serde_json::from_str(res.as_ref().unwrap()).unwrap();
        let mut statuses = response.get_mut("statuses").unwrap().as_array().unwrap().to_owned();
        statuses.reverse();
        for status in statuses {
            //println!("{}", status);
            let text = status.get("full_text").unwrap().as_str().unwrap();
            new_last_id = status.get("id_str").unwrap().as_str().unwrap().to_string();
            let created_at = status.get("created_at").unwrap().as_str().unwrap();
            let user = status.get("user").unwrap();
            let name = user.get("name").unwrap().as_str().unwrap();
            let screen_name = user.get("screen_name").unwrap().as_str().unwrap();
            println!("==========");
            println!("Text: {}", text);
            println!("Id: {}", &new_last_id);
            println!("created_at: {}", created_at);
            println!("Name: {}", name);
            println!("Screen Name: {}", screen_name);
            if let Some(message) = check_key("idea", text){
                let feedback = Feedback::new("idea", &new_last_id, text, name, screen_name, 0, "Twitter");
                feedback.post(url, token);
            }else if let Some(message) = check_key("pregunta", text){
                let feedback = Feedback::new("pregunta", &new_last_id, text, name, screen_name, 0, "Twitter");
                feedback.post(url, token);
            }else if let Some(option) = check_comment("comentario", text){
                let (commentario, reference) = option;
                if let Some(message) = commentario{
                    let id = match reference {
                        Some(value) => value,
                        None => new_last_id.clone(),
                    };
                    let feedback = Feedback::new("comentario", &id, &message, name, screen_name, 0, "Twitter");
                    feedback.post(url, token);
                }
            }else{
                let feedback = Feedback::new("mencion", &new_last_id, text, name, screen_name, 0, "Twitter");
                feedback.post(url, token);
            }
        }
    }
    if new_last_id != "" && new_last_id != last_id{
        return Some(new_last_id);
    }
    None
}
