use std::cmp::Ordering;

use anyhow::Result;
use rand::Rng;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::commands::{BotCommand, CommandStatus, InteractionUtil, SendEmbed};

/// A command to do an opposed roll following the Call of Cthulhu 7th Edition.
pub struct Op7Command;

#[derive(PartialEq)]
enum RollResult {
    ExtremeSuccess(i32),
    HardSuccess(i32),
    Success(i32),
    Failure(i32),
}

impl PartialOrd for RollResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            RollResult::ExtremeSuccess(num1) => match other {
                RollResult::ExtremeSuccess(num2) => Some(num1.cmp(num2)),
                RollResult::HardSuccess(_) => Some(Ordering::Greater),
                RollResult::Success(_) => Some(Ordering::Greater),
                RollResult::Failure(_) => Some(Ordering::Greater),
            },
            RollResult::HardSuccess(num1) => match other {
                RollResult::ExtremeSuccess(_) => Some(Ordering::Less),
                RollResult::HardSuccess(num2) => Some(num1.cmp(num2)),
                RollResult::Success(_) => Some(Ordering::Greater),
                RollResult::Failure(_) => Some(Ordering::Greater),
            },
            RollResult::Success(num1) => match other {
                RollResult::ExtremeSuccess(_) => Some(Ordering::Less),
                RollResult::HardSuccess(_) => Some(Ordering::Less),
                RollResult::Success(num2) => Some(num1.cmp(num2)),
                RollResult::Failure(_) => Some(Ordering::Greater),
            },
            RollResult::Failure(num1) => match other {
                RollResult::ExtremeSuccess(_) => Some(Ordering::Less),
                RollResult::HardSuccess(_) => Some(Ordering::Less),
                RollResult::Success(_) => Some(Ordering::Less),
                RollResult::Failure(num2) => Some(num1.cmp(num2)),
            },
        }
    }
}

impl ToString for RollResult {
    fn to_string(&self) -> String {
        match self {
            RollResult::ExtremeSuccess(_) => "극단적 성공".into(),
            RollResult::HardSuccess(_) => "어려운 성공".into(),
            RollResult::Success(_) => "보통 성공".into(),
            RollResult::Failure(_) => "실패".into(),
        }
    }
}

#[naming]
#[serenity::async_trait]
impl BotCommand for Op7Command {
    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("크툴루의 부름 7판 룰에 따라 대항 판정을 합니다.")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "status1",
                    "참가자 1의 특성치 (예: 근력 대항일 경우 근력 값)",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "status2",
                    "참가자 2의 특성치 (예: 민첩 대항일 경우 민첩 값)",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "bonus1",
                    "참가자 1의 보너스/패널티 주사위 (예: +1은 보너스, -1은 패널티)",
                )
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "bonus2",
                    "참가자 2의 보너스/패널티 주사위 (예: +1은 보너스, -1은 패널티)",
                )
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "name1",
                    "참가자 1의 이름",
                )
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "name2",
                    "참가자 2의 이름",
                )
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "comment",
                    "판정 설명",
                )
            )
    }

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<CommandStatus> {
        let status1 = interaction.get_int_option("status1".into()).unwrap();
        let status2 = interaction.get_int_option("status2".into()).unwrap();

        let bonus1 = interaction.get_int_option("bonus1".into()).unwrap_or(0);
        let bonus2 = interaction.get_int_option("bonus2".into()).unwrap_or(0);

        let name1 = interaction
            .get_string_option("name1".into())
            .unwrap_or("참가자 1");
        let name2 = interaction
            .get_string_option("name2".into())
            .unwrap_or("참가자 2");

        let comment = interaction
            .get_string_option("comment".into())
            .unwrap_or("대항 판정");

        fn roll_dice(status: i32, bonus: i32) -> (String, RollResult) {
            let mut rng = rand::thread_rng();
            let lower_digit = rng.gen_range(0..10);
            let results = (0..(1 + bonus.abs()))
                .map(|_| {
                    let res = rng.gen_range(0..10) * 10 + lower_digit;
                    if res == 0 {
                        100
                    } else {
                        res
                    }
                })
                .collect::<Vec<_>>();

            let (selected, selected_text) = if bonus == 0 {
                let res = results.iter().next().unwrap();
                (*res, res.to_string())
            } else if bonus > 0 {
                let minimum = results.iter().min().unwrap();
                let list = results
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                (*minimum, format!("min([{}]) = {}", list, minimum))
            } else {
                let maximum = results.iter().max().unwrap();
                let list = results
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                (*maximum, format!("max([{}]) = {}", list, maximum))
            };

            if selected <= status / 5 {
                (
                    format!(":trophy: {} <= {} / 5", selected_text, status),
                    RollResult::ExtremeSuccess(status),
                )
            } else if selected <= status / 2 {
                (
                    format!(":star: {} <= {} / 2", selected_text, status),
                    RollResult::HardSuccess(status),
                )
            } else if selected <= status {
                (
                    format!(":o: {} <= {}", selected_text, status),
                    RollResult::Success(status),
                )
            } else {
                (
                    format!(":x: {} > {}", selected_text, status),
                    RollResult::Failure(status),
                )
            }
        }

        let (mut result_text1, result1) = roll_dice(status1, bonus1);
        let (mut result_text2, result2) = roll_dice(status2, bonus2);

        let mut player1_won = result1 > result2;
        if result1 == result2 {
            let mut rng = rand::thread_rng();
            loop {
                let player1_roll = rng.gen_range(1..=100);
                let player2_roll = rng.gen_range(1..=100);

                result_text1 += format!(", {}", player1_roll).as_str();
                result_text2 += format!(", {}", player2_roll).as_str();

                if player1_roll != player2_roll {
                    player1_won = player1_roll < player2_roll;
                    break;
                }
            }
        }

        interaction
            .send_embed(
                ctx,
                CreateEmbed::new()
                    .title(comment)
                    .field(
                        format!(":first_place: {}", if player1_won { name1 } else { name2 }),
                        if player1_won {
                            &result_text1
                        } else {
                            &result_text2
                        },
                        false,
                    )
                    .field(
                        format!(":second_place: {}", if player1_won { name2 } else { name1 }),
                        if player1_won {
                            &result_text2
                        } else {
                            &result_text1
                        },
                        false,
                    ),
            )
            .await?;

        Ok(CommandStatus::Ok)
    }
}
