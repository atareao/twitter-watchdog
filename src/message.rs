use regex::Regex;

pub fn check_key(key: &str, message: &str) -> Option<String>{
    if message.contains(&format!(r#"#{}"#, key)){
        return Some(message.trim().to_string());
    }
    None
}

pub fn check_comment(key: &str, message: &str) -> Option<(Option<String>, Option<String>)>{
    if message.contains(&format!(r#"#{}"#, key)){
        let patron = format!(r#"#{}\s+(\d*)"#, key);
        let re = Regex::new(&patron).unwrap();
        return match re.captures(message) {
            Some(captures) => {
                let referencia = captures.get(1).unwrap().as_str().to_string();
                Some((Some(referencia), Some(message.to_string())))
            },
            None => Some((None, Some(message.to_string()))),
        };
    }
    None
}
