## 🎼 Orchestra: The Fullstack Rust Framework for Joyful Productivity

**Orchestra** is an opinionated, modular, RAM-efficient framework for building fullstack Rust applications—designed for humans, startups, and agentic AI tools.

### ✨ Philosophy

* **KISS by default, powerful when needed**
* **Convention over configuration**
* **Modular folder structure**
* **Agentic CLI orchestration**
* **RAM-aware, emotionally ergonomic**

### 📦 Default Stack

| Layer       | Tool                                                | Notes               |
| ----------- | --------------------------------------------------- | ------------------- |
| Database    | `sqlite3` (default), switchable to Postgres/MySQL |                     |
| ORM         | `sqlx` (default), optional `diesel`             |                     |
| Templates   | `askama`                                          | Compile-time safe   |
| CSS         | `tailwindcss`                                     | Admin panel & views |
| Auth        | Built-in starter kit                                |                     |
| Admin Panel | Auto-generated CRUD                                 |                     |
| Jobs        | Lazy-static + async                                 |                     |
| Migrations  | Built-in                                            |                     |
| CLI         | `orchestra` binary                                |                     |
| Structure   | MVC-inspired                                        |                     |

### 🧱 Folder Structure

Code

```
src/
├── commands/        # CLI extensions
├── controllers/     # Route handlers
├── models/          # DB structs
├── views/           # Askama templates
├── middleware/      # Auth, logging, etc.
├── jobs/            # Background tasks
├── services/        # Business logic
└── main.rs
migrations/
public/
config/
orchestrator/
```

### ⚙️ CLI Commands

bash

```
orchestra make:model User
orchestra make:model Teacher -a         # -a = all: model, migration, controller, view, admin
orchestra make:view Dashboard
orchestra make:controller Auth
orchestra make:admin Product
orchestra make:job SendEmail
orchestra deploy --env=prod --ssh=user@server.com
orchestra serve
```

### 🤖 Agentic AI Ready

* Templates for code generation
* CLI orchestration for AI workflows
* Low RAM footprint for edge devices
* Easy onboarding for contributors

### 🚀 Deployment

* Local dev server with hot reload
* SSH-based deploy with CI/CD hooks
* Systemd or Docker support
* GitHub Actions integration

### 💡 Next Steps

* [x] Draft README.md
* [x] Build CLI parser for `orchestra`
* [x] Scaffold starter repo with default structure
* [x] Create templates for model/view/controller generation
* [ ] Design admin panel layout with Tailwind
* [ ] Write migration engine (Rust or SQL-based)
