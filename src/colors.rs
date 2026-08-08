use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use ratatui::style::Color;

const DEFAULT_THEME_NAME: &str = "tokyo-night-moon";

static THEME: RwLock<Option<ActiveTheme>> = RwLock::new(None);

#[derive(Clone)]
struct ActiveTheme {
    name: String,
    colors: Theme,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Theme {
    bg_dark: Color,
    bg_highlight: Color,
    fg: Color,
    fg_dark: Color,
    comment: Color,
    dark5: Color,
    fg_gutter: Color,
    blue: Color,
    blue1: Color,
    cyan: Color,
    green: Color,
    teal: Color,
    purple: Color,
    orange: Color,
    red: Color,
    magenta: Color,
}

#[derive(Default)]
struct ThemeFile {
    colors: HashMap<String, Color>,
    ui: HashMap<String, String>,
    slurm: HashMap<String, String>,
}

pub fn init() {
    let (name, theme) = config_root()
        .map(|root| load_configured_theme(&root))
        .unwrap_or_else(|| (DEFAULT_THEME_NAME.to_string(), fallback()));
    set_active_theme(name, theme);
}

fn current() -> Theme {
    THEME
        .read()
        .unwrap()
        .as_ref()
        .map(|active| active.colors)
        .unwrap_or_else(fallback)
}

fn set_active_theme(name: String, theme: Theme) {
    *THEME.write().unwrap() = Some(ActiveTheme {
        name,
        colors: theme,
    });
}

pub fn current_theme_name() -> String {
    THEME
        .read()
        .unwrap()
        .as_ref()
        .map(|active| active.name.clone())
        .unwrap_or_else(|| DEFAULT_THEME_NAME.to_string())
}

pub fn available_theme_names() -> Vec<String> {
    config_root()
        .map(|root| {
            load_all_themes(&root)
                .into_iter()
                .map(|(name, _)| name)
                .collect()
        })
        .unwrap_or_default()
}

pub fn preview_theme(name: &str) -> bool {
    if !valid_theme_name(name) {
        return false;
    }

    let theme = config_root()
        .and_then(|root| {
            load_all_themes(&root)
                .into_iter()
                .find(|(theme_name, _)| theme_name == name)
                .map(|(_, theme)| theme)
        })
        .or_else(|| (name == DEFAULT_THEME_NAME).then(fallback));

    if let Some(theme) = theme {
        set_active_theme(name.to_string(), theme);
        true
    } else {
        false
    }
}

pub fn bg_dark() -> Color {
    current().bg_dark
}

pub fn bg_highlight() -> Color {
    current().bg_highlight
}

pub fn fg() -> Color {
    current().fg
}

pub fn fg_dark() -> Color {
    current().fg_dark
}

pub fn comment() -> Color {
    current().comment
}

pub fn dark5() -> Color {
    current().dark5
}

pub fn fg_gutter() -> Color {
    current().fg_gutter
}

pub fn blue() -> Color {
    current().blue
}

pub fn blue1() -> Color {
    current().blue1
}

pub fn cyan() -> Color {
    current().cyan
}

pub fn green() -> Color {
    current().green
}

pub fn teal() -> Color {
    current().teal
}

pub fn purple() -> Color {
    current().purple
}

pub fn orange() -> Color {
    current().orange
}

pub fn red() -> Color {
    current().red
}

pub fn magenta() -> Color {
    current().magenta
}

fn config_root() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
}

fn load_configured_theme(root: &Path) -> (String, Theme) {
    let Some(configured_path) = read_config_value(root, "theme") else {
        return (DEFAULT_THEME_NAME.to_string(), fallback());
    };
    let path = expand_home(root, &configured_path);
    if let Ok(content) = std::fs::read_to_string(&path)
        && let Some(theme) = parse_theme(&content, fallback())
    {
        return (theme_name(&path), theme);
    }
    (DEFAULT_THEME_NAME.to_string(), fallback())
}

