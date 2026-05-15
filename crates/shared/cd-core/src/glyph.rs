use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;
use thiserror::Error;

/// Ошибки, возникающие при парсинге глифов.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum GlyphError {
    #[error("invalid hex color length: {0:?}")]
    InvalidColorLength(String),
    #[error("invalid hex digits in color: {0:?}")]
    InvalidColorHex(String),
    #[error("invalid utf8 sequence")]
    InvalidUtf8Sequence,
    #[error("glyph char must be exactly one rune, got {0:?}")]
    MultipleChars(String),
    #[error("glyph char is empty")]
    EmptyChar,
    #[error("rune {_0:?} (U+{codepoint:04X}) is not supported in CP437", codepoint = *_0 as u32)]
    UnsupportedChar(char),
}

/// Упакованное представление цветного символа.
/// Формат `u32`:
/// - [0..8] биты: Символ (CP437)
/// - [8..32] биты: Цвет RGB (24 бита)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub struct Glyph(u32);

impl Glyph {
    const BITS_CHAR: u32 = 8;
    const MASK_CHAR: u32 = (1 << Self::BITS_CHAR) - 1;
    const MASK_COLOR: u32 = 0xFF_FF_FF;

    /// Создает новый Glyph из RGB-цвета и символа.
    /// Используется `const fn`, что позволяет вычислять глифы на этапе компиляции.
    #[inline(always)]
    pub const fn new(color_rgb: u32, ch: u8) -> Self {
        Self(((color_rgb & Self::MASK_COLOR) << Self::BITS_CHAR) | (ch as u32 & Self::MASK_CHAR))
    }

    /// Извлекает 24-битный RGB-цвет.
    #[inline(always)]
    pub const fn color(&self) -> u32 {
        (self.0 >> Self::BITS_CHAR) & Self::MASK_COLOR
    }

    /// Извлекает 8-битный символ.
    #[inline(always)]
    pub const fn ch(&self) -> u8 {
        (self.0 & Self::MASK_CHAR) as u8
    }

    /// Возвращает представление символа в виде Unicode (`char` в Rust).
    #[inline]
    pub fn to_char(&self) -> char {
        CP437_TO_UNICODE[self.ch() as usize]
    }

    /// Возвращает строковое HEX-представление цвета (например, "#00FF00").
    pub fn hex_color(&self) -> String {
        format!("#{:06X}", self.color())
    }

    #[inline]
    pub fn hex_color_bytes(&self) -> [u8; 7] {
        const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
        let c = self.color();
        [
            b'#',
            HEX_CHARS[((c >> 20) & 0xF) as usize],
            HEX_CHARS[((c >> 16) & 0xF) as usize],
            HEX_CHARS[((c >> 12) & 0xF) as usize],
            HEX_CHARS[((c >> 8) & 0xF) as usize],
            HEX_CHARS[((c >> 4) & 0xF) as usize],
            HEX_CHARS[(c & 0xF) as usize],
        ]
    }

    /// Парсит глиф из строковых значений (аналог ParseGlyphFromJSON).
    pub fn from_json(char_str: &str, color_str: &str) -> Result<Self, GlyphError> {
        let rgb = parse_hex_color_fast(color_str)?;
        let bytes = char_str.as_bytes();

        if bytes.is_empty() {
            return Ok(Self::new(rgb, b' '));
        }

        // --- FAST PATH (Максимальное ускорение) ---
        // Если это стандартный печатный ASCII (99% текста),
        // он занимает 1 байт. Мы байпасим парсинг UTF-8 и бинарный поиск.
        if bytes.len() == 1 {
            let b = bytes[0];
            if (32..=126).contains(&b) {
                return Ok(Self::new(rgb, b));
            }
        }

        // --- SLOW PATH (Unicode или управляющие символы) ---
        let mut chars = char_str.chars();
        let c = chars.next().ok_or(GlyphError::EmptyChar)?;

        if chars.next().is_some() {
            return Err(GlyphError::MultipleChars(char_str.to_string()));
        }

        // Бинарный поиск O(log N) по массиву из 256 элементов (максимум ~8 прыжков, все в кэше L1)
        if let Ok(idx) = UNICODE_TO_CP437_SORTED.binary_search_by_key(&c, |&(ch, _)| ch) {
            Ok(Self::new(rgb, UNICODE_TO_CP437_SORTED[idx].1))
        } else {
            Err(GlyphError::UnsupportedChar(c))
        }
    }
}

