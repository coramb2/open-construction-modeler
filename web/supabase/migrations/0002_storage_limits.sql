-- Open Construction Modeler — storage bucket size/type limits
-- Run this in the Supabase SQL editor after 0001_init.sql.
--
-- WHY: uploads now go straight from the browser to Storage (the previous
-- server-action upload path hit Next.js's 1 MB Server Action body limit and
-- Vercel's ~4.5 MB function-request cap — see web/SETUP.md). Because the file
-- bytes no longer pass through the app's server-side validation, the bucket
-- configuration IS the enforcement boundary for size and type. These limits
-- mirror MAX_IMAGE_FILE_BYTES / MAX_MODEL_FILE_BYTES in src/lib/uploads.ts —
-- keep the two in sync.

-- Images: 10 MiB, restricted to the formats the app renders.
update storage.buckets
set
  file_size_limit = 10485760, -- 10 * 1024 * 1024
  allowed_mime_types = array['image/jpeg', 'image/png', 'image/webp', 'image/gif']
where id = 'images';

-- Models: 100 MiB. No MIME allowlist — IFC/DXF/OCM have no registered MIME
-- type and arrive as application/octet-stream, so type is gated by the
-- extension allowlist in the app (MODEL_FILE_EXTENSIONS), not here.
update storage.buckets
set file_size_limit = 104857600 -- 100 * 1024 * 1024
where id = 'models';
