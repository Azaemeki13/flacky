use axum::{extract::State, extract::Path, Json};
use sqlx::PgPool;
use crate::User;
use crate::UserPayload;
use crate::Song;
use crate::SongPayload;
use crate::Playlist;
use crate::PlaylistPayload;
use crate::PlaylistResponse;

#[axum::debug_handler]
pub async fn ping_handler() -> &'static str 
{
    "pong"
}

#[axum::debug_handler]
pub async fn create_user_handler(State(pool): State<PgPool>,
    axum::extract::Json(payload): axum::extract::Json<UserPayload>,) -> Result<Json<User>, axum::http::StatusCode>
{
    let user = sqlx::query_as::<_, User>
        ("INSERT INTO users (username) VALUES ($1) RETURNING id, username, created_at")
        .bind(payload.username)
        .fetch_one(&pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok (Json(user))

}

pub async fn get_all_users_handler(State(pool): State<PgPool>, ) -> Result<Json<Vec<User>>, axum::http::StatusCode>
{
    let users = sqlx::query_as::<_,User>("SELECT id, username, created_at FROM users ORDER BY created_at ASC")
        .fetch_all(&pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(axum::Json(users))
}

pub async fn get_user_by_id_handler(State(pool): State<PgPool>, Path(user_id):Path<uuid::Uuid>,) ->Result<Json<User>, axum::http::StatusCode>
{
    let user = sqlx::query_as::<_,User>("SELECT id, username, created_at FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    match user
    {
        Some(found_user) => Ok(Json(found_user)),
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

pub async fn create_song_handler(State(pool): State<PgPool>, 
    axum::extract::Json(payload): axum::extract::Json<SongPayload>, )->Result<Json<Song>, axum::http::StatusCode>
{
    let song = sqlx::query_as!
        (Song, "INSERT INTO songs (title, artist, duration_seconds, audio_url, ml_features) VALUES ($1, $2, $3, $4, $5) RETURNING id, title, artist, duration_seconds, audio_url, ml_features, created_at", payload.title, payload.artist, payload.duration_seconds, payload.audio_url,
         payload.ml_features)
        .fetch_one(&pool)
        .await
        .map_err(|e| 
            {eprintln!("Database song error {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR})?;
    Ok(Json(song))
}

pub async fn create_playlist_handler(State(pool): State<PgPool>,
    axum::extract::Json(payload): axum::extract::Json<PlaylistPayload>, )->Result<Json<Playlist>, axum::http::StatusCode>
{
    let playlist = sqlx::query_as!
        (Playlist, "INSERT INTO playlists (name, owner_id) VALUES ($1, $2) RETURNING id, name, owner_id, created_at", payload.name, payload.owner_id)
        .fetch_one(&pool)
        .await
        .map_err(|e|
            {eprintln!("Database playlist error {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR})?;
    Ok(Json(playlist))
}

pub async fn add_song_to_playlist_handler(State(pool): State<PgPool>,
    axum::extract::Path((playlist_id, song_id)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,) -> Result<axum::http::StatusCode, axum::http::StatusCode>
{
    sqlx::query!
        ("INSERT INTO playlist_songs (playlist_id, song_id) VALUES ($1, $2)",
        playlist_id, song_id)
        .execute(&pool)
        .await
        .map_err
        (|e| 
         {
            eprintln!("Link error ! {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
         })?;
    Ok(axum::http::StatusCode::CREATED)

}

pub async fn get_playlist_by_id_handler(
    State(pool): State<PgPool>, axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    ) -> Result<Json<PlaylistResponse>, axum::http::StatusCode>
{
    let playlist = sqlx::query_as! (PlaylistResponse, r#"
                                            SELECT p.id, p.name, p.owner_id,
                                            COALESCE(json_agg(json_build_object('id', s.id, 'title', s.title, 'artist', s.artist, 'duration_seconds', s.duration_seconds))
                                            FILTER (WHERE s.id IS NOT NULL), '[]') as "songs!"
                                            FROM playlists p
                                            LEFT JOIN playlist_songs ps ON p.id = ps.playlist_id
                                            LEFT JOIN songs s ON ps.song_id = s.id
                                            WHERE p.id = $1 GROUP BY p.id"#, id)
        .fetch_optional(&pool)
        .await
        .map_err(|e|
            {
                eprintln!("Query erreur when getting playlist! {}", e);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR})?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(playlist))
}
