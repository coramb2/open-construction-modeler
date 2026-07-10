# Web platform — setup & deploy

This is the "Part 2" collaboration platform from the main repo's README: a
place to publish and browse construction models, from finished projects
down to individual made items. Next.js app, deployed to Vercel, backed by
Supabase (Postgres + Auth + Storage).

## 1. Create a Supabase project

1. Create a project at [supabase.com](https://supabase.com) (free tier is
   fine for testing).
2. In the SQL Editor, run `supabase/migrations/0001_init.sql` — it creates
   the `profiles`/`items`/`item_images` tables, all Row Level Security
   policies, the `models`/`images` storage buckets, and the trigger that
   auto-creates a profile row on signup. **Every table has RLS enabled and
   is denied by default** — nothing is readable/writable outside the
   policies the migration defines. Then run
   `supabase/migrations/0002_storage_limits.sql`, which sets the per-bucket
   size/type limits (10 MiB images, 100 MiB models). **This one is not
   optional:** uploads go straight from the browser to Storage, so the bucket
   config — not app code — is what actually caps file size. Skip it and users
   can upload arbitrarily large files.
3. In **Authentication > Sign In / Providers > GitHub**, enable the GitHub
   provider. You'll need a GitHub OAuth App:
   - Create one at <https://github.com/settings/developers> → "New OAuth App".
   - **Homepage URL**: your deployed URL (or `http://localhost:3000` for
     local dev).
   - **Authorization callback URL**: use the callback URL Supabase shows
     you on the GitHub provider setup page (it's your Supabase project URL
     + `/auth/v1/callback`, *not* this app's `/auth/callback` route).
   - Paste the resulting **Client ID / Client Secret** into Supabase's GitHub
     provider settings. ⚠️ Copy the actual **Client ID** from the OAuth App
     page (an opaque token like `Ov23li…`), *not* the app's name/slug —
     pasting the slug makes GitHub 404 the authorize page with
     `client_id=<slug>` and sign-in silently fails.
4. In **Authentication > URL Configuration**, set **Site URL** to your app's
   URL and add it to **Redirect URLs** (wildcards allowed), e.g.
   `https://your-app.vercel.app/**` and `http://localhost:3000/**`. This is
   an allow-list of **your own app URLs**, not users — Supabase only redirects
   the post-login `code` back to URLs listed here; if the app's
   `/auth/callback` URL isn't covered, Supabase falls back to the Site URL and
   the code lands on the wrong page (you stay signed out). One entry per
   domain covers every user.
5. Copy your project's URL and publishable (anon) key from
   **Settings > API** — you'll need them in the next step.

## 2. Configure environment variables

```bash
cp .env.example .env.local
```

Fill in `NEXT_PUBLIC_SUPABASE_URL` and `NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY`
from Supabase's API settings page (the "publishable" key may be labeled
"anon" key on older projects — same thing). `SUPABASE_SERVICE_ROLE_KEY` is
not used yet by any code in this app; leave it unset until something
actually needs it (server-side admin operations bypass RLS entirely, so
that key should never be added to more of the app's surface than
necessary).

## 3. Run locally

```bash
npm install
npm run dev
```

Visit `http://localhost:3000`. Sign in with GitHub to confirm the auth flow
works end-to-end (OAuth redirect → `/auth/callback` → session cookie set →
a `profiles` row created automatically by the database trigger).

## 4. Deploy to Vercel

1. Import this repo into Vercel, and set the project's **Root Directory**
   to `web` (this is a monorepo — the Next.js app lives in this
   subdirectory, not the repo root).
2. Add the same environment variables from step 2 in the Vercel project
   settings (**Settings > Environment Variables**).
3. Add your Vercel deployment URL as an authorized redirect in both:
   - GitHub OAuth App settings (**Authorization callback URL** stays
     pointed at Supabase's `/auth/v1/callback` — this doesn't change per
     deployment).
   - Supabase **Authentication > URL Configuration** — add your Vercel
     URL to **Redirect URLs** so `signInWithOAuth`'s `redirectTo` is
     allowed to complete.
4. Deploy. Vercel auto-builds on push once connected.

## Notes on what's *not* here yet

- No email/password or magic-link sign-in — GitHub OAuth only, for now.
  Supabase makes adding a second provider close to free; revisit if
  testers without GitHub accounts need access.
- Built: GitHub sign-in, the browse feed, the publish/upload flow, item
  detail pages, and public profiles. Not built yet: edit/delete of published
  items, and the fork/diff/merge features tracked in the main repo's project
  board.
- **Uploads go straight from the browser to Supabase Storage**, not through
  the `createItem` server action — a Server Action caps its request body at
  1 MB by default and Vercel caps a function request at ~4.5 MB, both well
  under the 10 MiB image / 100 MiB model limits. The client uploads to
  `{uid}/{folder}/…` (Storage RLS pins the first segment to the uploader) and
  then calls `createItem` with just the metadata + resulting paths; the action
  re-validates each path is inside the caller's own folder before recording
  it. Because the bytes bypass app code, the **bucket** size/type limits from
  `0002_storage_limits.sql` are the real enforcement — don't skip that
  migration.
- The in-browser 3D viewer renders glTF/GLB only — IFC/DXF parsing lives in
  Rust on the desktop app and hasn't been ported to the browser. Other file
  types are downloadable but not previewable, same as how every other "browse
  3D models" site (Sketchfab etc.) handles proprietary CAD formats.
- `npm run build` is pinned to `--webpack` instead of the Next.js 16
  default (Turbopack) because Turbopack's CSS build step hit a
  process-spawn timeout in the sandboxed dev environment this was built
  in. Turbopack should work fine on Vercel's own infrastructure — this is
  a defensive pin for reproducibility in case CI hits the same sandboxing
  constraint, not a known Turbopack bug. Safe to remove and re-test if it
  turns out to be a non-issue elsewhere.
