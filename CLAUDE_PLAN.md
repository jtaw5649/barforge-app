# Barforge Modules Marketplace - Implementation Plan

## Overview

Build a web marketplace + redesigned desktop app for Barforge modules with:
- **Website**: SvelteKit on Cloudflare Pages
- **Auth**: GitHub OAuth
- **API**: Extended barforge-registry-api
- **Desktop**: Updated barforge-app to match website design

---

## Phase Summary

| Phase | Duration | Description |
|-------|----------|-------------|
| 1 | Week 1-2 | Database & Auth Foundation |
| 2 | Week 2-3 | Reviews System |
| 3 | Week 3-4 | Module Submission Flow |
| 4 | Week 4-5 | Admin & Moderation |
| 5 | Week 5-8 | Website Frontend |
| 6 | Week 8-10 | Desktop App Updates |
| 7 | Week 10-11 | Polish & Launch |

---

## 1. Database Schema

### New Tables (Migration 0003)

```sql
-- Users (GitHub OAuth)
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    github_id INTEGER UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,
    display_name TEXT,
    avatar_url TEXT,
    bio TEXT,
    website_url TEXT,
    verified_author INTEGER DEFAULT 0,
    role TEXT DEFAULT 'user' CHECK(role IN ('user', 'moderator', 'admin')),
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Sessions
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id),
    token_hash TEXT UNIQUE NOT NULL,
    expires_at TEXT NOT NULL
);

-- Reviews
CREATE TABLE reviews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_uuid TEXT NOT NULL REFERENCES modules(uuid),
    user_id INTEGER NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK(rating >= 1 AND rating <= 5),
    title TEXT,
    body TEXT,
    helpful_count INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(module_uuid, user_id)
);

-- Module Submissions (moderation queue)
CREATE TABLE module_submissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    submitter_id INTEGER NOT NULL REFERENCES users(id),
    uuid TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    version TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    status TEXT DEFAULT 'pending' CHECK(status IN ('pending', 'approved', 'rejected')),
    submitted_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Module Versions
CREATE TABLE module_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_uuid TEXT NOT NULL REFERENCES modules(uuid),
    version TEXT NOT NULL,
    changelog TEXT,
    package_key TEXT NOT NULL,
    downloads INTEGER DEFAULT 0,
    published_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(module_uuid, version)
);

-- Featured Modules
CREATE TABLE featured_modules (
    module_uuid TEXT PRIMARY KEY REFERENCES modules(uuid),
    position INTEGER NOT NULL
);
```

---

## 2. New API Endpoints

### Auth Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/auth/github` | Initiate OAuth |
| GET | `/auth/github/callback` | OAuth callback |
| POST | `/auth/logout` | Logout |
| GET | `/auth/me` | Current user |

### User Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/users/:username` | Public profile |
| PATCH | `/api/v1/users/me` | Update profile |
| GET | `/api/v1/users/:username/modules` | User's modules |

### Module Management (Auth Required)
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/modules` | Submit module |
| GET | `/api/v1/modules/mine` | My modules |
| PUT | `/api/v1/modules/:uuid` | Update module |
| POST | `/api/v1/modules/:uuid/versions` | Publish version |
| DELETE | `/api/v1/modules/:uuid` | Unlist module |

### Reviews
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/:uuid/reviews` | Get reviews |
| POST | `/api/v1/modules/:uuid/reviews` | Create review |
| PUT | `/api/v1/modules/:uuid/reviews` | Update review |
| DELETE | `/api/v1/modules/:uuid/reviews` | Delete review |

### Admin (Moderator+)
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/admin/submissions` | Moderation queue |
| POST | `/api/v1/admin/submissions/:id/review` | Approve/reject |
| POST | `/api/v1/admin/users/:id/verify` | Verify author |
| GET | `/api/v1/admin/stats` | Dashboard stats |

### Public
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/featured` | Featured + popular + recent |

---

## 3. Website Pages

```
/                        Homepage (featured, popular, recent)
/browse                  Browse all modules
/browse?category=:cat    Filter by category
/modules/:uuid           Module detail (reviews, screenshots)
/authors/:username       Author profile

/login                   GitHub OAuth initiation
/submit                  Submit module form
/dashboard               User dashboard
/dashboard/modules       My modules
/dashboard/reviews       My reviews
/dashboard/settings      Account settings

/admin                   Admin dashboard
/admin/submissions       Moderation queue
/admin/users             User management
```

### Key Components
- `<ModuleCard />` - Grid card with name, author, rating, downloads
- `<ModuleDetail />` - Full page with screenshots, reviews, install
- `<ReviewSection />` - Rating distribution + review list
- `<SubmitForm />` - Multi-step wizard for module submission
- `<ModerationQueue />` - Table of pending submissions

---

## 4. Desktop App Changes

### Registry API Models
- Generate the Rust client into `crates/barforge-registry-client` from `barforge-registry-api/docs/openapi.yaml`.
- Map API models into app domain types in `src/api.rs` (Author, Review, RegistryModule).

### New Screens
- **Author Profile** - Show author info + their modules
- **Enhanced Module Detail** - Add reviews section, author card

### New Widgets
- `review_card.rs` - Display individual review
- `rating_distribution.rs` - Star distribution chart
- `author_card.rs` - Author info with avatar

### Files to Modify
- `src/app/state.rs` - Add `AuthorProfile(String)` screen
- `src/app/message.rs` - Add review/author messages
- `src/widget/module_detail_screen.rs` - Add reviews section
- `src/tasks/registry.rs` - Add review/author API calls

---

## 5. Design System

### Colors (Dark Theme)
```css
--color-primary: #617DFA;
--bg-base: #191B1F;
--bg-surface: #222428;
--bg-elevated: #2A2D32;
--text-normal: #F6F7F9;
--text-muted: #B0B5BE;
```

### Typography
```css
--font-sans: 'Inter', system-ui, sans-serif;
--font-mono: 'JetBrains Mono', monospace;
```

### Spacing
```css
--space-sm: 8px;
--space-md: 12px;
--space-lg: 16px;
--space-xl: 24px;
```

---

## 6. Critical Files

### API (barforge-registry-api)
- `src/lib.rs` - Register new routes
- `src/routes.rs` - Add handlers
- `src/db.rs` - Add queries
- `migrations/0003_marketplace.sql`

### Desktop (barforge-app)
- `crates/barforge-registry-client/` - Regenerate API client
- `src/api.rs` - Map API models into domain types
- `src/widget/module_detail_screen.rs` - Enhance
- `src/app/state.rs` - Add screen variant
- `src/theme/palette.rs` - Color reference

### Website (new repo: barforge-web)
- `src/routes/+page.svelte` - Homepage
- `src/routes/browse/+page.svelte` - Browse
- `src/routes/modules/[uuid]/+page.svelte` - Detail
- `src/lib/components/` - Shared components

---

## 7. Immediate Next Steps

1. **Regenerate API client** - Run `scripts/generate-registry-client.sh` after spec changes
2. **Create new repo** - barforge-web (SvelteKit)
3. **Start Phase 1** - Database migrations + GitHub OAuth
4. **Design mockups** - Create Figma/sketches for website

---

## Open Questions

1. Should website and API be same Cloudflare project or separate?
2. Package uploads: direct to R2 or through API proxy?
3. Minimum features for MVP launch?
