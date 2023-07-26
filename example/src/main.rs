use std::sync::{Arc, Mutex};

use axum::{extract::State, Router};
use cqrs_server::*;
use example::aspe_cts::tests::contracts::{
    manager::configuration::sites::{CreateSite, CreateSiteErrorCodes},
    technician::{MyWorkFor, WorkOrderDto}, shared::AddressDto,
};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

#[derive(Clone)]
struct AppState(Arc<Mutex<Vec<WorkOrderDto>>>);

async fn router() {
    let app = Router::new()
        .command(create_site)
        .query(my_work_for)
        .with_state(AppState(Arc::new(Mutex::new(Vec::new()))))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_site(
    State(state): State<AppState>,
    CQRSInput(input): CQRSInput<CreateSite>,
) -> CommandResult<CreateSite> {
    validate_site(&input)?;

    state.0.lock().unwrap().push(WorkOrderDto {
        site_id: input.name.clone(),
        order_id: input.name.clone(),
        site_name: input.name.clone(),
        site_address: AddressDto {
            line_1: "".to_string(),
            line_2: "".to_string(),
            line_3: "".to_string(),
            line_4: input.division_id.to_string(),
        },
        contract_number: input.name.clone(),
        contact: None,
        test_definitions: vec![],
    });
    CommandResult::success()
}

fn validate_site(input: &CreateSite) -> CommandResult<CreateSite> {
    if input.name.is_empty() {
        CommandResult::single_error_code(CreateSiteErrorCodes::NameIsEmpty)
    } else if input.name.len() > 10 {
        CommandResult::single_error_code(CreateSiteErrorCodes::NameIsTooLong)
    } else {
        CommandResult::success()
    }
}

async fn my_work_for(
    State(state): State<AppState>,
    CQRSInput(_): CQRSInput<MyWorkFor>,
) -> QueryResult<MyWorkFor> {
    let data = state.0.lock().unwrap();
    QueryResult::new(&*data)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    router().await;
}
