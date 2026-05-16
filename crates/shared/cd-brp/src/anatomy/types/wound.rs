//! Система ран и результатов урона для симулятивной анатомии.
//!
//! Этот модуль реализует детализированное представление повреждений,
//! полученных персонажем в бою. В отличие от простого вычитания ХП,
//! система ран учитывает:
//! - Глубину проникновения атаки через ткани тела
//! - Тип урона и его специфичные эффекты (кровотечение, перелом, некроз)
//! - Тяжесть повреждения и функциональные последствия
//! - Риски вторичных осложнений (инфекция, шок, потеря сознания)
//!
//! # Архитектура данных
//! ```text
//! DamageEvent (входящий урон)
//! ├── PenetrationProfile: глубина + тип острия
//! │
//! ▼
//! Anatomy.apply_damage_detailed()
//! ├── Расчёт проникновения через TissueLayer[]
//! ├── Определение затронутых тканей и органов
//! ├── Создание Wound с параметрами
//! │   ├── severity: классификация тяжести
//! │   ├── bleeding_rate: скорость кровопотери
//! │   ├── pain_level: влияние на сознание
//! │   └── infection_risk: шанс заражения
//! │
//! ▼
//! DamageResult (возврат в боевую систему)
//! ├── damage_dealt: фактический урон по ХП
//! ├── bleeding_added: прирост кровопотери
//! └── pain_caused: болевой шок
//! ```
//!
//! # Источники
//! - *Basic Roleplaying UGE*, стр. 14-15, 149-151: "Major Wounds", "Weapon Special Effects"
//! - *Dwarf Fortress*: механика `WOUND` с параметрами `SIZE`, `SEVERITY`, `PAIN_LEVEL` [[8]]
//! - *CDDA*: система ран с кровотечением, болью, риском инфекции
//!
//! # Пример использования
//! ```rust,ignore
//! use crate::anatomy::{WoundType, WoundSeverity, PenetrationProfile};
//!
//! // Создание профиля проникающей атаки (копьё)
//! let penetration = PenetrationProfile {
//!     depth_mm: 45.0,          // Глубина проникновения 4.5 см
//!     tip_type: WoundType::Piercing,
//! };
//!
//! // Нанесение урона
//! let result = anatomy.apply_damage_detailed(
//!     HitLocationType::Chest,  // Попадание в грудь
//!     12,                       // Базовый урон
//!     WoundType::Piercing,     // Тип урона
//!     penetration.depth_mm,    // Проникновение
//! );
//!
//! // Обработка результата
//! match result {
//!     DamageResult::Hit { damage_dealt, bleeding_added, pain_caused } => {
//!         // Урон применён, можно отправить визуальные/звуковые эффекты
//!         ui.show_damage_indicator(damage_dealt);
//!         if bleeding_added > 0.0 {
//!             ui.show_bleeding_warning();
//!         }
//!     }
//!     DamageResult::Blocked => {
//!         // Броня остановила атаку
//!         ui.show_block_effect();
//!     }
//!     DamageResult::Missed => {
//!         // Промах (не должен происходить на этом этапе)
//!     }
//! }
//! ```

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::anatomy::{TissueType, WoundSeverity, WoundType};

// ============================================================================
// Профиль проникновения
// ============================================================================

/// Параметры проникающей атаки для расчёта урона по тканям.
///
/// # Назначение
/// `PenetrationProfile` описывает физические характеристики атаки,
/// необходимые для симуляции прохождения урона через слои тела:
/// - `depth_mm`: насколько глубоко атака может проникнуть
/// - `tip_type`: как тип урона влияет на взаимодействие с тканями
///
/// # Механика проникновения
/// При расчёте урона система последовательно "проходит" через ткани
/// в порядке от внешних к внутренним, уменьшая оставшуюся проникающую
/// способность на толщину каждого слоя:
///
/// ```rust,ignore
/// let mut remaining_penetration = profile.depth_mm;
///
/// for tissue_type in TISSUE_PENETRATION_ORDER {
///     let Some(tissue) = part.tissues.get(&tissue_type) else { continue; };
///     
///     if remaining_penetration <= 0.0 {
///         break; // Урон не достиг более глубоких слоёв
///     }
///     
///     // Урон ткани пропорционален проникновению
///     let tissue_damage = remaining_penetration / tissue.thickness;
///     tissue.integrity -= tissue_damage;
///     remaining_penetration -= tissue.thickness;
/// }
/// ```
///
/// # Взаимодействие с типами урона
/// | TipType | Эффект на проникновение | Пример оружия |
/// |---------|------------------------|---------------|
/// | `Blunt` | Распределение урона по площади, низкая глубина | Дубина, кулак |
/// | `Cutting` | Чистый разрез, средняя глубина, высокое кровотечение | Меч, топор |
/// | `Piercing` | Максимальная глубина, риск повреждения органов | Копьё, стрела |
/// | `Burning` | Игнор части брони, некроз тканей | Огонь, кислота |
/// | `Tearing` | Множественные мелкие раны, высокий болевой шок | Когти, взрыв |
/// | `Crushing` | Структурные повреждения, риск переломов | Молот, давление |
///
/// # Пример расчёта для разных типов
/// ```rust,ignore
/// // Колющее оружие: глубина 50 мм, тип Piercing
/// let spear = PenetrationProfile {
///     depth_mm: 50.0,
///     tip_type: WoundType::Piercing,
/// };
/// // Проникает: кожа(2мм) → жир(5мм) → мышца(10мм) → возможно орган
///
/// // Тупое оружие: глубина 15 мм, тип Blunt
/// let club = PenetrationProfile {
///     depth_mm: 15.0,
///     tip_type: WoundType::Blunt,
/// };
/// // Проникает: кожа(2мм) → жир(5мм) → часть мышцы, урон распределяется
/// ```
///
/// # Ссылки
/// - *BRP UGE*, стр. 149: "Weapon Special Effects" — влияние типа оружия
/// - *Dwarf Fortress*: `ATTACK_EDGE` и `ATTACK_CONTACT_AREA` для расчёта проникновения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenetrationProfile {
    /// Глубина проникновения атаки в миллиметрах.
    ///
    /// # Механика
    /// Значение представляет максимальную глубину, на которую атака
    /// может проникнуть в тело до полной остановки. Используется для:
    /// - Определения, какие ткани будут затронуты
    /// - Расчёта урона по каждому слою пропорционально проникновению
    /// - Оценки риска повреждения внутренних органов
    ///
    /// # Типичные значения
    /// | Источник | Глубина (мм) | Комментарий |
    /// |----------|-------------|-------------|
    /// | Кулак/удар | 5-15 | Зависит от силы и защиты |
    /// | Тупое оружие | 10-25 | Дубина, молот, камень |
    /// | Режущее оружие | 20-40 | Меч, топор (зависит от заточки) |
    /// | Колющее оружие | 30-80 | Копьё, кинжал, стрела |
    /// | Огнестрел | 50-200+ | Зависит от калибра и дистанции |
    /// | Взрыв/когти | 5-30 | Множественные поверхностные раны |
    ///
    /// # Модификаторы
    /// - **Сила атаки (STR + weapon_bonus)**: увеличивает эффективную глубину
    /// - **Броня цели**: уменьшает глубину на значение защиты
    /// - **Угол попадания**: косые удары имеют меньшую эффективную глубину
    ///
    /// # Пример расчёта с бронёй
    /// ```rust,ignore
    /// let effective_depth = penetration.depth_mm
    ///     - armor_value as f32  // Вычитаем защиту брони
    ///     * angle_factor;        // Угол попадания (1.0 = прямой, 0.5 = косой)
    /// ```
    pub depth_mm: f32,

    /// Тип урона, определяющий механизм взаимодействия с тканями.
    ///
    /// # Влияние на расчёт
    /// `tip_type` модифицирует:
    /// - Коэффициент проникновения через разные ткани
    /// - Вероятность специфичных эффектов (кровотечение, перелом)
    /// - Множитель боли и риска инфекции
    ///
    /// # Примеры модификаторов по типу
    /// ```rust,ignore
    /// let penetration_factor = match profile.tip_type {
    ///     WoundType::Piercing => 1.2,  // +20% к глубине
    ///     WoundType::Cutting => 1.0,   // Базовое значение
    ///     WoundType::Blunt => 0.7,     // -30% к глубине, +50% к урону по кости
    ///     WoundType::Burning => 0.5,   // Игнорирует кожу, прямой урон тканям
    ///     _ => 1.0,
    /// };
    ///
    /// let bleed_factor = match profile.tip_type {
    ///     WoundType::Cutting => 2.0,   // Двойное кровотечение
    ///     WoundType::Piercing => 1.5,  // Повышенное кровотечение
    ///     WoundType::Blunt => 0.3,     // Минимальное кровотечение
    ///     _ => 1.0,
    /// };
    /// ```
    ///
    /// # Тактическое применение
    /// - Против тяжёлой брони: `Blunt` или `Crushing` (игнорируют часть защиты)
    /// - Против незащищённых целей: `Cutting` или `Piercing` (макс. урон)
    /// - Для контроля: `Blunt` (оглушение без летального исхода)
    ///
    /// # Ссылки
    /// - *BRP UGE*, стр. 149: таблица "Weapon Special Effects"
    /// - *Dwarf Fortress*: `ATTACK_TYPE` и взаимодействие с материалами брони
    pub tip_type: WoundType,
}

