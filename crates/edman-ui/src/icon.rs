use iced::{
    font::{Family, Stretch, Weight},
    Font,
};

pub const ANGLE_RIGHT: char = '\u{f105}';
pub const ANGLE_DOWN: char = '\u{f107}';

pub const ICON_FONT: Font = Font {
    family: Family::Name("Font Awesome 6 Free"),
    weight: Weight::Black,
    stretch: Stretch::Normal,
    monospaced: false,
};
