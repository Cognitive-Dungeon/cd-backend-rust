use bevy_ecs::prelude::*;
use cd_ecs::{
    Controller,
    components::{CombatBubble, InCombat},
};

use crate::systems::intents::IntentEndTurn;

pub fn combat_turn_system(
    mut reader: MessageReader<IntentEndTurn>,
    mut combatants: Query<&mut InCombat>,
    mut bubbles: Query<&mut CombatBubble>,
) {
    for intent in reader.read() {
        // 1. Проверяем, в бою ли сущность
        let Ok(in_combat) = combatants.get(intent.entity) else {
            tracing::warn!(
                "Entity {:?} tried to end turn, but is not in combat!",
                intent.entity
            );
            continue;
        };

        // 2. Получаем пузырь боя
        let Ok(mut bubble) = bubbles.get_mut(in_combat.bubble) else {
            continue;
        };

        // 3. Проверяем, его ли сейчас ход
        if bubble.current_actor() != Some(intent.entity) {
            tracing::warn!(
                "Entity {:?} tried to end turn, but it is NOT their turn!",
                intent.entity
            );
            continue;
        }

        // 4. Передаем ход следующему
        bubble.current_turn_idx = (bubble.current_turn_idx + 1) % bubble.turn_order.len();

        // Если круг замкнулся — начинается новый раунд
        if bubble.current_turn_idx == 0 {
            bubble.round += 1;
            tracing::info!(
                "Combat Bubble {:?} advanced to Round {}",
                in_combat.bubble,
                bubble.round
            );
        }

        let next_actor = bubble.current_actor().unwrap();
        tracing::info!("Turn passed! Next actor is {:?}", next_actor);

        // 5. Восстанавливаем Очки Действия (AP) и Движения (MP) новому ходящему
        if let Ok(mut next_in_combat) = combatants.get_mut(next_actor) {
            next_in_combat.action_points = 6;
            next_in_combat.movement_points = 10;
        }
    }
}

/// Простой ИИ: если сейчас ход NPC, он автоматически передает ход дальше.
pub fn npc_ai_system(
    bubbles: Query<&CombatBubble>,
    combatants: Query<(Entity, &InCombat, Option<&Controller>)>,
    mut intent_end_turn_writer: MessageWriter<IntentEndTurn>,
) {
    for bubble in bubbles.iter() {
        if let Some(actor) = bubble.current_actor()
            && let Ok((entity, _, controller)) = combatants.get(actor)
        {
            // Если у сущности НЕТ компонента Controller — значит это NPC
            if controller.is_none() {
                tracing::info!("NPC {:?} is thinking... and passes the turn!", entity);
                intent_end_turn_writer.write(IntentEndTurn { entity });
            }
        }
    }
}
