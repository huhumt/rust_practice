use chrono::Local;
use glob::glob;
use rand::Rng;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde_json::{self, Value};
use std::error::Error;
use std::fmt;
use std::fs::{remove_file, File};
use std::path::Path;

#[derive(Debug)]
pub enum JsonDataParseError {
    DataNotHashmapError,
    DataNotArrayError,
}

impl fmt::Display for JsonDataParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DataNotHashmapError => write!(f, "Not hashmap data"),
            Self::DataNotArrayError => write!(f, "Not array data"),
        }
    }
}

impl Error for JsonDataParseError {}

fn shorten_url(long_url: &str) -> Result<String, Box<dyn Error>> {
    let params = [("format", "json"), ("url", long_url)];
    let shorten_url = reqwest::Url::parse_with_params("https://is.gd/create.php", &params)?;
    let response = reqwest::blocking::Client::new()
        .post(shorten_url)
        .send()?
        .json::<Value>()?;
    let map = response
        .as_object()
        .ok_or(JsonDataParseError::DataNotHashmapError)?;
    match map.get("shorturl") {
        Some(short_url) => Ok(short_url.to_string()),
        None => Err("Shorturl not found".into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let date = Local::now().date_naive().format("%m_%d").to_string();
    let tmp_json_filename: &str = &format!("/tmp/rustnews_out{}.json", date);
    let response: Value;
    if !Path::new(tmp_json_filename).exists() {
        let mut url: String =
            "https://api.wikimedia.org/feed/v1/wikipedia/en/onthisday/events/".to_string();
        url.push_str(&date.replace('_', "/"));
        // response = reqwest::blocking::get(url)?.json::<Value>()?;
        response = reqwest::blocking::Client::new()
            .get(url)
            .send()?
            .json::<Value>()?;

        for json_path in glob("/tmp/rustnews_out*.json")? {
            remove_file(json_path?)?;
        }
        serde_json::to_writer_pretty(File::create(tmp_json_filename)?, &response)?;
    } else {
        response = serde_json::from_reader(File::open(tmp_json_filename)?)?;
    }

    let events = response["events"]
        .as_array()
        .ok_or(JsonDataParseError::DataNotArrayError)?;
    let rand_num = rand::thread_rng().gen_range(0..events.len());
    let cur_event = &events[rand_num];
    // println!("{}: {}", cur_event["year"], cur_event["text"]);
    let extract = &cur_event["pages"]
        .as_array()
        .ok_or(JsonDataParseError::DataNotArrayError)?[0]
        .as_object()
        .ok_or(JsonDataParseError::DataNotHashmapError)?["extract"];
    println!("\n\n{}", extract);

    // TOP_HEADLINES_URL = "https://newsapi.org/v2/top-headlines"
    // EVERYTHING_URL = "https://newsapi.org/v2/everything"
    // SOURCES_URL = "https://newsapi.org/v2/sources"
    // "Content-Type": "Application/JSON", "Authorization": a5645b9190f54f92bd7ab596d165343b
    // https://github.com/mattlisiv/newsapi-python/blob/master/newsapi/const.py
    if let Some(keyword) = &cur_event["text"]
        .to_string()
        .replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "")
        .rsplit_once(' ')
    {
        let params = [("language", "en"), ("q", keyword.1)];
        let news_api_url =
            reqwest::Url::parse_with_params("https://newsapi.org/v2/top-headlines", &params)?;
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(header::USER_AGENT, HeaderValue::from_static("Rust Reqwest"));
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("a5645b9190f54f92bd7ab596d165343b"),
        );
        let news = reqwest::blocking::Client::new()
            .get(news_api_url)
            .headers(headers)
            .send()?
            .json::<Value>()?;
        let news_article = news["articles"]
            .as_array()
            .ok_or(JsonDataParseError::DataNotArrayError)?;
        if !news_article.is_empty() {
            let rand_article_index = rand::thread_rng().gen_range(0..news_article.len());
            let rand_article = &news_article[rand_article_index];
            let rand_article_title = &rand_article["title"];
            let rand_article_url = &rand_article["url"].to_string();
            let short_url = shorten_url(rand_article_url)?;
            println!("\n\n{}, from {}", rand_article_title, short_url);
        }
    }

    Ok(())
}
