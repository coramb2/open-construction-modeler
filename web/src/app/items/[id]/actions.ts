'use server'

import { createClient } from '@/lib/supabase/server'
import { redirect } from 'next/navigation'
import { revalidatePath } from 'next/cache'

/**
 * Fork an item: create an independent, owned copy linked back to the source.
 *
 * Per the Server Action security model, the client supplies only the source
 * *id* (a reference); ownership is taken from the session and the source row is
 * re-read from the database, not trusted from the client. Files are public, so
 * the fork references the same storage paths — a future enhancement can copy
 * them into the forker's folder for true independence.
 */
export async function forkItem(sourceId: string): Promise<{ error: string } | undefined> {
  const supabase = await createClient()
  const { data: claims } = await supabase.auth.getClaims()
  const userId = claims?.claims.sub
  if (!userId) {
    return { error: 'You must be signed in to fork.' }
  }

  // RLS ensures the source is visible to this user (published, or their own).
  const { data: source, error: sourceError } = await supabase
    .from('items')
    .select(
      'id, title, description, category, trade, cover_image_path, model_file_path, model_file_type, published',
    )
    .eq('id', sourceId)
    .single()
  if (sourceError || !source) {
    return { error: 'Original not found or not accessible.' }
  }
  if (!source.published) {
    return { error: 'Only published items can be forked.' }
  }

  const { data: fork, error: insertError } = await supabase
    .from('items')
    .insert({
      owner_id: userId,
      title: source.title,
      description: source.description,
      category: source.category,
      trade: source.trade,
      cover_image_path: source.cover_image_path,
      model_file_path: source.model_file_path,
      model_file_type: source.model_file_type,
      forked_from: source.id,
      published: true,
    })
    .select('id')
    .single()
  if (insertError || !fork) {
    return { error: insertError?.message ?? 'Failed to create the fork.' }
  }

  revalidatePath('/')
  redirect(`/items/${fork.id}`)
}
