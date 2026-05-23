use anyhow::Result;
use log::{error, info};
use once_cell::sync::Lazy;
use serenity::builder::{
    CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::{Command, CommandInteraction};
use serenity::model::colour::Colour;
use serenity::prelude::Context;

use crate::commands::choose::ChooseCommand;
use crate::commands::create_sheet::{CSCommand, EditNameCommand, EditStatCommand, ShowSheetCommand};
use crate::commands::opposed::Op7Command;
use crate::commands::roll::RollCommand;
use crate::commands::skill::{SanRollCommand, SkillCommand, UseLuckCommand};
use crate::commands::desc::DescCommand;
use crate::commands::npc::NpcCommand;
use crate::logging::BotEventCounter;

/// Represents a handled result of the command.
/// Note that you cannot use this for internal errors.
pub enum CommandStatus {
    Ok,
    Err(String),
}

/// Represents a bot command.
#[serenity::async_trait]
pub trait BotCommand {
    /// Registers a command to Discord.
    fn create(&self) -> CreateCommand;

    /// Gets a name of the command.
    fn name(&self) -> &str;

    /// Executes the command.
    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<CommandStatus>;
}

/// The commands which can be invoked through the bot.
static REGISTERED_COMMANDS: Lazy<Vec<Box<dyn BotCommand + Sync + Send>>> = Lazy::new(|| {
    vec![
        Box::new(ChooseCommand),
        Box::new(CSCommand),
        Box::new(ShowSheetCommand),
        Box::new(EditNameCommand),
        Box::new(EditStatCommand),
        Box::new(RollCommand),
        Box::new(Op7Command),
        Box::new(SkillCommand),
        Box::new(UseLuckCommand),
        Box::new(SanRollCommand),
        Box::new(DescCommand),
        Box::new(NpcCommand),
    ]
});

/// Controls all of commands.
pub struct BotCommandManager;

impl BotCommandManager {
    /// Registers all commands to Discord.
    pub async fn register_all(ctx: &Context) -> Result<()> {
        let commands = REGISTERED_COMMANDS
            .iter()
            .map(|command| {
                info!("Registering /{}.", command.name());

                command.create()
            })
            .collect();

        Command::set_global_commands(ctx, commands).await?;

        info!("Registered all commands.");

        Ok(())
    }

    /// Executes a command.
    pub async fn run_command(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
        let mut command_executed = false;
        for command in REGISTERED_COMMANDS.iter() {
            let name = &interaction.data.name;
            if command.name() == name {
                if command_executed {
                    error!("Some commands are duplicated.");
                    return Ok(());
                }
                command_executed = true;

                let result = command.execute(ctx, interaction).await?;

                BotEventCounter::increment(name).await;

                if let CommandStatus::Err(message) = result {
                    Self::reply_error(ctx, interaction, message).await?;
                };
            }
        }
        if !command_executed {
            error!("Tried to execute an unknown command.");
        }
        Ok(())
    }

    /// Reports an error to the user.
    ///
    /// This method cannot be used to report an internal server error.
    async fn reply_error(
        ctx: &Context,
        interaction: &CommandInteraction,
        error: String,
    ) -> Result<()> {
        interaction
            .send_embed(
                ctx,
                CreateEmbed::default()
                    .title("ERROR")
                    .field("Message", error, false)
                    .colour(Colour::RED),
            )
            .await?;

        Ok(())
    }
}

/// An extension for `ApplicationCommandInteraction`.
pub trait InteractionUtil {
    /// Gets a nickname of the user who invoked the command.
    fn get_nickname(&self) -> String;

    /// Gets a value of option as `String`.
    fn get_string_option(&self, name: String) -> Option<&str>;

    /// Gets a value of option as `i32`.
    fn get_int_option(&self, name: String) -> Option<i32>;
}

impl InteractionUtil for CommandInteraction {
    fn get_nickname(&self) -> String {
        match &self.member {
            Some(member) => member.display_name().to_string(),
            None => self.user.name.clone(),
        }
    }

    fn get_string_option(&self, name: String) -> Option<&str> {
        self.data
            .options
            .iter()
            .find(|option| option.name == name)
            .map(|option| option.value.as_str().unwrap())
    }

    fn get_int_option(&self, name: String) -> Option<i32> {
        self.data
            .options
            .iter()
            .find(|option| option.name == name)
            .map(|option| option.value.as_i64().unwrap() as i32)
    }
}

/// An extension for `ApplicationCommandInteraction` to send an embed content easily.
#[serenity::async_trait]
pub trait SendEmbed<'l> {
    /// Sends an embed to the user.
    async fn send_embed(&'l self, ctx: &Context, embed: CreateEmbed) -> Result<()>;
}

#[serenity::async_trait]
impl<'l> SendEmbed<'l> for CommandInteraction {
    async fn send_embed(&'l self, ctx: &Context, embed: CreateEmbed) -> Result<()> {
        self.create_response(
            &ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::default().add_embed(embed),
            ),
        )
        .await?;
        Ok(())
    }
}

pub mod choose;
pub mod create_sheet;
pub mod opposed;
pub mod roll;
pub mod skill;
pub mod desc;
pub mod npc;
