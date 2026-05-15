//! BRP UGE Skills System — Bevy ECS + MMO Ready
//!
//! Архитектура:
//! - SkillData: "сырые" данные навыка (реплицируются на клиент)
//! - CachedSkillChance: рассчитанный итоговый шанс (сервер → клиент)
//! - SkillTemporaryModifiers: временные баффы/дебаффы (сервер-авторитет)
//! - Event-Driven проверки навыков с детерминированным RNG

use bevy::app::{App, FixedUpdate, Plugin};
use bevy::ecs::prelude::*;
use bevy::ecs::resource::Resource;
use bevy::prelude::{Deref, DerefMut};
use bevy::reflect::Reflect;
use cd_core::ObjectGuid;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use tracing::{debug, trace, warn};

// ============================================================================
// Типобезопасные проценты
// ============================================================================

/// Обёртка над процентами навыков с валидацией диапазона.
/// Экономит память (i16 vs i32) и предотвращает ошибки типов.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[serde(transparent)]
pub struct SkillPercent(i16);

impl SkillPercent {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const MAX: Self = Self(200); // Лимит по BRP UGE (настраиваемый)
    pub const MIN: Self = Self(1); // Минимальный шанс (автоматический провал при 0)

    /// Создаёт валидированное значение с clamp в допустимый диапазон.
    #[inline]
    pub const fn new(value: i16) -> Self {
        let v = if value < Self::MIN.0 {
            Self::MIN.0
        } else if value > Self::MAX.0 {
            Self::MAX.0
        } else {
            value
        };
        Self(v)
    }

    /// Создаёт значение без валидации (для внутреннего использования/десериализации).
    #[inline]
    pub const fn new_unchecked(value: i16) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn get(self) -> i16 {
        self.0
    }

    #[inline]
    pub const fn clamp(self, min: i16, max: i16) -> Self {
        let v = if self.0 < min {
            min
        } else if self.0 > max {
            max
        } else {
            self.0
        };
        Self(v)
    }
}

impl Default for SkillPercent {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<i16> for SkillPercent {
    fn from(value: i16) -> Self {
        SkillPercent::new(value)
    }
}

impl From<SkillPercent> for i16 {
    fn from(value: SkillPercent) -> Self {
        value.0
    }
}

// ============================================================================
// Категории навыков и модификаторы
// ============================================================================

/// Категории навыков в соответствии с BRP UGE (стр. 18)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Combat,
    Communication,
    Manipulation,
    Mental,
    Perception,
    #[default]
    Physical,
}

impl SkillCategory {
    pub const COUNT: usize = 6;

    #[must_use]
    #[inline]
    pub const fn as_index(self) -> usize {
        match self {
            Self::Combat => 0,
            Self::Communication => 1,
            Self::Manipulation => 2,
            Self::Mental => 3,
            Self::Perception => 4,
            Self::Physical => 5,
        }
    }

    #[must_use]
    #[inline]
    pub const fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Combat),
            1 => Some(Self::Communication),
            2 => Some(Self::Manipulation),
            3 => Some(Self::Mental),
            4 => Some(Self::Perception),
            5 => Some(Self::Physical),
            _ => None,
        }
    }

    #[must_use]
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Combat => "combat",
            Self::Communication => "communication",
            Self::Manipulation => "manipulation",
            Self::Mental => "mental",
            Self::Perception => "perception",
            Self::Physical => "physical",
        }
    }
}

/// Предрассчитанные модификаторы категорий на основе характеристик.
/// Хранится как массив для эффективного доступа и масштабируемости.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct CategoryModifiers {
    #[reflect(ignore)]
    modifiers: [i32; SkillCategory::COUNT],
}

