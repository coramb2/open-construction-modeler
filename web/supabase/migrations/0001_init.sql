-- Open Construction Modeler — Collaboration Platform schema
-- Run this in the Supabase SQL editor (or via `supabase db push`) on a
-- fresh project. Every table has Row Level Security enabled — there is no
-- table here that's readable/writable without an explicit policy below.

-- ============================================================================
-- profiles
-- ============================================================================
-- One row per auth.users row. Created automatically by the trigger below —
-- never insert into this table directly from the app.
create table if not exists public.profiles (
  id uuid primary key references auth.users (id) on delete cascade,
  username text unique not null,
  display_name text,
  avatar_url text,
  bio text,
  created_at timestamptz not null default now()
);

alter table public.profiles enable row level security;

create policy "profiles are publicly readable"
  on public.profiles for select
  using (true);

create policy "users can update their own profile"
  on public.profiles for update
  using (auth.uid() = id)
  with check (auth.uid() = id);

-- No insert/delete policy: profile rows are created only by the
-- handle_new_user trigger (as the postgres role, which bypasses RLS) and
-- are never deleted directly — they cascade from auth.users deletion.

-- Auto-create a profile row whenever a new user signs up. Username is
-- derived from GitHub's OAuth metadata (user_name) with a random-suffix
-- fallback to guarantee uniqueness even if two users share a GitHub handle
-- across different identity providers, or the metadata is missing.
create or replace function public.handle_new_user()
returns trigger
language plpgsql
security definer
set search_path = public
as $$
begin
  insert into public.profiles (id, username, display_name, avatar_url)
  values (
    new.id,
    coalesce(
      new.raw_user_meta_data ->> 'user_name',
      new.raw_user_meta_data ->> 'preferred_username',
      'user_' || substr(new.id::text, 1, 8)
    ),
    coalesce(new.raw_user_meta_data ->> 'full_name', new.raw_user_meta_data ->> 'user_name'),
    new.raw_user_meta_data ->> 'avatar_url'
  )
  on conflict (id) do nothing;
  return new;
exception
  when unique_violation then
    -- Username collision — fall back to a guaranteed-unique value rather
    -- than failing the signup entirely.
    insert into public.profiles (id, username, display_name, avatar_url)
    values (
      new.id,
      'user_' || replace(new.id::text, '-', ''),
      coalesce(new.raw_user_meta_data ->> 'full_name', new.raw_user_meta_data ->> 'user_name'),
      new.raw_user_meta_data ->> 'avatar_url'
    )
    on conflict (id) do nothing;
    return new;
end;
$$;

drop trigger if exists on_auth_user_created on auth.users;
create trigger on_auth_user_created
  after insert on auth.users
  for each row execute function public.handle_new_user();

-- ============================================================================
-- items — finished projects and individual made items (a "chair", etc.)
-- ============================================================================
create table if not exists public.items (
  id uuid primary key default gen_random_uuid(),
  owner_id uuid not null references public.profiles (id) on delete cascade,
  title text not null check (char_length(title) between 1 and 200),
  description text check (char_length(description) <= 10000),
  category text not null check (category in ('project', 'item')),
  trade text,
  model_file_path text,
  model_file_type text,
  cover_image_path text,
  published boolean not null default true,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists items_owner_id_idx on public.items (owner_id);
create index if not exists items_published_created_at_idx
  on public.items (created_at desc) where published;

alter table public.items enable row level security;

create policy "published items are publicly readable"
  on public.items for select
  using (published or owner_id = auth.uid());

create policy "users can insert their own items"
  on public.items for insert
  with check (owner_id = auth.uid());

create policy "users can update their own items"
  on public.items for update
  using (owner_id = auth.uid())
  with check (owner_id = auth.uid());

create policy "users can delete their own items"
  on public.items for delete
  using (owner_id = auth.uid());

create or replace function public.set_updated_at()
returns trigger
language plpgsql
as $$
begin
  new.updated_at = now();
  return new;
end;
$$;

drop trigger if exists items_set_updated_at on public.items;
create trigger items_set_updated_at
  before update on public.items
  for each row execute function public.set_updated_at();

-- ============================================================================
-- item_images — additional photos per item (beyond the single cover image)
-- ============================================================================
create table if not exists public.item_images (
  id uuid primary key default gen_random_uuid(),
  item_id uuid not null references public.items (id) on delete cascade,
  storage_path text not null,
  position integer not null default 0,
  created_at timestamptz not null default now()
);

create index if not exists item_images_item_id_idx on public.item_images (item_id);

alter table public.item_images enable row level security;

create policy "images of visible items are publicly readable"
  on public.item_images for select
  using (
    exists (
      select 1 from public.items
      where items.id = item_images.item_id
        and (items.published or items.owner_id = auth.uid())
    )
  );

create policy "users can insert images on their own items"
  on public.item_images for insert
  with check (
    exists (
      select 1 from public.items
      where items.id = item_images.item_id and items.owner_id = auth.uid()
    )
  );

create policy "users can delete images on their own items"
  on public.item_images for delete
  using (
    exists (
      select 1 from public.items
      where items.id = item_images.item_id and items.owner_id = auth.uid()
    )
  );

-- ============================================================================
-- Storage buckets
-- ============================================================================
-- Public buckets: once an item is published, its files are meant to be
-- viewable/downloadable by anyone (that's the point of a public gallery),
-- same model every "browse 3D models" site (Sketchfab etc.) uses. Access
-- control here is about who can WRITE, not who can read.
insert into storage.buckets (id, name, public)
values ('models', 'models', true)
on conflict (id) do nothing;

insert into storage.buckets (id, name, public)
values ('images', 'images', true)
on conflict (id) do nothing;

-- Explicit public-read policies (not just relying on bucket.public=true,
-- which only bypasses RLS for the public-URL endpoint) so authenticated
-- APIs like storage.list()/download() also work for any future "my
-- uploads" UI, not just getPublicUrl().
create policy "models are publicly readable"
  on storage.objects for select
  using (bucket_id = 'models');

create policy "images are publicly readable"
  on storage.objects for select
  using (bucket_id = 'images');

-- Upload path convention enforced by policy: {user_id}/{item_id}/{filename}
-- — a user can only write into a folder prefixed with their own uid, so one
-- user can never overwrite or delete another user's files.
create policy "authenticated users can upload to their own folder (models)"
  on storage.objects for insert
  with check (
    bucket_id = 'models'
    and (storage.foldername(name))[1] = auth.uid()::text
  );

create policy "users can update files in their own folder (models)"
  on storage.objects for update
  using (
    bucket_id = 'models'
    and (storage.foldername(name))[1] = auth.uid()::text
  );

create policy "users can delete files in their own folder (models)"
  on storage.objects for delete
  using (
    bucket_id = 'models'
    and (storage.foldername(name))[1] = auth.uid()::text
  );

create policy "authenticated users can upload to their own folder (images)"
  on storage.objects for insert
  with check (
    bucket_id = 'images'
    and (storage.foldername(name))[1] = auth.uid()::text
  );

create policy "users can update files in their own folder (images)"
  on storage.objects for update
  using (
    bucket_id = 'images'
    and (storage.foldername(name))[1] = auth.uid()::text
  );

create policy "users can delete files in their own folder (images)"
  on storage.objects for delete
  using (
    bucket_id = 'images'
    and (storage.foldername(name))[1] = auth.uid()::text
  );
