import { describe, it, expect } from 'vitest'
import {
  buildStoragePath,
  fileExtension,
  hasAllowedExtension,
  isGltfFile,
  isOwnedStoragePath,
  MAX_IMAGE_FILE_BYTES,
  MAX_MODEL_FILE_BYTES,
  validateUploadFile,
} from './uploads'

function makeFile(name: string, sizeBytes: number): File {
  // Building a real Blob of the target size is wasteful for a 100MB test —
  // fake the size instead, since validateUploadFile only reads .size/.name.
  const file = new File([''], name)
  Object.defineProperty(file, 'size', { value: sizeBytes })
  return file
}

describe('fileExtension', () => {
  it('extracts a lowercase extension', () => {
    expect(fileExtension('model.GLB')).toBe('glb')
    expect(fileExtension('photo.jpg')).toBe('jpg')
  })

  it('handles a multi-dot filename by taking the last segment', () => {
    expect(fileExtension('my.cool.model.glb')).toBe('glb')
  })

  it('returns empty string for no extension', () => {
    expect(fileExtension('README')).toBe('')
  })

  it('returns empty string for a trailing dot with nothing after it', () => {
    expect(fileExtension('file.')).toBe('')
  })
})

describe('isGltfFile', () => {
  it('recognizes glb and gltf', () => {
    expect(isGltfFile('model.glb')).toBe(true)
    expect(isGltfFile('model.gltf')).toBe(true)
    expect(isGltfFile('model.GLB')).toBe(true)
  })

  it('rejects other model formats', () => {
    expect(isGltfFile('model.ifc')).toBe(false)
    expect(isGltfFile('model.dxf')).toBe(false)
  })

  it('handles null/undefined without throwing', () => {
    expect(isGltfFile(null)).toBe(false)
    expect(isGltfFile(undefined)).toBe(false)
    expect(isGltfFile('')).toBe(false)
  })
})

describe('validateUploadFile', () => {
  it('accepts a valid model file', () => {
    const result = validateUploadFile(makeFile('house.ifc', 1024), 'model')
    expect(result.ok).toBe(true)
  })

  it('accepts a valid image file', () => {
    const result = validateUploadFile(makeFile('cover.png', 1024), 'image')
    expect(result.ok).toBe(true)
  })

  it('rejects a disallowed extension for the given kind', () => {
    const result = validateUploadFile(makeFile('cover.exe', 1024), 'image')
    expect(result.ok).toBe(false)
  })

  it('rejects an image file used where a model was expected (and vice versa)', () => {
    // .jpg is a valid image extension but not a valid model extension —
    // confirms the allowlist is actually scoped per `kind`, not shared.
    expect(validateUploadFile(makeFile('photo.jpg', 1024), 'model').ok).toBe(false)
    expect(validateUploadFile(makeFile('house.ifc', 1024), 'image').ok).toBe(false)
  })

  it('rejects a file over the size cap', () => {
    const result = validateUploadFile(makeFile('house.ifc', MAX_MODEL_FILE_BYTES + 1), 'model')
    expect(result.ok).toBe(false)
    if (!result.ok) expect(result.error).toMatch(/too large/)
  })

  it('accepts a file exactly at the size cap', () => {
    expect(validateUploadFile(makeFile('house.ifc', MAX_MODEL_FILE_BYTES), 'model').ok).toBe(true)
    expect(validateUploadFile(makeFile('cover.png', MAX_IMAGE_FILE_BYTES), 'image').ok).toBe(true)
  })

  it('rejects an empty file', () => {
    const result = validateUploadFile(makeFile('house.ifc', 0), 'model')
    expect(result.ok).toBe(false)
    if (!result.ok) expect(result.error).toMatch(/empty/)
  })

  it('rejects a file with no extension', () => {
    expect(validateUploadFile(makeFile('house', 1024), 'model').ok).toBe(false)
  })
})

describe('buildStoragePath', () => {
  it('builds a {user_id}/{item_id}/{kind}.{ext} path matching the RLS policy convention', () => {
    expect(buildStoragePath('user-1', 'item-1', 'cover', 'photo.jpg')).toBe(
      'user-1/item-1/cover.jpg',
    )
    expect(buildStoragePath('user-1', 'item-1', 'model', 'House.IFC')).toBe(
      'user-1/item-1/model.ifc',
    )
  })

  it('omits the extension segment cleanly when there is none', () => {
    expect(buildStoragePath('user-1', 'item-1', 'cover', 'noext')).toBe('user-1/item-1/cover')
  })
})

describe('isOwnedStoragePath', () => {
  it('accepts a well-formed path inside the user’s own folder', () => {
    expect(isOwnedStoragePath('user-1/folder-1/cover.jpg', 'user-1')).toBe(true)
    expect(isOwnedStoragePath('user-1/folder-1/model.ifc', 'user-1')).toBe(true)
  })

  it('rejects a path owned by a different user', () => {
    // The crux: a crafted request must not be able to record someone else’s
    // file path onto its own item row.
    expect(isOwnedStoragePath('user-2/folder-1/cover.jpg', 'user-1')).toBe(false)
  })

  it('rejects path traversal', () => {
    expect(isOwnedStoragePath('user-1/../user-2/folder/cover.jpg', 'user-1')).toBe(false)
    expect(isOwnedStoragePath('user-1/folder/../../secret', 'user-1')).toBe(false)
  })

  it('rejects the wrong number of segments (too few or too many)', () => {
    expect(isOwnedStoragePath('user-1/cover.jpg', 'user-1')).toBe(false)
    expect(isOwnedStoragePath('user-1/folder/sub/cover.jpg', 'user-1')).toBe(false)
    expect(isOwnedStoragePath('user-1', 'user-1')).toBe(false)
  })

  it('rejects empty segments and empty input', () => {
    expect(isOwnedStoragePath('user-1//cover.jpg', 'user-1')).toBe(false)
    expect(isOwnedStoragePath('user-1/folder/', 'user-1')).toBe(false)
    expect(isOwnedStoragePath('', 'user-1')).toBe(false)
  })
})

describe('hasAllowedExtension', () => {
  it('accepts allowed extensions for each kind', () => {
    expect(hasAllowedExtension('user/folder/cover.png', 'image')).toBe(true)
    expect(hasAllowedExtension('user/folder/model.ifc', 'model')).toBe(true)
    expect(hasAllowedExtension('user/folder/model.GLB', 'model')).toBe(true)
  })

  it('rejects an extension not in the kind allowlist (server-side gate)', () => {
    // A crafted request could report any path; the server must not accept an
    // arbitrary extension onto the item row.
    expect(hasAllowedExtension('user/folder/cover.exe', 'image')).toBe(false)
    expect(hasAllowedExtension('user/folder/cover.svg', 'image')).toBe(false)
    expect(hasAllowedExtension('user/folder/model.png', 'model')).toBe(false)
  })

  it('rejects a path with no extension', () => {
    expect(hasAllowedExtension('user/folder/cover', 'image')).toBe(false)
  })
})
