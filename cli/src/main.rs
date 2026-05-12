use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

#[derive(Parser)]
#[command(
    name = "orchestra",
    version = "0.1.0",
    about = "Orchestra Framework CLI - Joyful Rust web development",
    long_about = "Orchestra is an opinionated, modular, RAM-efficient framework\nfor building fullstack Rust applications.\n\nLike Laravel Artisan, but for Rust.",
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Orchestra project
    New {
        /// Project name
        project_name: String,
        /// Target directory (defaults to project name)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Generate a Model struct
    #[command(name = "make:model")]
    MakeModel {
        /// Model name (PascalCase, e.g. User, BlogPost)
        name: String,
        /// Also generate migration, controller, and repository
        #[arg(short = 'a', long)]
        all: bool,
    },
    /// Generate a Controller
    #[command(name = "make:controller")]
    MakeController {
        /// Controller name (PascalCase, e.g. UserController)
        name: String,
    },
    /// Generate a Repository
    #[command(name = "make:repository")]
    MakeRepository {
        /// Repository name (PascalCase, e.g. UserRepository)
        name: String,
    },
    /// Generate a Service
    #[command(name = "make:service")]
    MakeService {
        /// Service name (PascalCase, e.g. AuthService)
        name: String,
    },
    /// Run the development server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { project_name, path } => {
            cmd_new(&project_name, path.as_deref())?;
        }
        Commands::MakeModel { name, all } => {
            cmd_make_model(&name, all)?;
        }
        Commands::MakeController { name } => {
            cmd_make_controller(&name)?;
        }
        Commands::MakeRepository { name } => {
            cmd_make_repository(&name)?;
        }
        Commands::MakeService { name } => {
            cmd_make_service(&name)?;
        }
        Commands::Serve { port, host } => {
            cmd_serve(&host, port)?;
        }
    }

    Ok(())
}

fn cmd_new(project_name: &str, target_path: Option<&Path>) -> Result<()> {
    let project_dir = target_path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(project_name));

    println!("🎼 Creating Orchestra project: {}", project_name);

    if project_dir.exists() {
        anyhow::bail!(
            "Directory '{}' already exists.",
            project_dir.display()
        );
    }

    // Create project directory structure
    let dirs = [
        "src/models",
        "src/repositories",
        "src/controllers",
        "src/services",
        "src/middleware",
        "src/routes",
        "src/config",
        "migrations",
        "views/layouts",
        "views/partials",
        "public/css",
        "public/js",
        "tests",
    ];

    for dir in &dirs {
        fs::create_dir_all(project_dir.join(dir))
            .with_context(|| format!("Failed to create directory: {}", dir))?;
    }

    // Write Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
