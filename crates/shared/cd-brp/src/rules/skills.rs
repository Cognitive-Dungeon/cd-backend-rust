use crate::domain::CharacteristicBlock;
use crate::{
    BodyPlan, Characteristic, Dex, Edu, GameSessionConfig, Int, KnowledgeType, Pow, Stat,
    VehicleCategory,
};
// src/rules/skills.rs
use crate::math::frac_u16;
use crate::types::{SkillCategory, SkillType};

/// Контекст персонажа, необходимый для вычисления динамических шансов навыков.
pub struct BaseChanceContext<'a> {
    pub stats: &'a CharacteristicBlock,
    pub body_plan: BodyPlan,
    pub config: &'a GameSessionConfig,
}

impl SkillType {
    /// Возвращает категорию навыка.
    pub const fn category(&self) -> SkillCategory {
        use SkillCategory::*;
        use SkillType::*;

        match self {
            // Combat
            WeaponAttack(_, _) | Parry(_, _) | Shield(_) | Brawl | Grapple | MartialArts(_) => {
                Combat
            }

            // Communication
            Bargain | Command | Disguise | Etiquette(_) | FastTalk | LanguageOwn(_)
            | LanguageOther(_) | Perform(_) | Persuade | Status(_) | Teach => Communication,

            // Manipulation
            Art(_) | Craft(_) | Demolition | FineManipulation | HeavyMachine(_) | Repair(_)
            | SleightOfHand => Manipulation,

            // Mental
            Appraise | FirstAid | Gaming | Knowledge(_) | Literacy(_) | Medicine
            | Psychotherapy | Science(_) | Strategy | TechnicalSkill(_) => Mental,

            // Perception
            Insight | Listen | Navigate | Research | Sense | Spot | Track => Perception,

            // Physical
            Climb | Dodge | Drive(_) | Fly | Hide | Jump | Pilot(_) | Projection | Ride(_)
            | Stealth | Swim | Throw => Physical,
        }
    }

    /// Возвращает статичный базовый шанс навыка.
    /// Если навык вычисляется динамически (Dodge, Fly, LanguageOwn и т.д.), возвращает None.
    pub const fn static_base_chance(&self) -> Option<u16> {
        use SkillType::*;

        match self {
            // === ДИНАМИЧЕСКИЕ НАВЫКИ (Требуют характеристик персонажа) ===
            Dodge | Fly | Projection | Gaming | LanguageOwn(_) | Literacy(_) => None,

            // === УНИКАЛЬНЫЕ БАЗОВЫЕ ШАНСЫ (По рулбуку) ===
            Knowledge(knowledge_type) => match knowledge_type {
                // По правилам "Blasphemous Lore skill begins at 0%, not 05%"
                KnowledgeType::BlasphemousLore => Some(0),
                _ => Some(5),
            },

            LanguageOther(_) => Some(0), // Чужие языки всегда начинаются с 0, если не не приобретены

            Drive(vehicle_cat) => match vehicle_cat {
                // Наземный/Простой транспорт (20%)
                VehicleCategory::AnimalDrawn
                | VehicleCategory::Automobile
                | VehicleCategory::Motorcycle
                | VehicleCategory::Train
                | VehicleCategory::Hovercraft
                | VehicleCategory::LandSkimmer => Some(20),

                // Heavy/Military/Uncommon for Drive (1%)
                VehicleCategory::Tank
                | VehicleCategory::Mech
                | VehicleCategory::Boat
                | VehicleCategory::Ship
                | VehicleCategory::Submarine
                | VehicleCategory::AirVehicle
                | VehicleCategory::Spacecraft => Some(1),
            },

            // Во всем, что летает/плавает/ходит (Мехи), Pilot - это 1% (стр. 39)
            Pilot(_) => Some(1),

            // === ОРУЖИЕ (Зависит от чертежа, база здесь 0) ===
            WeaponAttack(_, _) | Parry(_, _) | Shield(_) => Some(0),

            // === 40% ===
            Climb => Some(40),

            // === 30% ===
            FirstAid => Some(30),

            // === 25% ===
            Brawl | Grapple | Jump | Listen | Research | Spot | Swim | Throw => Some(25),

            // === 15% ===
            Appraise | Persuade | Repair(_) | Status(_) => Some(15),

            // === 10% ===
            Hide | Navigate | Sense | Stealth | Teach | Track => Some(10),

            // === 05% ===
            Art(_) | Bargain | Command | Craft(_) | Etiquette(_) | FastTalk | FineManipulation
            | Insight | Medicine | Perform(_) | Ride(_) | SleightOfHand | TechnicalSkill(_) => {
                Some(5)
            }

            // === 01% ===
            Demolition | Disguise | HeavyMachine(_) | MartialArts(_) | Psychotherapy
            | Science(_) | Strategy => Some(1),
        }
    }

