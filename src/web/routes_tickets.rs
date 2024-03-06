use axum::{
    extract::{Path, State},
    routing::{delete, post},
    Json, Router,
};

use crate::{
    model::{ModelController, Ticket, TicketForCreate},
    Result,
};

// NOTE: possibility to use Appstate and substates as well.
// See video at 37:15
pub fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/tickets", post(create_ticket).get(list_tickets))
        .route("/tickets/:id", delete(delete_ticket))
        .with_state(mc)
}

/// State allows us to share the model controller across handlers
/// Axum ensures that the shared data is immutable and accessed in a thread-safe manner.
async fn create_ticket(
    State(mc): State<ModelController>,
    Json(ticket_fc): Json<TicketForCreate>,
) -> Result<Json<Ticket>> {
    println!("->> {:12} - create_ticket", "HANDLER");
    let ticket = mc.create_ticket(ticket_fc).await?;
    Ok(Json(ticket))
}

async fn list_tickets(State(mc): State<ModelController>) -> Result<Json<Vec<Ticket>>> {
    println!("->> {:12} - list_tickets", "HANDLER");
    let tickets = mc.list_tickets().await?;
    Ok(Json(tickets))
}

async fn delete_ticket(
    State(mc): State<ModelController>,
    Path(id): Path<u64>,
) -> Result<Json<Ticket>> {
    println!("->> {:12} - delete_ticket", "HANDLER");
    let ticket = mc.delete_ticket(id).await?;
    Ok(Json(ticket))
}
