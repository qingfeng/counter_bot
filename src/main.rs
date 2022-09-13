use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup,
    },
    utils::command::BotCommands,
};

static COUNTER: AtomicUsize  = AtomicUsize::new(0);

#[derive(BotCommands)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Click the Add button and start counting, it's that simple.")]
    Help,
    #[command(description = "Start & Restart the counter.")]
    Start,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    log::info!("Starting counter bot...");

    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler).enable_ctrlc_handler().build().dispatch().await;
    Ok(())
}

fn make_keyboard() -> InlineKeyboardMarkup {
    let keyboard = ["Add"]
                .map(|btn| InlineKeyboardButton::callback(btn, btn));
    InlineKeyboardMarkup::new([keyboard])
}

async fn message_handler(
    m: Message,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = m.text() {
        match BotCommands::parse(text, "buttons") {
            Ok(Command::Help) => {
                bot.send_message(m.chat.id, Command::descriptions().to_string()).await?;
            }
            Ok(Command::Start) => {
                COUNTER.store(0, Ordering::SeqCst);
                bot.send_message(m.chat.id, "0000").reply_markup(make_keyboard()).await?;
            }

            Err(_) => {
                bot.send_message(m.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}

async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(data) = q.data {
        if data == "Add" {
            COUNTER.fetch_add(1, Ordering::Relaxed);
            let text = format!("{:04}", COUNTER.load(Ordering::Relaxed));
            match q.message {
                Some(Message { id, chat, .. }) => {
                    log::debug!("text: {}", text);
                    bot.edit_message_text(chat.id, id, text).reply_markup(make_keyboard()).await?;
                }
                None => {
                    log::debug!("text: {}", text);
                }
            }
        }
    }

    Ok(())
}