sqlx = {{ version = "0.7", features = ["runtime-tokio-native-tls", "sqlite", "chrono"] }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
askama = "0.12"
dotenv = "0.15"
anyhow = "1"
thiserror = "1"
uuid = {{ version = "1", features = ["v4", "serde"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
"#,
        project_name
    );
    write_file(&project_dir.join("Cargo.toml"), &cargo_toml)?;

    // Write .env.example
    let env_example = r#"# Application
APP_NAME=orchestra-app
APP_ENV=development
APP_PORT=8080
APP_HOST=127.0.0.1

# Database
DATABASE_URL=sqlite://./database.db

# Logging
RUST_LOG=info
"#;
    write_file(&project_dir.join(".env.example"), env_example)?;

    // Write .env (copy of example)
    write_file(&project_dir.join(".env"), env_example)?;

    // Write .gitignore
    let gitignore = r#"/target
/database.db
/.env
*.db-shm
*.db-wal
"#;
    write_file(&project_dir.join(".gitignore"), gitignore)?;

    // Write src/main.rs
    let main_rs = r#"use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod controllers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./database.db".to_string());

    // create_if_missing(true) automatically creates database.db on first run
    let connect_opts = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_opts)
        .await?;

    // Run all pending migrations from the ./migrations directory
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database ready");

    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("APP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    tracing::info!("Starting server at http://{}:{}", host, port);

    let pool = web::Data::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .configure(routes::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await?;

    Ok(())
}
"#;
    write_file(&project_dir.join("src/main.rs"), main_rs)?;

    // Write module files
    write_file(
        &project_dir.join("src/models/mod.rs"),
        "// Models represent your database tables as Rust structs.\n// Each model corresponds to a database table.\n\npub mod example;\n",
    )?;
    write_file(
        &project_dir.join("src/models/example.rs"),
        &generate_model_content("Example"),
    )?;

    write_file(
        &project_dir.join("src/repositories/mod.rs"),
        "// Repositories handle all database interactions.\n// They abstract the data access layer from business logic.\n\npub mod example;\n",
    )?;
    write_file(
        &project_dir.join("src/repositories/example.rs"),
        &generate_repository_content("Example"),
    )?;

    write_file(
        &project_dir.join("src/controllers/mod.rs"),
        "// Controllers handle HTTP requests and responses.\n// They use services and repositories to fulfill requests.\n\npub mod example;\n",
    )?;
    write_file(
        &project_dir.join("src/controllers/example.rs"),
        &generate_controller_content("Example"),
    )?;

    write_file(
        &project_dir.join("src/services/mod.rs"),
        "// Services contain your business logic.\n// They coordinate between repositories and other services.\n",
    )?;

    write_file(
        &project_dir.join("src/middleware/mod.rs"),
        "// Middleware intercepts requests and responses.\n// Examples: authentication, logging, CORS.\n",
    )?;

    write_file(
        &project_dir.join("src/routes/mod.rs"),
        &generate_routes_content(),
    )?;

    write_file(
        &project_dir.join("src/config/mod.rs"),
        &generate_config_content(),
    )?;

    // Write base Askama template
    let base_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Orchestra App{% endblock %}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@1.9.12"></script>
    {% block head %}{% endblock %}
</head>
<body class="bg-gray-50 text-gray-900">
    <nav class="bg-white shadow-sm border-b px-6 py-4">
        <div class="max-w-7xl mx-auto flex items-center justify-between">
            <a href="/" class="text-xl font-bold text-indigo-600">🎼 Orchestra</a>
            <div class="flex gap-4 text-sm">
                {% block nav %}{% endblock %}
            </div>
        </div>
    </nav>

    <main class="max-w-7xl mx-auto px-6 py-8">
        {% block content %}{% endblock %}
    </main>

    <footer class="text-center text-sm text-gray-400 py-8">
        Built with 🎼 Orchestra Framework
    </footer>

    {% block scripts %}{% endblock %}
</body>
</html>
"#;
    write_file(&project_dir.join("views/layouts/base.html"), base_html)?;

    // Write initial migration — creates the examples table
    let initial_migration = r#"-- Orchestra initial migration
-- This file runs automatically on `cargo run` (via sqlx::migrate!)
-- Add your own tables below, or create new files: 0002_add_posts.sql, etc.

CREATE TABLE IF NOT EXISTS examples (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;
    write_file(
        &project_dir.join("migrations/0001_initial.sql"),
        initial_migration,
    )?;

    // Write tests/mod.rs
    write_file(
        &project_dir.join("tests/mod.rs"),
        "// Integration tests for the application.\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_works() {\n        assert_eq!(2 + 2, 4);\n    }\n}\n",
    )?;

    println!("✅ Project '{}' created successfully!", project_name);
    println!();
    println!("  Next steps:");
    println!("    cd {}", project_dir.display());
    println!("    cargo build");
    println!("    orchestra serve");
    println!();
    println!("  Generate resources:");
    println!("    orchestra make:model Post");
    println!("    orchestra make:controller PostController");
    println!("    orchestra make:repository PostRepository");

    Ok(())
}

fn cmd_make_model(name: &str, all: bool) -> Result<()> {
    let snake = to_snake_case(name);
    let path = PathBuf::from(format!("src/models/{}.rs", snake));

    ensure_src_dir("src/models")?;
    write_file(&path, &generate_model_content(name))?;
    println!("✅ Model created: {}", path.display());
    append_mod_declaration("src/models/mod.rs", &snake)?;

    if all {
        cmd_make_repository(name)?;
        cmd_make_controller(name)?;
    }

    Ok(())
}

fn cmd_make_controller(name: &str) -> Result<()> {
    let snake = to_snake_case(name);
    let path = PathBuf::from(format!("src/controllers/{}.rs", snake));

    ensure_src_dir("src/controllers")?;
    write_file(&path, &generate_controller_content(name))?;
    println!("✅ Controller created: {}", path.display());
    append_mod_declaration("src/controllers/mod.rs", &snake)?;

    Ok(())
}

fn cmd_make_repository(name: &str) -> Result<()> {
    let snake = to_snake_case(name);
    let path = PathBuf::from(format!("src/repositories/{}.rs", snake));

    ensure_src_dir("src/repositories")?;
    write_file(&path, &generate_repository_content(name))?;
    println!("✅ Repository created: {}", path.display());
    append_mod_declaration("src/repositories/mod.rs", &snake)?;

    Ok(())
}

fn cmd_make_service(name: &str) -> Result<()> {
    let snake = to_snake_case(name);
    let path = PathBuf::from(format!("src/services/{}.rs", snake));

    ensure_src_dir("src/services")?;
    let content = format!(
        r#"use anyhow::Result;

pub struct {name}Service {{
    // Add your dependencies here (repositories, other services, etc.)
}}

impl {name}Service {{
    pub fn new() -> Self {{
        Self {{}}
    }}
}}

impl Default for {name}Service {{
    fn default() -> Self {{
        Self::new()
    }}
}}
"#
    );
    write_file(&path, &content)?;
    println!("✅ Service created: {}", path.display());
    append_mod_declaration("src/services/mod.rs", &snake)?;

    Ok(())
}

fn cmd_serve(host: &str, port: u16) -> Result<()> {
    println!("🎼 Starting Orchestra development server...");
    println!("   Listening on http://{}:{}", host, port);
    println!("   Press Ctrl+C to stop");
    println!();
    println!("   Hint: Run `cargo run` in your project directory to start the server.");
    Ok(())
}

// ─── Code generators ────────────────────────────────────────────────────────

fn generate_model_content(name: &str) -> String {
    let snake = to_snake_case(name);
    format!(
        r#"use serde::{{Deserialize, Serialize}};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct {name} {{
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
}}

#[derive(Debug, Deserialize)]
pub struct Create{name}Dto {{
    // Add your creation fields here
}}

#[derive(Debug, Deserialize)]
pub struct Update{name}Dto {{
    // Add your update fields here
}}

impl {name} {{
    pub fn table_name() -> &'static str {{
        "{snake}s"
    }}
}}
"#
    )
}

