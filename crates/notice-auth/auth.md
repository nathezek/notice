## Auth Middleware:
✅ JWT-based authentication middleware
   - AuthUser extractor (required — returns 401 if missing)
   - OptionalAuthUser extractor (optional — None if not logged in)
   - Extracts user_id + username from JWT claims

✅ Automatic personalization via Bearer token
   - No more passing user_id in query params
   - Just add: Authorization: Bearer <token>
   - Search automatically uses the authenticated user's KG

✅ Protected routes
   - /api/auth/me — view your own profile (auth required)
   - /api/me/kg — view your own KG (auth required)
   - /api/me/kg/context — view your search context (auth required)

✅ Public routes unchanged
   - /api/search — works anonymous AND authenticated
   - /api/submit — works anonymous, tracks submitter if logged in
   - /health, /api/auth/register, /api/auth/login — no auth needed

✅ Clear error messages
   - 401 with human-readable messages for missing/invalid/expired tokens

### What changed:
BEFORE (current):
  GET /api/search?q=ownership&user_id=37ef3f09-...
  (user passes their own ID — insecure, anyone can impersonate)

AFTER:
  GET /api/search?q=ownership
  Authorization: Bearer eyJ0eXAi...
  (server extracts user_id from JWT automatically)

### The Diagram:

```text
┌──────────────────────────────────────────────────────┐
│                   Request Flow                       │
│                                                      │
│  Request ──▶ Auth Middleware ──▶ Route Handler        │
│                   │                                  │
│           ┌───────┴────────┐                         │
│           │ Has Bearer     │                         │
│           │ token?         │                         │
│           └───────┬────────┘                         │
│              YES  │  NO                              │
│           ┌───────┴────────┐                         │
│           │ Verify JWT     │  AuthUser = None         │
│           │ Extract claims │  (anonymous access OK)   │
│           └───────┬────────┘                         │
│              VALID │ INVALID                         │
│           ┌───────┴────────┐                         │
│  AuthUser │ = Some(user_id)│  → 401 Unauthorized     │
│           └────────────────┘                         │
└──────────────────────────────────────────────────────┘

Routes:
  /api/search           → optional auth (anonymous OK, personalized if logged in)
  /api/submit           → optional auth (tracks who submitted)
  /api/crawl            → optional auth
  /api/users/{id}/kg    → required auth (only own KG)
  /api/admin/resync     → required auth (admin only, future)
  /api/auth/register    → no auth
  /api/auth/login       → no auth
  /health               → no auth

```

### The Flow:

```text
Register/Login → JWT token
     │
     ▼
Add to requests: Authorization: Bearer <token>
     │
     ▼
Middleware extracts user_id from JWT
     │
     ├── Required auth routes → 401 if missing
     └── Optional auth routes → anonymous if missing, personalized if present

```

### Why this is better:

1. **Security**: Users can't fake their user_id anymore
2. **Convenience**: No need to pass user_id in every request
3. **Standardization**: Follows standard JWT authentication patterns
4. **Flexibility**: Easy to add more claims to the JWT later (roles, permissions, etc.)
