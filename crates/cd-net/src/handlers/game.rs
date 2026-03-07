use crate::error::NetResult;
use crate::session::Session;
use cd_core::WorldPos;
use cd_engine::{CommandSender, InputCmd};

pub async fn handle_move(session: Session, cmd_tx: CommandSender, x: i32, y: i32) -> NetResult<()> {
    let guid = session.require_guid().await?;

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
