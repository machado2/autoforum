use std::error::Error;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::{
    dice_roll::dice_roll,
    flarum::Forum,
    language::Language,
    llm::{get_llm_response, LlmError},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    pub user_id: i32,
    pub system_message: String,
}

impl Character {
    async fn ask_gpt(&self, user_message: &str) -> Result<String, LlmError> {
        get_llm_response(&self.system_message, user_message).await
    }

    pub async fn create_title_for_new_topic(
        &self,
        lang: &dyn Language,
    ) -> Result<String, LlmError> {
        self.ask_gpt(&lang.get_new_topic_title_prompt()).await
    }

    pub async fn create_post_new_topic(
        &self,
        lang: &dyn Language,
        title: &str,
    ) -> Result<String, LlmError> {
        let prompt = lang.get_new_topic_prompt(title);
        self.ask_gpt(&prompt).await
    }

    pub async fn create_new_topic(
        &self,
        lang: &dyn Language,
        forum: &Forum,
    ) -> Result<(), Box<dyn Error>> {
        let title = self.create_title_for_new_topic(lang).await?;
        let post = self.create_post_new_topic(lang, &title).await?;
        forum
            .create_new_discussion(self.user_id, &title, &post)
            .await?;
        Ok(())
    }

    pub async fn post_on_discussion(
        &self,
        lang: &dyn Language,
        forum: &Forum,
        discussion_id: i32,
    ) -> Result<(), Box<dyn Error>> {
        let last_comments = forum.fetch_discussion(discussion_id).await?;
        let history = last_comments
            .recent_posts
            .iter()
            .map(|c| c.content.clone())
            .collect::<Vec<String>>()
            .join("\n\n");
        let msg = lang.get_reply_prompt(&last_comments.title, &history);
        let content = self.ask_gpt(msg.as_str()).await?;
        forum
            .write_post(self.user_id, discussion_id, &content)
            .await?;
        Ok(())
    }

    pub async fn interact_with_forum(
        &self,
        lang: &dyn Language,
        forum: &Forum,
    ) -> Result<(), Box<dyn Error>> {
        let mut should_create_new_topic = dice_roll(20);
        if !should_create_new_topic {
            let random_discussion = forum.get_random_discussion(self.user_id).await?;
            match random_discussion {
                Some(discussion) => {
                    self.post_on_discussion(lang, forum, discussion.id).await?;
                }
                None => {
                    println!("No discussions found, creating a new one");
                    should_create_new_topic = true;
                }
            }
        }
        if should_create_new_topic {
            self.create_new_topic(lang, forum).await?;
        }
        Ok(())
    }
}

pub fn get_character(lang: &dyn Language, user_id: Option<i32>) -> Character {
    match user_id {
        None => get_random_character(lang),
        Some(user_id) => lang
            .get_character_list()
            .iter()
            .find(|c| c.user_id == user_id)
            .unwrap()
            .clone(),
    }
}

fn get_random_character(lang: &dyn Language) -> Character {
    let mut rng = rand::thread_rng();
    lang.get_character_list().choose(&mut rng).unwrap().clone()
}
