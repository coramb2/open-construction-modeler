'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { createClient } from '@/lib/supabase/client'
import { createItem } from './actions'
import {
  buildStoragePath,
  validateUploadFile,
  IMAGE_FILE_EXTENSIONS,
  MODEL_FILE_EXTENSIONS,
} from '@/lib/uploads'

function fileOrNull(value: FormDataEntryValue | null): File | null {
  return value instanceof File && value.size > 0 ? value : null
}

export default function NewItemForm() {
  const router = useRouter()
  const [error, setError] = useState<string | null>(null)
  const [pending, setPending] = useState(false)

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setError(null)
    setPending(true)

    try {
      const formData = new FormData(event.currentTarget)
      const title = String(formData.get('title') ?? '').trim()
      const description = String(formData.get('description') ?? '').trim()
      const category = String(formData.get('category') ?? '')
      const trade = String(formData.get('trade') ?? '').trim()
      const coverFile = fileOrNull(formData.get('cover_image'))
      const modelFile = fileOrNull(formData.get('model_file'))

      // Fast client-side feedback. Not a security boundary — the real limits
      // are enforced by the Storage bucket config (size/type) and the server
      // action (metadata). See 0002_storage_limits.sql.
      if (!title || title.length > 200) {
        throw new Error('Title is required (1–200 characters).')
      }
      if (description.length > 10000) {
        throw new Error('Description is too long (max 10,000 characters).')
      }
      if (coverFile) {
        const check = validateUploadFile(coverFile, 'image')
        if (!check.ok) throw new Error(check.error)
      }
      if (modelFile) {
        const check = validateUploadFile(modelFile, 'model')
        if (!check.ok) throw new Error(check.error)
      }

      const supabase = createClient()
      const { data: userData, error: userError } = await supabase.auth.getUser()
      const userId = userData.user?.id
      if (userError || !userId) {
        throw new Error('You must be signed in to publish.')
      }

      // One random folder per upload, under the user's own uid — matches the
      // `{uid}/…` prefix the Storage RLS write policy requires.
      const folder = crypto.randomUUID()
      let coverPath: string | null = null
      let modelPath: string | null = null

      if (coverFile) {
        coverPath = buildStoragePath(userId, folder, 'cover', coverFile.name)
        const { error: uploadError } = await supabase.storage
          .from('images')
          .upload(coverPath, coverFile, { upsert: true })
        if (uploadError) {
          throw new Error(`Cover image upload failed: ${uploadError.message}`)
        }
      }

      if (modelFile) {
        modelPath = buildStoragePath(userId, folder, 'model', modelFile.name)
        const { error: uploadError } = await supabase.storage
          .from('models')
          .upload(modelPath, modelFile, { upsert: true })
        if (uploadError) {
          throw new Error(`Model file upload failed: ${uploadError.message}`)
        }
      }

      const result = await createItem({ title, description, category, trade, coverPath, modelPath })
      if ('error' in result) {
        throw new Error(result.error)
      }

      // Leave `pending` true — we're navigating away on success.
      router.push(`/items/${result.itemId}`)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Something went wrong.')
      setPending(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="mx-auto max-w-xl space-y-5 px-6 py-8">
      <h1 className="text-xl font-bold text-blue-400">Publish</h1>

      {error && (
        <p className="rounded border border-red-800 bg-red-950 px-3 py-2 text-sm text-red-400">
          {error}
        </p>
      )}

      <div>
        <label htmlFor="title" className="mb-1 block text-xs uppercase text-gray-400">
          Title
        </label>
        <input
          id="title"
          name="title"
          required
          maxLength={200}
          className="w-full rounded border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100"
        />
      </div>

      <div>
        <label htmlFor="category" className="mb-1 block text-xs uppercase text-gray-400">
          Category
        </label>
        <select
          id="category"
          name="category"
          required
          defaultValue="item"
          className="w-full rounded border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100"
        >
          <option value="item">Item — a single made thing (a chair, a bracket, ...)</option>
          <option value="project">Project — a finished build</option>
        </select>
      </div>

      <div>
        <label htmlFor="trade" className="mb-1 block text-xs uppercase text-gray-400">
          Trade <span className="normal-case text-gray-600">(optional)</span>
        </label>
        <input
          id="trade"
          name="trade"
          placeholder="e.g. Structural, Woodworking, Electrical…"
          className="w-full rounded border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100"
        />
      </div>

      <div>
        <label htmlFor="description" className="mb-1 block text-xs uppercase text-gray-400">
          Description <span className="normal-case text-gray-600">(optional)</span>
        </label>
        <textarea
          id="description"
          name="description"
          rows={5}
          maxLength={10000}
          className="w-full rounded border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-100"
        />
      </div>

      <div>
        <label htmlFor="cover_image" className="mb-1 block text-xs uppercase text-gray-400">
          Cover image <span className="normal-case text-gray-600">(optional)</span>
        </label>
        <input
          id="cover_image"
          name="cover_image"
          type="file"
          accept={IMAGE_FILE_EXTENSIONS.map((e) => `.${e}`).join(',')}
          className="w-full text-sm text-gray-300 file:mr-3 file:rounded file:border-0 file:bg-gray-700 file:px-3 file:py-1.5 file:text-gray-100"
        />
      </div>

      <div>
        <label htmlFor="model_file" className="mb-1 block text-xs uppercase text-gray-400">
          Model file <span className="normal-case text-gray-600">(optional — glTF/GLB gets an in-browser preview, other formats are downloadable only)</span>
        </label>
        <input
          id="model_file"
          name="model_file"
          type="file"
          accept={MODEL_FILE_EXTENSIONS.map((e) => `.${e}`).join(',')}
          className="w-full text-sm text-gray-300 file:mr-3 file:rounded file:border-0 file:bg-gray-700 file:px-3 file:py-1.5 file:text-gray-100"
        />
      </div>

      <button
        type="submit"
        disabled={pending}
        className="w-full rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed"
      >
        {pending ? 'Publishing…' : 'Publish'}
      </button>
    </form>
  )
}
