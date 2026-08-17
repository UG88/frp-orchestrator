use crate::api::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use frp_shared::api_types::CreateMappingRequest;
use frp_shared::models::{Allocation, AllocationStatus};
use serde_json::json;

pub async fn list_allocations(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.list_allocations() {
        Ok(allocs) => (StatusCode::OK, Json(json!({ "allocations": allocs }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn list_mappings(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.list_mappings() {
        Ok(mappings) => (StatusCode::OK, Json(json!({ "mappings": mappings }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn create_mapping(
    State(state): State<AppState>,
    Json(payload): Json<CreateMappingRequest>,
) -> impl IntoResponse {
    let node = match state.db.get_node(&payload.node_id) {
        Ok(Some(n)) => n,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Node {} not registered", payload.node_id) })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let alloc = Allocation {
        id: payload.allocation_id.clone(),
        node_id: payload.node_id.clone(),
        server_id: payload.server_id,
        server_name: payload.server_name.clone(),
        pterodactyl_allocation_id: 0,
        local_ip: payload.local_ip,
        local_port: payload.local_port,
        protocol: payload.protocol,
        custom_alias: payload.custom_alias,
        status: AllocationStatus::Pending,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if let Err(e) = state.db.upsert_allocation(&alloc) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to save allocation: {}", e) })),
        )
            .into_response();
    }

    match state
        .allocation_mgr
        .provision_mapping(&node, &alloc, None, None)
        .await
    {
        Ok(mapping) => (StatusCode::CREATED, Json(json!({ "mapping": mapping }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_mapping(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.allocation_mgr.delete_mapping(&id).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "deleted" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
