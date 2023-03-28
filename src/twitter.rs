use reqwest;
use oauth::Token;
use std::{format, collections::HashMap};

static BASE_URI: &str = "https://api.twitter.com";

#[derive(oauth::Request)]
struct MentionsTimeline{
    since_id: String,
    include_entities: String,
}

impl MentionsTimeline{
    fn new(since_id: &str) -> Self{
        MentionsTimeline {
            since_id: since_id.to_string(),
            include_entities: "false".to_string(),
        }
    }
    fn get_params(self) -> HashMap<String, String>{
        let mut result = HashMap::new();
        result.insert("since_id".to_string(), self.since_id);
        result.insert("include_entities".to_string(), self.include_entities);
        result
    }
}

#[derive(oauth::Request)]
struct SearchTweets{
    q: String,
    include_entities: String,
    result_type: String,
    tweet_mode: String,
    since_id: String,
}

impl SearchTweets{
    fn new(query: &str, since_id: &str) -> Self{
        SearchTweets {
            q: query.to_string(),
            include_entities: "false".to_string(),
            result_type: "recent".to_string(),
            tweet_mode: "extended".to_string(),
            since_id: since_id.to_string(),
        }
    }
    fn get_params(self) -> HashMap<String, String>{
        let mut result = HashMap::new();
        result.insert("q".to_string(), self.q);
        result.insert("include_entities".to_string(), self.include_entities);
        result.insert("result_type".to_string(), self.result_type);
        result.insert("tweet_mode".to_string(), self.tweet_mode);
        result.insert("since_id".to_string(), self.since_id);
        result
    }
}

#[derive(oauth::Request)]
struct Tweet{
    status: String,
    in_reply_to_status_id: String,
}

impl Tweet{
    pub fn new(message: &str, in_reply_to_status_id: &str)->Self{
        Self{
            status: message.to_string(),
            in_reply_to_status_id: in_reply_to_status_id.to_string()
        }
    }
    pub fn get_params(self) -> HashMap<String, String>{
        let mut result = HashMap::new();
        result.insert("status".to_string(), self.status.to_string());
        result
    }
}

pub struct Twitter{
    token: Token,
}

impl Twitter{
    pub fn new(client_identifier: &str, client_secret: &str, token: &str, token_secret: &str)->Self{
        Twitter{
            token: oauth::Token::from_parts(
                client_identifier.to_string(),
                client_secret.to_string(),
                token.to_string(),
                token_secret.to_string()),
        }
    }

    pub async fn post(&self, message: &str, in_reply_to_status_id: &str){
        let uri = format!("{}/1.1/statuses/update.json", BASE_URI);
        let request = Tweet::new(message, in_reply_to_status_id);
        let authorization_header = oauth::post(&uri, &request, &self.token,
            oauth::HMAC_SHA1);
        let client = reqwest::Client::new();
        let res = client
            .post(&uri)
            .header("Authorization", authorization_header)
            .form(&request.get_params())
            .send()
            .await;
    }

    pub async fn get_mentions(&self, since_id: &str) -> Result<String, reqwest::Error>{
        let encoded_message = "@atareao AND -filter:retweets AND -filter:replies";
        let uri = format!("{}/1.1/search/tweets.json", BASE_URI);
        let search = SearchTweets::new(&encoded_message, since_id);
        let authorization_header = oauth::get(&uri, &search, &self.token,
            oauth::HMAC_SHA1);
        let uri = oauth::to_query(uri.to_owned(), &search);
        println!("{}", &uri);
        println!("{}", &authorization_header);
        let client = reqwest::Client::new();
        let res = client
            .get(&uri)
            .header("Authorization", authorization_header)
            .send()
            .await?
            .text()
            .await?;
        Ok(res)
    }

    pub async fn get_mentions_timeline(&self) -> Result<String, reqwest::Error>{
        let uri = format!("{}/1.1/statuses/mentions_timeline.json", BASE_URI);
        let search = MentionsTimeline::new("123456");
        let authorization_header = oauth::get(&uri, &search, &self.token,
            oauth::HMAC_SHA1);
        let uri = oauth::to_query(uri.to_owned(), &search);
        println!("{}", &uri);
        println!("{}", &authorization_header);
        let client = reqwest::Client::new();
        let res = client
            .get(&uri)
            .header("Authorization", authorization_header)
            .send()
            .await?
            .text()
            .await?;
        Ok(res)
    }
}
