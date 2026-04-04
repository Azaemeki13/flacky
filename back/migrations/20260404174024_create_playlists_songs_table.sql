CREATE TABLE playlist_songs (
	playlist_id UUID NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
	song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
	added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	PRIMARY KEY(playlist_id, song_id)
);
