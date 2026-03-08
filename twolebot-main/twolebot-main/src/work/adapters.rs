use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::work::models::PaginatedResponse;

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn paginated_payload<T: serde::Serialize>(
    paginated: PaginatedResponse<T>,
) -> serde_json::Value {
    let total_pages = if paginated.limit > 0 {
        ((paginated.total as i32) + paginated.limit - 1) / paginated.limit
    } else {
        1
    };

    serde_json::json!({
        "data": paginated.items,
        "pagination": {
            "page": paginated.page,
            "limit": paginated.limit,
            "total": paginated.total,
            "totalPages": total_pages,
        }
    })
}

pub fn work_error(status: StatusCode, msg: impl Into<String>) -> impl IntoResponse {
    (status, Json(ErrorResponse { error: msg.into() }))
}

pub fn to_err_response(e: crate::TwolebotError) -> impl IntoResponse {
    e.into_response()
}
