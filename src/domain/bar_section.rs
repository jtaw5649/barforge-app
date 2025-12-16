use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BarSection {
    Left,
    #[default]
    Center,
    Right,
}

impl BarSection {
    pub fn all() -> &'static [Self] {
        &[Self::Left, Self::Center, Self::Right]
    }

    pub fn array_key(&self) -> &'static str {
        match self {
            BarSection::Left => "modules-left",
            BarSection::Center => "modules-center",
            BarSection::Right => "modules-right",
        }
    }
}

impl std::fmt::Display for BarSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BarSection::Left => write!(f, "Left"),
            BarSection::Center => write!(f, "Center"),
            BarSection::Right => write!(f, "Right"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModulePosition {
    pub section: BarSection,
    #[serde(default)]
    pub order: Option<u32>,
}

impl ModulePosition {
    pub fn new(section: BarSection) -> Self {
        Self {
            section,
            order: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_section_default_is_center() {
        assert_eq!(BarSection::default(), BarSection::Center);
    }

    #[test]
    fn test_bar_section_array_keys() {
        assert_eq!(BarSection::Left.array_key(), "modules-left");
        assert_eq!(BarSection::Center.array_key(), "modules-center");
        assert_eq!(BarSection::Right.array_key(), "modules-right");
    }

    #[test]
    fn test_bar_section_display() {
        assert_eq!(format!("{}", BarSection::Left), "Left");
        assert_eq!(format!("{}", BarSection::Center), "Center");
        assert_eq!(format!("{}", BarSection::Right), "Right");
    }

    #[test]
    fn test_module_position_new() {
        let pos = ModulePosition::new(BarSection::Right);
        assert_eq!(pos.section, BarSection::Right);
        assert!(pos.order.is_none());
    }

    #[test]
    fn test_bar_section_serialize() {
        let section = BarSection::Left;
        let json = serde_json::to_string(&section).unwrap();
        assert_eq!(json, r#""left""#);
    }

    #[test]
    fn test_bar_section_deserialize() {
        let section: BarSection = serde_json::from_str(r#""right""#).unwrap();
        assert_eq!(section, BarSection::Right);
    }
}
