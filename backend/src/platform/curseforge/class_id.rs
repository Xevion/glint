/// CurseForge class IDs for Minecraft content types.
/// Used for filtering search requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ClassId {
    BukkitPlugins = 5,
    Mods = 6,
    ResourcePacks = 12,
    Worlds = 17,
    Modpacks = 4471,
    Customizations = 4546,
    Shaders = 6552,
    Datapacks = 6945,
}
