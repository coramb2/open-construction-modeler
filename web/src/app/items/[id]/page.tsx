import Link from 'next/link'
import { notFound } from 'next/navigation'
import { createClient } from '@/lib/supabase/server'
import { storagePublicUrl } from '@/lib/storage-url'
import { isGltfFile } from '@/lib/uploads'
import GltfViewer from '@/components/GltfViewer'
import IfcModelInfo from '@/components/IfcModelInfo'
import AlignmentReportCard from '@/components/AlignmentReportCard'
import ForkButton from '@/components/ForkButton'

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

  const { data: claims } = await supabase.auth.getClaims()
  const signedIn = Boolean(claims?.claims?.sub)

  // Lineage: the item this was forked from (if any), and how many forks it has.
  let forkSource: { id: string; title: string; username: string | null } | null = null
  if (item.forked_from) {
    const { data: src } = await supabase
      .from('items')
      .select('id, title, profiles(username)')
      .eq('id', item.forked_from)
      .single()
    if (src) {
      forkSource = { id: src.id, title: src.title, username: src.profiles?.username ?? null }
    }
  }

  const { count: forkCount } = await supabase
    .from('items')
    .select('id', { count: 'exact', head: true })
    .eq('forked_from', id)

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

        {forkSource && (
          <div className="mt-1 text-xs text-gray-500">
            <span aria-hidden>⑂</span> forked from{' '}
            <Link href={`/items/${forkSource.id}`} className="text-blue-400 hover:underline">
              {forkSource.title}
            </Link>
            {forkSource.username && <> by {forkSource.username}</>}
          </div>
        )}

        <div className="mt-3 flex items-center gap-3">
          {signedIn && <ForkButton itemId={item.id} />}
          {(forkCount ?? 0) > 0 && (
            <span className="text-xs text-gray-500">
              {forkCount} fork{forkCount === 1 ? '' : 's'}
            </span>
          )}
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

      {modelUrl && item.model_file_type === 'ifc' && (
        <div className="mt-4 space-y-4">
          <IfcModelInfo modelUrl={modelUrl} />
          <AlignmentReportCard modelUrl={modelUrl} />
        </div>
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