impl PenetrationProfile {
    /// Создаёт профиль для тупой атаки с базовыми параметрами.
    ///
    /// # Возвращаемые значения
    /// - `depth_mm`: 15.0 мм (среднее для ударов без оружия)
    /// - `tip_type`: `WoundType::Blunt`
    ///
    /// # Использование
    /// Подходит для:
    /// - Ударов кулаками, ногами
    /// - Атак дубинами, молотами, камнями
    /// - Падений с высоты, ударов об окружение
    ///
    /// # Пример
    /// ```rust,ignore
    /// let fist_attack = PenetrationProfile::blunt();
    /// let result = anatomy.apply_damage_detailed(
    ///     location,
    ///     damage,
    ///     fist_attack.tip_type,
    ///     fist_attack.depth_mm,
    /// );
    /// ```
    #[must_use]
    pub fn blunt() -> Self {
        Self {
            depth_mm: 15.0,
            tip_type: WoundType::Blunt,
        }
    }

    /// Создаёт профиль для режущей атаки с базовыми параметрами.
    ///
    /// # Возвращаемые значения
    /// - `depth_mm`: 30.0 мм (среднее для мечей/топоров)
    /// - `tip_type`: `WoundType::Cutting`
    ///
    /// # Использование
    /// Подходит для:
    /// - Атак мечами, топорами, серпами
    /// - Порезов от острых краёв окружения
    /// - Специальных приёмов "slash" с повышенной глубиной
    ///
    /// # Особые правила
    /// Режущие атаки имеют повышенный шанс:
    /// - Кровотечения (`Bleeding`): ×2 к базовой скорости
    /// - Отсечения (`Severed`): при уроне ≥2× макс. ХП части
    ///
    /// # Пример
    /// ```rust,ignore
    /// let sword_attack = PenetrationProfile::cutting();
    /// // При попадании в конечность:
    /// // - Шанс кровотечения: 50%
    /// // - Шанс ампутации: если урон >= 2 * part.max_hp
    /// ```
    #[must_use]
    pub fn cutting() -> Self {
        Self {
            depth_mm: 30.0,
            tip_type: WoundType::Cutting,
        }
    }

    /// Создаёт профиль для колющей атаки с базовыми параметрами.
    ///
    /// # Возвращаемые значения
    /// - `depth_mm`: 50.0 мм (среднее для копий/кинжалов)
    /// - `tip_type`: `WoundType::Piercing`
    ///
    /// # Использование
    /// Подходит для:
    /// - Атак копьями, рапирами, кинжалами
    /// - Стрел, арбалетных болтов
    /// - Клыков, когтей с острым концом
    ///
    /// # Особые правила
    /// Колющие атаки:
    /// - Имеют максимальную глубину проникновения
    /// - Игнорируют 25-50% мягкой брони (кожа, ткань)
    /// - Высокий риск повреждения внутренних органов при попадании в торс
    ///
    /// # Пример расчёта риска для органов
    /// ```rust,ignore
    /// // При попадании в грудь с проникновением 50 мм:
    /// // Кожа(2) + Жир(5) + Мышца(10) = 17 мм до органов
    /// // Оставшееся проникновение: 50 - 17 = 33 мм → достаточное для сердца/лёгких
    ///
    /// if remaining_penetration > 20.0 && location == HitLocationType::Chest {
    ///     // Высокий риск повреждения критических органов
    ///     apply_organ_damage(OrganType::Heart, remaining_penetration);
    /// }
    /// ```
    #[must_use]
    pub fn piercing() -> Self {
        Self {
            depth_mm: 50.0,
            tip_type: WoundType::Piercing,
        }
    }

    /// Рассчитывает эффективную глубину с учётом брони и угла.
    ///
    /// # Параметры
    /// - `armor`: значение защиты брони в данной локации
    /// - `angle_factor`: множитель от угла попадания (1.0 = прямой, 0.0 = касательный)
    ///
    /// # Возвращаемое значение
    /// Эффективная глубина проникновения после применения модификаторов.
    ///
    /// # Формула расчёта
    /// ```math
    /// effective_depth = max(0, depth_mm - armor) × angle_factor × type_modifier
    /// ```
    /// где `type_modifier` зависит от `tip_type`:
    /// - `Piercing`: ×1.2 против мягкой брони, ×0.8 против лат
    /// - `Cutting`: ×1.0 базовое
    /// - `Blunt`: ×0.7 базовое, но ×1.5 урон по кости
    ///
    /// # Пример
    /// ```rust,ignore
    /// let profile = PenetrationProfile::piercing(); // depth=50, type=Piercing
    /// let effective = profile.effective_depth(10, 0.9); // броня 10, угол 90%
    /// // Расчёт: (50 - 10) × 0.9 × 1.2 = 43.2 мм
    /// ```
    #[must_use]
    pub fn effective_depth(&self, armor: i32, angle_factor: f32) -> f32 {
        let after_armor = (self.depth_mm - armor as f32).max(0.0);
        let type_modifier = match self.tip_type {
            WoundType::Piercing => 1.2,
            WoundType::Cutting => 1.0,
            WoundType::Blunt => 0.7,
            WoundType::Burning => 0.5, // Игнорирует часть брони
            _ => 1.0,
        };
        (after_armor * angle_factor * type_modifier).max(0.0)
    }
}

