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
    /// Generate authentication scaffolding (register, login, JWT, roles)
    #[command(name = "make:auth")]
    MakeAuth {
        /// Include role-based authorization (admin, manager, user...)
        #[arg(long)]
        with_role: bool,
        /// Include forgot password flow
        #[arg(long)]
        forgot: bool,
        /// Login identifier (email, phone, or both)
        #[arg(long, default_value = "email")]
        login_via: String,
        /// JSON-only API (no template views, just JSON responses)
        #[arg(long)]
        json_only: bool,
        /// Comma-separated roles list (e.g. admin,manager,user)
        #[arg(long)]
        roles: Option<String>,
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
        Commands::MakeAuth {
            with_role,
            forgot,
            login_via,
            json_only,
            roles,
        } => {
            cmd_make_auth(with_role, forgot, &login_via, json_only, roles.as_deref())?;
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
        "templates/layouts",
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
mod views;

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
        "// Controllers handle HTTP requests and responses.\n// They use services and repositories to fulfill requests.\n\npub mod home;\npub mod example;\n",
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

    // Write views module
    let views_mod = r#"use askama::Template;

#[derive(Template)]
#[template(path = "welcome.html")]
pub struct WelcomeTemplate;
"#;
    write_file(&project_dir.join("src/views/mod.rs"), views_mod)?;

    // Write home controller
    let home_controller = r#"use actix_web::HttpResponse;
use askama::Template;
use crate::views::WelcomeTemplate;

pub async fn welcome() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(WelcomeTemplate.render().unwrap())
}
"#;
    write_file(&project_dir.join("src/controllers/home.rs"), home_controller)?;

    // Write Askama templates
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
    write_file(&project_dir.join("templates/layouts/base.html"), base_html)?;

    let welcome_html = r#"{% extends "layouts/base.html" %}

{% block content %}
<div class="text-center py-16">
    <h1 class="text-5xl font-bold text-indigo-600 mb-4">🎼 Orchestra</h1>
    <p class="text-xl text-gray-600 mb-8">Your Rust web framework is ready.</p>
    <div class="flex justify-center gap-4">
        <a href="/health" class="px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700">Health Check</a>
    </div>
</div>
{% endblock %}
"#;
    write_file(&project_dir.join("templates/welcome.html"), welcome_html)?;

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

fn cmd_make_auth(
    with_role: bool,
    forgot: bool,
    login_via: &str,
    json_only: bool,
    roles: Option<&str>,
) -> Result<()> {
    println!("🔐 Generating authentication scaffolding...\n");

    let roles_list: Vec<&str> = parse_roles(roles);
    let ts = 20260526120000u64;

    // ── Ensure required directories ──────────────────────────────────────────
    for dir in &[
        "src/models",
        "src/controllers",
        "src/middleware",
        "src/routes",
        "migrations",
    ] {
        ensure_src_dir(dir)?;
    }
    if !json_only {
        ensure_src_dir("views/auth")?;
    }

    // ── Migration: users table ───────────────────────────────────────────────
    let login_col = match login_via {
        "phone" => "    phone       TEXT UNIQUE NOT NULL,",
        "both" => "    email       TEXT UNIQUE,\n    phone       TEXT UNIQUE,",
        _ => "    email       TEXT UNIQUE NOT NULL,\n    phone       TEXT UNIQUE,",
    };
    let users_migration = format!(
        r#"-- Orchestra auth migration: users table
-- Generated by `orchestra make:auth`

CREATE TABLE IF NOT EXISTS users (
    id           TEXT PRIMARY KEY,
{login_col}
    password_hash TEXT NOT NULL,
    name         TEXT NOT NULL DEFAULT '',
    verified_at  TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
"#
    );
    write_file(&PathBuf::from(format!("migrations/{}_create_users_table.sql", ts)), &users_migration)?;
    println!("  ✓ migrations/{}_create_users_table.sql", ts);

    // ── Migration: roles table (if --with-role) ──────────────────────────────
    if with_role {
        let roles_migration = r#"-- Orchestra auth migration: roles table

CREATE TABLE IF NOT EXISTS roles (
    id         TEXT PRIMARY KEY,
    name       TEXT UNIQUE NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;
        write_file(
            &PathBuf::from(format!("migrations/{}_create_roles_table.sql", ts + 1)),
            roles_migration,
        )?;
        println!("  ✓ migrations/{}_create_roles_table.sql", ts + 1);

        let junction_migration = r#"-- Orchestra auth migration: user_roles junction table

CREATE TABLE IF NOT EXISTS user_roles (
    user_id   TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id   TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);
"#;
        write_file(
            &PathBuf::from(format!("migrations/{}_create_user_roles.sql", ts + 2)),
            junction_migration,
        )?;
        println!("  ✓ migrations/{}_create_user_roles.sql", ts + 2);
    }

    // ── Model: user.rs ────────────────────────────────────────────────────────
    let user_model = generate_auth_user_model(login_via);
    write_file(&PathBuf::from("src/models/user.rs"), &user_model)?;
    println!("  ✓ src/models/user.rs");

    // ── Model: role.rs (if --with-role) ──────────────────────────────────────
    if with_role {
        let role_model = r#"use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRole {
    pub user_id: String,
    pub role_id: String,
}
"#;
        write_file(&PathBuf::from("src/models/role.rs"), role_model)?;
        println!("  ✓ src/models/role.rs");
    }

    // ── Middleware: auth.rs ──────────────────────────────────────────────────
    let auth_middleware = r#"use actix_web::{HttpRequest, HttpResponse};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_token(user_id: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now,
        exp: now + 86_400 * 7,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub fn extract_user(req: &HttpRequest) -> Result<String, HttpResponse> {
    let secret = std::env::var("JWT_SECRET").map_err(|_| {
        HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "JWT_SECRET not configured"}))
    })?;

    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| {
            HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing Authorization header"}))
        })?;

    let auth_str = auth_header.to_str().map_err(|_| {
        HttpResponse::Unauthorized()
            .json(serde_json::json!({"error": "Invalid Authorization header"}))
    })?;

    if !auth_str.starts_with("Bearer ") {
        return Err(HttpResponse::Unauthorized()
            .json(serde_json::json!({"error": "Expected Bearer token"})));
    }

    let token = &auth_str[7..];
    let claims =
        verify_token(token, &secret).map_err(|_| {
            HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid or expired token"}))
        })?;

    Ok(claims.sub)
}
"#;
    write_file(&PathBuf::from("src/middleware/auth.rs"), auth_middleware)?;
    println!("  ✓ src/middleware/auth.rs");

    // ── Controller: auth.rs ──────────────────────────────────────────────────
    let auth_controller = generate_auth_controller(login_via, with_role, forgot);
    write_file(&PathBuf::from("src/controllers/auth.rs"), &auth_controller)?;
    println!("  ✓ src/controllers/auth.rs");

    // ── Routes: auth_routes.rs ───────────────────────────────────────────────
    let auth_routes = generate_auth_routes(forgot);
    write_file(&PathBuf::from("src/routes/auth_routes.rs"), &auth_routes)?;
    println!("  ✓ src/routes/auth_routes.rs");

    // ── Views (if not --json-only) ──────────────────────────────────────────
    if !json_only {
        let login_view = r#"{% extends "layouts/base.html" %}

{% block title %}Login - Orchestra{% endblock %}

{% block content %}
<div class="max-w-md mx-auto mt-16">
    <h1 class="text-3xl font-bold text-center mb-8">Sign In</h1>
    <form method="POST" action="/api/auth/login" class="space-y-4">
        <input type="email" name="email" placeholder="Email"
               class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500">
        <input type="password" name="password" placeholder="Password"
               class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500">
        <button type="submit"
                class="w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 font-medium">
            Sign In
        </button>
    </form>
    <p class="text-center mt-4 text-sm text-gray-500">
        Don't have an account? <a href="/register" class="text-indigo-600">Register</a>
    </p>
</div>
{% endblock %}
"#;
        write_file(&PathBuf::from("views/auth/login.html"), login_view)?;
        println!("  ✓ views/auth/login.html");

        let register_view = r#"{% extends "layouts/base.html" %}

{% block title %}Register - Orchestra{% endblock %}

{% block content %}
<div class="max-w-md mx-auto mt-16">
    <h1 class="text-3xl font-bold text-center mb-8">Create Account</h1>
    <form method="POST" action="/api/auth/register" class="space-y-4">
        <input type="text" name="name" placeholder="Full Name" required
               class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500">
        <input type="email" name="email" placeholder="Email"
               class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500">
        <input type="password" name="password" placeholder="Password" required
               class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500">
        <button type="submit"
                class="w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 font-medium">
            Register
        </button>
    </form>
    <p class="text-center mt-4 text-sm text-gray-500">
        Already have an account? <a href="/login" class="text-indigo-600">Sign In</a>
    </p>
</div>
{% endblock %}
"#;
        write_file(&PathBuf::from("views/auth/register.html"), register_view)?;
        println!("  ✓ views/auth/register.html");

        if forgot {
            let forgot_view = r#"{% extends "layouts/base.html" %}

{% block title %}Forgot Password - Orchestra{% endblock %}

{% block content %}
<div class="max-w-md mx-auto mt-16">
    <h1 class="text-3xl font-bold text-center mb-8">Reset Password</h1>
    <form method="POST" action="/api/auth/forgot" class="space-y-4">
        <input type="email" name="email" placeholder="Email address"
               class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500">
        <button type="submit"
                class="w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 font-medium">
            Send Reset Link
        </button>
    </form>
</div>
{% endblock %}
"#;
            write_file(&PathBuf::from("views/auth/forgot.html"), forgot_view)?;
            println!("  ✓ views/auth/forgot.html");
        }
    }

    // ── Update Cargo.toml: add bcrypt & jsonwebtoken ────────────────────────
    let cargo_path = Path::new("Cargo.toml");
    if cargo_path.exists() {
        let mut cargo_content = fs::read_to_string(cargo_path)?;
        let mut changed = false;

        for (dep, ver) in &[("bcrypt", "0.16"), ("jsonwebtoken", "9")] {
            if !cargo_content.contains(&format!("{} = ", dep)) {
                if let Some(pos) = cargo_content.find("[dependencies]") {
                    let after_line =
                        cargo_content[pos..].find('\n').map(|p| pos + p + 1).unwrap_or(cargo_content.len());
                    cargo_content.insert_str(after_line, &format!("{} = \"{}\"\n", dep, ver));
                    changed = true;
                }
            }
        }

        if changed {
            fs::write(cargo_path, &cargo_content)?;
            println!("  ✓ Cargo.toml (added bcrypt, jsonwebtoken)");
        }
    }

    // ── Update .env with JWT_SECRET ─────────────────────────────────────────
    let jwt_secret = generate_secret();
    for env_file in &[".env", ".env.example"] {
        let env_path = Path::new(env_file);
        if env_path.exists() {
            let env_content = fs::read_to_string(env_path)?;
            if !env_content.contains("JWT_SECRET") {
                let mut updated = env_content;
                updated.push_str(&format!("\n# JWT\nJWT_SECRET={}\n", jwt_secret));
                fs::write(env_path, updated)?;
                println!("  ✓ {} (added JWT_SECRET)", env_file);
            }
        }
    }

    // ── Update mod.rs files ─────────────────────────────────────────────────
    append_mod_declaration("src/models/mod.rs", "user")?;
    if with_role {
        append_mod_declaration("src/models/mod.rs", "role")?;
    }
    append_mod_declaration("src/controllers/mod.rs", "auth")?;
    append_mod_declaration("src/middleware/mod.rs", "auth")?;
    append_mod_declaration("src/routes/mod.rs", "auth_routes")?;

    // ── Wire auth routes into routes/mod.rs ─────────────────────────────────
    let routes_path = Path::new("src/routes/mod.rs");
    if routes_path.exists() {
        let routes_content = fs::read_to_string(routes_path)?;
        let marker = "pub fn configure(cfg: &mut web::ServiceConfig) {";
        if routes_content.contains(marker)
            && !routes_content.contains("auth_routes::configure")
        {
            let insert = "    // Auth routes\n    auth_routes::configure(cfg);\n\n";
            if let Some(pos) = routes_content.find(marker) {
                let body_start = routes_content[pos..]
                    .find('{')
                    .map(|p| pos + p + 1)
                    .unwrap_or(routes_content.len());
                let updated = format!(
                    "{}{}{}",
                    &routes_content[..body_start],
                    insert,
                    &routes_content[body_start..]
                );
                fs::write(routes_path, updated)?;
                println!("  ✓ src/routes/mod.rs (wired auth routes)");
            }
        }
    }

    // ── Summary ─────────────────────────────────────────────────────────────
    println!();
    println!("🔐 Authentication scaffolding complete!");
    println!();
    println!("  API Endpoints:");
    println!("    POST /api/auth/register  — Create account");
    println!("    POST /api/auth/login     — Sign in");
    println!("    GET  /api/auth/me        — Current user");
    if forgot {
        println!("    POST /api/auth/forgot   — Request password reset");
        println!("    POST /api/auth/reset    — Reset password");
    }
    if with_role {
        println!("    GET  /api/auth/roles    — User roles");
    }
    println!();
    println!("  Next steps:");
    println!("    1. Add migrations to your database:");
    println!("       cargo run  (migrations run automatically)");
    println!("    2. Test auth flow:");
    println!("       curl -X POST http://localhost:8080/api/auth/register \\");
    println!("         -H 'Content-Type: application/json' \\");
    println!("         -d '{{\"name\":\"Test\",\"email\":\"test@test.com\",\"password\":\"secret\"}}'");
    if with_role {
        let rls = if roles_list.is_empty() {
            "admin,user"
        } else {
            &roles_list.join(", ")
        };
        println!("    3. Available roles: {}", rls);
    }

    Ok(())
}

