use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use poem::{
    http::StatusCode,
    listener::TcpListener,
    middleware::{Compression, Tracing},
    web::CompressionLevel,
    EndpointExt, Response, Route, Server,
};
use poem_openapi::{
    param::{self, Path},
    payload::{self, Json},
    Object, OpenApi, OpenApiService,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use lazy_static::lazy_static;

use rusqlite::Connection;
use utils::{format_radix::format_radix, json_error_middleware::JsonErrorMiddleware, raw_poem_response::RawPoemResponse};

mod utils;

#[derive(Debug, Deserialize, Object, Serialize)]
struct UrlCreationRequest {
    pub url: String,
}

lazy_static! {
    static ref DATA: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref SQLITE: Arc<Mutex<Connection>> = Arc::new(Mutex::new(
        Connection::open("db.sqlite").expect("can't open sqlite database")
    ));
}

struct ShortenApi {
    nested_path: String,
}

#[OpenApi]
impl ShortenApi {
    /// Get item uri from shorthand
    #[oai(path = "/:id", method = "get")]
    async fn get_url(&self, id: Path<String>) -> RawPoemResponse {
        let sqlite = SQLITE.clone();
        let conn = sqlite.lock().unwrap();

        let mut stmt = conn
            .prepare("SELECT key, uri FROM links WHERE key = ? ;")
            .unwrap();
        let mut result = stmt.query([id.0]).unwrap();

        match result.next().unwrap() {
            Some(uri) => RawPoemResponse(
                Response::builder()
                    .status(StatusCode::MOVED_PERMANENTLY)
                    .header("Location", uri.get::<usize, String>(1).unwrap())
                    .finish(),
            ),
            None => RawPoemResponse(Response::builder().status(StatusCode::NOT_FOUND).finish()),
        }
    }

    #[oai(path = "/", method = "post")]
    async fn create_short_url(
        &self,
        host: param::Header<String>,
        input: payload::Json<UrlCreationRequest>,
    ) -> Json<serde_json::Value> {
        let key = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let key = format_radix(key, 36);

        let sqlite = SQLITE.clone();
        let conn = sqlite.lock().unwrap();

        conn.execute(
            "INSERT INTO links (key, uri) VALUES (?1, ?2)",
            (&key, &input.url),
        )
        .unwrap();

        Json(json!({
            "url": format!("{}{}{}", host.0,self.nested_path, key)
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }

    {
        // just drop if can't initialize database
        let con = SQLITE.lock().unwrap();
        con.execute(
            "CREATE TABLE IF NOT EXISTS links (
                key   TEXT PRIMARY KEY,
                uri   TEXT NOT NULL
            )",
            (),
        )
        .unwrap();
    }

    // tracing_subscriber::fmt::init();
    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let api_service = OpenApiService::new(
        ShortenApi {
            nested_path: '/'.to_string(),
        },
        "Link shortener",
        "1.0",
    );
    let ui = api_service.swagger_ui();
    let spec = api_service.spec();

    Server::new(TcpListener::bind("0.0.0.0:3366"))
        .run_with_graceful_shutdown(
            Route::new()
                .nest("/docs", ui)
                .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
                .nest(
                    "/",
                    api_service.with(Compression::new().with_quality(CompressionLevel::Best)),
                )
                .with(Tracing)
                .with(JsonErrorMiddleware),
            async move {
                let _ = tokio::signal::ctrl_c().await;
            },
            Some(Duration::from_secs(5)),
        )
        .await?;
    // TODO add on close routine for normal server stop
    info!("application: stop");

    Ok(())
}
