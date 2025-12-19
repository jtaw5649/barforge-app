mod bar_section;
mod installed;

pub use bar_section::{BarSection, ModulePosition};
pub use installed::InstalledModule;
pub use waybar_registry_types::{
    Author, AuthorProfile, CategoryInfo, ModuleCategory, ModuleUuid, ModuleUuidError,
    ModuleVersion, RegistryIndex, RegistryModule, Review, ReviewUser, ReviewsResponse,
};