#[cfg(test)]
fn load_named_theme(root: &Path, name: &str) -> Theme {
    load_all_themes(root)
        .into_iter()
        .find(|(theme_name, _)| theme_name == name)
        .map(|(_, theme)| theme)
        .unwrap_or_else(fallback)
}

fn load_all_themes(root: &Path) -> Vec<(String, Theme)> {
    let mut themes = Vec::new();
    if let Some(catalog_path) = read_config_value(root, "theme_catalog") {
        for path in load_catalog_paths(&expand_home(root, &catalog_path)) {
            load_theme_entry(&mut themes, &path, root);
        }
    }
    if let Some(selected_path) = read_config_value(root, "theme") {
        load_theme_entry(&mut themes, &selected_path, root);
    }
    themes.sort_by(|(left, _), (right, _)| left.cmp(right));
    themes
}

fn load_catalog_paths(catalog_path: &Path) -> Vec<String> {
    let Ok(contents) = std::fs::read_to_string(catalog_path) else {
        return Vec::new();
    };
    let Ok(catalog) = contents.parse::<toml::Value>() else {
        return Vec::new();
    };
    catalog
        .get("themes")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}

fn load_theme_entry(themes: &mut Vec<(String, Theme)>, configured_path: &str, root: &Path) {
    let path = expand_home(root, configured_path);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    let Some(theme) = parse_theme(&content, fallback()) else {
        return;
    };
    let name = theme_name(&path);
    themes.retain(|(existing, _)| existing != &name);
    themes.push((name, theme));
}

fn expand_home(root: &Path, configured_path: &str) -> PathBuf {
    configured_path
        .strip_prefix("~/")
        .map(|rest| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .or_else(|| root.parent().map(Path::to_path_buf))
                .unwrap_or_default()
                .join(rest)
        })
        .unwrap_or_else(|| PathBuf::from(configured_path))
}

fn theme_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(DEFAULT_THEME_NAME)
        .to_string()
}

fn valid_theme_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn read_config_value(root: &Path, requested_key: &str) -> Option<String> {
    let content = std::fs::read_to_string(root.join("slurmtui").join("config.toml")).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            break;
        }
        if let Some((key, value)) = parse_kv(line)
            && key == requested_key
            && !value.is_empty()
        {
            return Some(value.to_string());
        }
    }
    None
}

