use druid::Selector;
use uuid::Uuid;

use crate::state::TrackImport;

// Query
pub const QUERY_RUN: Selector = Selector::new("query.run");
pub const QUERY_PLAY: Selector = Selector::new("query.play");

pub const UI_TRACK_EDIT_OPEN: Selector<Uuid> = Selector::new("ui.track-edit.open");
pub const UI_TRACK_EDIT_CLOSE: Selector = Selector::new("ui.track-edit.close");
pub const UI_TRACK_IMPORT_OPEN: Selector<TrackImport> = Selector::new("ui.track-import.open");
pub const UI_TRACK_ADD_CLOSE: Selector = Selector::new("ui.track-add.close");

pub const TAG_SEARCH: Selector<String> = Selector::new("tag.search");

// Database editing
pub const TRACK_ADD: Selector<tf_db::Track> = Selector::new("track.add");
pub const TRACK_DELETE: Selector<Uuid> = Selector::new("track.delete");
pub const TRACK_EDIT_TAG: Selector<(Uuid, String, f32)> = Selector::new("track.edit-tag");
