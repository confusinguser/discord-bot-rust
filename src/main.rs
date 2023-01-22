use std::env;

use serenity::async_trait;
use serenity::framework::standard::StandardFramework;
use serenity::model::channel::{Message, Reaction};
use serenity::model::prelude::Ready;
use serenity::prelude::*;

use commands::snake;

mod commands;
mod utils;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        snake::message(ctx, msg).await;
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        snake::reaction_add(ctx, reaction).await;
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        // for x in ctx
        //     .http()
        //     .get_guild_roles(877165960124067850)
        //     .await
        //     .unwrap()
        //     .iter()
        // {
        //     x.guild_id
        //         .member(&ctx, 244786205873405952)
        //         .await
        //         .unwrap()
        //         .add_role(&ctx, x.id)
        //         .await
        //         .ok();
        // }
    }
}


#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(""));

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("TOKEN");
    let intents = GatewayIntents::all();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");
    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