fn parse_theme(content: &str, base: Theme) -> Option<Theme> {
    let file = parse_theme_file(content)?;

    let raw = |names: &[&str]| {
        names
            .iter()
            .find_map(|name| file.colors.get(*name).copied())
    };
    let resolve = |values: &HashMap<String, String>, names: &[&str]| {
        names.iter().find_map(|name| {
            values
                .get(*name)
                .and_then(|value| resolve_color(value, &file.colors))
        })
    };
    let ui = |names: &[&str]| resolve(&file.ui, names);
    let slurm = |names: &[&str]| resolve(&file.slurm, names);

    Some(Theme {
        bg_dark: slurm(&["background"])
            .or_else(|| ui(&["background_dark", "background"]))
            .or_else(|| raw(&["bg_dark", "mantle", "crust", "bg", "base"]))
            .unwrap_or(base.bg_dark),
        bg_highlight: slurm(&["selection"])
            .or_else(|| ui(&["cursor_bg"]))
            .or_else(|| raw(&["bg_highlight", "surface0", "surface1"]))
            .unwrap_or(base.bg_highlight),
        fg: slurm(&["text"])
            .or_else(|| ui(&["text", "text_bright"]))
            .or_else(|| raw(&["fg", "text"]))
            .unwrap_or(base.fg),
        fg_dark: slurm(&["text_dim"])
            .or_else(|| ui(&["text_dim", "text_muted"]))
            .or_else(|| raw(&["fg_dark", "subtext1", "subtext0", "comment"]))
            .unwrap_or(base.fg_dark),
        comment: slurm(&["text_muted"])
            .or_else(|| ui(&["text_dim", "text_muted"]))
            .or_else(|| raw(&["comment", "overlay0", "fg_dark"]))
            .unwrap_or(base.comment),
        dark5: slurm(&["hint"])
            .or_else(|| ui(&["text_dim", "text_muted"]))
            .or_else(|| raw(&["dark5", "overlay1", "comment", "fg_dark"]))
            .unwrap_or(base.dark5),
        fg_gutter: slurm(&["border"])
            .or_else(|| ui(&["border", "text_muted"]))
            .or_else(|| raw(&["fg_gutter", "surface1", "surface0", "overlay0"]))
            .unwrap_or(base.fg_gutter),
        blue: slurm(&["heading"])
            .or_else(|| ui(&["heading", "selection"]))
            .or_else(|| raw(&["blue", "sapphire"]))
            .unwrap_or(base.blue),
        blue1: slurm(&["completing"])
            .or_else(|| ui(&["picker_directory", "key"]))
            .or_else(|| raw(&["blue1", "sapphire", "blue", "cyan"]))
            .unwrap_or(base.blue1),
        cyan: slurm(&["key"])
            .or_else(|| ui(&["key", "picker_loading"]))
            .or_else(|| raw(&["cyan", "sky", "sapphire", "teal"]))
            .unwrap_or(base.cyan),
        green: slurm(&["success"])
            .or_else(|| ui(&["success"]))
            .or_else(|| raw(&["green"]))
            .unwrap_or(base.green),
        teal: slurm(&["completed"])
            .or_else(|| raw(&["teal", "cyan", "green"]))
            .unwrap_or(base.teal),
        purple: slurm(&["pending", "accent"])
            .or_else(|| raw(&["purple", "pink"]))
            .or_else(|| ui(&["accent", "picker_accent"]))
            .or_else(|| raw(&["magenta", "mauve"]))
            .unwrap_or(base.purple),
        orange: slurm(&["warning"])
            .or_else(|| raw(&["orange", "peach", "yellow"]))
            .unwrap_or(base.orange),
        red: slurm(&["error"])
            .or_else(|| ui(&["error"]))
            .or_else(|| raw(&["red", "maroon"]))
            .unwrap_or(base.red),
        magenta: slurm(&["metric"])
            .or_else(|| raw(&["magenta", "mauve", "purple", "pink"]))
            .or_else(|| ui(&["accent"]))
            .unwrap_or(base.magenta),
    })
}

fn parse_theme_file(content: &str) -> Option<ThemeFile> {
    let mut file = ThemeFile::default();
    let mut section = "";

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            section = match line {
                "[colors]" => "colors",
                "[ui]" => "ui",
                "[slurm]" => "slurm",
                _ => "",
            };
            continue;
        }
        let Some((key, value)) = parse_kv(line) else {
            continue;
        };
        match section {
            "colors" => {
                if let Some(color) = parse_hex_color(value) {
                    file.colors.insert(key.to_string(), color);
                }
            }
            "ui" => {
                file.ui.insert(key.to_string(), value.to_string());
            }
            "slurm" => {
                file.slurm.insert(key.to_string(), value.to_string());
            }
            _ => {}
        }
    }

    if file.colors.is_empty() && file.ui.is_empty() && file.slurm.is_empty() {
        None
    } else {
        Some(file)
    }
}

fn resolve_color(value: &str, colors: &HashMap<String, Color>) -> Option<Color> {
    parse_hex_color(value).or_else(|| colors.get(value).copied())
}

fn parse_kv(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once('=')?;
    Some((
        key.trim(),
        value.trim().trim_matches('"').trim_matches('\''),
    ))
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let value = value.strip_prefix('#')?;
    if value.len() != 6 {
        return None;
    }
    Some(Color::Rgb(
        u8::from_str_radix(&value[0..2], 16).ok()?,
        u8::from_str_radix(&value[2..4], 16).ok()?,
        u8::from_str_radix(&value[4..6], 16).ok()?,
    ))
}

