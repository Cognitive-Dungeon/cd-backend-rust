//! Доменные объекты: персонажи, экипировка, их статические чертежи (Blueprints).

pub mod character;
pub mod chars;
pub mod gear;
pub mod validation;

pub use chars::CharacteristicBlock;
pub use gear::WeaponBlueprint;
