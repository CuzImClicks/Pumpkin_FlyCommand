use async_trait::async_trait;
use log::info;
use pumpkin::{
    PERMISSION_REGISTRY,
    command::{
        CommandExecutor, CommandSender,
        args::{
            Arg, ConsumedArgs,
            bounded_num::{BoundedNumArgumentConsumer, ToFromNumber},
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
};

const NAMES: [&str; 1] = ["fly"];
const DESCRIPTION: &str = "Gives you the ability to fly.";

const PERMISSION_NODE: &str = "fly_command:fly_command";

#[plugin_method]
async fn on_load(&mut self, server: &Context) -> Result<(), String> {
    pumpkin::init_log!();
    info!("Fly_Command plugin loaded!");

    let command = CommandTree::new(NAMES, DESCRIPTION)
        .then(argument_default_name(PlayersArgumentConsumer).execute(NoSpeedExecutor))
        .then(
            argument(
                "speed",
                BoundedNumArgumentConsumer::new().name("speed").min(0.0),
            )
            .execute(WithSpeedExecutor),
        );

    let perm = Permission::new(
        PERMISSION_NODE,
        "description",
        PermissionDefault::Op(PermissionLvl::Zero),
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
        let Some(Arg::Players(targets)) = args.get("players") else {
            return Err(CommandError::InvalidConsumption(Some(
                "players".to_string(),
            )));
        };

        for player in targets {
            let mut abilities_lock = player.abilities.lock().await;

            abilities_lock.allow_flying = !abilities_lock.allow_flying;
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
        let Some(Arg::Players(targets)) = args.get("players") else {
            return Err(CommandError::InvalidConsumption(Some(
                "players".to_string(),
            )));
        };

        let Some(Arg::Num(Ok(num))) = args.get("speed") else {
            return Err(CommandError::InvalidConsumption(Some("speed".to_string())));
        };

        for player in targets {
            let mut abilities_lock = player.abilities.lock().await;

            abilities_lock.allow_flying = true;
            let Some(speed) = f32::from_number(num) else {
                return Err(CommandError::InvalidConsumption(Some("speed".to_string())));
            };
            abilities_lock.fly_speed = speed;
            player.send_abilities_update().await;
        }
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
