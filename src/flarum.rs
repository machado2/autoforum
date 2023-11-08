extern crate reqwest;
extern crate serde;

use std::{collections::HashMap, error::Error};

use html2md::parse_html;
use rand::seq::SliceRandom;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use simple_error::SimpleError;

fn get_flarum_api_key() -> String {
    dotenvy::dotenv().ok();
    dotenvy::var("FLARUM_API_KEY").unwrap_or_else(|_| panic!("FLARUM_API_KEY must be set"))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Discussion {
    pub id: i32,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IdType {
    id: String,
    #[serde(rename = "type")]
    otype: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DataSingleIdType {
    data: IdType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FlarumObj {
    pub id: String,
    #[serde(rename = "type")]
    pub otype: String,
    pub attributes: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub content: String,
}

pub struct Forum {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DiscussionsResponse {
    data: Vec<FlarumObj>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DiscussionPostsResponse {
    included: Vec<FlarumObj>,
}

pub struct DiscussionData {
    pub title: String,
    pub recent_posts: Vec<Post>,
}

impl Forum {
    pub fn new(baseurl: &str) -> Self {
        Self {
            base_url: baseurl.to_string(),
            client: reqwest::Client::new(),
        }
    }

    fn get_headers(&self, user_id: i32) -> Result<HeaderMap, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let api_key = get_flarum_api_key();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(format!("Token {}; userId={}", &api_key, user_id).as_str())?,
        );
        Ok(headers)
    }

    async fn get(&self, user_id: i32, url: &str) -> Result<Value, Box<dyn Error>> {
        let headers = self.get_headers(user_id)?;
        let value = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await?
            .json::<Value>()
            .await?;
        Ok(value)
    }

    pub async fn list_recent_discussions(
        &self,
        user_id: i32,
    ) -> Result<Vec<Discussion>, Box<dyn Error>> {
        let url = format!("{}/discussions", self.base_url);
        let value = self.get(user_id, &url).await?;
        let discussions: Vec<_> = value["data"]
            .as_array()
            .ok_or_else(|| SimpleError::new("Invalid response"))?
            .iter()
            .filter_map(|d| {
                if d["relationships"]["lastPostedUser"]["data"]["id"]
                    .as_str()
                    .and_then(|s| s.parse::<i32>().ok())
                    == Some(user_id)
                {
                    return None;
                }
                if d["attributes"]["isLocked"].as_bool() == Some(true) {
                    return None;
                }
                if d["attributes"]["isSticky"].as_bool() == Some(true) {
                    return None;
                }
                if d["attributes"]["isHidden"].as_bool() == Some(true) {
                    return None;
                }
                if d["attributes"]["canReply"].as_bool() == Some(false) {
                    return None;
                }
                let id = d["id"].as_str().and_then(|s| s.parse::<i32>().ok());
                let title = d["attributes"]["title"].as_str();
                title.map(|t| {
                    id.map(|i| Discussion {
                        id: i,
                        title: t.to_string(),
                    })
                })
            })
            .flatten()
            .collect();
        Ok(discussions)
    }

    pub async fn get_random_discussion(
        &self,
        user_id: i32,
    ) -> Result<Option<Discussion>, Box<dyn Error>> {
        let discussions = self.list_recent_discussions(user_id).await?;
        let mut rng = rand::thread_rng();
        let random_discussion = discussions.choose(&mut rng).cloned();
        Ok(random_discussion)
    }

    pub async fn fetch_discussion(&self, id: i32) -> Result<DiscussionData, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let api_key = get_flarum_api_key();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(format!("Token {}", &api_key).as_str())?,
        );

        let response = self
            .client
            .get(format!("{}/discussions/{}", self.base_url, id))
            .headers(headers)
            .send()
            .await?
            .json::<Value>()
            .await?;

        let title = response["data"]["attributes"]
            .as_str()
            .unwrap_or("No title")
            .to_string();

        let mut posts: Vec<_> = response["included"]
            .as_array()
            .ok_or_else(|| SimpleError::new("Could not get included posts"))?
            .iter()
            .filter(|p| p["type"] == "posts")
            .collect();
        posts.sort_by_key(|p| p["attributes"]["number"].as_i64().unwrap_or(0));

        let posts: Vec<_> = posts
            .iter()
            .map(|p| Post {
                content: parse_html(p["attributes"]["contentHtml"].as_str().unwrap_or("")),
            })
            .collect();

        Ok(DiscussionData {
            title,
            recent_posts: posts,
        })
    }

    pub async fn create_new_discussion(
        &self,
        user_id: i32,
        title: &str,
        content: &str,
    ) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/discussions", self.base_url);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let api_key = get_flarum_api_key();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(format!("Token {}; userId={}", &api_key, user_id).as_str())?,
        );
        self.client
            .post(url)
            .headers(headers)
            .json(&json!({
                "data": {
                    "type": "discussions",
                    "attributes": {
                        "title": title,
                        "content": content
                    },
                    "relationships": {
                        "tags": {
                            "data": [
                                {
                                    "type": "tags",
                                    "id": "1"
                                }
                            ]
                        }
                    }
                }
            }))
            .send()
            .await?;
        Ok(())
    }

    pub async fn write_post(
        &self,
        user_id: i32,
        discussion_id: i32,
        content: &str,
    ) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/posts", self.base_url);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let api_key = get_flarum_api_key();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(format!("Token {}; userId={}", &api_key, user_id).as_str())?,
        );
        self.client
            .post(url)
            .headers(headers)
            .json(&json!(
                {"data":{"type":"posts","attributes":{"content":content},"relationships":{"discussion":{"data":{"type":"discussions","id":discussion_id.to_string()}}}}}
            ))
            .send()
            .await?;
        Ok(())
    }
}
