use bevy::ecs::message::Message;
use cd_core::ObjectGuid;
use serde::{Deserialize, Serialize};

use crate::{
    FumbleTableType, HitLocation, HitPoints, SpecialSuccessEffect, SuccessLevel,
    TemporaryInsanityType,
};

/// То, что сервер рассылает клиентам для применения изменений и отрисовки анимаций
#[derive(Message)]
pub enum CombatEffect {
    Missed {
        attacker_id: ObjectGuid,
        target_id: ObjectGuid,
    },
    Hit {
        target_id: ObjectGuid,
        damage_taken: HitPoints,
        armor_mitigated: HitPoints,
        special_applied: SpecialSuccessEffect,
        is_critical: bool,
        hit_location: Option<HitLocation>,
    },
    ItemDamaged {
        owner_id: ObjectGuid,
        item_id: ObjectGuid,
        damage: HitPoints,
    },
    FumbleApplied {
        entity_id: ObjectGuid,
        fumble_type: FumbleTableType,
    },
}

/// То, что сервер рассылает в брокер сообщений (Kafka/RabbitMQ) или напрямую в LLM-сервис.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NarrativeEvent {
    CombatHit {
        attacker_name: String,
        target_name: String,
        weapon_name: String,
        // Строгие данные для расчета
        damage_dealt: u16,
        hit_location: Option<HitLocation>, // Если включена опция

        // --- БОГАТЫЙ КОНТЕКСТ ДЛЯ LLM ---
        success_level: SuccessLevel, // Critical, Special и т.д.
        special_effect_applied: SpecialSuccessEffect, // Напр: Impaled (пронзил)

        // Факты симуляции (если включены опциональные правила)
        armor_penetrated: bool,
        bone_fractured: bool,
        major_wound_triggered: bool,
        knockback_meters: u8,
    },

    PsychologicalShock {
        target_name: String,
        sanity_lost: u16,
        temporary_insanity: Option<TemporaryInsanityType>, // Если провал > порога
    },
}
