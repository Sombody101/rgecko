pub fn get_format_code_from_label(label: &str) -> Option<&'static str> {
    return __find_in_map(FORMAT_CODES, label);
}

// fn get_control_from_key(key: &str) -> Option<&'static str> {
//     return __find_in_map(CONTROL_CHARS, key);
// }

pub fn __find_in_map(map: &[(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    return map.iter().find(|&&(k, _)| k == key).map(|&(_, v)| v);
}

pub static FORMAT_CODES: &[(&'static str, &'static str)] = &[
    ("bold", ";1"),
    ("dim", ";2"),
    ("italic", ";3"),
    ("underlined", ";4"),
    ("blinking", ";5"),
    ("fblinking", ";6"),
    ("swap", ";7"),
    ("striked", ";9"),
    ("dunderlined", ";21"),
    ("overlined", ";53"),
];

pub static CONTROL_CHARS: &[(&'static str, &'static str)] = &[
    ("\\003", "\x03"),
    ("\\x1b", "\x1b"),
    ("\\n", "\n"),
    ("\\r", "\r"),
    ("\\t", "\t"),
    ("\\a", "\x07"),
    ("\\b", "\x08"),
    ("\\f", "\x0c"),
    ("\\v", "\x0b"),
];
