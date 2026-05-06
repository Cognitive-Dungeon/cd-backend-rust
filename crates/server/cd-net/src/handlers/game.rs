use crate::error::NetResult;
use crate::session::Session;
use cd_core::WorldPos;
use cd_engine::{CommandSender, InputCmd};

pub async fn handle_move(session: Session, cmd_tx: CommandSender, x: i32, y: i32) -> NetResult<()> {
    let guid = session.require_guid().await?;
    tracing::info!("Network: Player {} requested move to ({}, {})", guid, x, y);

    let cmd = InputCmd::Move {
        entity_guid: guid,
        target: WorldPos::new(x, y, 0),
    };

    cmd_tx
        .send(cmd)
        .await
        .map_err(|_| crate::error::NetError::EngineDead)?;

    Ok(())
}

pub async fn handle_cast(session: Session, cmd_tx: CommandSender, spell: String) -> NetResult<()> {
    let guid = session.require_guid().await?;

    cmd_tx
        .send(InputCmd::CastSpell {
            entity_guid: guid,
            spell_slug: spell,
        })
        .await
        .map_err(|_| crate::error::NetError::EngineDead)?;

    Ok(())
}
