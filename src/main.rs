use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::{
    sqlite::{SqlitePoolOptions, SqliteRow},
    Row, SqlitePool,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use mortgage_calculator::{compare_mortgages, models::*};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_project,
        list_projects,
        get_project,
        delete_project,
        create_offer,
        list_offers,
        save_project_offers,
        update_offer_name,
        delete_offer,
        compare_offers
    ),
    components(schemas(
        Project,
        SavedOffer,
        CreateProjectRequest,
        CreateOfferRequest,
        SaveProjectOffersRequest,
        SaveOfferRequest,
        UpdateOfferNameRequest,
        CompareRequest,
        CompareResponse,
        MortgageInput,
        MortgageResult
    )),
    tags(
        (name = "Projects", description = "Project management"),
        (name = "Offers", description = "Saved mortgage offers"),
        (name = "Calculator", description = "Mortgage comparison")
    )
)]
struct ApiDoc;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

#[utoipa::path(
    post,
    path = "/api/projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created successfully", body = Project),
        (status = 400, description = "Project name must not be empty"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
async fn create_project(
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO projects (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(&now)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(Project {
            id,
            name: name.to_owned(),
            created_at: now,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/projects",
    responses(
        (status = 200, description = "List of projects", body = Vec<Project>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<Project>>, StatusCode> {
    let rows = sqlx::query("SELECT id, name, created_at FROM projects ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let projects: Vec<Project> = rows
        .iter()
        .map(|row| Project {
            id: row.get(0),
            name: row.get(1),
            created_at: row.get(2),
        })
        .collect();

    Ok(Json(projects))
}

#[utoipa::path(
    get,
    path = "/api/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project details", body = Project),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, StatusCode> {
    let row = sqlx::query("SELECT id, name, created_at FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(Project {
        id: row.get(0),
        name: row.get(1),
        created_at: row.get(2),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses(
        (status = 204, description = "Project deleted"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("DELETE FROM offers WHERE project_id = ?")
        .bind(&id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(&id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/offers",
    request_body = CreateOfferRequest,
    responses(
        (status = 201, description = "Offer created successfully", body = SavedOffer),
        (status = 400, description = "Offer name or mortgage values are invalid"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Offers"
)]
async fn create_offer(
    State(state): State<AppState>,
    Json(req): Json<CreateOfferRequest>,
) -> Result<(StatusCode, Json<SavedOffer>), StatusCode> {
    if req.name.trim().is_empty()
        || !valid_mortgage(
            req.principal,
            req.annual_rate,
            req.years,
            req.insurance_rate,
        )
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let project_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)")
            .bind(&req.project_id)
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !project_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let name = req.name.trim().to_owned();

    sqlx::query("INSERT INTO offers (id, project_id, name, principal, annual_rate, years, insurance_rate, is_best_monthly, is_best_total, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&req.project_id)
        .bind(&name)
        .bind(req.principal)
        .bind(req.annual_rate)
        .bind(req.years as i64)
        .bind(req.insurance_rate)
        .bind(false)
        .bind(false)
        .bind(&now)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(SavedOffer {
            id,
            project_id: req.project_id,
            name,
            principal: req.principal,
            annual_rate: req.annual_rate,
            years: req.years,
            insurance_rate: req.insurance_rate,
            is_best_monthly: false,
            is_best_total: false,
            created_at: now,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/projects/{id}/offers",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "List of offers for a project", body = Vec<SavedOffer>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Offers"
)]
async fn list_offers(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<SavedOffer>>, StatusCode> {
    let rows = sqlx::query("SELECT id, project_id, name, principal, annual_rate, years, insurance_rate, is_best_monthly, is_best_total, created_at FROM offers WHERE project_id = ? ORDER BY created_at ASC")
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let offers = rows.iter().map(saved_offer_from_row).collect();

    Ok(Json(offers))
}

#[utoipa::path(
    put,
    path = "/api/projects/{id}/offers",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    request_body = SaveProjectOffersRequest,
    responses(
        (status = 200, description = "Project offers saved", body = Vec<SavedOffer>),
        (status = 400, description = "Offer names or mortgage values are invalid"),
        (status = 404, description = "Project or offer not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Offers"
)]
async fn save_project_offers(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(req): Json<SaveProjectOffersRequest>,
) -> Result<Json<Vec<SavedOffer>>, StatusCode> {
    let mut submitted_ids = std::collections::HashSet::new();
    for offer in &req.offers {
        if offer.name.trim().is_empty()
            || !valid_mortgage(
                offer.principal,
                offer.annual_rate,
                offer.years,
                offer.insurance_rate,
            )
            || offer
                .id
                .as_ref()
                .is_some_and(|id| !submitted_ids.insert(id.clone()))
        {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let project_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)")
            .bind(&project_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !project_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    let existing_rows = sqlx::query("SELECT id FROM offers WHERE project_id = ?")
        .bind(&project_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let existing_ids: Vec<String> = existing_rows.iter().map(|row| row.get(0)).collect();

    for offer in &req.offers {
        let name = offer.name.trim();
        if let Some(id) = &offer.id {
            let result = sqlx::query(
                "UPDATE offers SET name = ?, principal = ?, annual_rate = ?, years = ?, insurance_rate = ? WHERE id = ? AND project_id = ?",
            )
            .bind(name)
            .bind(offer.principal)
            .bind(offer.annual_rate)
            .bind(offer.years as i64)
            .bind(offer.insurance_rate)
            .bind(id)
            .bind(&project_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if result.rows_affected() == 0 {
                return Err(StatusCode::NOT_FOUND);
            }
        } else {
            let id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO offers (id, project_id, name, principal, annual_rate, years, insurance_rate, is_best_monthly, is_best_total, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(&project_id)
            .bind(name)
            .bind(offer.principal)
            .bind(offer.annual_rate)
            .bind(offer.years as i64)
            .bind(offer.insurance_rate)
            .bind(false)
            .bind(false)
            .bind(now)
            .execute(&mut *transaction)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    for id in existing_ids {
        if !submitted_ids.contains(&id) {
            sqlx::query("DELETE FROM offers WHERE id = ? AND project_id = ?")
                .bind(id)
                .bind(&project_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = sqlx::query("SELECT id, project_id, name, principal, annual_rate, years, insurance_rate, is_best_monthly, is_best_total, created_at FROM offers WHERE project_id = ? ORDER BY created_at ASC")
        .bind(&project_id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows.iter().map(saved_offer_from_row).collect()))
}

fn saved_offer_from_row(row: &SqliteRow) -> SavedOffer {
    SavedOffer {
        id: row.get(0),
        project_id: row.get(1),
        name: row.get(2),
        principal: row.get(3),
        annual_rate: row.get(4),
        years: row.get::<i64, _>(5) as u32,
        insurance_rate: row.get(6),
        is_best_monthly: row.get::<i64, _>(7) != 0,
        is_best_total: row.get::<i64, _>(8) != 0,
        created_at: row.get(9),
    }
}

#[utoipa::path(
    put,
    path = "/api/offers/{id}/name",
    params(
        ("id" = String, Path, description = "Offer ID")
    ),
    request_body = UpdateOfferNameRequest,
    responses(
        (status = 200, description = "Offer name updated", body = SavedOffer),
        (status = 400, description = "Offer name must not be empty"),
        (status = 404, description = "Offer not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Offers"
)]
async fn update_offer_name(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateOfferNameRequest>,
) -> Result<Json<SavedOffer>, StatusCode> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query("UPDATE offers SET name = ? WHERE id = ?")
        .bind(name)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query("SELECT id, project_id, name, principal, annual_rate, years, insurance_rate, is_best_monthly, is_best_total, created_at FROM offers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(SavedOffer {
        id: row.get(0),
        project_id: row.get(1),
        name: row.get(2),
        principal: row.get(3),
        annual_rate: row.get(4),
        years: row.get::<i64, _>(5) as u32,
        insurance_rate: row.get(6),
        is_best_monthly: row.get::<i64, _>(7) != 0,
        is_best_total: row.get::<i64, _>(8) != 0,
        created_at: row.get(9),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/offers/{id}",
    params(
        ("id" = String, Path, description = "Offer ID")
    ),
    responses(
        (status = 204, description = "Offer deleted"),
        (status = 404, description = "Offer not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Offers"
)]
async fn delete_offer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM offers WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/compare",
    request_body = CompareRequest,
    responses(
        (status = 200, description = "Comparison results", body = CompareResponse),
        (status = 400, description = "At least one valid mortgage offer is required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Calculator"
)]
async fn compare_offers(
    Json(req): Json<CompareRequest>,
) -> Result<Json<CompareResponse>, StatusCode> {
    if req.offers.is_empty()
        || req.offers.iter().any(|offer| {
            !valid_mortgage(
                offer.principal,
                offer.annual_rate,
                offer.years,
                offer.insurance_rate,
            )
        })
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let comparison = compare_mortgages(mortgage_calculator::ComparisonRequest {
        mortgages: req.offers,
    });

    Ok(Json(CompareResponse {
        results: comparison.results,
        best_monthly: comparison.best_monthly,
        best_total: comparison.best_total,
    }))
}

fn valid_mortgage(
    principal: f64,
    annual_rate: f64,
    years: u32,
    insurance_rate: Option<f64>,
) -> bool {
    principal.is_finite()
        && principal > 0.0
        && annual_rate.is_finite()
        && annual_rate >= 0.0
        && (1..=40).contains(&years)
        && insurance_rate.is_none_or(|rate| rate.is_finite() && rate >= 0.0)
}

async fn init_database() -> Result<SqlitePool, sqlx::Error> {
    let db_path = "./mortgage_calculator.db";
    let database_url = format!("sqlite:{}?mode=rwc", db_path);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS offers (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            principal REAL NOT NULL,
            annual_rate REAL NOT NULL,
            years INTEGER NOT NULL,
            insurance_rate REAL,
            is_best_monthly BOOLEAN NOT NULL DEFAULT 0,
            is_best_total BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_offers_project_id ON offers(project_id)")
        .execute(&pool)
        .await?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let host = args.next().unwrap_or_else(|| "127.0.0.1".to_owned());
    let port = args
        .next()
        .unwrap_or_else(|| "8081".to_owned())
        .parse::<u16>()?;

    let pool = init_database().await?;
    let state = AppState { db: pool };

    let api_routes = Router::new()
        .route("/api/projects", get(list_projects).post(create_project))
        .route("/api/projects/:id", get(get_project).delete(delete_project))
        .route(
            "/api/projects/:id/offers",
            get(list_offers).put(save_project_offers),
        )
        .route("/api/offers", post(create_offer))
        .route("/api/offers/:id", delete(delete_offer))
        .route("/api/offers/:id/name", put(update_offer_name))
        .route("/api/compare", post(compare_offers))
        .with_state(state.clone());

    let app = Router::new()
        .merge(api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()))
        .fallback_service(ServeDir::new("."));

    let listener = TcpListener::bind((host.as_str(), port)).await?;
    println!("🚀 Server running at http://{host}:{port}");
    println!("📚 Swagger UI at http://{host}:{port}/swagger-ui");
    println!("📄 OpenAPI JSON at http://{host}:{port}/api/openapi.json");
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn saves_project_offers_and_replaces_removed_offers() {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&db)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE offers (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                principal REAL NOT NULL,
                annual_rate REAL NOT NULL,
                years INTEGER NOT NULL,
                insurance_rate REAL,
                is_best_monthly BOOLEAN NOT NULL DEFAULT 0,
                is_best_total BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&db)
        .await
        .unwrap();
        let state = AppState { db };
        let (_, Json(project)) = create_project(
            State(state.clone()),
            Json(CreateProjectRequest {
                name: "Projet test".to_owned(),
            }),
        )
        .await
        .unwrap();

        let make_offer_request = |name: &str| CreateOfferRequest {
            project_id: project.id.clone(),
            name: name.to_owned(),
            principal: 200_000.0,
            annual_rate: 3.5,
            years: 20,
            insurance_rate: Some(0.36),
        };
        let (_, Json(first_offer)) = create_offer(
            State(state.clone()),
            Json(make_offer_request("Offre initiale")),
        )
        .await
        .unwrap();
        let (_, Json(removed_offer)) = create_offer(
            State(state.clone()),
            Json(make_offer_request("Offre supprimée")),
        )
        .await
        .unwrap();

        let Json(saved_offers) = save_project_offers(
            State(state.clone()),
            Path(project.id.clone()),
            Json(SaveProjectOffersRequest {
                offers: vec![
                    SaveOfferRequest {
                        id: Some(first_offer.id.clone()),
                        name: "Offre modifiée".to_owned(),
                        principal: 225_000.0,
                        annual_rate: 3.1,
                        years: 25,
                        insurance_rate: None,
                    },
                    SaveOfferRequest {
                        id: None,
                        name: "Nouvelle offre".to_owned(),
                        principal: 180_000.0,
                        annual_rate: 3.2,
                        years: 15,
                        insurance_rate: Some(0.25),
                    },
                ],
            }),
        )
        .await
        .unwrap();
        assert_eq!(saved_offers.len(), 2);

        let Json(offers) = list_offers(State(state), Path(project.id)).await.unwrap();
        assert_eq!(offers.len(), 2);
        assert!(offers.iter().all(|offer| offer.id != removed_offer.id));
        let updated_offer = offers
            .iter()
            .find(|offer| offer.id == first_offer.id)
            .unwrap();
        assert_eq!(updated_offer.name, "Offre modifiée");
        assert_eq!(updated_offer.principal, 225_000.0);
        assert_eq!(updated_offer.annual_rate, 3.1);
        assert_eq!(updated_offer.years, 25);
        assert_eq!(updated_offer.insurance_rate, None);
        assert!(offers.iter().any(|offer| offer.name == "Nouvelle offre"));
    }
}
