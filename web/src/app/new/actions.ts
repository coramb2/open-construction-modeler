'use server'

import { createClient } from '@/lib/supabase/server'
import { fileExtension, isOwnedStoragePath } from '@/lib/uploads'
import type { ItemCategory } from '@/lib/database.types'
import { revalidatePath } from 'next/cache'

// Files are uploaded straight from the browser to Supabase Storage (bypassing
// the Server Action / Vercel request-body size limits). This action never sees
// the bytes — the client hands it the resulting storage *paths* plus the item
// metadata. Per the Next.js Server Actions security guidance: derive ownership
// from the session, and re-validate every client-supplied value (including the
// paths) before writing it.
export type CreateItemInput = {
  title: string
  description: string
  category: string
  trade: string
  coverPath: string | null
  modelPath: string | null
}

export type CreateItemResult = { error: string } | { itemId: string }

export async function createItem(input: CreateItemInput): Promise<CreateItemResult> {
  const supabase = await createClient()
  const { data: claims } = await supabase.auth.getClaims()
  const userId = claims?.claims.sub
  if (!userId) {
    return { error: 'You must be signed in.' }
  }

  const title = input.title?.trim() ?? ''
  const description = input.description?.trim() ?? ''
  const trade = input.trade?.trim() ?? ''
  const category = input.category

  if (!title || title.length > 200) {
    return { error: 'Title is required (1–200 characters).' }
  }
  if (description.length > 10000) {
    return { error: 'Description is too long (max 10,000 characters).' }
  }
  if (category !== 'project' && category !== 'item') {
    return { error: 'Invalid category.' }
  }

  // The bytes were uploaded client-side; here we only accept a path that
  // provably belongs to this user's folder. Storage RLS already blocks a
  // cross-user write, but a path is still untrusted input on the way to the
  // items row — re-check it (null means "no file", which is allowed).
  if (input.coverPath !== null && !isOwnedStoragePath(input.coverPath, userId)) {
    return { error: 'Invalid cover image path.' }
  }
  if (input.modelPath !== null && !isOwnedStoragePath(input.modelPath, userId)) {
    return { error: 'Invalid model file path.' }
  }

  const modelType = input.modelPath ? fileExtension(input.modelPath) : null

  const { data: item, error } = await supabase
    .from('items')
    .insert({
      owner_id: userId,
      title,
      description: description || null,
      category: category as ItemCategory,
      trade: trade || null,
      cover_image_path: input.coverPath,
      model_file_path: input.modelPath,
      model_file_type: modelType,
      published: true,
    })
    .select('id')
    .single()

  if (error || !item) {
    return { error: error?.message ?? 'Failed to create item.' }
  }

  // Refresh the home feed so the new item shows immediately.
  revalidatePath('/')
  return { itemId: item.id }
}
