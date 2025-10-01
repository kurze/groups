<!--
SYNC IMPACT REPORT
==================
Version Change: [NONE] → 1.0.0 (initial constitution)
Modified Principles: N/A (initial creation)
Added Sections:
  - Core Principles (4 principles focused on code quality, testing, UX, performance)
  - Quality Standards
  - Development Workflow
  - Governance
Removed Sections: N/A
Templates Status:
  ✅ plan-template.md - reviewed, Constitution Check section compatible
  ✅ spec-template.md - reviewed, requirements align with principles
  ✅ tasks-template.md - reviewed, test-first workflow compatible but not strictly TDD
  ✅ agent-file-template.md - reviewed, no changes needed
Follow-up TODOs: None
Ratification Date: 2025-10-01 (initial adoption)
-->

# Groups Project Constitution

## Core Principles

### I. Code Quality & Maintainability (NON-NEGOTIABLE)

All code MUST adhere to these non-negotiable quality standards:

- **KISS (Keep It Simple, Stupid)**: Prefer simple, obvious solutions over clever ones. Code complexity requires explicit justification in reviews.
- **YAGNI (You Aren't Gonna Need It)**: Implement only current requirements. Speculative features are prohibited unless justified by immediate roadmap needs.
- **DRY (Don't Repeat Yourself) - Pragmatically**: Extract duplication only when patterns stabilize (Rule of Three). Premature abstraction is worse than localized repetition.
- **Single Responsibility**: Each module, struct, or function serves one clear purpose. Mixed concerns require refactoring before merge.
- **Explicit Over Implicit**: Use explicit function returns rather than side effects. State changes must be traceable through return values and parameters.
- **Module Boundaries**: Maintain clear separation between API layer, business logic (services), and data access. Cross-layer violations are blocked.
- **Minimal Dependencies**: New external dependencies require justification of necessity and evaluation of maintenance burden.
- **Document "Why" Not "What"**: Code should be self-explanatory. Comments explain rationale, trade-offs, and non-obvious decisions—never paraphrase the code itself.

**Rationale**: This project is a learning exercise for Rust. Maintainability and clarity trump sophistication. Future contributors (including "future you") must understand code intent without archaeology.

### II. Testability & Test Coverage

Code MUST be testable and adequately covered by tests:

- **Testable Design**: Write code that can be tested. Avoid global state, hidden dependencies, and untestable side effects. If code is hard to test, it's poorly designed.
- **Test Coverage Requirements**:
  - **Contract Tests**: Every REST API endpoint SHOULD have contract tests validating request/response schemas.
  - **Integration Tests**: User flows spanning multiple layers (API → Service → Database) MUST have integration tests for critical paths.
  - **Unit Tests**: Complex business logic with branching, validation, or computation SHOULD have unit tests.
  - **End-to-End Tests**: Critical user journeys MUST have Playwright E2E tests (login, group creation, etc.).
- **Test Isolation**: Tests MUST NOT depend on execution order. Use test database (`DATABASE_TEST_URL` on port 5433) to avoid state pollution.
- **Test Execution**: `task pre-commit` (format + test) MUST pass before committing. CI failures block merges.
- **Test Doubles**: Prefer real dependencies (database, HTTP clients) in tests to catch integration issues. Mocks only for external systems (email, payment gateways).

**Note on TDD**: While tests-first development is encouraged for complex features, strict TDD (red-green-refactor) is not mandatory. Write tests when they add value, in whatever order makes sense.

**Rationale**: Tests ensure correctness, enable refactoring, and serve as documentation. Testable code tends to be better-designed code. However, dogmatic TDD can slow down exploration and learning—flexibility is valued.

### III. User Experience Consistency

User-facing interfaces MUST deliver predictable, coherent experiences:

- **Template Consistency**: Tera templates MUST use shared layout base (`templates/base.html`) for navigation, styling, and structure.
- **Error Handling**: User-facing errors MUST be actionable (e.g., "Email already registered. Try logging in?" not "Database constraint violation").
- **Form Validation**: Client-side validation provides immediate feedback. Server-side validation is authoritative and MUST provide field-specific error messages.
- **Session Management**: Authentication state MUST persist correctly. RequireAuth middleware redirects to `/login` with return URL preservation.
- **Responsive Design**: HTML/CSS MUST function on mobile and desktop. Test at 320px (mobile) and 1920px (desktop) viewports.
- **Loading States**: htmz interactions MUST provide visual feedback (loading spinners, disabled buttons) to prevent double-submission.
- **Accessibility Basics**: Forms MUST have labels, buttons MUST have descriptive text, and keyboard navigation MUST work.

**Rationale**: This project mimics Meetup.com. Inconsistent UX erodes trust and usability. Small details (error messages, loading states) separate toy projects from production-ready systems.

### IV. Performance Through Simplicity (NON-NEGOTIABLE)

Performance MUST be achieved through architectural simplicity, not complexity:

- **No Premature Optimization**: DO NOT add caching layers, CDNs, Redis, or similar infrastructure unless measurements prove necessity. Simple, direct code is faster to write, debug, and maintain.
- **Simplicity First**: The fastest code is code that doesn't exist. Favor minimal logic, straightforward algorithms, and direct database queries over elaborate optimization schemes.
- **Baseline Targets**:
  - **API Response Times**: REST endpoints respond in <200ms p95 for simple queries, <500ms for complex queries—achieved through efficient queries, not caching.
  - **Database Efficiency**: Use SQLx compile-time query verification. Avoid N+1 queries via joins or batching, not query caches.
  - **Pagination**: Listing endpoints paginate (default 20 items, max 100) to keep queries and payloads small.
  - **Startup Time**: Application starts in <5 seconds locally (excluding migrations).
  - **Memory Footprint**: Development server runs comfortably in 512MB RAM.
- **Static Assets**: Serve CSS/JS from `/static` with appropriate cache headers. Keep assets minimal—no heavy frameworks or bundlers unless justified.
- **Perceived Performance**: Fast, lightweight pages feel snappier than heavily optimized bloat. Prioritize small payloads, minimal dependencies, and fast initial renders.

**Rationale**: Snappiness comes from keeping the system light and simple. Adding caching, workers, queues, and other layers creates complexity that slows development and introduces failure modes. For this scale (learning project, modest traffic), simple is fast enough—and stays fast.

## Quality Standards

### Code Review Requirements

All pull requests MUST satisfy:

1. **Automated Checks**: CI passes (`task pre-commit`, `task e2e`, `task test-docker`).
2. **Constitution Compliance**: No violations of Core Principles unless explicitly justified in PR description with "Complexity Tracking" section.
3. **Test Evidence**: New features include appropriate tests. Bug fixes include regression tests where applicable.
4. **Documentation Updates**: API changes update `CLAUDE.md`. New commands update `Taskfile.yml`.
5. **No Debug Artifacts**: Remove `println!`, `dbg!`, commented code, and TODO comments without issue tracking.

### Security Standards

- **Password Handling**: MUST use Argon2id (`password::hash_password`, `password::verify_password`). Never log or display passwords.
- **Session Security**: `SESSION_SECRET_KEY` MUST be 64+ characters in production. Rotate after suspected compromise.
- **SQL Injection**: MUST use SQLx parameterized queries. String interpolation in SQL is prohibited.
- **Input Validation**: Sanitize and validate all user input. Reject malformed data at API boundary.
- **Secrets Management**: NEVER commit `.env` or credentials. Use `.env.example` for templates.

### Documentation Standards

- **README.md**: Maintained as onboarding guide (setup, architecture, commands).
- **CLAUDE.md**: Single source of truth for AI-assisted development context. Updated incrementally (O(1) operation, <150 lines).
- **Inline Comments**: Explain "why" for non-obvious decisions (e.g., "Use blocking I/O here to prevent race condition in migration").
- **API Documentation**: REST endpoints documented in `CLAUDE.md` under API Endpoints section.

## Development Workflow

### Task Automation

The project uses Task (taskfile.dev) for all common operations:

- **Development**: `task dev` starts database + server. `task dev-clean` resets state.
- **Testing**: `task test` runs unit/integration tests. `task e2e` runs Playwright. `task pre-commit` runs full quality gate.
- **Database**: `task db-up/down/reset/shell` manages PostgreSQL lifecycle. Separate test DB on port 5433.
- **Code Quality**: `task fmt` formats Rust code. `cargo fmt` and `cargo clippy` MUST pass.

**New commands MUST be added to `Taskfile.yml` with clear descriptions.**

### Git Workflow

- **Branch Naming**: Feature branches: `feature/short-description`. Bug fixes: `fix/issue-number-description`.
- **Commit Messages**: Conventional Commits format: `type(scope): description` (e.g., `feat(auth): add password reset flow`).
- **Pull Requests**: Descriptive titles, link to issues, include test evidence. Squash before merge to keep history clean.
- **Protected Main**: Direct pushes to `main` prohibited. Require PR approval + passing CI.

### Database Migrations

- **Migration Files**: SQL migrations in `migrations/` directory, managed by SQLx.
- **Forward-Only**: Migrations MUST be idempotent and forward-only. Rollbacks via new migration, not deletion.
- **Testing**: Test migrations on clean database (`task db-test-up`, run migrations, verify schema, `task db-test-down`).

## Governance

### Amendment Process

1. **Proposal**: Open GitHub issue with "Constitution Amendment" label. Describe principle change, rationale, and impact.
2. **Discussion**: Allow 7 days for team feedback. Address concerns.
3. **Implementation**: Update constitution, increment version (see Versioning Policy), propagate to templates.
4. **Migration Plan**: If breaking change, provide migration guide for in-flight work.
5. **Approval**: Requires unanimous approval for MAJOR version bumps (removing/redefining principles), majority for MINOR/PATCH.

### Versioning Policy

Constitution follows semantic versioning:

- **MAJOR (X.0.0)**: Backward-incompatible changes (removing principles, redefining core requirements). Requires migration plan.
- **MINOR (x.Y.0)**: New principles added, material expansions to existing guidance. Review existing work for compliance.
- **PATCH (x.y.Z)**: Clarifications, wording improvements, typo fixes. No behavioral changes expected.

### Compliance Review

- **Pre-Merge**: All PRs reviewed against constitution. Violations documented in "Complexity Tracking" section if justified, otherwise rejected.
- **Quarterly Audit**: Every 3 months, review constitution for outdated guidance. Prune or update stale principles.
- **Deviation Tracking**: Justified deviations logged in PR descriptions. Repeated patterns trigger principle re-evaluation.

### Relationship to Development Guidance

- **Authority**: Constitution supersedes conflicting practices in `CLAUDE.md`, `README.md`, or inline comments.
- **Runtime Guidance**: Use `CLAUDE.md` for tactical development guidance (API endpoints, file locations, dependency versions). Use constitution for strategic principles.
- **Template Alignment**: Plan, spec, and task templates MUST reference constitution version and include "Constitution Check" gates.

**Version**: 1.0.0 | **Ratified**: 2025-10-01 | **Last Amended**: 2025-10-01