// ============================================================================
// Рана
// ============================================================================

/// Детализированное представление повреждения части тела.
///
/// # Архитектурное назначение
/// `Wound` хранит полную информацию о полученном повреждении для:
/// - Расчёта продолжающихся эффектов (кровотечение, боль, инфекция)
/// - Определения функциональных штрафов к персонажу
/// - Отслеживания прогрессии заживления или осложнений
/// - Визуализации состояния персонажа (интерфейс, анимации)
///
/// # Жизненный цикл раны
/// ```text
/// 1. Создание: при нанесении урона в apply_damage_detailed()
///    ├── Определяется затронутые ткани
///    ├── Рассчитывается тяжесть (severity)
///    ├── Инициализируются параметры (боль, кровотечение, риск)
///
/// 2. Активная фаза: каждый игровой тик
///    ├── Кровотечение: отнимает ХП, если не остановлено
///    ├── Боль: влияет на сознание, штрафы к действиям
///    ├── Инфекция: прогрессия по шансу infection_risk
///
/// 3. Заживление или осложнения
///    ├── Лечение: снижение severity, остановка кровотечения
///    ├── Естественное восстановление: только для Minor ран
///    ├── Осложнения: инфекция → сепсис, некроз → ампутация
///
/// 4. Разрешение
///    ├── Зажившая: удаление из списка, возможен шрам (косметика)
///    ├── Хроническая: перманентный штраф к функции
///    ├── Фатальная: смерть персонажа
/// ```
///
/// # Взаимодействие с другими системами
/// ```text
/// Anatomy.parts[location].wounds: Vec<Wound>
/// ├── update_vitals_system:
/// │   ├── Суммирует pain_level → anatomy.vitals.pain
/// │   ├── Суммирует bleeding_rate → anatomy.substances.blood_loss_rate
/// │   └── Проверяет infection_risk → добавляет патогены
/// │
/// ├── healing_tick_system:
/// │   ├── Снижает severity для Minor ран
/// │   ├── Уменьшает bleeding_rate при лечении
/// │   └── Проверяет infection_progression
/// │
/// └── action_points.rs:
///     ├── Проверяет severity → штраф к доступным действиям
///     └── Проверяет affected_tissues → специфичные ограничения
/// ```
///
/// # Пример создания раны
/// ```rust,ignore
/// // При нанесении колющего урона в ногу
/// let wound = Wound {
///     wound_type: WoundType::Piercing,
///     severity: WoundSeverity::Inhibited, // Ограничивает функцию
///     affected_tissues: vec![TissueType::Skin, TissueType::Muscle],
///     depth: 25.0, // 2.5 см проникновения
///     bleeding_rate: 0.8, // 0.8 мл/сек кровопотери
///     pain_level: 35.0, // Уровень боли
///     infection_risk: 0.15, // 15% шанс заражения
///     created_at: current_timestamp(),
/// };
///
/// body_part.wounds.push(wound);
/// ```
///
/// # Ссылки
/// - *BRP UGE*, стр. 15: "Major Wounds" и их последствия
/// - *Dwarf Fortress*: `WOUND` с параметрами `SIZE`, `PAIN_LEVEL`, `INFECTION_CHANCE`
/// - *CDDA*: система ран с кровотечением, болью, риском заражения
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Wound {
    /// Тип урона, вызвавшего рану.
    ///
    /// # Назначение
    /// Определяет:
    /// - Визуальное представление раны (спрайт, частицы, звук)
    /// - Специфичные эффекты (например, `Burning` → некроз)
    /// - Методы лечения (разные типы требуют разных средств)
    ///
    /// # Влияние на геймплей
    /// | WoundType | Особый эффект | Лечение |
    /// |-----------|--------------|---------|
    /// | `Blunt` | Риск перелома, ушиб | Покой, иммобилизация |
    /// | `Cutting` | Повышенное кровотечение | Бинты, жгут, хирургия |
    /// | `Piercing` | Риск повреждения органов | Хирургия, диагностика |
    /// | `Burning` | Некроз тканей, продолж. урон | Мази, магия, ампутация |
    /// | `Tearing` | Множественные раны, высокий шок | Комплексное лечение |
    /// | `Crushing` | Структурные повреждения, оглушение | Репозиция, поддержка |
    ///
    /// # Пример проверки типа
    /// ```rust,ignore
    /// match wound.wound_type {
    ///     WoundType::Cutting | WoundType::Piercing => {
    ///         // Повышенный риск кровотечения
    ///         if wound.bleeding_rate > 0.5 {
    ///             ui.show_bleeding_warning();
    ///         }
    ///     }
    ///     WoundType::Burning => {
    ///         // Риск некроза — требуется срочное лечение
    ///         if wound.severity >= WoundSeverity::Inhibited {
    ///             quest_tracker.add_objective("Treat burn wound");
    ///         }
    ///     }
    ///     _ => {}
    /// }
    /// ```
    pub wound_type: WoundType,

    /// Степень тяжести раны.
    ///
    /// # Шкала тяжести
    /// ```
    /// None < Minor < Inhibited < FunctionLoss < Broken < Missing
    /// ```
    ///
    /// # Геймплейные эффекты по уровням
    /// | Severity | Штраф к функции | Боль | Лечение |
    /// |----------|----------------|------|---------|
    /// | `None` | 0% | 0 | Не требуется |
    /// | `Minor` | -5% | +10 | Заживает само за 1-2 дня |
    /// | `Inhibited` | -50% | +25 | Требует лечения, риск ухудшения |
    /// | `FunctionLoss` | -100% | +50 | Срочное лечение, риск смерти |
    /// | `Broken` | Структурное разрушение | +75 | Хирургия, долгая реабилитация |
    /// | `Missing` | Часть утрачена | 0* | Только протез/магия |
    ///
    /// *Ампутированная часть не болит, но может вызывать фантомные боли
    ///
    /// # Определение тяжести при создании
    /// ```rust,ignore
    /// let severity = if part.is_destroyed {
    ///     WoundSeverity::Missing
    /// } else if part.is_useless {
    ///     WoundSeverity::FunctionLoss
    /// } else if affected_tissues.contains(&TissueType::Bone) {
    ///     WoundSeverity::Broken
    /// } else if damage_ratio > 0.5 {
    ///     WoundSeverity::Inhibited
    /// } else {
    ///     WoundSeverity::Minor
    /// };
    /// ```
    ///
    /// # Взаимодействие с системой действий
    /// ```rust,ignore
    /// // В action_points.rs
    /// let penalty = match wound.severity {
    ///     WoundSeverity::None | WoundSeverity::Minor => 0,
    ///     WoundSeverity::Inhibited => -50, // -50% к действиям с этой частью
    ///     WoundSeverity::FunctionLoss | WoundSeverity::Broken => -100, // Невозможно
    ///     WoundSeverity::Missing => 0, // Часть отсутствует, другие штрафы
    /// };
    /// action_points = (action_points + penalty).max(0);
    /// ```
    pub severity: WoundSeverity,

    /// Список типов тканей, затронутых раной.
    ///
    /// # Назначение
    /// Позволяет:
    /// - Точно определить функциональные последствия (какие ткани повреждены)
    /// - Рассчитать суммарную боль по `pain_receptors` каждой ткани
    /// - Определить специфичные риски (например, повреждение `Artery` = кровотечение)
    ///
    /// # Порядок и дубликаты
    /// Список может содержать несколько вхождений одного типа,
    /// если рана затронула ткань в нескольких местах. Порядок не важен.
    ///
    /// # Примеры комбинаций
    /// ```rust,ignore
    /// // Поверхностный порез
    /// vec![TissueType::Skin]
    ///
    /// // Глубокий порез мышцы
    /// vec![TissueType::Skin, TissueType::Fat, TissueType::Muscle]
    ///
    /// // Колющее ранение с повреждением сосуда
    /// vec![TissueType::Skin, TissueType::Muscle, TissueType::Artery]
    ///
    /// // Перелом с повреждением нерва
    /// vec![TissueType::Skin, TissueType::Muscle, TissueType::Bone, TissueType::Nerve]
    /// ```
    ///
    /// # Расчёт боли по затронутым тканям
    /// ```rust,ignore
    /// // В BodyPart::calculate_pain()
    /// let total_pain: f32 = wound.affected_tissues.iter()
    ///     .filter_map(|t| self.tissues.get(t))
    ///     .map(|t| t.pain_receptors)
    ///     .sum::<f32>()
    ///     * wound.severity.pain_multiplier(); // ×1/2/3 в зависимости от тяжести
    /// ```
    ///
    /// # Проверка специфичных эффектов
    /// ```rust,ignore
    /// // Проверка на риск паралича (повреждение нерва)
    /// if wound.affected_tissues.contains(&TissueType::Nerve)
    ///     && wound.severity >= WoundSeverity::Inhibited {
    ///     body_part.is_useless = true;
    /// }
    ///
    /// // Проверка на массивное кровотечение (повреждение артерии)
    /// if wound.affected_tissues.contains(&TissueType::Artery) {
    ///     anatomy.substances.blood_loss_rate += wound.bleeding_rate * 3.0;
    /// }
    /// ```
    pub affected_tissues: Vec<TissueType>,

    /// Глубина проникновения раны в миллиметрах.
    ///
    /// # Назначение
    /// - Визуализация: глубина влияет на отображение раны
    /// - Лечение: глубокие раны сложнее заживают, требуют хирургии
    /// - Осложнения: глубина >30 мм = повышенный риск повреждения органов
    ///
    /// # Пороговые значения
    /// | Глубина | Эффект |
    /// |---------|--------|
    /// | 0-10 мм | Поверхностная, заживает сама |
    /// | 10-25 мм | Средняя, требует базового лечения |
    /// | 25-40 мм | Глубокая, риск повреждения органов |
    /// | >40 мм | Критическая, срочное хирургическое вмешательство |
    ///
    /// # Взаимодействие с локацией
    /// Одна и та же глубина имеет разные последствия в разных зонах:
    /// ```rust,ignore
    /// match location {
    ///     HitLocationType::Head => {
    ///         // Череп ~10 мм, мозг сразу за ним
    ///         if depth > 12.0 { risk_brain_damage = true; }
    ///     }
    ///     HitLocationType::Chest => {
    ///         // Грудная стенка ~20 мм, затем органы
    ///         if depth > 25.0 { risk_organ_damage = true; }
    ///     }
    ///     HitLocationType::RightArm => {
    ///         // Рука ~40 мм до кости
    ///         if depth > 45.0 { risk_bone_fracture = true; }
    ///     }
    ///     _ => {}
    /// }
    /// ```
    pub depth: f32,

    /// Скорость кровотечения в миллилитрах в секунду.
    ///
    /// # Механика кровопотери
    /// Значение добавляется к глобальной `blood_loss_rate` персонажа:
    /// ```rust,ignore
    /// // В update_vitals_system
    /// anatomy.substances.blood_loss_rate += wound.bleeding_rate;
    ///
    /// // Каждый игровой тик (например, 0.1 сек)
    /// let blood_lost = anatomy.substances.blood_loss_rate * 0.1;
    /// anatomy.substances.blood_volume -= blood_lost;
    /// ```
    ///
    /// # Типичные значения
    /// | Тип повреждения | bleeding_rate (мл/сек) | Комментарий |
    /// |----------------|------------------------|-------------|
    /// | Поверхностный порез | 0.1-0.3 | Капиллярное, останавливается само |
    /// | Глубокий порез мышцы | 0.5-1.5 | Требует бинтов |
    /// | Повреждение вены | 1.0-3.0 | Давящая повязка обязательна |
    /// | Разрыв артерии | 3.0-10.0+ | Жгут или смерть за минуты |
    /// | Внутреннее кровотечение | 0.5-2.0 | Сложно диагностировать |
    ///
    /// # Факторы влияния
    /// - **Тип ткани**: `Artery` ×5, `Vein` ×2, `Muscle` ×1, `Bone` ×0
    /// - **Тяжесть раны**: `Broken`/`Missing` увеличивают скорость
    /// - **Лечение**: бинты/жгуты устанавливают `bleeding_rate = 0`
    ///
    /// # Остановка кровотечения
    /// ```rust,ignore
    /// // Первая помощь: бинты
    /// if skill_check(FirstAid, difficulty) {
    ///     wound.bleeding_rate = 0.0;
    ///     ui.show_message("Bleeding stopped");
    /// }
    ///
    /// // Экстренно: жгут (только для конечностей)
    /// if location.is_limb() && wound.bleeding_rate > 2.0 {
    ///     wound.bleeding_rate = 0.0;
    ///     // Но: риск некроза через 2 часа без снятия
    ///     wound.infection_risk += 0.2;
    /// }
    /// ```
    ///
    /// # Критические пороги кровопотери
    /// | Потеря крови | Эффект |
    /// |--------------|--------|
    /// | 0-15% (150 мл) | Нет симптомов |
    /// | 15-30% (150-300 мл) | Тахикардия, штраф к физическим действиям |
    /// | 30-40% (300-400 мл) | Гиповолемия, спутанность сознания |
    /// | 40-50% (400-500 мл) | Шок, потеря сознания |
    /// | >50% (>500 мл) | Смерть без немедленного вмешательства |
    pub bleeding_rate: f32,

    /// Уровень боли, причиняемой раной (0.0-200.0).
    ///
    /// # Механика боли
    /// Боль рассчитывается при создании раны и может меняться:
    /// ```rust,ignore
    /// // Базовый расчёт
    /// let base_pain = affected_tissues.iter()
    ///     .filter_map(|t| tissues.get(t))
    ///     .map(|t| t.pain_receptors)
    ///     .sum::<f32>();
    ///
    /// let severity_mult = match severity {
    ///     WoundSeverity::Minor => 1.0,
    ///     WoundSeverity::Inhibited => 2.0,
    ///     WoundSeverity::FunctionLoss | WoundSeverity::Broken => 3.0,
    ///     _ => 0.0,
    /// };
    ///
    /// pain_level = (base_pain * severity_mult * (depth / 10.0).min(1.0))
    ///     .clamp(0.0, 200.0);
    /// ```
    ///
    /// # Влияние на геймплей
    /// | Боль | Эффект |
    /// |------|--------|
    /// | 0-25 | Нет штрафов |
    /// | 25-50 | -10% к точности действий, лёгкий дискомфорт |
    /// | 50-100 | -25% к навыкам, штраф к концентрации |
    /// | 100-150 | -50% к действиям, риск потери сознания при нагрузке |
    /// | >150 | Автоматическая потеря сознания (по правилам DF) |
    ///
    /// # Управление болью
    /// ```rust,ignore
    /// // Обезболивающие
    /// if has_medication(MedicationType::Analgesic) {
    ///     wound.pain_level *= 0.5; // Снижение на 50%
    /// }
    ///
    /// // Естественное снижение со временем
    /// if !wound.is_bleeding() && wound.severity <= WoundSeverity::Minor {
    ///     wound.pain_level *= 0.95; // -5% за тик
    /// }
    /// ```
    ///
    /// # Фантомная боль
    /// При ампутации (`Missing`) рана может периодически вызывать
    /// фантомную боль (проверка каждые 24 игровых часа):
    /// ```rust,ignore
    /// if wound.severity == WoundSeverity::Missing && rng.random_bool(0.1) {
    ///     anatomy.vitals.pain += 20.0; // Внезапный приступ
    ///     ui.show_message("Phantom pain...");
    /// }
    /// ```
    pub pain_level: f32,

    /// Вероятность заражения раны (0.0-1.0).
    ///
    /// # Механика инфекции
    /// Каждый игровой час проводится проверка:
    /// ```rust,ignore
    /// // В infection_progress_system
    /// if rng.random::<f32>() < wound.infection_risk {
    ///     // Заражение произошло
    ///     anatomy.substances.pathogens.entry(PathogenId::Bacteria)
    ///         .or_insert_with(|| Infection {
    ///             pathogen: PathogenId::Bacteria,
    ///             virulence: 0.4,
    ///             incubation_remaining: 12.0, // 12 часов до симптомов
    ///             symptoms: vec![InfectionSymptom::Fever(0.3)],
    ///         });
    /// }
    /// ```
    ///
    /// # Факторы риска
    /// | Фактор | Влияние на infection_risk |
    /// |--------|--------------------------|
    /// | Тип урона: `Cutting`/`Piercing` | +0.1 (открытая рана) |
    /// | Тип урона: `Burning` | -0.1 (стерилизация огнём) |
    /// | Глубина >30 мм | +0.15 (сложнее очистить) |
    /// | Загрязнённая среда | +0.2 (грязь, вода, трупы) |
    /// | Наличие лечения | -0.3 (антисептики, магия) |
    /// | Иммунитет персонажа | Множитель: `(POW + CON) / 20` |
    ///
    /// # Профилактика и лечение
    /// ```rust,ignore
    /// // Очистка раны (первая помощь)
    /// if skill_check(FirstAid, difficulty) {
    ///     wound.infection_risk *= 0.3; // Снижение на 70%
    /// }
    ///
    /// // Антисептики
    /// if has_item(ItemType::Antiseptic) {
    ///     wound.infection_risk = 0.0; // Полная защита
    /// }
    ///
    /// // Магия исцеления
    /// if cast_spell(Spell::Purify) {
    ///     wound.infection_risk = 0.0;
    ///     if wound.severity == WoundSeverity::Minor {
    ///         wound.severity = WoundSeverity::None; // Мгновенное заживление
    ///     }
    /// }
    /// ```
    ///
    /// # Прогрессия инфекции
    /// При заражении симптомы развиваются:
    /// ```text
    /// Инкубация (0-12 часов) → Лихорадка → Воспаление →
    /// → (без лечения) Некроз/Сепсис → Смерть
    /// ```
    ///
    /// # Пример проверки риска
    /// ```rust,ignore
    /// // Предупреждение игроку
    /// if wound.infection_risk > 0.3 && !wound.is_treated() {
    ///     ui.show_warning("Wound is at high risk of infection!");
    ///     quest_tracker.add_objective("Clean and bandage the wound");
    /// }
    /// ```
    pub infection_risk: f32,

    /// Время создания раны (таймстемп в секундах).
    ///
    /// # Назначение
    /// - Отслеживание длительности раны для расчёта заживления
    /// - Определение приоритета лечения (старые раны = выше риск осложнений)
    /// - Логирование и отладка боевых событий
    ///
    /// # Использование
    /// ```rust,ignore
    /// // Проверка: рана старше 24 часов без лечения
    /// let age_hours = (current_time - wound.created_at) / 3600.0;
    /// if age_hours > 24.0 && wound.severity >= WoundSeverity::Inhibited {
    ///     // Повышенный риск инфекции/некроза
    ///     wound.infection_risk *= 1.5;
    /// }
    ///
    /// // Расчёт прогресса заживления
    /// let healing_progress = (current_time - wound.created_at) as f32
    ///     * healing_rate(wound.severity, treatment_quality);
    /// ```
    ///
    /// # Формат времени
    /// Рекомендуется использовать:
    /// - Игровые секунды (для детерминизма и воспроизводимости)
    /// - Или `std::time::SystemTime` для реального времени в single-player
    ///
    /// # Пример получения текущего времени
    /// ```rust,ignore
    /// // В Bevy с ресурсом времени
    /// use bevy::time::Time;
    ///
    /// fn get_timestamp(time: Res<Time>) -> f64 {
    ///     time.elapsed_secs_f64()
    /// }
    /// ```
    pub created_at: f64,
}

