use crate::{ctx::Ctx, Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize)]
pub struct Ticket {
    pub id: u64,
    pub cid: u64, // creator user id
    pub title: String,
}

#[derive(Deserialize)]
pub struct TicketForCreate {
    pub title: String,
}

#[derive(Clone)]
pub struct ModelController {
    // `tickets_store` is an Arc (atomic reference count) wrapping a Mutex, which
    // protects a vector of optional tickets.
    //
    // The Arc allows multiple ownership of the underlying data, ensuring that
    // even if we clone the `ModelController` multiple times, they all share the
    // same underlying `tickets_store`.
    //
    // The Mutex ensures exclusive access to the vector. This means that even though
    // multiple `ModelController` instances share the same data, access to the vector
    // is synchronized, preventing data races and ensuring safe concurrent access.
    //
    // So, cloning a `ModelController` results in a new instance that shares the
    // same underlying data, providing a convenient way to work with shared state
    // while ensuring thread safety.
    tickets_store: Arc<Mutex<Vec<Option<Ticket>>>>,
}

// we create the constructor to control the signature
impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tickets_store: Arc::default(),
        })
    }
}

// CRUD impl
impl ModelController {
    pub async fn create_ticket(&self, ctx: Ctx, ticket_fc: TicketForCreate) -> Result<Ticket> {
        let mut store = self.tickets_store.lock().unwrap();

        // hacky: basically index +1, works because rust guarantees that we have
        // exclusive access to the store array
        let id = store.len() as u64;
        let ticket = Ticket {
            id,
            cid: ctx.user_id(),
            title: ticket_fc.title,
        };
        store.push(Some(ticket.clone()));
        Ok(ticket)
    }
    pub async fn list_tickets(&self, _ctx: Ctx) -> Result<Vec<Ticket>> {
        let store = self.tickets_store.lock().unwrap();

        let tickets = store.iter().filter_map(|t| t.clone()).collect();

        Ok(tickets)
    }

    pub async fn delete_ticket(&self, _ctx: Ctx, id: u64) -> Result<Ticket> {
        let mut store = self.tickets_store.lock().unwrap();

        let ticket = store.get_mut(id as usize).and_then(|t| t.take());

        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }
}
