//! The icon themes of XP, for the start menu, the title bars and the task
//! buttons.

/// The themes to search, in order, by the start of their directory name.
/// XP's own icons come from `WinXPSVG` or `WindowsXP`, when one is
/// installed; they come with suffixes such as
/// `WinXPSVG-plasma5up-scalable-icontheme-blackysgate.de`.
pub const THEMES: [&str; 7] = [
    "WinXPSVG",
    "WindowsXP",
    "Adwaita",
    "breeze",
    "Papirus",
    "Humanity",
    "hicolor",
];