impl Wound {
    /// Проверяет, является ли рана активной (требует внимания).
    ///
    /// # Возвращаемое значение
    /// - `true`: рана кровоточит, болит или имеет риск инфекции
    /// - `false`: рана зажила или не требует лечения
    ///
    /// # Использование
    /// Полезно для:
    /// - ИИ: определение необходимости лечения/отступления
    /// - Интерфейс: отображение активных ран игроку
    /// - Оптимизация: пропуск расчётов для заживших ран
    ///
    /// # Пример
    /// ```rust,ignore
    /// // В системе обновления виталов
    /// for wound in &body_part.wounds {
    ///     if wound.is_active() {
    ///         anatomy.vitals.pain += wound.pain_level;
    ///         anatomy.substances.blood_loss_rate += wound.bleeding_rate;
    ///     }
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub fn is_active(&self) -> bool {
        self.severity > WoundSeverity::None || self.bleeding_rate > 0.0 || self.infection_risk > 0.0
    }

    /// Проверяет, остановлено ли кровотечение.
    ///
    /// # Возвращаемое значение
    /// - `true`: `bleeding_rate == 0.0` — кровь не теряется
    /// - `false`: активная кровопотеря
    ///
    /// # Использование
    /// ```rust,ignore
    /// // Проверка перед применением жгута
    /// if !wound.is_bleeding_stopped() && location.is_limb() {
    ///     // Можно применить жгут
    ///     wound.bleeding_rate = 0.0;
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub fn is_bleeding_stopped(&self) -> bool {
        self.bleeding_rate <= 0.0
    }