    /// Возвращает базовый шанс для любого навыка
    pub fn base_chance(&self, ctx: &BaseChanceContext) -> u16 {
        // Если навык имеет жесткую статику (например, Climb 40%), сразу возвращаем её
        if let Some(static_chance) = self.static_base_chance() {
            return static_chance;
        }

        // Если статики нет (возвращен None), навык ОБЯЗАН быть рассчитан динамически.
        match self {
            SkillType::Dodge => calc_dodge_base(ctx.stats.dex),
            SkillType::Projection => calc_projection_base(ctx.stats.dex),

            SkillType::Fly => {
                let has_wings = matches!(
                    ctx.body_plan,
                    BodyPlan::Winged
                        | BodyPlan::WingedFourLegged
                        | BodyPlan::WingedFourLeggedWithTail
                        | BodyPlan::WingedHumanoid
                );
                calc_fly_base(ctx.stats.dex, has_wings)
            }
            SkillType::Gaming => calc_gaming_base(ctx.stats.int, ctx.stats.pow),

            SkillType::LanguageOwn(_) | SkillType::Literacy(_) => {
                calc_language_own_base(ctx.stats.int, ctx.stats.edu, ctx.config.use_education_stat)
            }

            // Если мы добавили навык, для которого static_base_chance() вернул None,
            // но забыли прописать его формулу здесь, программа упадет на этапе тестирования.
            _ => unreachable!(
                "Skill {:?} returned None for static_base_chance but has no dynamic formula!",
                self
            ),
        }
    }

    /// Определяет ключевую характеристику, от которой зависит навык.
    /// Позволяет глобальным эффектам (болезни, перегруз, ослепление)
    /// автоматически штрафовать целые группы навыков.
    pub const fn primary_characteristic(&self) -> Characteristic {
        use Characteristic::*;
        use SkillType::*;

        // Все боевые навыки (и оружие, и рукопашная) требуют координации (Dex)
        if matches!(self.category(), crate::types::SkillCategory::Combat) {
            return Dex;
        }

        match self {
            // === Ловкость и Координация (DEX) ===
            // Сюда попадает всё оружие, уклонение, скрытность, вождение и манипуляции
            WeaponAttack(_, _)
            | Parry(_, _)
            | Shield(_)
            | MartialArts(_)
            | Dodge
            | Stealth
            | Hide
            | SleightOfHand
            | FineManipulation
            | Drive(_)
            | Pilot(_)
            | Fly
            | Ride(_) => Dex,

            // === Физическая Сила (STR) ===
            // Навыки, где чистая физика важнее координации
            Brawl | Grapple | Climb | Jump | Swim | Throw => Str,

            // === Интеллект и Знания (INT) ===
            // Ремесло, медицина, науки, языки
            Appraise | Craft(_) | Demolition | FirstAid | Medicine | Navigate | Repair(_)
            | Science(_) | Strategy | TechnicalSkill(_) | Knowledge(_) | LanguageOwn(_)
            | LanguageOther(_) | Literacy(_) | Gaming => Int,

            // === Восприятие и Воля (POW / INT) ===
            // В BRP Восприятие часто привязано к POW или INT (мы используем POW для чутья)
            Insight | Listen | Sense | Spot | Track | Projection | Psychotherapy | Research => Pow,

            // === Общение и Харизма (CHA) ===
            Bargain | Command | Disguise | Etiquette(_) | FastTalk | Perform(_) | Persuade
            | Status(_) | Teach | Art(_) => Cha,

            // HeavyMachine может быть INT или DEX, пусть будет INT (понимание механизмов)
            HeavyMachine(_) => Int,
        }
    }

    /// Определяет, может ли навык получать "галочки опыта" (Experience Checks).
    /// Стр. 45: "Language (Own) cannot normally be improved by experience checks."
    /// Стр. 70: Status меняется по решению Мастера, а не кубиками.
    pub const fn can_improve_by_experience(&self) -> bool {
        use SkillType::*;
        !matches!(self, LanguageOwn(_) | Status(_))
    }
}

const fn calc_dodge_base(dex: Stat<Dex>) -> u16 {
    dex.get().saturating_mul(2)
}

const fn calc_projection_base(dex: Stat<Dex>) -> u16 {
    dex.get().saturating_mul(2)
}

const fn calc_fly_base(dex: Stat<Dex>, has_wings: bool) -> u16 {
    if has_wings {
        dex.get().saturating_mul(4)
    } else {
        frac_u16::half_ceil(dex.get())
    }
}

const fn calc_gaming_base(int: Stat<Int>, pow: Stat<Pow>) -> u16 {
    int.get().saturating_add(pow.get())
}

fn calc_language_own_base(int: Stat<Int>, edu: Option<Stat<Edu>>, use_edu_rule: bool) -> u16 {
    let stat = if use_edu_rule {
        edu.map(|e| e.get()).unwrap_or_else(|| int.get())
    } else {
        int.get()
    };
    stat.saturating_mul(5)
}
