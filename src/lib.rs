use std::sync::Arc;

use async_trait::async_trait;
use log::info;
use pumpkin::{
    command::{
        CommandExecutor, CommandSender,
        args::{
            Arg, ConsumedArgs, FindArgDefaultName,
            bounded_num::BoundedNumArgumentConsumer,
            players::PlayersArgumentConsumer,
        },
        dispatcher::CommandError,
        tree::{
            CommandTree,
            builder::{argument, argument_default_name},
        },
    },
    plugin::Context,
    server::Server,
};
use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault},
    text::TextComponent,
};

const NAMES: [&str; 1] = ["fly"];
const DESCRIPTION: &str = "Gives you the ability to fly.";

const PERMISSION_NODE: &str = "fly_command:fly_command";

fn speed_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::<f32>::new()
        .name("speed")
        .min(0.0)
}

#[plugin_method]
async fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
    pumpkin::init_log!();
    info!("Fly_Command plugin loaded!");

    let command = CommandTree::new(NAMES, DESCRIPTION)
        .then(
            argument_default_name(PlayersArgumentConsumer)
                .execute(NoSpeedExecutor)
                .then(argument("speed", speed_consumer()).execute(WithSpeedExecutor)),
        )
        .execute(BaseExecutor);

    let perm = Permission::new(
        PERMISSION_NODE,
        "description",
        PermissionDefault::Op(PermissionLvl::One),
    );
    server.register_permission(perm).await?;

    server.register_command(command, PERMISSION_NODE).await;
    Ok(())
}

struct NoSpeedExecutor;

#[async_trait]
impl CommandExecutor for NoSpeedExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender,
        _: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get("target") else {
            return Err(CommandError::InvalidConsumption(Some("target".to_string())));
        };

        for player in targets {
            {
                let mut abilities_lock = player.abilities.lock().await;

                abilities_lock.allow_flying = !abilities_lock.allow_flying;

                if !abilities_lock.allow_flying {
                    abilities_lock.flying = false;
                }
            }

            player.send_abilities_update().await;
        }
        Ok(())
    }
}

struct WithSpeedExecutor;

#[async_trait]
impl CommandExecutor for WithSpeedExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender,
        _: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get("target") else {
            return Err(CommandError::InvalidConsumption(Some("target".to_string())));
        };

        let Ok(Ok(speed)) = speed_consumer().find_arg_default_name(args) else {
            return Err(CommandError::InvalidConsumption(Some("speed".to_string())));
        };

        for player in targets {
            {
                let mut abilities_lock = player.abilities.lock().await;

                abilities_lock.allow_flying = true;
                abilities_lock.fly_speed = speed;
            }
            player.send_abilities_update().await;
        }
        Ok(())
    }
}

struct BaseExecutor;

#[async_trait]
impl CommandExecutor for BaseExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _: &Server,
        _: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(player) = sender.as_player() else {
            return Err(CommandError::CommandFailed(
                Box::new(TextComponent::text("Failed to get sender as player.")),
            ));
        };

        {
            let mut abilities_lock = player.abilities.lock().await;

            abilities_lock.allow_flying = !abilities_lock.allow_flying;

            if !abilities_lock.allow_flying {
                abilities_lock.flying = false;
            }
        }

        player.send_abilities_update().await;

        Ok(())
    }
}

// ---

#[plugin_impl]
pub struct FlyCommand {}

impl FlyCommand {
    pub fn new() -> Self {
        FlyCommand {}
    }
}

impl Default for FlyCommand {
    fn default() -> Self {
        Self::new()
    }
}
