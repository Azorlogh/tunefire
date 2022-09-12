use druid::Selector;
use uuid::Uuid;

use crate::state::NewTrack;

// Query
pub const QUERY_RUN: Selector = Selector::new("query.run");
pub const QUERY_PLAY: Selector = Selector::new("query.play");

pub const UI_TRACK_EDIT_OPEN: Selector<Uuid> = Selector::new("ui.track-edit.open");
pub const UI_TRACK_EDIT_CLOSE: Selector = Selector::new("ui.track-edit.close");
pub const UI_TRACK_ADD_OPEN: Selector<String> = Selector::new("ui.track-add.open");
pub const UI_TRACK_ADD_CLOSE: Selector = Selector::new("ui.track-add.close");

pub const TAG_SEARCH: Selector<String> = Selector::new("tag.search");

// Database editing
pub const TRACK_ADD: Selector<NewTrack> = Selector::new("track.add");
pub const TRACK_DELETE: Selector<Uuid> = Selector::new("track.delete");