    /// Рассчитывает текущий уровень боли с учётом лечения.
    ///
    /// # Параметры
    /// - `pain_medication`: множитель обезболивания (1.0 = нет, 0.5 = ×50%)
    ///
    /// # Возвращаемое значение
    /// Актуальный уровень боли после применения модификаторов.
    ///
    /// # Пример
    /// ```rust,ignore
    /// // С обезболивающим
    /// let current_pain = wound.calculate_current_pain(0.5); // -50%
    ///
    /// // Без лечения
    /// let current_pain = wound.calculate_current_pain(1.0); // Базовое значение
    /// ```
    #[must_use]
    pub fn calculate_current_pain(&self, pain_medication: f32) -> f32 {
        (self.pain_level * pain_medication.clamp(0.0, 1.0)).max(0.0)
    }

    /// Применяет лечение к ране.
    ///
    /// # Параметры
    /// - `treatment_quality`: качество лечения (0.0-1.0)
    /// - `treatment_type`: тип вмешательства (бинты, хирургия, магия)
    ///
    /// # Эффекты
    /// - Снижает `bleeding_rate` пропорционально качеству
    /// - Уменьшает `infection_risk` при антисептической обработке
    /// - Может снизить `severity` при успешном лечении
    ///
    /// # Возвращаемое значение
    /// `true` если лечение успешно применено, `false` если рана не требует лечения.
    ///
    /// # Пример
    /// ```rust,ignore
    /// // Первая помощь: бинты
    /// if wound.apply_treatment(0.6, TreatmentType::Bandage) {
    ///     ui.show_message("Wound bandaged");
    /// }
    ///
    /// // Хирургия: сложное лечение
    /// if skill_check(Surgery, difficulty) {
    ///     wound.apply_treatment(0.95, TreatmentType::Surgery);
    /// }
    /// ```
    pub fn apply_treatment(&mut self, quality: f32, _treatment_type: TreatmentType) -> bool {
        if !self.is_active() {
            return false;
        }

        let q = quality.clamp(0.0, 1.0);

        // Остановка кровотечения
        self.bleeding_rate *= 1.0 - (q * 0.9); // До 90% снижения

        // Снижение риска инфекции
        self.infection_risk *= 1.0 - (q * 0.7); // До 70% снижения

        // Снижение боли
        self.pain_level *= 1.0 - (q * 0.4); // До 40% снижения

        // Возможное улучшение тяжести (только для Minor/Inhibited)
        if self.severity <= WoundSeverity::Inhibited && q > 0.8 {
            self.severity = match self.severity {
                WoundSeverity::Inhibited => WoundSeverity::Minor,
                WoundSeverity::Minor => WoundSeverity::None,
                _ => self.severity,
            };
        }

        true
    }