impl CategoryModifiers {
    /// Рассчитывает модификаторы по формулам BRP UGE (стр. 18).
    ///
    /// - Primary (DEX/INT): +1 за каждый пункт >10, -1 за каждый <10
    /// - Secondary (STR/POW/CON/CHA/EDU): +1 за каждые 2 пункта >10, -1 за каждые 2 <10
    /// - Negative (SIZ): -1 за каждый пункт >10, +1 за каждый <10
    #[must_use]
    pub fn calculate(chars: &crate::characteristics::Characteristics) -> Self {
        let primary = |val: i32| val - 10;
        let secondary = |val: i32| (val - 10) / 2; // Целочисленное деление: округление к нулю
        let negative = |val: i32| 10 - val;

        Self {
            modifiers: [
                primary(chars.dex) + secondary(chars.int) + secondary(chars.str), // Combat
                primary(chars.int) + secondary(chars.pow) + secondary(chars.cha), // Communication
                primary(chars.dex) + secondary(chars.int) + secondary(chars.str), // Manipulation
                primary(chars.int) + secondary(chars.pow) + secondary(chars.edu), // Mental
                primary(chars.int) + secondary(chars.pow) + secondary(chars.con), // Perception
                primary(chars.dex)
                    + secondary(chars.str)
                    + secondary(chars.con)
                    + negative(chars.siz), // Physical
            ],
        }
    }

    #[inline]
    #[must_use]
    pub fn get_modifier(&self, category: SkillCategory) -> i32 {
        self.modifiers[category.as_index()]
    }

    #[inline]
    #[must_use]
    pub const fn as_array(&self) -> &[i32; SkillCategory::COUNT] {
        &self.modifiers
    }

    /// Итератор по категориям и их модификаторам.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (SkillCategory, i32)> + '_ {
        SkillCategoryIter::all().map(|cat| (cat, self.get_modifier(cat)))
    }
}

impl std::ops::Index<SkillCategory> for CategoryModifiers {
    type Output = i32;
    #[inline]
    fn index(&self, category: SkillCategory) -> &Self::Output {
        &self.modifiers[category.as_index()]
    }
}

// Вспомогательный итератор по всем категориям
#[derive(Debug, Clone, Copy)]
pub struct SkillCategoryIter {
    index: usize,
}

impl SkillCategoryIter {
    #[inline]
    pub fn all() -> Self {
        Self { index: 0 }
    }
}

impl Iterator for SkillCategoryIter {
    type Item = SkillCategory;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < SkillCategory::COUNT {
            let cat = SkillCategory::from_index(self.index);
            self.index += 1;
            cat
        } else {
            None
        }
    }
}

// ============================================================================
// Данные навыка (компонент Bevy)
// ============================================================================

/// Уникальный идентификатор навыка (для репликации и контент-менеджмента).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, Component)]
pub struct SkillId(pub ObjectGuid);

impl SkillId {
    #[inline]
    #[must_use]
    pub fn new(guid: ObjectGuid) -> Self {
        Self(guid)
    }

    /// Хэш для детерминированного RNG (используем сырое значение GUID).
    #[inline]
    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.0.as_u64()
    }
}

/// Источник временного модификатора (для отслеживания и очистки).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Reflect)]
pub enum ModifierSource {
    Item(ObjectGuid),    // UUID предмета в инвентаре
    Spell(String),       // ID заклинания/эффекта
    Environment(String), // Погода, локация и т.д.
    Custom(String),      // Произвольный источник (для отладки/модов)
}

/// Временный модификатор навыка (бафф, дебафф, ситуационный бонус).
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TempModifier {
    pub source: ModifierSource,
    pub value: i16,
    /// Серверное время истечения (в тиках или секундах). None = постоянный.
    pub expires_at: Option<u64>,
}

impl TempModifier {
    #[inline]
    #[must_use]
    pub fn is_expired(&self, current_tick: u64) -> bool {
        self.expires_at.is_some_and(|exp| current_tick >= exp)
    }
}

/// Компонент: временные модификаторы навыка.
/// Хранится отдельно от базовых данных для эффективного обновления.
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct SkillTemporaryModifiers(pub Vec<TempModifier>);

