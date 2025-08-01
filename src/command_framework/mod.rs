pub mod command;
pub mod command_framework;

use async_trait::async_trait;
use std::fmt::Debug;
#[derive(Debug)]
pub struct User {
    pub id: String,
    pub login: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Message {
    pub id: String,
    pub user: User,
    pub content: String,
}

#[derive(Debug)]
pub enum Event {
    Message(Message),
}

#[derive(Debug)]
pub struct Context {}

#[async_trait]
pub trait Framework: Send + Sync + Debug {
    async fn dispatch(&self, ctx: Context, event: Event);
}