    // /// Рассчитывает шанс развития инфекции на основе параметров раны
    // #[must_use]
    // pub fn calculate_infection_chance(&self, environment: InfectionEnvironment) -> f32 {
    //     let base = self.infection_risk;
    //     let env_factor = match environment {
    //         InfectionEnvironment::Clean => 0.5,
    //         InfectionEnvironment::Normal => 1.0,
    //         InfectionEnvironment::Dirty => 1.5,
    //         InfectionEnvironment::Contaminated => 2.0,
    //     };
    //     (base * env_factor).clamp(0.0, 1.0)
    // }

    /// Проверяет, требует ли рана хирургического вмешательства
    #[must_use]
    pub fn requires_surgery(&self) -> bool {
        self.depth > 30.0
            || self.severity >= WoundSeverity::Broken
            || self
                .affected_tissues
                .iter()
                .any(|t| matches!(t, TissueType::OrganTissue | TissueType::Artery))
    }
}

/// Типы медицинского вмешательства для лечения ран.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreatmentType {
    /// Базовая первая помощь: очистка, бинтование
    FirstAid,
    /// Хирургическое вмешательство: швы, репозиция, удаление инородных тел
    Surgery,
    /// Магическое исцеление: мгновенное заживление (фэнтези)
    Magic,
    /// Технологическое лечение: нанороботы, регенерация (киберпанк)
    Advanced,
}

// ============================================================================
// Результат урона
// ============================================================================

