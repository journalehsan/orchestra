# Orchestra Framework - Agent Guide

This guide helps AI agents work effectively with the Orchestra framework codebase.

## Project Overview

Orchestra is an opinionated, modular, RAM-efficient framework for building fullstack Rust applications. It's designed for humans, startups, and agentic AI tools with a focus on KISS principles, convention over configuration, and agentic CLI orchestration.

## Current Project Status

This is a **conceptual framework in planning phase**. The repository currently contains only documentation (README.md and ROADMAP.md) and no actual implementation yet.

## Planned Technology Stack

| Layer       | Tool                                  | Notes                          |
| ----------- | ------------------------------------- | ------------------------------ |
| Web Framework | Actix-web (primary), Axum (alt)      | High-performance async web     |
| ORM         | SQLx (default), Diesel (optional)     | Compile-time checked SQL       |
| Templates   | Askama                                | Compile-time safe templates    |
| CSS         | TailwindCSS                           | Admin panel & views            |
| Database    | SQLite3 (default), Postgres/MySQL     | Switchable database backends   |
| Auth        | Built-in starter kit                  | JWT + Session support          |
| Admin Panel | Auto-generated CRUD                   | Tailwind-based UI              |
| CLI         | `orchestra` binary                    | Code generation & management    |

## Planned Directory Structure

```
src/
├── commands/        # CLI extensions
├── controllers/     # Route handlers
├── models/          # DB structs
├── views/           # Askama templates
├── middleware/      # Auth, logging, etc.
├── jobs/            # Background tasks
├── services/        # Business logic
├── repositories/    # Data access layer (SQLx/Diesel)
├── routes/          # Route definitions
├── config/          # Configuration
└── main.rs
migrations/
public/
config/
orchestrator/
```

## Planned CLI Commands

```bash
orchestra make:model User
orchestra make:model Teacher -a         # -a = all: model, migration, controller, view, admin
orchestra make:view Dashboard
orchestra make:controller Auth
orchestra make:admin Product
orchestra make:job SendEmail
orchestra deploy --env=prod --ssh=user@server.com
orchestra serve
```

## Development Philosophy

- **KISS by default, powerful when needed**
- **Convention over configuration**
- **Modular folder structure**
- **Agentic CLI orchestration**
- **RAM-aware, emotionally ergonomic**
- **AI as assistant, not replacement**
- **Performance first** - maintain Rust's speed advantages
- **Zero-cost abstractions** - no runtime overhead

## Planned Architecture Patterns

- **Model-Repository-Controller pattern** (similar to MVC)
- **Repository Pattern** for data access abstraction
- **Service Layer** for business logic
- **Request/Response DTOs** for validation + serialization
- **Middleware Stack** for auth, logging, CORS
- **Unified Error Handling** with custom error types

## When Working on This Project

1. **Implementation Phase**: This project is in planning phase. When implementing, follow the roadmap in ROADMAP.md which outlines a 32-36 week development plan.

2. **Start with Foundation**: Begin with Phase 1 - Core CLI & Project Scaffolding, focusing on the `orchestra` CLI tool and basic Actix-web setup.

3. **Follow Rust Best Practices**: Use idiomatic Rust, maintain zero-cost abstractions, and ensure compile-time safety where possible.

4. **AI Integration Focus**: This framework is designed to be AI-friendly. Consider how code generation and AI assistants will interact with each component.

5. **Performance Priority**: Always optimize for RAM efficiency and maintain performance advantages of Rust.

## Key Implementation Priorities

Based on ROADMAP.md, focus on these high-priority items:
- Project scaffolding with Actix-web
- SQLx + optional Diesel integration
- Model-Repository-Controller generator
- Basic AI code generation
- Route conflict detection

## Testing Strategy

When implemented, the framework should include:
- Auto-generated tests (`rf-cli test:generate`)
- Test runner with coverage (`rf-cli test:run`)
- Built-in benchmarking
- Performance profiling capabilities

## Documentation Standards

- Maintain comprehensive Markdown documentation
- Include code examples for all features
- Document CLI commands with usage examples
- Keep README.md updated with current implementation status

## Contributing Guidelines

Since this is a framework for developers:
- Prioritize developer experience
- Maintain clear error messages
- Ensure AI agents can effectively use the CLI
- Test all code generation templates
- Validate all generated code compiles on first try