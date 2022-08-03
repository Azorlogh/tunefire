use druid::{Color, Env, Key};

use crate::State;

pub const ACCENT: Key<Color> = Key::new("theme.accent");
pub const ACCENT_DIM: Key<Color> = Key::new("theme.accent-dim");
pub const BACKGROUND: Key<Color> = Key::new("theme.background");
pub const BACKGROUND_HIGHLIGHT0: Key<Color> = Key::new("theme.background-highlight-0");
pub const BACKGROUND_HIGHLIGHT1: Key<Color> = Key::new("theme.background-highlight-1");
pub const FOREGROUND: Key<Color> = Key::new("theme.foreground");

const fn color(code: usize) -> Color {
	Color::rgb8((code >> 16) as u8, (code >> 8) as u8, code as u8)
}

mod colors {
	use druid::Color;

	use super::color;

	pub const ACCENT: Color = color(0x98c379);
	pub const ACCENT_DIM: Color = color(0x8bb16e);
	pub const BACKGROUND: Color = color(0x282C34);
	pub const BACKGROUND_HIGHLIGHT0: Color = color(0x2c313a);
	pub const BACKGROUND_HIGHLIGHT1: Color = color(0x3a404c);
	pub const FOREGROUND: Color = color(0xffffff);
}

pub fn apply(env: &mut Env, _data: &State) {
	env.set(ACCENT, colors::ACCENT);
	env.set(ACCENT_DIM, colors::ACCENT_DIM);
	env.set(BACKGROUND, colors::BACKGROUND);
	env.set(BACKGROUND_HIGHLIGHT0, colors::BACKGROUND_HIGHLIGHT0);
	env.set(BACKGROUND_HIGHLIGHT1, colors::BACKGROUND_HIGHLIGHT1);
	env.set(FOREGROUND, colors::FOREGROUND);

	{
		use druid::theme::*;
		env.set(WINDOW_BACKGROUND_COLOR, color(0x282C34));
		env.set(BACKGROUND_DARK, color(0x282C34));
		env.set(BACKGROUND_LIGHT, color(0x363b43));

		env.set(TEXT_COLOR, colors::FOREGROUND);
		env.set(CURSOR_COLOR, colors::FOREGROUND);

		env.set(BUTTON_DARK, color(0x282C34));
		env.set(BUTTON_LIGHT, color(0x282C34));

		env.set(BORDER_LIGHT, color(0xFFFFFF));
		env.set(BORDER_DARK, color(0x21252b));
		env.set(PRIMARY_LIGHT, color(0x98c379));

		env.set(WIDGET_PADDING_HORIZONTAL, 10.0);
		env.set(WIDGET_PADDING_VERTICAL, 10.0);
		env.set(TEXTBOX_INSETS, 8.0);
	}
}