/// Результат применения урона к анатомии персонажа.
///
/// # Архитектурное назначение
/// `DamageResult` служит интерфейсом между:
/// - Системой боя (расчёт попадания, урона, спецэффектов)
/// - Системой анатомии (применение урона к тканям, органам, виталам)
/// - Системами обратной связи (интерфейс, звук, частицы, логирование)
///
/// # Преимущества event-driven подхода
/// ```text
/// Боевая система
///     │
///     ▼
/// DamageEvent { target, location, damage, type, penetration }
///     │
///     ▼
/// apply_damage_system (слушает EventReader<DamageEvent>)
///     │
///     ├── Применяет урон к Anatomy
///     ├── Создаёт Wound при необходимости
///     └── Возвращает DamageResult
///           │
///           ▼
///     Обратная связь:
///     ├── Визуал: частицы крови, анимация боли
///     ├── Звук: крик, удар, разрыв ткани
///     ├── Интерфейс: индикатор урона, предупреждение о кровотечении
///     └── Лог: запись события для реплеев/отладки
/// ```
///
/// # Пример использования в боевой системе
/// ```rust,ignore
/// // В системе обработки атаки
/// let result = anatomy.apply_damage_detailed(
///     hit_location,
///     total_damage,
///     weapon_type,
///     penetration_depth,
/// );
///
/// match result {
///     DamageResult::Hit { damage_dealt, bleeding_added, pain_caused } => {
///         // Визуальные эффекты
///         spawn_blood_particles(hit_location, damage_dealt);
///         
///         // Звуковые эффекты
///         play_sound(match damage_dealt {
///             0..=5 => Sound::LightHit,
///             6..=15 => Sound::MediumHit,
///             _ => Sound::HeavyHit,
///         });
///         
///         // Интерфейс
///         if bleeding_added > 0.5 {
///             ui.show_warning("Target is bleeding!");
///         }
///         
///         // Проверка на потерю сознания от боли
///         if pain_caused > 150.0 {
///             target_state = CharacterState::Unconscious;
///         }
///     }
///     DamageResult::Blocked => {
///         play_sound(Sound::ArmorBlock);
///         spawn_spark_particles(hit_location);
///     }
///     DamageResult::Missed => {
///         // Не должно происходить на этом этапе, но на всякий случай
///         play_sound(Sound::Miss);
///     }
/// }
/// ```
///
/// # Сетевая репликация
/// Для MMO-проектов `DamageResult` можно сериализовать и отправлять
/// клиентам для детерминированного воспроизведения эффектов:
/// ```rust,ignore
/// // На сервере
/// let result = anatomy.apply_damage_detailed(...);
/// network.send_to_clients(DamageApplied {
///     target_id,
///     location,
///     damage: result.damage_dealt(),
///     effects: result.to_network_effects(),
/// });
///
/// // На клиенте
/// fn on_damage_applied(event: DamageApplied) {
///     spawn_visual_effects(event.location, event.damage);
///     play_impact_sound(event.damage);
///     update_health_bar(event.target_id, event.damage);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DamageResult {
    /// Атака не достигла цели (промах на этапе попадания).
    ///
    /// # Когда возвращается
    /// - Неверная локация (не существует в `Anatomy::parts`)
    /// - Промах на более раннем этапе (не должен доходить до этой системы)
    ///
    /// # Обработка
    /// Обычно игнорируется или логируется для отладки:
    /// ```rust,ignore
    /// DamageResult::Missed => {
    ///     tracing::warn!("Damage applied to invalid location: {:?}", location);
    /// }
    /// ```
    Missed,

    /// Атака заблокирована бронёй или щитом.
    ///
    /// # Когда возвращается
    /// - `raw_damage <= armor_value` после вычета защиты
    /// - Специальный эффект парирования (полная блокировка)
    ///
    /// # Геймплейные эффекты
    /// - Звук удара по броне (металлический звон, глухой стук)
    /// - Визуальные искры/частицы при попадании в металл
    /// - Возможное повреждение брони (отдельная механика)
    ///
    /// # Пример обработки
    /// ```rust,ignore
    /// DamageResult::Blocked => {
    ///     // Звук и визуал
    ///     audio.play(Sound::ArmorBlock);
    ///     particles.spawn(SparkEffect { location: hit_pos });
    ///     
    ///     // Шанс повреждения брони при сильном ударе
    ///     if raw_damage > armor_value * 2 {
    ///         armor_durability -= 1;
    ///         if armor_durability <= 0 {
    ///             ui.show_message("Armor damaged!");
    ///         }
    ///     }
    /// }
    /// ```
    Blocked,

    /// Успешное нанесение урона с детализированными параметрами.
    ///
    /// # Поля
    /// | Параметр | Тип | Описание |
    /// |----------|-----|----------|
    /// | `damage_dealt` | `i32` | Фактический урон по ХП (после брони и лимитов) |
    /// | `bleeding_added` | `f32` | Прирост скорости кровопотери (мл/сек) |
    /// | `pain_caused` | `f32` | Уровень боли, добавленный к персонажу |
    ///
    /// # Использование параметров
    /// ```rust,ignore
    /// DamageResult::Hit { damage_dealt, bleeding_added, pain_caused } => {
    ///     // Обновление интерфейса
    ///     health_bar.update(-damage_dealt);
    ///     
    ///     // Предупреждение о кровотечении
    ///     if bleeding_added > 0.5 {
    ///         ui.show_bleeding_indicator(bleeding_added);
    ///     }
    ///     
    ///     // Проверка на болевой шок
    ///     if pain_caused > 100.0 {
    ///         ui.show_pain_warning();
    ///     }
    ///     
    ///     // Логирование для баланса
    ///     debug!("Hit: {} dmg, {:.2} bleed, {:.1} pain",
    ///            damage_dealt, bleeding_added, pain_caused);
    /// }
    /// ```
    ///
    /// # Пример значений
    /// | Ситуация | damage_dealt | bleeding_added | pain_caused |
    /// |----------|-------------|----------------|-------------|
    /// | Лёгкий порез | 2-5 | 0.1-0.3 | 10-25 |
    /// | Глубокий порез | 8-15 | 0.5-1.5 | 30-60 |
    /// | Колющее в орган | 10-25 | 1.0-3.0 | 50-100 |
    /// | Критический удар | 20-50+ | 2.0-10.0 | 80-200 |
    Hit {
        /// Фактический урон, нанесённый по ХП персонажа.
        ///
        /// # Расчёт
        /// ```rust,ignore
        /// let after_armor = (raw_damage - armor).max(0);
        /// let capped = after_armor.min(part.max_hp * 2); // Лимит BRP
        /// damage_dealt = capped;
        /// ```
        ///
        /// # Ограничения
        /// - Не может превышать `part.max_hp * 2` за один удар (правило BRP)
        /// - Всегда >= 0 (отрицательный урон невозможен)
        /// - Может быть 0, если атака поглощена тканями без повреждения ХП
        damage_dealt: i32,

        /// Прирост скорости кровотечения в миллилитрах в секунду.
        ///
        /// # Источник
        /// Суммируется из `bleeding_rate` всех затронутых тканей:
        /// ```rust,ignore
        /// bleeding_added = affected_tissues.iter()
        ///     .filter_map(|t| tissues.get(t))
        ///     .map(|t| t.bleeding_rate * damage_ratio)
        ///     .sum();
        /// ```
        ///
        /// # Глобальное влияние
        /// Значение добавляется к `anatomy.substances.blood_loss_rate`,
        /// что влияет на:
        /// - Скорость потери ХП от кровопотери
        /// - Риск гиповолемического шока
        /// - Необходимость срочного лечения
        bleeding_added: f32,

        /// Уровень боли, причинённой ударом.
        ///
        /// # Расчёт
        /// ```rust,ignore
        /// pain_caused = affected_tissues.iter()
        ///     .filter_map(|t| tissues.get(t))
        ///     .map(|t| t.pain_receptors)
        ///     .sum::<f32>()
        ///     * severity_multiplier
        ///     * (depth / 10.0).min(1.0);
        /// ```
        ///
        /// # Влияние на персонажа
        /// - Добавляется к `anatomy.vitals.pain`
        /// - При `pain > 150`: автоматическая потеря сознания
        /// - При `pain > 100`: штраф -50% к точности действий
        pain_caused: f32,
    },
}

impl DamageResult {
    /// Извлекает значение нанесённого урона из результата.
    ///
    /// # Возвращаемое значение
    /// - Для `Hit`: значение `damage_dealt`
    /// - Для `Missed`/`Blocked`: 0
    ///
    /// # Использование
    /// Удобно для ситуаций, где нужен только числовой урон:
    /// ```rust,ignore
    /// // Обновление полосы здоровья
    /// let damage = result.damage_dealt();
    /// health_bar.value -= damage;
    ///
    /// // Подсчёт урона за бой
    /// total_damage_dealt += result.damage_dealt();
    /// ```
    ///
    /// # Пример
    /// ```rust,ignore
    /// assert_eq!(DamageResult::Missed.damage_dealt(), 0);
    /// assert_eq!(DamageResult::Blocked.damage_dealt(), 0);
    /// assert_eq!(
    ///     DamageResult::Hit { damage_dealt: 10, bleeding_added: 0.5, pain_caused: 30.0 }
    ///         .damage_dealt(),
    ///     10
    /// );
    /// ```
    #[must_use]
    #[inline]
    pub fn damage_dealt(&self) -> i32 {
        match self {
            Self::Hit { damage_dealt, .. } => *damage_dealt,
            _ => 0,
        }
    }

