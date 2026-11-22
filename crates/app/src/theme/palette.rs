/// The colors of the interface, as `0xRRGGBB`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub window: u32,
    pub sidebar: u32,
    pub surface: u32,
    pub surface_hover: u32,
    pub surface_active: u32,
    pub border: u32,
    pub accent: u32,
    pub text: u32,
    pub text_muted: u32,
    pub text_faint: u32,
    pub attention: u32,
    pub working: u32,
    pub danger: u32,
    /// Text that can be clicked: the ports of a card.
    pub link: u32,
    pub badge: u32,
    /// Text on `badge` and on `accent`.
    pub on_accent: u32,
    /// Where text is typed.
    pub field: u32,
}

pub const DARK: Palette = Palette {
    window: 0x161618,
    sidebar: 0x1c1c1f,
    surface: 0x232327,
    surface_hover: 0x2b2b30,
    surface_active: 0x33333a,
    border: 0x2a2a2e,
    accent: 0x3f6fd0,
    text: 0xe6e6ea,
    text_muted: 0xb4b4bc,
    text_faint: 0x9c9cab,
    attention: 0xe5a50a,
    working: 0x57a773,
    danger: 0xf07a75,
    link: 0x8fb4f5,
    badge: 0x3d6fd6,
    on_accent: 0xffffff,
    field: 0x101012,
};

pub const LIGHT: Palette = Palette {
    window: 0xffffff,
    sidebar: 0xf3f3f5,
    surface: 0xe9e9ed,
    surface_hover: 0xe0e0e6,
    surface_active: 0xd5d5dd,
    border: 0xd0d0d8,
    accent: 0x2a5fc0,
    text: 0x1c1c1f,
    text_muted: 0x4a4a54,
    text_faint: 0x585862,
    attention: 0x7a5200,
    working: 0x2e7d4f,
    danger: 0xa82a1e,
    link: 0x1f4fa8,
    badge: 0x2a5fc0,
    on_accent: 0xffffff,
    field: 0xffffff,
};