// Ручная реализация Deref и DerefMut вместо макросов
impl std::ops::Deref for SkillTemporaryModifiers {
    type Target = Vec<TempModifier>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for SkillTemporaryModifiers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SkillTemporaryModifiers {
    /// Добавляет модификатор и автоматически очищает истёкшие.
    #[inline]
    pub fn add(&mut self, modifier: TempModifier, current_tick: u64) {
        self.0.retain(|m| !m.is_expired(current_tick));
        self.0.push(modifier);
    }

    /// Удаляет все модификаторы от указанного источника.
    #[inline]
    pub fn remove_by_source(&mut self, source: &ModifierSource) {
        self.0.retain(|m| &m.source != source);
    }

    /// Суммарное значение всех активных модификаторов.
    #[inline]
    #[must_use]
    pub fn total(&self, current_tick: u64) -> i16 {
        self.0
            .iter()
            .filter(|m| !m.is_expired(current_tick))
            .map(|m| m.value)
            .sum()
    }
}

/// Компонент: базовые данные навыка (реплицируются на клиент).
#[derive(Component, Clone, Serialize, Deserialize, Debug, Reflect)]
pub struct SkillData {
    pub id: SkillId,
    pub name: String,
    pub category: SkillCategory,
    pub base_chance: SkillPercent,
    pub allocated_points: SkillPercent,
    #[serde(default)]
    #[reflect(default)]
    pub is_default: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[reflect(default)]
    pub default_base: Option<SkillPercent>,
    #[serde(default)]
    #[reflect(default)]
    pub description: String,
    #[serde(default)]
    #[reflect(default)]
    pub tags: Vec<String>,
}

impl SkillData {
    /// Конструктор с валидацией.
    #[inline]
    #[must_use]
    pub fn new(
        id: ObjectGuid,
        name: impl Into<String>,
        category: SkillCategory,
        base_chance: i16,
    ) -> Self {
        Self {
            id: SkillId::new(id),
            name: name.into(),
            category,
            base_chance: SkillPercent::new(base_chance),
            allocated_points: SkillPercent::ZERO,
            is_default: false,
            default_base: None,
            description: String::new(),
            tags: Vec::new(),
        }
    }

    /// Builder-паттерн для удобного создания.
    #[inline]
    #[must_use]
    pub fn builder(
        id: ObjectGuid,
        name: impl Into<String>,
        category: SkillCategory,
        base: i16,
    ) -> SkillDataBuilder {
        SkillDataBuilder {
            data: Self::new(id, name, category, base),
        }
    }

    /// Эффективный базовый шанс (учитывает is_default/default_base).
    #[inline]
    #[must_use]
    pub fn effective_base(&self) -> SkillPercent {
        if self.allocated_points.get() > 0 || !self.is_default {
            self.base_chance
        } else {
            self.default_base.unwrap_or(SkillPercent::ZERO)
        }
    }

    /// Базовый шанс + вложенные очки (без модификаторов категорий/временных).
    #[inline]
    #[must_use]
    pub fn base_total(&self) -> i16 {
        self.effective_base().get() + self.allocated_points.get()
    }
}

/// Builder для SkillData.
#[derive(Debug, Clone)]
pub struct SkillDataBuilder {
    data: SkillData,
}

impl SkillDataBuilder {
    #[inline]
    pub fn points(mut self, value: i16) -> Self {
        self.data.allocated_points = SkillPercent::new(value);
        self
    }

    #[inline]
    pub fn default(mut self, base: Option<i16>) -> Self {
        self.data.is_default = true;
        self.data.default_base = base.map(SkillPercent::new);
        self
    }

    #[inline]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.data.description = desc.into();
        self
    }

    #[inline]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.data.tags.push(tag.into());
        self
    }

    #[inline]
    pub fn tags(mut self, tags: impl IntoIterator<Item = String>) -> Self {
        self.data.tags.extend(tags);
        self
    }

    #[inline]
    pub fn build(self) -> SkillData {
        self.data
    }
}

/// Компонент: кэшированный итоговый шанс навыка.
/// Рассчитывается системами, реплицируется на клиент только для отображения.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct CachedSkillChance(pub SkillPercent);

impl CachedSkillChance {
    #[inline]
    #[must_use]
    pub const fn new(value: i16) -> Self {
        Self(SkillPercent::new(value))
    }

    #[inline]
    #[must_use]
    pub fn get(self) -> i16 {
        self.0.get()
    }
}

// ============================================================================
// События: проверка навыков (Event-Driven архитектура)
// ============================================================================

/// Сложность проверки навыка (множитель к целевому значению).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum SkillDifficulty {
    /// ×1.0 — стандартная проверка
    Easy,
    /// ×0.5 — половина от навыка
    Medium,
    /// ×0.25 — четверть от навыка
    Hard,
    /// ×0.1 — экстремально сложно
    Critical,
}

