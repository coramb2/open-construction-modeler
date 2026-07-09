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
   policies the migration defines.
3. In **Authentication > Sign In / Providers > GitHub**, enable the GitHub
   provider. You'll need a GitHub OAuth App:
   - Create one at <https://github.com/settings/developers> → "New OAuth App".
   - **Homepage URL**: your deployed URL (or `http://localhost:3000` for
     local dev).
   - **Authorization callback URL**: use the callback URL Supabase shows
     you on the GitHub provider setup page (it's your Supabase project URL
     + `/auth/v1/callback`, *not* this app's `/auth/callback` route).
   - Paste the resulting Client ID / Client Secret into Supabase's GitHub
     provider settings.
4. Copy your project's URL and publishable (anon) key from
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
- The browse feed, upload flow, item detail/viewer pages, and public
  profile pages aren't built yet — this PR is auth + schema only. See the
  main repo's task tracker / open PRs for what's landed since.
- The in-browser 3D viewer (once built) will only render glTF/GLB files —
  IFC/DXF parsing lives in Rust on the desktop app and hasn't been ported
  to the browser. Other file types will be downloadable but not
  previewable, same as how every other "browse 3D models" site (Sketchfab
  etc.) handles proprietary CAD formats.
- `npm run build` is pinned to `--webpack` instead of the Next.js 16
  default (Turbopack) because Turbopack's CSS build step hit a
  process-spawn timeout in the sandboxed dev environment this was built
  in. Turbopack should work fine on Vercel's own infrastructure — this is
  a defensive pin for reproducibility in case CI hits the same sandboxing
  constraint, not a known Turbopack bug. Safe to remove and re-test if it
  turns out to be a non-issue elsewhere.