/// Реализация аналога метода String() в Go
impl fmt::Display for Glyph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Мы НЕ вызываем self.hex_color(), чтобы избежать аллокации String.
        // Форматируем цвет напрямую в поток вывода.
        write!(
            f,
            "Glyph{{char='{}', color=#{:06X}}}",
            self.to_char().escape_debug(),
            self.color()
        )
    }
}

/// Ультрабыстрый парсер HEX цветов. Работает напрямую с байтами,
/// минуя тяжелый внутренний механизм `u32::from_str_radix` (проверки знаков, основ и т.д.).
#[inline]
fn parse_hex_color_fast(s: &str) -> Result<u32, GlyphError> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return Ok(0);
    }

    let bytes = if bytes[0] == b'#' { &bytes[1..] } else { bytes };
    if bytes.len() != 6 {
        return Err(GlyphError::InvalidColorLength(s.to_string()));
    }

    let mut color = 0u32;
    for &b in bytes {
        let val = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => return Err(GlyphError::InvalidColorHex(s.to_string())),
        };
        color = (color << 4) | (val as u32);
    }

    Ok(color)
}

/// Вспомогательная функция для парсинга HEX-цветов.
fn parse_hex_color(s: &str) -> Result<u32, GlyphError> {
    if s.is_empty() {
        return Ok(0);
    }

    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return Err(GlyphError::InvalidColorLength(s.to_string()));
    }

    u32::from_str_radix(s, 16).map_err(|_| GlyphError::InvalidColorHex(s.to_string()))
}

/// Ленивая инициализация обратной таблицы Unicode -> CP437
static UNICODE_TO_CP437: LazyLock<HashMap<char, u8>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(256);
    for (i, &c) in CP437_TO_UNICODE.iter().enumerate() {
        if c != '\0' {
            map.insert(c, i as u8);
        }
    }
    map
});

