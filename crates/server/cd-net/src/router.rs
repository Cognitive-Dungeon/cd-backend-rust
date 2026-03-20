use crate::error::NetResult;
use crate::handlers;
use crate::protocol::ClientPacket;
use crate::session::Session;
use cd_engine::CommandSender;

pub struct Router {
    cmd_tx: CommandSender,
}

impl Router {
    pub fn new(cmd_tx: CommandSender) -> Self {
        Self { cmd_tx }
    }

    pub async fn dispatch(&self, session: Session, packet: ClientPacket) -> NetResult<()> {
        // Маршрутизация
        match packet {
            ClientPacket::Login { token } => {
                handlers::auth::handle_login(session, self.cmd_tx.clone(), token).await
            }
            ClientPacket::Move { x, y } => {
                handlers::game::handle_move(session, self.cmd_tx.clone(), x, y).await
            }
            ClientPacket::Cast { spell } => {
                handlers::game::handle_cast(session, self.cmd_tx.clone(), spell).await
            }
        }
    }
}
