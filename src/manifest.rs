use axum::extract::Path;
use axum::response::IntoResponse;

pub async fn head(Path((name, reference)): Path<(String, String)>) -> impl IntoResponse {
    todo!()
}