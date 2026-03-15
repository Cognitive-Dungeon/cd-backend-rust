use crate::error::NetResult;
use crate::protocol::{ClientPacket, ServerPacket};
use crate::session::Session;
use cd_core::{ObjectGuid, WorldPos};
use cd_engine::{CommandSender, InputCmd};
use std::hash::{Hash, Hasher};

pub async fn handle_login(session: Session, cmd_tx: CommandSender, token: String) -> NetResult<()> {
    // 1. Логика генерации/проверки (пока dummy)
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    token.hash(&mut hasher);
    let guid = ObjectGuid::new(1, 1, 1, hasher.finish() as u32);

    // 2. Обновляем сессию
    session.set_authenticated(guid).await;

    // 3. Шлем команду движку
    let cmd = InputCmd::SpawnPlayer {
        entity_guid: guid,
        name: token,
    };
    cmd_tx
        .send(cmd)
        .await
        .map_err(|_| crate::error::NetError::EngineDead)?;

    // 4. Отвечаем клиенту
    let resp = ServerPacket::AuthSuccess {
        guid: guid.to_string(),
    };
    session.send_packet(resp).await;

    tracing::info!("Session authenticated: {:?}", guid);
    Ok(())
}
