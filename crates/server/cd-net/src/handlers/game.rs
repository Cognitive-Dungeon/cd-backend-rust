use crate::error::NetResult;
use crate::session::Session;
use cd_core::{Direction, WorldPos};
use cd_engine::{CommandSender, InputCmd};

pub async fn handle_move(session: Session, cmd_tx: CommandSender, dir: Direction) -> NetResult<()> {
    let guid = session.require_guid().await?;
    tracing::info!("Network: Player {} requested move {:?}", guid, dir);

    let cmd = InputCmd::Move {
        entity_guid: guid,
        direction: dir,
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

pub async fn handle_end_turn(session: Session, cmd_tx: CommandSender) -> NetResult<()> {
    let guid = session.require_guid().await?;

    tracing::info!("Network: Player {} requested End Turn", guid);

    cmd_tx
        .send(InputCmd::EndTurn { entity_guid: guid })
        .await
        .map_err(|_| crate::error::NetError::EngineDead)?;

    Ok(())
}
