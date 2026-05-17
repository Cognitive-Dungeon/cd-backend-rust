use bevy::ecs::prelude::*;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::anatomy::Anatomy;
use crate::characteristics::Characteristics;
use crate::encumbrance::EncumbrancePenalties;
use crate::rules::*;
use crate::skills::{SkillCategory, SkillPercent};

/// Компонент усталости (Stamina / Fatigue Points).
/// Отвечает за способность персонажа совершать интенсивные действия.
#[derive(Debug, Clone, Serialize, Deserialize, Component, Reflect)]
pub struct Fatigue {
    /// Текущее значение выносливости (f32 для плавного тика регенерации)
    pub current: f32,
    /// Максимальный запас выносливости
    pub max: f32,
    /// Флаг истощения (current <= 0). Влияет на навыки и передвижение.
    pub is_exhausted: bool,
}

impl Fatigue {
    /// Инициализирует выносливость по правилам BRP (STR + CON)
    pub fn new(chars: &Characteristics) -> Self {
        let max_fp = (chars.str + chars.con) as f32;
        Self {
            current: max_fp,
            max: max_fp,
            is_exhausted: false,
        }
    }

    /// Попытка потратить выносливость на действие (атака, бег, заклинание).
    /// Возвращает `true` если действие успешно, `false` если сил нет.
    pub fn try_consume(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            if self.current <= 0.0 {
                self.is_exhausted = true;
            }
            true
        } else {
            // Если сил не хватает, действие проваливается, а выносливость падает до 0
            self.current = 0.0;
            self.is_exhausted = true;
            false
        }
    }

    /// Процесс восстановления (или потери) выносливости с течением времени.
    pub fn process_tick(
        &mut self,
        delta_secs: f32,
        chars: &Characteristics,
        anatomy: Option<&Anatomy>,
        encumbrance: Option<&EncumbrancePenalties>,
    ) {
        // 1. Пассивный расход от сильного перегруза
        let mut passive_drain = 0.0;
        if let Some(enc) = encumbrance {
            if enc.level.causes_passive_stamina_drain() {
                passive_drain = FATIGUE_ENCUMBRANCE_DRAIN_RATE * delta_secs;
            }
        }

        if passive_drain > 0.0 {
            self.current = (self.current - passive_drain).max(0.0);
        } else {
            // 2. Регенерация (если нет пассивного расхода)
            let mut regen_rate = FATIGUE_BASE_REGEN_RATE + (chars.con as f32 * 0.1);

            // Симуляционные модификаторы от физиологии (если есть тело)
            if let Some(anat) = anatomy {
                if anat.substances.oxygen_saturation < FATIGUE_HYPOXIA_THRESHOLD {
                    regen_rate *= 0.5; // Задыхается — медленно восстанавливает силы
                }
                if anat.substances.hydration < FATIGUE_DEHYDRATION_THRESHOLD {
                    regen_rate *= 0.5; // Обезвожен
                }
            }

            self.current = (self.current + regen_rate * delta_secs).min(self.max);
        }

        // 3. Обновление флага истощения
        // Даем небольшой "буфер" перед снятием флага, чтобы он не мигал туда-сюда при 0.1 FP
        if self.current >= 1.0 {
            self.is_exhausted = false;
        } else if self.current <= 0.0 {
            self.is_exhausted = true;
        }
    }
}

// ============================================================================
// ECS Системы
// ============================================================================

/// Система, обновляющая выносливость с течением времени.
pub fn fatigue_tick_system(
    time: Res<bevy::time::Time>,
    mut query: Query<(
        &mut Fatigue,
        &Characteristics,
        Option<&Anatomy>,
        Option<&EncumbrancePenalties>,
    )>,
) {
    let delta = time.delta_secs();

    for (mut fatigue, chars, anatomy, encumbrance) in query.iter_mut() {
        fatigue.process_tick(delta, chars, anatomy, encumbrance);
    }
}

/// Система применения штрафов истощения к навыкам.
/// Работает аналогично `apply_encumbrance_to_cached_skills`.
pub fn apply_fatigue_to_cached_skills(
    fatigue_query: Query<(Entity, &Fatigue)>,
    mut skills_query: Query<(
        &mut crate::skills::CachedSkillChance,
        &crate::skills::SkillData,
    )>,
) {
    for (entity, fatigue) in fatigue_query.iter() {
        if fatigue.is_exhausted {
            if let Ok((mut cached, skill_data)) = skills_query.get_mut(entity) {
                // Штрафуем только те навыки, которые требуют физических усилий
                if matches!(
                    skill_data.category,
                    SkillCategory::Combat | SkillCategory::Physical | SkillCategory::Manipulation
                ) {
                    cached.0 = SkillPercent::new(cached.0.get() + FATIGUE_EXHAUSTION_SKILL_PENALTY);
                }
            }
        }
    }
}
