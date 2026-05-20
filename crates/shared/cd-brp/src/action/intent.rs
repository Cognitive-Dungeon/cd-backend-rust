use cd_core::ObjectGuid;

/// То, что игрок шлет на сервер
pub enum CombatIntent {
    MeleeAttack {
        target_id: ObjectGuid,
        weapon_id: ObjectGuid,
    },
    RangedAttack {
        target_id: ObjectGuid,
        weapon_id: ObjectGuid,
        aim_bonus: bool,
    },
    Parry {
        weapon_id: ObjectGuid,
    },
    Dodge,
}
