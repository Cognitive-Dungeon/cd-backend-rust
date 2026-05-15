use bevy::ecs::message::{Message, MessageReader};
use bevy::ecs::prelude::*;

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
}

/// Система, которая слушает входящий урон и применяет его к анатомии.
pub fn apply_damage_system(
    mut damage_messages: MessageReader<DamageMessage>,
    mut anatomy_query: Query<&mut crate::anatomy::Anatomy, With<Damageable>>,
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
        let result = anatomy.apply_damage_detailed(
            message.location,
            message.raw_damage,
            message.penetration.clone(),
        );

        // В будущем здесь можно генерировать исходящие события (например, BloodSplatterEvent)
        // на основе результата (result.bleeding_added, result.pain_caused).
        tracing::debug!(
            "Damage applied to entity {:?} at {:?}: Result: {:?}",
            message.target,
            message.location,
            result
        );
    }
}
