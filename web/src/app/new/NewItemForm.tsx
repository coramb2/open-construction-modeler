'use client'

import { useActionState } from 'react'
import { createItem } from './actions'
import { IMAGE_FILE_EXTENSIONS, MODEL_FILE_EXTENSIONS } from '@/lib/uploads'

export default function NewItemForm() {
  const [state, formAction, pending] = useActionState(createItem, undefined)

  return (
    <form action={formAction} className="mx-auto max-w-xl space-y-5 px-6 py-8">
      <h1 className="text-xl font-bold text-blue-400">Publish</h1>

      {state?.error && (
        <p className="rounded border border-red-800 bg-red-950 px-3 py-2 text-sm text-red-400">
          {state.error}
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
          Cover image
        </label>
        <input
          id="cover_image"
          name="cover_image"
          type="file"
          required
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
