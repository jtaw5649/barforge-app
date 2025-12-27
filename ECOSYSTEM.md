# Barforge Ecosystem

## Repos and Roles

- **barforge-registry-api**: Cloudflare Workers REST API for modules, users, reviews, collections, versions, screenshots, and package delivery via R2.
- **barforge-web**: SvelteKit web hub (Cloudflare Pages). Uses the registry API for browsing, auth, reviews, stars, collections, screenshots, and user profiles. API base comes from `PUBLIC_API_BASE_URL` with a default of `https://api.barforge.dev`.
- **barforge-app**: Iced desktop app for installing and managing modules. CURRENTLY OUTDATED, but actively consumes the registry API and will be updated to match web + API soon.

## Shared Contracts

- **`barforge-registry-api/docs/openapi.yaml`** is the source of truth for HTTP contracts.
- **barforge-app** generates a Rust client in `barforge-app/crates/barforge-registry-client` via `barforge-app/scripts/generate-registry-client.sh`, then maps API models into app domain types in `barforge-app/src/api.rs`.
- **barforge-web** generates `barforge-web/src/lib/api-types.ts` via `barforge-web/scripts/generate-api-types.mjs` and re-exports aliases in `barforge-web/src/lib/types.ts`.
- **Module timestamps**: `RegistryModule.last_updated` is canonical for recency/trending/new badges; it is set on module creation and updated on releases, so clients should not rely on `created_at`.
- **Stars**: `StarredModule` extends `RegistryModule` with `starred_at`.
- **RegistryModule requirements**: `repo_url` and `tags` are required in API responses and tests/mocks; `repo_url` must use `https://` (HTTP is rejected by server validation).
- **Module creation**: `CreateModuleRequest.license` is required and must be a valid SPDX license identifier.
- **Error responses**: API returns RFC 9457 Problem Details (`application/problem+json`) with `type=about:blank` for standard errors and `code`/`error_id` extensions.

## Integration Points

- Base API: `https://api.barforge.dev`
- Auth sync: `POST /api/v1/auth/sync`
- Index: `/api/v1/index`
- Modules: `/api/v1/modules`, `/api/v1/modules/:uuid`, `/api/v1/modules/search?q=&category=`
- Categories: `/api/v1/categories`
- Users: `/api/v1/users/:username`, `/api/v1/users/:username/modules`, `/api/v1/users/me` (GET/PATCH/DELETE)
- Reviews: `/api/v1/modules/:uuid/reviews`
- Versions: `/api/v1/modules/:uuid/versions`
- Screenshots: `/api/v1/modules/:uuid/screenshots`
- Collections: `/api/v1/collections`, `/api/v1/collections/:id`, `/api/v1/users/:username/collections`
- Stars: `/api/v1/users/me/stars`, `/api/v1/modules/:uuid/star`
- Notifications: `/api/v1/notifications`, `/api/v1/notifications/stream`, `/api/v1/notifications/announcements`, `/api/v1/notifications/:id/read`, `/api/v1/notifications/mark-all-read`, `/api/v1/notifications/preferences`
- Downloads: `POST /api/v1/modules/:uuid/download` (legacy when `DOWNLOADS_MODE=logpush`; counts come from Workers Trace Events Logpush package logs)
- Package delivery: `/packages/:uuid/:version/:filename` (R2)
- Security: `/security/check?uuid=&version=`

## Data Flow

- Web/app -> Registry API -> D1/R2/KV.
- Workers Trace Events Logpush -> R2 (LOGPUSH_BUCKET) -> Cron ingest -> D1 download rollups -> module/version counts.
- Web handles GitHub auth (Auth.js) with `user:email` scope and calls the API for profile, collections, reviews, stars, and notifications.
- App caches the registry index locally and downloads packages from the R2-backed endpoints.

## Repo Topology Recommendation

- Keep multi-repo for now to preserve independent release cadence.
- Add a contract-first API spec in `barforge-registry-api` (OpenAPI or JSON Schema).
- Generate TS types for `barforge-web` and a Rust client for `barforge-app` from the OpenAPI spec.
- Add a lightweight compatibility check in CI to detect schema drift.
- Revisit monorepo only if coordinated changes across all three repos become the norm.

