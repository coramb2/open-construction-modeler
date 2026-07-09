'use server'

import { createClient } from '@/lib/supabase/server'
import { buildStoragePath, fileExtension, validateUploadFile } from '@/lib/uploads'
import { redirect } from 'next/navigation'

export type CreateItemState = { error: string } | undefined

function asString(value: FormDataEntryValue | null): string {
  return typeof value === 'string' ? value.trim() : ''
}

function asFile(value: FormDataEntryValue | null): File | null {
  return value instanceof File && value.size > 0 ? value : null
}

export async function createItem(
  _prevState: CreateItemState,
  formData: FormData,
): Promise<CreateItemState> {
  const supabase = await createClient()
  const { data: claims } = await supabase.auth.getClaims()
  const userId = claims?.claims.sub
  if (!userId) {
    return { error: 'You must be signed in.' }
  }

  const title = asString(formData.get('title'))
  const description = asString(formData.get('description'))
  const category = asString(formData.get('category'))
  const trade = asString(formData.get('trade'))
  const coverImage = asFile(formData.get('cover_image'))
  const modelFile = asFile(formData.get('model_file'))

  if (!title || title.length > 200) {
    return { error: 'Title is required (1–200 characters).' }
  }
  if (description.length > 10000) {
    return { error: 'Description is too long (max 10,000 characters).' }
  }
  if (category !== 'project' && category !== 'item') {
    return { error: 'Invalid category.' }
  }
  if (!coverImage) {
    return { error: 'A cover image is required.' }
  }

  const coverCheck = validateUploadFile(coverImage, 'image')
  if (!coverCheck.ok) return { error: coverCheck.error }

  if (modelFile) {
    const modelCheck = validateUploadFile(modelFile, 'model')
    if (!modelCheck.ok) return { error: modelCheck.error }
  }

  // Insert as unpublished first — the row must not be visible in the public
  // feed until every upload has actually succeeded. Flipping `published`
  // only happens in the final update, once everything else is confirmed.
  const { data: item, error: insertError } = await supabase
    .from('items')
    .insert({
      owner_id: userId,
      title,
      description: description || null,
      category,
      trade: trade || null,
      published: false,
    })
    .select()
    .single()

  if (insertError || !item) {
    return { error: insertError?.message ?? 'Failed to create item.' }
  }

  const coverPath = buildStoragePath(userId, item.id, 'cover', coverImage.name)
  const { error: coverUploadError } = await supabase.storage
    .from('images')
    .upload(coverPath, coverImage, { upsert: true })
  if (coverUploadError) {
    return { error: `Cover image upload failed: ${coverUploadError.message}` }
  }

  let modelPath: string | null = null
  let modelType: string | null = null
  if (modelFile) {
    modelPath = buildStoragePath(userId, item.id, 'model', modelFile.name)
    modelType = fileExtension(modelFile.name)
    const { error: modelUploadError } = await supabase.storage
      .from('models')
      .upload(modelPath, modelFile, { upsert: true })
    if (modelUploadError) {
      return { error: `Model file upload failed: ${modelUploadError.message}` }
    }
  }

  const { error: updateError } = await supabase
    .from('items')
    .update({
      cover_image_path: coverPath,
      model_file_path: modelPath,
      model_file_type: modelType,
      published: true,
    })
    .eq('id', item.id)

  if (updateError) {
    return { error: updateError.message }
  }

  redirect(`/items/${item.id}`)
}
