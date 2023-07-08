mod twitter;
mod feedback;
mod config;
mod message;
mod mattermost;
mod zinc;

use dotenv::dotenv;
use std::{thread, time, env};
use tokio;
use crate::{twitter::Twitter, mattermost::Mattermost, config::Config, feedback::Feedback, zinc::Zinc};
use serde_json::{Map, Value, json};
use crate::message::{check_key, check_comment};

const FILENAME: &str = "lastid.toml";


#[tokio::main]
async fn main() {
    dotenv().ok();

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
    let mattermost_base_uri = env::var("MATTERMOST_BASE_URI").expect("Not found Mattermost Base Uri");
    let mattermost_token = env::var("MATTERMOST_ACCESS_TOKEN").expect("Not found Mattermost token");
    let mattermost = Mattermost::new(&mattermost_base_uri, &mattermost_token);
    let idea_channel = mattermost.get_channel_by_name("atareao_idea").await.unwrap();
    let pregunta_channel = mattermost.get_channel_by_name("atareao_pregunta").await.unwrap();
    let comentario_channel = mattermost.get_channel_by_name("atareao_comentario").await.unwrap();
    let mencion_channel = mattermost.get_channel_by_name("atareao_mencion").await.unwrap();
    let zinc_base_url = env::var("ZINC_BASE_URL").expect("Not found zinc base url");
    let zinc_indice = env::var("ZINC_INDICE").expect("Not found zinc indice");
    let zinc_token = env::var("ZINC_TOKEN").expect("Not found token");
    let zinc = Zinc::new(&zinc_base_url, &zinc_indice, &zinc_token);
    loop {
        thread::sleep(sleep_time);
        match search(&url, &token, &twitter, &last_id, &mattermost, 
                &idea_channel, &pregunta_channel, &comentario_channel,
                &mencion_channel, &zinc).await{
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

async fn search(url: &str, token: &str, twitter: &Twitter, last_id: &str,
        mattermost: &Mattermost, idea_channel: &str, pregunta_channel: &str,
        comentario_channel: &str, mencion_channel: &str, zinc: &Zinc) -> Option<String>{
    let mut new_last_id: String = "".to_string();
    let res = &twitter.get_mentions(&last_id).await;
    if res.is_ok(){
        let mut response: Map<String,Value> = serde_json::from_str(res.as_ref().unwrap()).unwrap();
        println!("{:?}", response);
        //let mut statuses = response.get_mut("statuses").unwrap().as_array().unwrap().to_owned();
        let mut statuses = match response.get_mut("statuses"){
            Some(statuses) => {
                match statuses.as_array(){
                    Some(array) => array.to_owned(),
                    None => Vec::new(),
                }
            },
            None => Vec::new()
        };
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
                let feedback = Feedback::new("idea", &new_last_id, &message, name, screen_name, 0, "Twitter");
                feedback.post(url, token).await;
                let thanks_message = format!("muchas gracias por tu idea, @{}", screen_name);
                twitter.post(&thanks_message, &new_last_id).await;
                let mm_message = format!("Src: Twitter. From: @{}. Content: {}", &screen_name, &message);
                mattermost.post_message(idea_channel, &mm_message, None).await;
                zinc.publish(&json!([{
                    "src": "Twitter",
                    "type": "idea",
                    "from": format!("@{}", &screen_name),
                    "message": &message,
                }])).await.unwrap();
            }else if let Some(message) = check_key("pregunta", text){
                let feedback = Feedback::new("pregunta", &new_last_id, &message, name, screen_name, 0, "Twitter");
                feedback.post(url, token).await;
                let thanks_message = format!("muchas gracias por tu pregunta, @{}", screen_name);
                twitter.post(&thanks_message, &new_last_id).await;
                let mm_message = format!("Src: Twitter. From: @{}. Content: {}", &screen_name, &message);
                mattermost.post_message(pregunta_channel, &mm_message, None).await;
                zinc.publish(&json!([{
                    "src": "Twitter",
                    "type": "pregunta",
                    "from": format!("@{}", &screen_name),
                    "message": &message,
                }])).await.unwrap();
            }else if let Some(option) = check_comment("comentario", text){
                let (commentario, reference) = option;
                if let Some(message) = commentario{
                    let id = match reference {
                        Some(value) => value,
                        None => new_last_id.clone(),
                    };
                    let feedback = Feedback::new("comentario", &id, &message, name, screen_name, 0, "Twitter");
                    feedback.post(url, token).await;
                    let thanks_message = format!("muchas gracias por tu comentario, @{}", screen_name);
                    twitter.post(&thanks_message, &new_last_id).await;
                    let mm_message = format!("Src: Twitter. From: @{}. Content: {}", &screen_name, &message);
                    mattermost.post_message(comentario_channel, &mm_message, None).await;
                    zinc.publish(&json!([{
                        "src": "Twitter",
                        "type": "comentario",
                        "from": format!("@{}", &screen_name),
                        "message": &message,
                    }])).await.unwrap();
                }
            }else{
                let feedback = Feedback::new("mencion", &new_last_id, text, name, screen_name, 0, "Twitter");
                feedback.post(url, token).await;
                let mm_message = format!("Src: Twitter. From: @{}. Content: {}", &screen_name, &text);
                mattermost.post_message(mencion_channel, &mm_message, None).await;
                zinc.publish(&json!([{
                    "src": "Twitter",
                    "type": "mencion",
                    "from": format!("@{}", &screen_name),
                    "message": &text,
                }])).await.unwrap();
            }
        }
    }
    if new_last_id != "" && new_last_id != last_id{
        return Some(new_last_id);
    }
    None
}