/// Таблица конвертации CP437 в Unicode.
#[rustfmt::skip]
pub const CP437_TO_UNICODE: [char; 256] = [
    // 0x00 - 0x1F (Control chars mapped to graphical representations in CP437)
	'\u{0000}', '\u{263A}', '\u{263B}', '\u{2665}', '\u{2666}', '\u{2663}', '\u{2660}', '\u{2022}', // ☺ ☻ ♥ ♦ ♣ ♠ •
	'\u{25D8}', '\u{25CB}', '\u{25D9}', '\u{2642}', '\u{2640}', '\u{266A}', '\u{266B}', '\u{263C}', // ◘ ○ ◙ ♂ ♀ ♪ ♫ ☼
	'\u{25BA}', '\u{25C4}', '\u{2195}', '\u{203C}', '\u{00B6}', '\u{00A7}', '\u{25AC}', '\u{21A8}', // ► ◄ ↕ ‼ ¶ § ▬ ↨
	'\u{2191}', '\u{2193}', '\u{2192}', '\u{2190}', '\u{221F}', '\u{2194}', '\u{25B2}', '\u{25BC}', // ↑ ↓ → ← ∟ ↔ ▲ ▼

	// 0x20 - 0x7E (Standard ASCII)
	' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
	'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
	'@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
	'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
	'`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
	'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '\u{2302}', // 0x7F = ⌂

	// 0x80 - 0xFF (Extended characters)
	'\u{00C7}', '\u{00FC}', '\u{00E9}', '\u{00E2}', '\u{00E4}', '\u{00E0}', '\u{00E5}', '\u{00E7}', // Ç ü é â ä à å ç
	'\u{00EA}', '\u{00EB}', '\u{00E8}', '\u{00EF}', '\u{00EE}', '\u{00EC}', '\u{00C4}', '\u{00C5}', // ê ë è ï î ì Ä Å
	'\u{00C9}', '\u{00E6}', '\u{00C6}', '\u{00F4}', '\u{00F6}', '\u{00F2}', '\u{00FB}', '\u{00F9}', // É æ Æ ô ö ò û ù
	'\u{00FF}', '\u{00D6}', '\u{00DC}', '\u{00A2}', '\u{00A3}', '\u{00A5}', '\u{20A7}', '\u{0192}', // ÿ Ö Ü ¢ £ ¥ ₧ ƒ
	'\u{00E1}', '\u{00ED}', '\u{00F3}', '\u{00FA}', '\u{00F1}', '\u{00D1}', '\u{00AA}', '\u{00BA}', // á í ó ú ñ Ñ ª º
	'\u{00BF}', '\u{2310}', '\u{00AC}', '\u{00BD}', '\u{00BC}', '\u{00A1}', '\u{00AB}', '\u{00BB}', // ¿ ⌐ ¬ ½ ¼ ¡ « »
	'\u{2591}', '\u{2592}', '\u{2593}', '\u{2502}', '\u{2524}', '\u{2561}', '\u{2562}', '\u{2556}', // ░ ▒ ▓ │ ┤ ╡ ╢ ╖
	'\u{2555}', '\u{2563}', '\u{2551}', '\u{2557}', '\u{255D}', '\u{255C}', '\u{255B}', '\u{2510}', // ╕ ╣ ║ ╗ ╝ ✜ ╛ ┐
	'\u{2514}', '\u{2534}', '\u{252C}', '\u{251C}', '\u{2500}', '\u{253C}', '\u{255E}', '\u{255F}', // └ ┴ ┬ ├ ─ ┼ ╞ ╟
	'\u{255A}', '\u{2554}', '\u{2569}', '\u{2566}', '\u{2560}', '\u{2550}', '\u{256C}', '\u{2567}', // ╚ ╔ ╩ ╦ ╠ ═ ╬ ╧
	'\u{2568}', '\u{2564}', '\u{2565}', '\u{2559}', '\u{2558}', '\u{2552}', '\u{2553}', '\u{256B}', // ╨ ╤ ╥ ╙ ╘ ╒ ╓ ╫
	'\u{256A}', '\u{2518}', '\u{250C}', '\u{2588}', '\u{2584}', '\u{258C}', '\u{2590}', '\u{2580}', // ╪ ┘ ┌ █ ▄ ▌ ▐ ▀
	'\u{03B1}', '\u{00DF}', '\u{0393}', '\u{03C0}', '\u{03A3}', '\u{03C3}', '\u{00B5}', '\u{03C4}', // α ß Γ π Σ σ µ τ
	'\u{03A6}', '\u{0398}', '\u{03A9}', '\u{03B4}', '\u{221E}', '\u{03C6}', '\u{03B5}', '\u{2229}', // Φ Θ Ω δ ∞ φ ε ∩ 
	'\u{2261}', '\u{00B1}', '\u{2265}', '\u{2264}', '\u{2320}', '\u{2321}', '\u{00F7}', '\u{2248}', // ≡ ± ≥ ≤ ⌠ ⌡ ÷ ≈ 
	'\u{00B0}', '\u{2219}', '\u{00B7}', '\u{221A}', '\u{207F}', '\u{00B2}', '\u{25A0}', '\u{00A0}', // ° ∙ · √ ⁿ ² ■ (NBSP)
];