impl SkillDifficulty {
    #[inline]
    #[must_use]
    pub const fn multiplier(self) -> f32 {
        match self {
            Self::Easy => 1.0,
            Self::Medium => 0.5,
            Self::Hard => 0.25,
            Self::Critical => 0.1,
        }
    }
}

/// Запрос на проверку навыка (от клиента → серверу).
#[derive(Event, Clone, Debug, Message)]
pub struct SkillCheckRequest {
    pub entity: Entity,
    pub skill_id: SkillId,
    pub difficulty: SkillDifficulty,
    pub context: Option<String>,
    pub client_tick: u64, // для анти-спуфинга
}

/// Результат проверки навыка (от сервера → клиенту).
#[derive(Event, Clone, Debug, Message)]
pub struct SkillCheckResult {
    pub entity: Entity,
    pub skill_id: SkillId,
    /// Выпавшее значение (1..100)
    pub rolled: u16,
    /// Целевое значение после всех модификаторов
    pub target: SkillPercent,
    /// Успех/неудача
    pub success: bool,
    /// Критический успех (≤5% от базы или 01)
    pub critical: bool,
    /// Фамбл (96-100 при неудаче)
    pub fumble: bool,
    pub context: Option<String>,
}

// ============================================================================
// Ресурсы и утилиты
// ============================================================================

/// Ресурс: кэш модификаторов категорий (пересчитывается при изменении характеристик).
#[derive(Resource, Clone, Copy, Debug)]
pub struct CategoryModifiersCache {
    pub value: CategoryModifiers,
    pub chars_version: u64,
}

impl Default for CategoryModifiersCache {
    fn default() -> Self {
        Self {
            value: CategoryModifiers {
                modifiers: [0; SkillCategory::COUNT],
            },
            chars_version: 0,
        }
    }
}

/// Текущий тик сервера (для детерминированного RNG и истечения баффов).
#[derive(Resource, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub struct ServerTick(pub u64);

/// Детерминированный RNG для сетевой синхронизации.
/// Использует SeaHash для скорости и детерминизма.
#[inline]
#[must_use]
pub fn deterministic_d100(entity_idx: u32, skill_hash: u64, tick: u64) -> u16 {
    use seahash::SeaHasher;

    let mut hasher = SeaHasher::with_seeds(0x1234_5678, 0x9ABC_DEF0, 0xFEDC_BA98, 0x7654_3210);
    entity_idx.hash(&mut hasher);
    skill_hash.hash(&mut hasher);
    tick.hash(&mut hasher);

    (hasher.finish() % 100 + 1) as u16
}

// ============================================================================
// Системы Bevy
// ============================================================================

/// Обновляет кэш модификаторов категорий при изменении характеристик.
/// Запускается только при реальном изменении (Changed<Characteristics>).
pub fn update_category_modifiers_cache(
    mut cache: ResMut<CategoryModifiersCache>,
    chars_query: Query<
        &crate::characteristics::Characteristics,
        Changed<crate::characteristics::Characteristics>,
    >,
) {
    if let Ok(chars) = chars_query.single() {
        let new_version = {
            use seahash::SeaHasher;
            let mut hasher = SeaHasher::new();
            chars.str.hash(&mut hasher);
            chars.con.hash(&mut hasher);
            chars.siz.hash(&mut hasher);
            chars.int.hash(&mut hasher);
            chars.pow.hash(&mut hasher);
            chars.dex.hash(&mut hasher);
            chars.cha.hash(&mut hasher);
            chars.edu.hash(&mut hasher);
            hasher.finish()
        };

        if cache.chars_version != new_version {
            cache.value = CategoryModifiers::calculate(chars);
            cache.chars_version = new_version;
            trace!("Category modifiers updated: {:?}", cache.value);
        }
    }
}

