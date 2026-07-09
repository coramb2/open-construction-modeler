import { notFound } from 'next/navigation'
import { createClient } from '@/lib/supabase/server'
import ItemCard, { type ItemWithOwner } from '@/components/ItemCard'

export default async function ProfilePage({
  params,
}: {
  params: Promise<{ username: string }>
}) {
  const { username } = await params
  const supabase = await createClient()

  const { data: profile } = await supabase
    .from('profiles')
    .select('*')
    .eq('username', username)
    .single()

  if (!profile) {
    notFound()
  }

  const { data } = await supabase
    .from('items')
    .select('*, profiles(username)')
    .eq('owner_id', profile.id)
    .eq('published', true)
    .order('created_at', { ascending: false })

  const items: ItemWithOwner[] =
    data?.map((row) => {
      const { profiles, ...item } = row
      return { ...item, owner_username: profiles?.username ?? username }
    }) ?? []

  return (
    <div className="mx-auto max-w-6xl px-6 py-8">
      <div className="mb-8 flex items-center gap-4">
        {profile.avatar_url && (
          // eslint-disable-next-line @next/next/no-img-element
          <img
            src={profile.avatar_url}
            alt={profile.username}
            className="h-16 w-16 rounded-full border border-gray-700"
          />
        )}
        <div>
          <h1 className="text-xl font-bold text-gray-100">
            {profile.display_name || profile.username}
          </h1>
          <p className="text-sm text-gray-500">@{profile.username}</p>
          {profile.bio && <p className="mt-1 text-sm text-gray-400">{profile.bio}</p>}
        </div>
      </div>

      {items.length === 0 ? (
        <p className="text-sm text-gray-500">No published items yet.</p>
      ) : (
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
          {items.map((item) => (
            <ItemCard key={item.id} item={item} />
          ))}
        </div>
      )}
    </div>
  )
}