    /// Проверяет, был ли нанесён урон (не промах и не блок).
    ///
    /// # Возвращаемое значение
    /// - `true`: вариант `Hit` с `damage_dealt > 0`
    /// - `false`: `Missed`, `Blocked` или `Hit` с нулевым уроном
    ///
    /// # Использование
    /// ```rust,ignore
    /// // Подсчёт попаданий
    /// if result.is_hit() {
    ///     stats.hits_landed += 1;
    /// }
    ///
    /// // Триггер для эффектов только при реальном уроне
    /// if result.is_hit() {
    ///     spawn_blood_splatter();
    ///     play_grunt_sound();
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub fn is_hit(&self) -> bool {
        matches!(self, Self::Hit { damage_dealt, .. } if *damage_dealt > 0)
    }

    /// Проверяет, добавила ли атака кровотечение.
    ///
    /// # Возвращаемое значение
    /// - `true`: `bleeding_added > 0.0`
    /// - `false`: нет прироста кровопотери
    ///
    /// # Использование
    /// ```rust,ignore
    /// // Предупреждение о кровотечении
    /// if result.is_bleeding() {
    ///     ui.show_status_effect(StatusEffect::Bleeding);
    /// }
    ///
    /// // ИИ: приоритет лечения кровоточащих союзников
    /// if ally_result.is_bleeding() {
    ///     ai_queue_action(Action::ApplyBandage, ally_entity);
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub fn is_bleeding(&self) -> bool {
        matches!(self, Self::Hit { bleeding_added, .. } if *bleeding_added > 0.0)
    }

    /// Проверяет, причинила ли атака значительную боль.
    ///
    /// # Параметры
    /// - `threshold`: порог боли для считания "значительной" (по умолчанию 50.0)
    ///
    /// # Возвращаемое значение
    /// - `true`: `pain_caused >= threshold`
    /// - `false`: боль ниже порога или атака не попала
    ///
    /// # Использование
    /// ```rust,ignore
    /// // Крик от боли
    /// if result.is_painful(50.0) {
    ///     audio.play(Sound::PainGrunt);
    /// }
    ///
    /// // Проверка на потерю сознания
    /// if result.is_painful(150.0) {
    ///     target.state = CharacterState::Unconscious;
    /// }
    /// ```
    #[must_use]
    pub fn is_painful(&self, threshold: f32) -> bool {
        matches!(self, Self::Hit { pain_caused, .. } if *pain_caused >= threshold)
    }
}

impl std::fmt::Display for DamageResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missed => write!(f, "Missed"),
            Self::Blocked => write!(f, "Blocked"),
            Self::Hit {
                damage_dealt,
                bleeding_added,
                pain_caused,
            } => {
                write!(
                    f,
                    "Hit: {}dmg, +{:.2}ml/s bleed, +{:.1} pain",
                    damage_dealt, bleeding_added, pain_caused
                )
            }
        }
    }
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_penetration_profile_effective_depth() {
        let profile = PenetrationProfile::piercing(); // depth=50, type=Piercing

        // Без брони, прямой угол
        assert!((profile.effective_depth(0, 1.0) - 60.0).abs() < 0.1); // 50 × 1.0 × 1.2

        // С бронёй 20, прямой угол
        assert!((profile.effective_depth(20, 1.0) - 36.0).abs() < 0.1); // (50-20) × 1.2

        // С бронёй 10, косой угол 50%
        assert!((profile.effective_depth(10, 0.5) - 24.0).abs() < 0.1); // (50-10) × 0.5 × 1.2

        // Броня больше глубины
        assert!(profile.effective_depth(100, 1.0) <= 0.0);
    }

    #[test]
    fn test_wound_is_active() {
        let minor_wound = Wound {
            wound_type: WoundType::Cutting,
            severity: WoundSeverity::Minor,
            affected_tissues: vec![TissueType::Skin],
            depth: 5.0,
            bleeding_rate: 0.0,
            pain_level: 10.0,
            infection_risk: 0.0,
            created_at: 0.0,
        };
        assert!(minor_wound.is_active()); // severity > None

        let healed_wound = Wound {
            severity: WoundSeverity::None,
            bleeding_rate: 0.0,
            infection_risk: 0.0,
            ..minor_wound
        };
        assert!(!healed_wound.is_active());
    }

    #[test]
    fn test_wound_apply_treatment() {
        let mut wound = Wound {
            wound_type: WoundType::Cutting,
            severity: WoundSeverity::Inhibited,
            affected_tissues: vec![TissueType::Skin, TissueType::Muscle],
            depth: 20.0,
            bleeding_rate: 1.0,
            pain_level: 40.0,
            infection_risk: 0.3,
            created_at: 0.0,
        };

        // Лечение с качеством 80%
        let success = wound.apply_treatment(0.8, TreatmentType::FirstAid);
        assert!(success);

        // Проверка изменений
        assert!(wound.bleeding_rate < 0.3); // Было 1.0, должно снизиться на ~72%
        assert!(wound.infection_risk < 0.15); // Было 0.3, должно снизиться на ~56%
        assert!(wound.pain_level < 30.0); // Было 40, должно снизиться на ~32%

        // При высоком качестве и низкой тяжести — улучшение severity
        let mut minor_wound = Wound {
            severity: WoundSeverity::Minor,
            bleeding_rate: 0.2,
            pain_level: 15.0,
            infection_risk: 0.1,
            ..wound
        };

        minor_wound.apply_treatment(0.9, TreatmentType::FirstAid);
        assert_eq!(minor_wound.severity, WoundSeverity::None);
    }

    #[test]
    fn test_damage_result_accessors() {
        let missed = DamageResult::Missed;
        assert_eq!(missed.damage_dealt(), 0);
        assert!(!missed.is_hit());
        assert!(!missed.is_bleeding());
        assert!(!missed.is_painful(0.0));

        let blocked = DamageResult::Blocked;
        assert_eq!(blocked.damage_dealt(), 0);
        assert!(!blocked.is_hit());

        let hit = DamageResult::Hit {
            damage_dealt: 12,
            bleeding_added: 0.8,
            pain_caused: 45.0,
        };
        assert_eq!(hit.damage_dealt(), 12);
        assert!(hit.is_hit());
        assert!(hit.is_bleeding());
        assert!(hit.is_painful(40.0));
        assert!(!hit.is_painful(50.0));
    }

    #[test]
    fn test_damage_result_zero_damage_not_hit() {
        // Edge case: Hit с damage_dealt = 0 не считается успешным попаданием
        let zero_hit = DamageResult::Hit {
            damage_dealt: 0,
            bleeding_added: 0.0,
            pain_caused: 0.0,
        };
        assert!(!zero_hit.is_hit());
    }
}