fn parse_roles(roles: Option<&str>) -> Vec<&str> {
    match roles {
        Some(s) if !s.is_empty() => {
            if s.starts_with('[') {
                let inner = s.trim_start_matches('[').trim_end_matches(']');
                inner
                    .split(',')
                    .map(|r| r.trim().trim_matches('"').trim_matches('\''))
                    .filter(|r| !r.is_empty())
                    .collect()
            } else {
                s.split(',').map(|r| r.trim()).filter(|r| !r.is_empty()).collect()
            }
        }
        _ => vec![],
    }
}

fn generate_secret() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("orchestra-auth-{:x}-change-me-in-production", nanos)
}

fn generate_auth_user_model(_login_via: &str) -> String {
    r#"use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password_hash: String,
    pub name: String,
    pub verified_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterDto {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginDto {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}
"#
    .to_string()
}

fn generate_auth_controller(login_via: &str, with_role: bool, forgot: bool) -> String {
    let register_check = match login_via {
        "phone" => r#"        // Check if phone already registered
        let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE phone = ?")
            .bind(&body.phone)
            .fetch_one(pool.get_ref())
            .await
            .unwrap_or(0);
        if existing > 0 {
            return HttpResponse::Conflict()
                .json(serde_json::json!({"error": "Phone already registered"}));
        }"#,
        "both" => r#"        // Check if email or phone already registered
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE email = ? OR phone = ?",
        )
        .bind(&body.email)
        .bind(&body.phone)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
        if existing > 0 {
            return HttpResponse::Conflict()
                .json(serde_json::json!({"error": "Email or phone already registered"}));
        }"#,
        _ => r#"        // Check if email already registered
        let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(&body.email)
            .fetch_one(pool.get_ref())
            .await
            .unwrap_or(0);
        if existing > 0 {
            return HttpResponse::Conflict()
                .json(serde_json::json!({"error": "Email already registered"}));
        }"#,
    };

    let register_impl = match login_via {
        "phone" => r#"        let user_id = uuid::Uuid::new_v4().to_string();
        let password_hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).unwrap();

        sqlx::query("INSERT INTO users (id, phone, password_hash, name) VALUES (?1, ?2, ?3, ?4)")
            .bind(&user_id)
            .bind(&body.phone)
            .bind(&password_hash)
            .bind(&body.name)
            .execute(pool.get_ref())
            .await
            .unwrap();

        let secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let token = crate::middleware::auth::create_token(&user_id, &secret).unwrap();

        HttpResponse::Created().json(serde_json::json!({
            "token": token,
            "user": {
                "id": user_id,
                "name": body.name,
                "email": null,
                "phone": body.phone,
            }
        }))"#,
        "both" => r#"        let user_id = uuid::Uuid::new_v4().to_string();
        let password_hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).unwrap();

        sqlx::query("INSERT INTO users (id, email, phone, password_hash, name) VALUES (?1, ?2, ?3, ?4, ?5)")
            .bind(&user_id)
            .bind(&body.email)
            .bind(&body.phone)
            .bind(&password_hash)
            .bind(&body.name)
            .execute(pool.get_ref())
            .await
            .unwrap();

        let secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let token = crate::middleware::auth::create_token(&user_id, &secret).unwrap();

        HttpResponse::Created().json(serde_json::json!({
            "token": token,
            "user": {
                "id": user_id,
                "name": body.name,
                "email": body.email,
                "phone": body.phone,
            }
        }))"#,
        _ => r#"        let user_id = uuid::Uuid::new_v4().to_string();
        let password_hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).unwrap();

        sqlx::query("INSERT INTO users (id, email, phone, password_hash, name) VALUES (?1, ?2, ?3, ?4, ?5)")
            .bind(&user_id)
            .bind(&body.email)
            .bind(&body.phone)
            .bind(&password_hash)
            .bind(&body.name)
            .execute(pool.get_ref())
            .await
            .unwrap();

        let secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let token = crate::middleware::auth::create_token(&user_id, &secret).unwrap();

        HttpResponse::Created().json(serde_json::json!({
            "token": token,
            "user": {
                "id": user_id,
                "name": body.name,
                "email": body.email,
                "phone": body.phone,
            }
        }))"#,
    };

    let login_impl = match login_via {
        "phone" => r#"        let user = sqlx::query_as::<_, crate::models::user::User>(
            "SELECT * FROM users WHERE phone = ?"
        )
        .bind(&body.phone)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap();

        let user = match user {
            Some(u) => u,
            None => return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"})),
        };

        let valid = bcrypt::verify(&body.password, &user.password_hash).unwrap_or(false);
        if !valid {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"}));
        }

        let secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let token = crate::middleware::auth::create_token(&user.id, &secret).unwrap();

        HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "user": {
                "id": user.id,
                "name": user.name,
                "email": user.email,
                "phone": user.phone,
            }
        }))"#,
        "both" => r#"        let user = sqlx::query_as::<_, crate::models::user::User>(
            "SELECT * FROM users WHERE email = ? OR phone = ?"
        )
        .bind(&body.email)
        .bind(&body.phone)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap();

        let user = match user {
            Some(u) => u,
            None => return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"})),
        };

        let valid = bcrypt::verify(&body.password, &user.password_hash).unwrap_or(false);
        if !valid {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"}));
        }

        let secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let token = crate::middleware::auth::create_token(&user.id, &secret).unwrap();

        HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "user": {
                "id": user.id,
                "name": user.name,
                "email": user.email,
                "phone": user.phone,
            }
        }))"#,
        _ => r#"        let user = sqlx::query_as::<_, crate::models::user::User>(
            "SELECT * FROM users WHERE email = ?"
        )
        .bind(&body.email)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap();

        let user = match user {
            Some(u) => u,
            None => return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"})),
        };

        let valid = bcrypt::verify(&body.password, &user.password_hash).unwrap_or(false);
        if !valid {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"}));
        }

        let secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let token = crate::middleware::auth::create_token(&user.id, &secret).unwrap();

        HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "user": {
                "id": user.id,
                "name": user.name,
                "email": user.email,
                "phone": user.phone,
            }
        }))"#,
    };

    let forgot_impl = if forgot {
        r#"
pub async fn forgot(
    pool: web::Data<SqlitePool>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let email = body.get("email").and_then(|v| v.as_str());
    let phone = body.get("phone").and_then(|v| v.as_str());

    // In production, send an email/SMS with a reset link.
    // For now, we return a reset token directly.
    if let Some(email_val) = email {
        let user = sqlx::query_as::<_, crate::models::user::User>(
            "SELECT * FROM users WHERE email = ?",
        )
        .bind(email_val)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap();

        if let Some(u) = user {
            let secret = std::env::var("JWT_SECRET").unwrap_or_default();
            let reset_token = crate::middleware::auth::create_token(&u.id, &secret).unwrap();
            tracing::info!("Password reset requested for email: {}", email_val);
            return HttpResponse::Ok().json(serde_json::json!({
                "message": "Reset link sent",
                "reset_token": reset_token,
            }));
        }
    }

    if let Some(phone_val) = phone {
        let user = sqlx::query_as::<_, crate::models::user::User>(
            "SELECT * FROM users WHERE phone = ?",
        )
        .bind(phone_val)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap();

        if let Some(u) = user {
            let secret = std::env::var("JWT_SECRET").unwrap_or_default();
            let reset_token = crate::middleware::auth::create_token(&u.id, &secret).unwrap();
            tracing::info!("Password reset requested for phone: {}", phone_val);
            return HttpResponse::Ok().json(serde_json::json!({
                "message": "Reset link sent",
                "reset_token": reset_token,
            }));
        }
    }

    // Always return OK to avoid leaking user existence
    HttpResponse::Ok().json(serde_json::json!({
        "message": "If the account exists, a reset link has been sent."
    }))
}

pub async fn reset(
    pool: web::Data<SqlitePool>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let token = body.get("token").and_then(|v| v.as_str());
    let new_password = body.get("password").and_then(|v| v.as_str());

    let (token, password) = match (token, new_password) {
        (Some(t), Some(p)) if !t.is_empty() && p.len() >= 6 => (t, p),
        _ => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid token or password too short (min 6 chars)"
        })),
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_default();
    match crate::middleware::auth::verify_token(token, &secret) {
        Ok(claims) => {
            let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
            sqlx::query("UPDATE users SET password_hash = ?1, updated_at = datetime('now') WHERE id = ?2")
                .bind(&password_hash)
                .bind(&claims.sub)
                .execute(pool.get_ref())
                .await
                .unwrap();
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Password updated successfully"
            }))
        }
        Err(_) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid or expired reset token"
        })),
    }
}
"#
    } else {
        ""
    };

    let roles_endpoint = if with_role {
        let roles_sql = "SELECT r.name FROM roles r \
            INNER JOIN user_roles ur ON ur.role_id = r.id \
            WHERE ur.user_id = ?1";
        format!(
            r#"
pub async fn roles(
    req: HttpRequest,
    pool: web::Data<SqlitePool>,
) -> HttpResponse {{
    let user_id = match crate::middleware::auth::extract_user(&req) {{
        Ok(id) => id,
        Err(resp) => return resp,
    }};

    let user_roles = sqlx::query_as::<_, (String,)>(
        "{roles_sql}",
    )
    .bind(&user_id)
    .fetch_all(pool.get_ref())
    .await
    .unwrap();

    let role_names: Vec<String> = user_roles.into_iter().map(|(name,)| name).collect();

    HttpResponse::Ok().json(serde_json::json!({{
        "user_id": user_id,
        "roles": role_names,
    }}))
}}
"#
        )
    } else {
        String::new()
    };

    format!(
        r#"use actix_web::{{web, HttpRequest, HttpResponse}};
use sqlx::SqlitePool;

pub async fn register(
    pool: web::Data<SqlitePool>,
    body: web::Json<crate::models::user::RegisterDto>,
) -> HttpResponse {{
    {register_check}

    {register_impl}
}}

pub async fn login(
    pool: web::Data<SqlitePool>,
    body: web::Json<crate::models::user::LoginDto>,
) -> HttpResponse {{
    {login_impl}
}}

pub async fn me(
    req: HttpRequest,
    pool: web::Data<SqlitePool>,
) -> HttpResponse {{
    let user_id = match crate::middleware::auth::extract_user(&req) {{
        Ok(id) => id,
        Err(resp) => return resp,
    }};

    let user = sqlx::query_as::<_, crate::models::user::User>(
        "SELECT * FROM users WHERE id = ?",
    )
    .bind(&user_id)
    .fetch_optional(pool.get_ref())
    .await
    .unwrap();

    match user {{
        Some(u) => HttpResponse::Ok().json(serde_json::json!({{
            "id": u.id,
            "name": u.name,
            "email": u.email,
            "phone": u.phone,
        }})),
        None => HttpResponse::NotFound().json(serde_json::json!({{
            "error": "User not found"
        }})),
    }}
}}
{forgot_impl}
{roles_endpoint}
"#
    )
}

fn generate_auth_routes(forgot: bool) -> String {
    let forgot_routes = if forgot {
        r#"            .route("/forgot", web::post().to(crate::controllers::auth::forgot))
            .route("/reset", web::post().to(crate::controllers::auth::reset))"#
    } else {
        ""
    };
    format!(
        r#"use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {{
    cfg.service(
        web::scope("/api/auth")
            .route("/register", web::post().to(crate::controllers::auth::register))
            .route("/login", web::post().to(crate::controllers::auth::login))
            .route("/me", web::get().to(crate::controllers::auth::me))
            {forgot_routes}
            ,
    );
}}
"#
    )
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
    let table = format!("{snake}s");
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

use crate::controllers::{example, home};

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Root route — welcome page
    cfg.route("/", web::get().to(home::welcome));

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
