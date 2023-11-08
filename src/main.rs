use characters::get_character;
use dotenvy::dotenv;
use english::EnglishLanguage;
use flarum::Forum;
use language::Language;
use portuguese::PortugueseLanguage;
use structopt::StructOpt;

mod characters;
mod dice_roll;
mod english;
mod flarum;
mod language;
mod llm;
mod portuguese;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "u", long = "user_id")]
    user_id: Option<i32>,

    #[structopt(short = "d", long = "discussion_id")]
    discussion_id: Option<i32>,

    #[structopt(short = "c", long = "create_new_topic")]
    create_new_topic: bool,

    #[structopt(short = "l", long = "language")]
    language: Option<String>,
}

#[tokio::main]
async fn main() {
    _ = dotenv();
    let opt = Opt::from_args();
    let lang: Box<dyn Language>;
    let forum;
    if opt.language == Some("pt".to_string()) {
        lang = Box::new(PortugueseLanguage);
        forum = Forum::new("https://forumbr.fbmac.net/api");
    } else {
        lang = Box::new(EnglishLanguage);
        forum = Forum::new("https://forum.fbmac.net/api");
    }
    let car = get_character(lang.as_ref(), opt.user_id);
    let result;
    if opt.create_new_topic {
        result = car.create_new_topic(lang.as_ref(), &forum).await;
    } else {
        match opt.discussion_id {
            Some(discussion_id) => {
                result = car.post_on_discussion(lang.as_ref(), &forum, discussion_id).await;
            }
            None => {
                result = car.interact_with_forum(lang.as_ref(), &forum).await;
            }
        }
    }
    match result {
        Ok(_) => println!("Success!"),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
