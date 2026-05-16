use bevy::ecs::message::{Message, MessageReader};
use bevy::ecs::prelude::*;

use crate::anatomy::{AnatomyEvent, DamageInput};
use crate::{HitLocationType, anatomy::PenetrationProfile};

/// Компонент-маркер. Указывает, что сущность может получать урон по системе анатомии.
#[derive(Component)]
pub struct Damageable;

/// Сообщение о входящем уроне.
/// Генерируется боевой системой после того, как учтены уклонения и броски атаки.
#[derive(Message, Clone, Debug)]
pub struct DamageMessage {
    pub target: Entity,
    pub location: HitLocationType,
    pub raw_damage: i32,
    pub penetration: PenetrationProfile,
    pub timestamp_secs: f64,
}

/// Система, которая слушает входящий урон и применяет его к анатомии.
pub fn apply_damage_system(
    mut damage_messages: MessageReader<DamageMessage>,
    mut anatomy_query: Query<&mut crate::anatomy::Anatomy, With<Damageable>>,
    mut event_writer: MessageWriter<AnatomyEvent>,
) {
    for message in damage_messages.read() {
        // Пытаемся найти анатомию у цели. Если её нет или у неё нет Damageable — игнорируем.
        let Ok(mut anatomy) = anatomy_query.get_mut(message.target) else {
            tracing::warn!(
                "Target {:?} has no Anatomy or Damageable component!",
                message.target
            );
            continue;
        };

        // Передаем весь профиль проникновения (включая тип раны и глубину)
        let output = anatomy.apply_damage_detailed(DamageInput {
            location: message.location,
            raw_damage: message.raw_damage,
            profile: message.penetration.clone(),
            timestamp_secs: message.timestamp_secs,
        });

        for event in output.events {
            event_writer.write(event); // Рассылаем по шине ECS!
        }

        // В будущем здесь можно генерировать исходящие события (например, BloodSplatterEvent)
        // на основе результата (result.bleeding_added, result.pain_caused).
        tracing::debug!(
            "Damage applied to entity {:?} at {:?}: Result: {:?}",
            message.target,
            message.location,
            output.damage_result
        );
    }
}
