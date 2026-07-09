import Link from 'next/link'
import { notFound } from 'next/navigation'
import { createClient } from '@/lib/supabase/server'
import { storagePublicUrl } from '@/lib/storage-url'
import { isGltfFile } from '@/lib/uploads'
import GltfViewer from '@/components/GltfViewer'

export default async function ItemDetailPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = await params
  const supabase = await createClient()

  const { data: item } = await supabase
    .from('items')
    .select('*, profiles(username), item_images(id, storage_path, position)')
    .eq('id', id)
    .single()

  if (!item) {
    notFound()
  }

  const modelUrl = item.model_file_path ? storagePublicUrl('models', item.model_file_path) : null
  const showViewer = modelUrl && isGltfFile(item.model_file_path)
  const images = (item.item_images ?? []).slice().sort((a, b) => a.position - b.position)

  return (
    <div className="mx-auto max-w-5xl px-6 py-8">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-100">{item.title}</h1>
        <div className="mt-1 flex items-center gap-3 text-sm text-gray-400">
          <span className="capitalize">{item.category}</span>
          {item.trade && <span>· {item.trade}</span>}
          <span>
            by{' '}
            <Link href={`/profile/${item.profiles?.username}`} className="text-blue-400 hover:underline">
              {item.profiles?.username ?? 'unknown'}
            </Link>
          </span>
        </div>
      </div>

      {showViewer ? (
        <div className="mb-6 h-[480px] w-full overflow-hidden rounded-lg border border-gray-700">
          <GltfViewer url={modelUrl} />
        </div>
      ) : item.cover_image_path ? (
        // eslint-disable-next-line @next/next/no-img-element
        <img
          src={storagePublicUrl('images', item.cover_image_path)}
          alt={item.title}
          className="mb-6 max-h-[480px] w-full rounded-lg border border-gray-700 object-contain bg-gray-900"
        />
      ) : null}

      {item.description && (
        <p className="mb-6 whitespace-pre-wrap text-sm text-gray-300">{item.description}</p>
      )}

      {modelUrl && (
        <a
          href={modelUrl}
          download
          className="inline-block rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500"
        >
          Download model file{item.model_file_type ? ` (.${item.model_file_type})` : ''}
        </a>
      )}
      {!showViewer && modelUrl && (
        <p className="mt-2 text-xs text-gray-500">
          In-browser preview is only available for glTF/GLB files — download to view this one.
        </p>
      )}

      {images.length > 0 && (
        <div className="mt-8 grid grid-cols-3 gap-3 sm:grid-cols-4">
          {images.map((img) => (
            // eslint-disable-next-line @next/next/no-img-element
            <img
              key={img.id}
              src={storagePublicUrl('images', img.storage_path)}
              alt=""
              className="aspect-square w-full rounded border border-gray-700 object-cover"
            />
          ))}
        </div>
      )}
    </div>
  )
}
