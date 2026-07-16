-- Fork lineage (issue #24): an item can be an independent copy of another,
-- linked back to its source. Run in the Supabase SQL editor after 0002.

alter table public.items
  add column if not exists forked_from uuid references public.items (id) on delete set null;

-- Indexed for the "how many forks does this item have?" count and for walking
-- lineage. `on delete set null` means a fork survives its parent being deleted
-- (the link just goes null) rather than cascading away.
create index if not exists items_forked_from_idx on public.items (forked_from);

-- No new RLS needed: forked_from is just another column on the items row,
-- already governed by the existing policies (owner-scoped writes; published-or-
-- owned reads). A fork is a normal insert the owner makes with this column set.