## Source Links

- `barforge-registry-api/README.md`
- `barforge-registry-api/src/lib.rs`
- `barforge-registry-api/src/routes.rs`
- `barforge-registry-api/src/db.rs`
- `barforge-registry-api/wrangler.toml`
- `barforge-registry-api/docs/openapi.yaml`
- `barforge-web/src/lib/index.ts`
- `barforge-web/src/lib/types.ts`
- `barforge-web/src/lib/api-types.ts`
- `barforge-app/src/services/paths.rs`
- `barforge-app/src/api.rs`
- `barforge-app/src/tasks/registry.rs`
- `barforge-app/crates/barforge-registry-client`
- `barforge-app/scripts/generate-registry-client.sh`
- `barforge-registry-api/.github/workflows/ci.yml`
- `barforge-web/scripts/generate-api-types.mjs`
- `barforge-web/src/lib/hooks/useModuleCard.svelte.ts`
- `barforge-web/src/lib/utils/popularity.ts`
- `barforge-web/src/lib/stores/stars.svelte.ts`
- `https://developers.cloudflare.com/turnstile/get-started/server-side-validation/`
- `https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events`
- `https://html.spec.whatwg.org/multipage/server-sent-events.html`
- `https://doc.rust-lang.org/reference/comments.html`
- `https://github.com/sveltejs/kit/blob/main/documentation/docs/20-core-concepts/30-form-actions.md`
- `https://github.com/sveltejs/svelte/blob/main/documentation/docs/02-runes/03-$derived.md`
- `https://github.com/sveltejs/svelte/blob/main/documentation/docs/02-runes/04-$effect.md`
- `https://github.com/sveltejs/svelte/blob/main/documentation/docs/07-misc/07-v5-migration-guide.md`
- `https://tsdoc.org/`
- `https://svelte.dev/docs/kit/form-actions`
- `https://svelte.dev/docs/svelte/$derived`
- `https://hpbn.co/server-sent-events-sse/`
- `https://eslint.org/docs/latest/rules/no-warning-comments`
- `https://google.github.io/styleguide/pyguide.html`
- `https://stackoverflow.blog/2021/12/23/best-practices-for-writing-code-comments/`
- `https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Strict_Transport_Security_Cheat_Sheet.html`
- `https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html`
- `https://developer.mozilla.org/en-US/docs/Web/API/HTMLImageElement/loading`
- `https://ogp.me/`
- `https://svelte.dev/docs/kit/configuration`
- `https://authjs.dev/guides/refresh-token-rotation`
- `https://authjs.dev/reference/core`
- `https://authjs.dev/reference/sveltekit/types`
- `https://authjs.dev/guides/extending-the-session`
- `https://authjs.dev/reference/core#callbacks`
- `https://developers.cloudflare.com/d1/platform/limits/`
- `https://developers.cloudflare.com/d1/best-practices/use-indexes/`
- `https://developers.cloudflare.com/workers/runtime-apis/cache/`
- `https://developers.cloudflare.com/workers/runtime-apis/streams/`
- `https://developers.cloudflare.com/r2/api/workers/workers-api-reference/`
- `https://developers.cloudflare.com/workers/platform/limits/`
- `https://doc.rust-lang.org/cargo/reference/registries.html`
- `https://raw.githubusercontent.com/rust-lang/crates.io/master/README.md`
- `https://www.verdaccio.org/docs/caching/`
- `https://developers.cloudflare.com/waf/rate-limiting-rules/best-practices/`
- `https://blog.rust-lang.org/2024/03/11/crates-io-download-changes/`
- `https://github.com/rust-lang/crates.io/issues/1442`
- `https://blog.npmjs.org/post/164799520460/api-rate-limiting-rolling-out.html`
- `https://blog.npmjs.org/post/187698412060/acceptible-use.html`
- `https://github.com/npm/registry/blob/main/docs/download-counts.md`
- `https://developers.cloudflare.com/logs/logpush/logpush-job/enable-destinations/r2/`
- `https://developers.cloudflare.com/logs/logpush/`
- `https://developers.cloudflare.com/logs/logpull/enabling-log-retention/`
- `https://developers.cloudflare.com/workers/observability/logs/logpush/`
- `https://developers.cloudflare.com/workers/platform/pricing/`
- `https://developers.cloudflare.com/cloudflare-one/insights/logs/logpush/`
- `https://developers.cloudflare.com/logs/logpush/logpush-job/datasets/account/workers_trace_events/`
- `https://blog.cloudflare.com/workers-logpush-ga/`
- `https://www.rfc-editor.org/rfc/rfc9457.html`
- `https://developers.cloudflare.com/workers/runtime-apis/request/`
- `https://developers.cloudflare.com/durable-objects/examples/build-a-rate-limiter/`
- `https://learn.openapis.org/best-practices.html`
- `https://developers.cloudflare.com/logs/logpush/logpush-job/datasets/`
- `https://spdx.org/licenses/`
- `https://spdx.dev/learn/overview/`
- `https://github.com/spdx/license-list-data`
- `https://fossa.com/blog/understanding-using-spdx-license-identifiers-license-expressions/`
- `https://docs.fedoraproject.org/en-US/legal/spdx/`
- `https://docs.rs/spdx/latest/spdx/`
- `https://raw.githubusercontent.com/EmbarkStudios/spdx/master/src/lib.rs`
- `https://raw.githubusercontent.com/EmbarkStudios/spdx/master/src/expression.rs`
- `https://raw.githubusercontent.com/EmbarkStudios/spdx/master/src/licensee.rs`
- `https://developers.cloudflare.com/logs/logpush/logpush-job/datasets/zone/http_requests/`
- `https://developers.cloudflare.com/analytics/analytics-engine/`
- `https://developers.cloudflare.com/analytics/analytics-engine/get-started/`
- `https://developers.cloudflare.com/workers/configuration/cron-triggers/`
- `https://developers.cloudflare.com/r2/platform/audit-logs/`
- `https://docs.github.com/en/rest/users/emails`
- `https://github.com/cloudflare/workers-rs/blob/main/README.md`
- `https://developers.cloudflare.com/r2/api/tokens/`
- `https://developers.cloudflare.com/logs/reference/filters/`
- `https://developers.cloudflare.com/logs/reference/log-fields/`
- `https://developers.cloudflare.com/logs/tutorials/examples/example-logpush-curl/`
- `https://developers.cloudflare.com/api/resources/logpush/subresources/jobs/methods/create/`
- `https://developers.cloudflare.com/fundamentals/api/how-to/create-via-api/`
- `https://developers.cloudflare.com/api/resources/user/subresources/tokens/methods/create/`
- `https://developers.cloudflare.com/api/node/resources/r2/subresources/temporary_credentials/methods/create/`
- `https://developers.cloudflare.com/fundamentals/api/get-started/account-owned-tokens/`
- `https://developers.cloudflare.com/api/resources/accounts/subresources/tokens/methods/create/`
- `https://developers.cloudflare.com/api/resources/accounts/subresources/tokens/methods/update/`
- `https://engineering.salesforce.com/investigate-issues-with-ease-by-adding-a-correlation-id-to-your-api-7a689413ff44/`
- `http://work.haufegroup.io/api-style-guide/correlation-id/correlation-id.html`
- `https://learn.openapis.org/best-practices.html`
- `https://openapi-generator.tech/`
- `https://raw.githubusercontent.com/OpenAPITools/openapi-generator/master/README.md`
- `https://openapi-generator.tech/docs/generators/rust/`
- `https://softwareengineering.stackexchange.com/questions/445809/handling-openapi-generated-code-in-a-projects-repository`
- `https://antondevtips.com/blog/a-better-way-to-handle-entity-identification-in-dotnet-with-strongly-typed-ids`
- `https://spec.openapis.org/oas/v3.1.0.html`
- `https://json-schema.org/understanding-json-schema/reference/string.html`
- `https://owasp.org/www-community/attacks/Path_Traversal`
- `https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Strict-Transport-Security`
- `https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy`
- `https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Permitted-Cross-Domain-Policies`
- `https://developers.google.com/search/docs/crawling-indexing/robots/intro`
- `https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview`
- `https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CSP`
- `https://github.com/sveltejs/kit/blob/main/documentation/docs/30-advanced/20-hooks.md`
