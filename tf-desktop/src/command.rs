use std::time::Duration;

use druid::Selector;
use uuid::Uuid;

use crate::state::NewTrack;

// Query
pub const QUERY_RUN: Selector = Selector::new("query.run");
pub const QUERY_PLAY: Selector = Selector::new("query.play");

// Player
pub const PLAYER_PLAY_PAUSE: Selector = Selector::new("player.play-pause");
pub const PLAYER_TICK: Selector = Selector::new("player.tick");
pub const PLAYER_SEEK: Selector<Duration> = Selector::new("player.seek");
pub const PLAYER_PREV: Selector = Selector::new("player.prev");
pub const PLAYER_NEXT: Selector = Selector::new("player.next");

pub const TRACK_PLAY: Selector<Uuid> = Selector::new("track-play");
pub const UI_TRACK_EDIT_OPEN: Selector<Uuid> = Selector::new("ui.track-edit.open");
pub const UI_TRACK_EDIT_CLOSE: Selector = Selector::new("ui.track-edit.close");
pub const UI_TRACK_ADD_OPEN: Selector<String> = Selector::new("ui.track-add.open");
pub const UI_TRACK_ADD_CLOSE: Selector = Selector::new("ui.track-add.close");

// Database editing
pub const TRACK_ADD: Selector<NewTrack> = Selector::new("track.add");
pub const TRACK_DELETE: Selector<Uuid> = Selector::new("track.delete");

pub const TAG_ADD: Selector<Uuid> = Selector::new("tag.add");
pub const TAG_RENAME: Selector<(String, String)> = Selector::new("tag.rename");
pub const TAG_TWEAK: Selector<(String, f32)> = Selector::new("tag.tweak");
pub const TAG_DELETE: Selector<String> = Selector::new("tag.delete");
