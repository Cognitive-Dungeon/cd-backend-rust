/// Трейт-маркер для всех характеристик BRP.
pub trait CharacteristicMarker {
    const NAME: &'static str;
    const ABBREVIATION: &'static str;
}

macro_rules! define_marker {
    ($struct_name:ident, $full_name:expr, $abbr:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $struct_name;

        impl CharacteristicMarker for $struct_name {
            const NAME: &'static str = $full_name;
            const ABBREVIATION: &'static str = $abbr;
        }
    };
}

define_marker!(Str, "Strength", "STR");
define_marker!(Con, "Constitution", "CON");
define_marker!(Siz, "Size", "SIZ");
define_marker!(Int, "Intelligence", "INT");
define_marker!(Pow, "Power", "POW");
define_marker!(Dex, "Dexterity", "DEX");
define_marker!(Cha, "Charisma", "CHA");
define_marker!(Edu, "Education", "EDU");
