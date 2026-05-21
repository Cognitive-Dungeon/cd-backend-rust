use serde::{Deserialize, Serialize};

/// Категории транспорта
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleCategory {
    AnimalDrawn, // Телеги, колесницы
    Automobile,  // Машины, грузовики
    Motorcycle,  // Мотоциклы
    Boat,        // Лодки
    Ship,        // Корабли
    Submarine,   // Подлодки
    AirVehicle,  // Самолеты, вертолеты, дирижабли
    Spacecraft,  // Космические корабли
    Train,       // Поезда
    Mech,        // Мехи (Mecha, стр. 212)
    Tank,        // Танки
    Hovercraft,  // Транспорт на воздушной подушке
    LandSkimmer, //
}

/// Маневры во время погони (стр. 88)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChaseManeuver {
    Turn,
    HighSpeedTurn,
    BootleggerReverse,
    Collide,
    Ram,
}