/// Пересчитывает кэшированные шансы навыков при изменении данных или модификаторов.
/// Использует ParIter для параллельной обработки в MMO-масштабе.
pub fn recalculate_skill_chances(
    cache: Res<CategoryModifiersCache>,
    tick: Res<ServerTick>,
    mut query: Query<
        (
            &SkillData,
            &mut CachedSkillChance,
            Option<&SkillTemporaryModifiers>,
        ),
        Or<(Changed<SkillData>, Changed<SkillTemporaryModifiers>)>,
    >,
) {
    query
        .par_iter_mut()
        .for_each(|(skill, mut cached, temp_mods)| {
            let category_mod = cache.value.get_modifier(skill.category);
            let base = skill.base_total();
            let temp = temp_mods.map_or(0, |m| m.total(**tick));

            let total = (base as i32 + category_mod + temp as i32)
                .clamp(SkillPercent::MIN.0 as i32, SkillPercent::MAX.0 as i32);

            cached.0 = SkillPercent::new(total as i16);
        });
}

/// Обрабатывает запросы на проверку навыков (ТОЛЬКО НА СЕРВЕРЕ).
/// Отправляет результаты обратно через события.
pub fn process_skill_checks(
    tick: Res<ServerTick>,
    mut requests: MessageReader<SkillCheckRequest>,
    mut results: MessageWriter<SkillCheckResult>,
    skills_query: Query<(&SkillData, &CachedSkillChance)>,
) {
    for req in requests.read() {
        let Ok((skill, cached)) = skills_query.get(req.entity) else {
            warn!("Entity {:?} not found for skill check", req.entity);
            continue;
        };

        if skill.id != req.skill_id {
            warn!(
                "Skill ID mismatch: requested {:?}, entity has {:?}",
                req.skill_id, skill.id
            );
            continue;
        }

        // Детерминированный бросок
        let rolled = deterministic_d100(
            req.entity.index_u32(),
            skill.id.as_u64(),
            **tick + req.client_tick,
        );

        // Применяем множитель сложности
        let mut target = cached.get() as f32 * req.difficulty.multiplier();
        target = target.clamp(1.0, SkillPercent::MAX.0 as f32);
        let target_int = target as i16;

        // Крит/фамбл по правилам BRP
        let base_chance = skill.effective_base().get();
        let critical_threshold = (base_chance / 20).max(1); // 5% от базы, минимум 1
        let critical = rolled as i16 <= critical_threshold || rolled == 1;
        let success = rolled as i16 <= target_int;
        let fumble = !success && rolled >= 96;

        results.write(SkillCheckResult {
            entity: req.entity,
            skill_id: req.skill_id,
            rolled,
            target: SkillPercent::new(target_int),
            success,
            critical,
            fumble,
            context: req.context.clone(),
        });

        debug!(
            "Skill check: {:?} = {} vs {} → success={}, crit={}, fumble={}",
            req.skill_id, rolled, target_int, success, critical, fumble
        );
    }
}

/// Очищает истёкшие временные модификаторы (ежекадрово или по таймеру).
pub fn cleanup_expired_modifiers(
    tick: Res<ServerTick>,
    mut query: Query<&mut SkillTemporaryModifiers>,
) {
    for mut mods in query.iter_mut() {
        mods.0.retain(|m| !m.is_expired(**tick));
    }
}

// ============================================================================
// Bevy Plugin
// ============================================================================

/// Набор систем для группировки и настройки порядка выполнения.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SkillsSet {
    /// Обновление кэшей и пересчёт значений
    Update,
    /// Обработка событий проверок
    ProcessChecks,
    /// Очистка устаревших данных
    Cleanup,
    /// Репликация на клиент (интегрируется с lightyear/bevy_replicon)
    Replicate,
}

/// Плагин системы навыков BRP для Bevy.
/// Добавляет компоненты, события, ресурсы и системы.
#[derive(Default)]
pub struct BrpSkillsPlugin;