const UNICODE_TO_CP437_SORTED: [(char, u8); 256] = {
    let mut arr = [('\0', 0); 256];
    let mut i = 0;
    while i < 256 {
        arr[i] = (CP437_TO_UNICODE[i], i as u8);
        i += 1;
    }

    // Сортировка пузырьком (выполняется rustc мгновенно при сборке)
    let mut i = 0;
    while i < 256 {
        let mut j = 0;
        while j < 255 - i {
            if arr[j].0 as u32 > arr[j + 1].0 as u32 {
                let tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
            }
            j += 1;
        }
        i += 1;
    }
    arr
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_glyph() {
        assert_eq!(Glyph::new(0xFFA500, b'A').0, 0xFFA50041);
        assert_eq!(Glyph::new(0x000000, b' ').0, 0x00000020);
        assert_eq!(Glyph::new(0xFFFFFF, b'\n').0, 0xFFFFFF0A);
    }

    #[test]
    fn test_glyph_properties() {
        let g = Glyph::new(0xFFA500, b'A');
        assert_eq!(g.color(), 0xFFA500);
        assert_eq!(g.ch(), b'A');
        assert_eq!(g.hex_color(), "#FFA500");
        assert_eq!(g.to_char(), 'A');
    }

    #[test]
    fn test_hex_color_bytes_zero_allocation() {
        let g = Glyph::new(0x0A1B2C, b'@');
        let bytes = g.hex_color_bytes();

        // Массив байтов совпадает с ASCII представлением цвета
        assert_eq!(&bytes, b"#0A1B2C");

        // Эту конструкцию вы можете использовать для сериализатора без аллокаций
        let str_slice = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(str_slice, "#0A1B2C");
        assert_eq!(str_slice, g.hex_color());
    }

    #[test]
    fn test_parse_json_fast_path() {
        // Обычные ASCII-символы должны парситься мгновенно
        let g = Glyph::from_json("a", "112233").unwrap();
        assert_eq!(g, Glyph::new(0x112233, b'a'));

        let g_space = Glyph::from_json(" ", "#FFFFFF").unwrap();
        assert_eq!(g_space, Glyph::new(0xFFFFFF, b' '));
    }

    #[test]
    fn test_parse_json_slow_path_unicode() {
        // Символы, выходящие за 32..126
        let g_symbol = Glyph::from_json("☺", "FF0000").unwrap();
        assert_eq!(g_symbol, Glyph::new(0xFF0000, 0x01)); // U+263A -> 0x01

        let g_cyrillic = Glyph::from_json("α", "00FF00").unwrap();
        assert_eq!(g_cyrillic, Glyph::new(0x00FF00, 0xE0)); // U+03B1 -> 0xE0
    }

    #[test]
    fn test_parse_json_errors() {
        // Неподдерживаемый юникод (например, русская буква, которой нет в CP437)
        let err = Glyph::from_json("Д", "FFFFFF").unwrap_err();
        assert!(matches!(err, GlyphError::UnsupportedChar('Д')));

        // Слишком много символов
        let err = Glyph::from_json("abc", "000000").unwrap_err();
        assert!(matches!(err, GlyphError::MultipleChars(_)));

        // Пустой символ трактуется как пробел (особенность из вашего исходного кода)
        let g_empty = Glyph::from_json("", "123456").unwrap();
        assert_eq!(g_empty.ch(), b' ');
    }

    #[test]
    fn test_fast_hex_parser_edge_cases() {
        // Поддерживает и с решеткой и без
        assert_eq!(parse_hex_color_fast("#AABBCC").unwrap(), 0xAABBCC);
        assert_eq!(parse_hex_color_fast("AABBCC").unwrap(), 0xAABBCC);

        // Поддерживает нижний регистр
        assert_eq!(parse_hex_color_fast("#aabbcc").unwrap(), 0xAABBCC);

        // Ошибки
        assert!(parse_hex_color_fast("12345").is_err()); // короткий
        assert!(parse_hex_color_fast("1234567").is_err()); // длинный
        assert!(parse_hex_color_fast("XXYYZZ").is_err()); // невалидные символы
    }

    #[test]
    fn test_const_array_is_properly_sorted() {
        // Удостоверимся, что алгоритм пузырьковой сортировки в const fn отработал верно
        for i in 0..255 {
            assert!(
                UNICODE_TO_CP437_SORTED[i].0 as u32 <= UNICODE_TO_CP437_SORTED[i + 1].0 as u32,
                "Массив не отсортирован на индексе {}",
                i
            );
        }
    }

    #[test]
    fn test_chunk_serialization_simulation() {
        // Создаем чанк карты (256 глифов)
        let chunk: Vec<Glyph> = vec![Glyph::new(0x445566, b'#'); 256];

        // Симулируем подготовку данных для клиента без аллокаций памяти под строки цветов
        let mut client_payload_chars = Vec::with_capacity(256);
        let mut client_payload_colors = Vec::with_capacity(256);

        for glyph in &chunk {
            client_payload_chars.push(glyph.to_char());
            // Вместо .hex_color() берем байты. Это можно отправить по сети или в JSON
            client_payload_colors.push(glyph.hex_color_bytes());
        }

        assert_eq!(client_payload_chars.len(), 256);
        assert_eq!(client_payload_colors.len(), 256);
        assert_eq!(&client_payload_colors[0], b"#445566");
    }
}
