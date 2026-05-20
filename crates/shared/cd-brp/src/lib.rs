//! # cd-brp
//!
//! Ядро правил, типов и логики системы Basic Roleplaying (BRP: UGE).
//! Содержит чистые доменные типы, систему расчётов, сетевые интенты/эффекты
//! и интеграцию с Bevy ECS.

#![warn(missing_docs)]
#![deny(unsafe_code)]

// Подпапки с логикой
pub mod action;
pub mod bevy;
pub mod constants;
pub mod domain;
pub mod math;
pub mod rules;
pub mod types;

// Эргономичные реэкспорты на уровень крейта
pub use constants::*;
pub use rules::*;
pub use types::*;

pub use cd_core::ObjectGuid;
