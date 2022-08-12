use std::time::Duration;

use druid::Selector;
use uuid::Uuid;

use crate::state::NewSong;

// Query
pub const QUERY_RUN: Selector = Selector::new("query.run");
pub const QUERY_PLAY: Selector = Selector::new("query.play");

// Player
pub const PLAYER_PLAY_PAUSE: Selector = Selector::new("player.play-pause");
pub const PLAYER_TICK: Selector = Selector::new("player.tick");
pub const PLAYER_SEEK: Selector<Duration> = Selector::new("player.seek");
pub const PLAYER_PREV: Selector = Selector::new("player.prev");
pub const PLAYER_NEXT: Selector = Selector::new("player.next");

pub const SONG_PLAY: Selector<Uuid> = Selector::new("song-play");
pub const UI_SONG_EDIT_OPEN: Selector<Uuid> = Selector::new("ui.song-edit.open");
pub const UI_SONG_EDIT_CLOSE: Selector = Selector::new("ui.song-edit.close");
pub const UI_SONG_ADD_OPEN: Selector<String> = Selector::new("ui.song-add.open");
pub const UI_SONG_ADD_CLOSE: Selector = Selector::new("ui.song-add.close");

// Database editing
pub const SONG_ADD: Selector<NewSong> = Selector::new("song.add");
pub const SONG_DELETE: Selector<Uuid> = Selector::new("song.delete");

pub const TAG_ADD: Selector<Uuid> = Selector::new("tag.add");
pub const TAG_RENAME: Selector<(String, String)> = Selector::new("tag.rename");
pub const TAG_TWEAK: Selector<(String, f32)> = Selector::new("tag.tweak");
pub const TAG_DELETE: Selector<String> = Selector::new("tag.delete");