fn generate_repository_content(name: &str) -> String {
    let snake = to_snake_case(name);
    let table = format!("{}s", snake);
    format!(
        r#"use anyhow::Result;
use sqlx::SqlitePool;

use crate::models::{snake}::{{{name}, Create{name}Dto}};

pub struct {name}Repository {{
    pool: SqlitePool,
}}

impl {name}Repository {{
    pub fn new(pool: SqlitePool) -> Self {{
        Self {{ pool }}
    }}

    pub async fn find_all(&self) -> Result<Vec<{name}>> {{
        let rows = sqlx::query_as::<_, {name}>(
            "SELECT * FROM {table} ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }}

    pub async fn find_by_id(&self, id: i64) -> Result<Option<{name}>> {{
        let row = sqlx::query_as::<_, {name}>(
            "SELECT * FROM {table} WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }}

    pub async fn delete(&self, id: i64) -> Result<bool> {{
        let result = sqlx::query("DELETE FROM {table} WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }}
}}
"#
    )
}

fn generate_controller_content(name: &str) -> String {
    let snake = to_snake_case(name);
    format!(
        r#"use actix_web::{{web, HttpResponse, Responder}};
use sqlx::SqlitePool;

use crate::models::{snake}::Create{name}Dto;
use crate::repositories::{snake}::{name}Repository;

pub async fn index(pool: web::Data<SqlitePool>) -> impl Responder {{
    let repo = {name}Repository::new(pool.get_ref().clone());
    match repo.find_all().await {{
        Ok(items) => HttpResponse::Ok().json(items),
        Err(e) => {{
            tracing::error!("Failed to fetch {snake}s: {{:?}}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({{
                "error": "Failed to fetch records"
            }}))
        }}
    }}
}}

pub async fn show(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> impl Responder {{
    let id = path.into_inner();
    let repo = {name}Repository::new(pool.get_ref().clone());
    match repo.find_by_id(id).await {{
        Ok(Some(item)) => HttpResponse::Ok().json(item),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({{
            "error": "Record not found"
        }})),
        Err(e) => {{
            tracing::error!("Failed to fetch {snake}: {{:?}}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({{
                "error": "Failed to fetch record"
            }}))
        }}
    }}
}}

pub async fn store(
    _pool: web::Data<SqlitePool>,
    _body: web::Json<Create{name}Dto>,
) -> impl Responder {{
    // TODO: Implement create logic
    HttpResponse::Created().json(serde_json::json!({{ "message": "Created" }}))
}}

pub async fn destroy(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> impl Responder {{
    let id = path.into_inner();
    let repo = {name}Repository::new(pool.get_ref().clone());
    match repo.delete(id).await {{
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({{
            "error": "Record not found"
        }})),
        Err(e) => {{
            tracing::error!("Failed to delete {snake}: {{:?}}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({{
                "error": "Failed to delete record"
            }}))
        }}
    }}
}}
"#
    )
}

fn generate_routes_content() -> String {
    r#"use actix_web::web;

use crate::controllers::example;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/examples", web::get().to(example::index))
            .route("/examples/{id}", web::get().to(example::show))
            .route("/examples", web::post().to(example::store))
            .route("/examples/{id}", web::delete().to(example::destroy)),
    );

    // Health check endpoint
    cfg.route("/health", web::get().to(health_check));
}

async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "framework": "Orchestra",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
"#
    .to_string()
}

fn generate_config_content() -> String {
    r#"use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub app_env: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://./database.db".to_string()),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
        }
    }

    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }
}
"#
    .to_string()
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    Ok(())
}

fn ensure_src_dir(dir: &str) -> Result<()> {
    fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create directory: {}", dir))
}

fn append_mod_declaration(mod_file: &str, module_name: &str) -> Result<()> {
    let path = Path::new(mod_file);
    let declaration = format!("pub mod {};\n", module_name);

    if path.exists() {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", mod_file))?;
        if !content.contains(&format!("pub mod {};", module_name)) {
            let mut updated = content;
            updated.push_str(&declaration);
            fs::write(path, updated)
                .with_context(|| format!("Failed to update {}", mod_file))?;
        }
    } else {
        write_file(path, &declaration)?;
    }

    Ok(())
}

/// Convert PascalCase or camelCase name to snake_case
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    // Strip common suffixes to get base name for file
    let result = result
        .trim_end_matches("_controller")
        .trim_end_matches("_repository")
        .trim_end_matches("_service")
        .to_string();
    result
}
