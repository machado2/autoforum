use crate::characters::Character;

pub trait Language {
    fn get_character_list(&self) -> Vec<Character>;
    fn get_new_topic_title_prompt(&self) -> String;
    fn get_new_topic_prompt(&self, title: &str) -> String;
    fn get_reply_prompt(&self, title: &str, history: &str) -> String;
}
