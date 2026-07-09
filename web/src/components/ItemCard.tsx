import Link from 'next/link'
import type { Item } from '@/lib/database.types'
import { storagePublicUrl } from '@/lib/storage-url'

export type ItemWithOwner = Item & { owner_username: string }

export default function ItemCard({ item }: { item: ItemWithOwner }) {
  return (
    <Link
      href={`/items/${item.id}`}
      className="block overflow-hidden rounded-lg border border-gray-700 bg-gray-800 transition-colors hover:border-gray-600"
    >
      <div className="aspect-square w-full bg-gray-900">
        {item.cover_image_path ? (
          // User-uploaded Supabase storage URLs aren't in next.config's image domains.
          // eslint-disable-next-line @next/next/no-img-element
          <img
            src={storagePublicUrl('images', item.cover_image_path)}
            alt={item.title}
            className="h-full w-full object-cover"
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center text-xs text-gray-600">
            No image
          </div>
        )}
      </div>
      <div className="p-3">
        <div className="truncate text-sm font-medium text-gray-100">{item.title}</div>
        <div className="mt-1 flex items-center justify-between text-xs text-gray-500">
          <span className="capitalize">{item.category}</span>
          <span>{item.owner_username}</span>
        </div>
      </div>
    </Link>
  )
}