impl Plugin for BrpSkillsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Ресурсы
            .init_resource::<CategoryModifiersCache>()
            .init_resource::<ServerTick>()
            // События
            .init_resource::<Messages<SkillCheckRequest>>()
            .init_resource::<Messages<SkillCheckResult>>()
            // Компоненты (регистрация для Reflect/сериализации)
            .register_type::<SkillCategory>()
            .register_type::<SkillData>()
            .register_type::<SkillTemporaryModifiers>()
            .register_type::<CachedSkillChance>()
            .register_type::<ModifierSource>()
            .register_type::<TempModifier>()
            .register_type::<SkillDifficulty>()
            // Системы
            .configure_sets(
                FixedUpdate,
                (
                    SkillsSet::Update,
                    SkillsSet::ProcessChecks,
                    SkillsSet::Cleanup,
                    SkillsSet::Replicate,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    // 1. Обновляем кэш модификаторов категорий
                    update_category_modifiers_cache.in_set(SkillsSet::Update),
                    // 2. Пересчитываем шансы навыков
                    recalculate_skill_chances.in_set(SkillsSet::Update),
                    // 3. Обрабатываем запросы проверок
                    process_skill_checks.in_set(SkillsSet::ProcessChecks),
                    // 4. Чистим истёкшие баффы
                    cleanup_expired_modifiers.in_set(SkillsSet::Cleanup),
                ),
            );
    }
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::characteristics::Characteristics;

    #[test]
    fn test_skill_percent_clamp() {
        assert_eq!(SkillPercent::new(-50).get(), 1);
        assert_eq!(SkillPercent::new(500).get(), 200);
        assert_eq!(SkillPercent::new(75).get(), 75);
    }

    #[test]
    fn test_category_index_roundtrip() {
        for i in 0..SkillCategory::COUNT {
            let cat = SkillCategory::from_index(i).unwrap();
            assert_eq!(cat.as_index(), i);
        }
        assert_eq!(SkillCategory::from_index(99), None);
    }

    #[test]
    fn test_brp_uge_example_page_18() {
        // Тест строго по примеру из BRP UGE, стр. 18:
        // "STR 14, CON 13, INT 8, SIZ 12, POW 10, DEX 12, CHA 8"
        let chars = Characteristics::new(14, 13, 12, 8, 10, 12, 8, 10);
        let mods = CategoryModifiers::calculate(&chars);

        assert_eq!(mods[SkillCategory::Combat], 3);
        assert_eq!(mods[SkillCategory::Communication], -3);
        assert_eq!(mods[SkillCategory::Manipulation], 3);
        assert_eq!(mods[SkillCategory::Mental], -2);
        assert_eq!(mods[SkillCategory::Perception], -1);
        assert_eq!(mods[SkillCategory::Physical], 3);
    }

    #[test]
    fn test_skill_builder() {
        let skill = SkillData::builder(
            ObjectGuid::new(1, 2, 3, 4),
            "Stealth",
            SkillCategory::Physical,
            30,
        )
        .points(15)
        .default(Some(10))
        .description("Hide in shadows")
        .tag("agility")
        .build();

        assert_eq!(skill.id.0, ObjectGuid::new(1, 2, 3, 4));
        assert_eq!(skill.base_chance.get(), 30);
        assert_eq!(skill.allocated_points.get(), 15);
        assert!(skill.is_default);
        assert_eq!(skill.default_base.unwrap().get(), 10);
        assert!(skill.tags.contains(&"agility".to_string()));
    }

    #[test]
    fn test_temp_modifiers_cleanup() {
        let mut mods = SkillTemporaryModifiers::default();
        let source = ModifierSource::Custom("test".into());

        mods.add(
            TempModifier {
                source: source.clone(),
                value: 10,
                expires_at: Some(100),
            },
            50,
        );
        mods.add(
            TempModifier {
                source: source.clone(),
                value: -5,
                expires_at: Some(75),
            },
            50,
        );

        assert_eq!(mods.total(50), 5); // 10 + (-5)
        assert_eq!(mods.total(80), 10); // -5 истёк
        assert_eq!(mods.total(100), 0); // оба истекли
    }

    #[test]
    fn test_deterministic_rng() {
        let r1 = deterministic_d100(42, SkillId::new(ObjectGuid::new(1, 2, 3, 4)).as_u64(), 1000);
        let r2 = deterministic_d100(42, SkillId::new(ObjectGuid::new(1, 2, 3, 4)).as_u64(), 1000);
        let r3 = deterministic_d100(42, SkillId::new(ObjectGuid::new(1, 2, 3, 4)).as_u64(), 1001);

        assert_eq!(r1, r2); // одинаковые входы → одинаковый выход
        assert_ne!(r1, r3); // разный tick → другой результат
        assert!((1..=100).contains(&r1)); // диапазон 1..100
    }
}
