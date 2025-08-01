use crate::command_framework::command::Command;
use crate::command_framework::{Context, Event, Framework, Message};
use async_trait::async_trait;
use regex::Regex;
use tracing::{error, info, instrument};

#[derive(Debug)]
pub struct CommandFramework {
    commands: Vec<Box<dyn Command>>,
}

impl CommandFramework {
    pub fn new(commands: Vec<Box<dyn Command>>) -> Self {
        Self { commands }
    }
}

#[async_trait]
impl Framework for CommandFramework {
    async fn dispatch(&self, ctx: Context, event: Event) {
        dispatch_event(&self, ctx, event).await;
    }
}

async fn dispatch_event(framework: &CommandFramework, ctx: Context, event: Event) {
    match event {
        Event::Message(message) => {
            dispatch_message(framework, ctx, message).await;
        }
    }
}

#[instrument]
async fn dispatch_message(framework: &CommandFramework, ctx: Context, message: Message) {
    // TODO: configure prefix
    let reg = Regex::new(r"^!([A-Za-z-]+)(?:\s(.*))?").expect("Failed to compile regex");

    let captures = reg.captures(&*message.content).and_then(|caps| {
        Some((
            caps.get(1)?.as_str(),
            caps.get(2).and_then(|el| Some(el.as_str())),
        ))
    });

    info!("{:?}", captures);

    if let Some(captures) = captures {
        for command in &framework.commands {
            if command.name() == captures.0 {
                let _ = command.action(captures.1.map(|t| t.to_owned())).await; // TODO: handle errors
            }
        }
    }
}
