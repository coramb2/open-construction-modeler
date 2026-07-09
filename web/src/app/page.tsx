import { createClient } from '@/lib/supabase/server'
import ItemCard, { type ItemWithOwner } from '@/components/ItemCard'

const FEED_PAGE_SIZE = 24

export default async function Home() {
  const supabase = await createClient()
  const { data, error } = await supabase
    .from('items')
    .select('*, profiles(username)')
    .eq('published', true)
    .order('created_at', { ascending: false })
    .limit(FEED_PAGE_SIZE)

  const items: ItemWithOwner[] =
    data?.map((row) => {
      const { profiles, ...item } = row
      return { ...item, owner_username: profiles?.username ?? 'unknown' }
    }) ?? []

  return (
    <div className="mx-auto max-w-6xl px-6 py-10">
      <div className="mb-8 text-center">
        <h1 className="text-3xl font-bold text-blue-400">
          Open Construction Modeler
        </h1>
        <p className="mx-auto mt-3 max-w-2xl text-gray-400">
          A place to publish and browse construction models — from finished
          projects down to individual made items. Think GitHub, for things
          that get built.
        </p>
      </div>

      {error && (
        <p className="text-center text-sm text-red-400">
          Couldn&apos;t load items: {error.message}
        </p>
      )}

      {!error && items.length === 0 && (
        <p className="text-center text-sm text-gray-500">
          Nothing published yet — be the first.
        </p>
      )}

      {items.length > 0 && (
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
          {items.map((item) => (
            <ItemCard key={item.id} item={item} />
          ))}
        </div>
      )}
    </div>
  )
}