const fn hex(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

const fn fallback() -> Theme {
    Theme {
        bg_dark: hex(0x1e, 0x20, 0x30),
        bg_highlight: hex(0x2f, 0x33, 0x4d),
        fg: hex(0xc8, 0xd3, 0xf5),
        fg_dark: hex(0x82, 0x8b, 0xb8),
        comment: hex(0x63, 0x6d, 0xa6),
        dark5: hex(0x73, 0x7a, 0xa2),
        fg_gutter: hex(0x3b, 0x42, 0x61),
        blue: hex(0x82, 0xaa, 0xff),
        blue1: hex(0x65, 0xbc, 0xff),
        cyan: hex(0x86, 0xe1, 0xfc),
        green: hex(0xc3, 0xe8, 0x8d),
        teal: hex(0x4f, 0xd6, 0xbe),
        purple: hex(0xfc, 0xa7, 0xea),
        orange: hex(0xff, 0x96, 0x6c),
        red: hex(0xff, 0x75, 0x7f),
        magenta: hex(0xc0, 0x99, 0xff),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    #[test]
    fn maps_shared_semantic_theme_roles() {
        let theme = parse_theme(
            r##"
                [colors]
                base = "#1e1e2e"
                surface0 = "#313244"
                text = "#cdd6f4"
                overlay0 = "#6c7086"
                blue = "#89b4fa"
                sapphire = "#74c7ec"
                sky = "#89dceb"
                green = "#a6e3a1"
                teal = "#94e2d5"
                mauve = "#cba6f7"
                peach = "#fab387"
                red = "#f38ba8"

                [ui]
                background = "base"
                cursor_bg = "surface0"
                text = "text"
                text_dim = "overlay0"
                border = "surface0"
                heading = "blue"
                accent = "mauve"
                key = "sky"
            "##,
            fallback(),
        )
        .unwrap();

        assert_eq!(theme.bg_dark, hex(0x1e, 0x1e, 0x2e));
        assert_eq!(theme.fg, hex(0xcd, 0xd6, 0xf4));
        assert_eq!(theme.blue, hex(0x89, 0xb4, 0xfa));
        assert_eq!(theme.cyan, hex(0x89, 0xdc, 0xeb));
        assert_eq!(theme.purple, hex(0xcb, 0xa6, 0xf7));
        assert_eq!(theme.green, hex(0xa6, 0xe3, 0xa1));
    }

    #[test]
    fn catalog_loads_only_explicit_theme_paths() {
        let root = test_config_root();
        let themes_dir = root.join("themes");
        std::fs::create_dir_all(&themes_dir).unwrap();
        std::fs::create_dir_all(root.join("slurmtui")).unwrap();
        std::fs::write(
            themes_dir.join("synthetic-theme.toml"),
            "[colors]\nblue = \"#112233\"\npurple = \"#445566\"\n",
        )
        .unwrap();
        std::fs::write(
            themes_dir.join("unlisted.toml"),
            "[colors]\nblue = \"#ffffff\"\n",
        )
        .unwrap();
        let catalog = root.join("catalog.toml");
        std::fs::write(
            &catalog,
            format!(
                "themes = [\"{}\"]\n",
                themes_dir.join("synthetic-theme.toml").display()
            ),
        )
        .unwrap();
        std::fs::write(
            root.join("slurmtui/config.toml"),
            format!(
                "theme = \"{}\"\ntheme_catalog = \"{}\"\n",
                themes_dir.join("synthetic-theme.toml").display(),
                catalog.display()
            ),
        )
        .unwrap();

        let theme = load_named_theme(&root, "synthetic-theme");
        assert_eq!(theme.blue, hex(0x11, 0x22, 0x33));
        assert_eq!(theme.purple, hex(0x44, 0x55, 0x66));
        assert_eq!(
            load_all_themes(&root)
                .into_iter()
                .map(|(name, _)| name)
                .collect::<Vec<_>>(),
            ["synthetic-theme"]
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_theme_names_that_escape_the_catalog() {
        assert!(valid_theme_name("tokyo-night-moon"));
        assert!(!valid_theme_name("../secrets"));
        assert!(!valid_theme_name("moon.toml"));
    }

    fn test_config_root() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "slurmtui-theme-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
