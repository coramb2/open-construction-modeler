// Hand-written to match supabase/migrations/0001_init.sql. If the schema
// changes, update this file in the same commit — there's no live Supabase
// project wired up yet to generate these automatically via the CLI.

// `type`, not `interface` — Database['public']['Tables'][x]['Row'] must be
// structurally assignable to Record<string, unknown> (postgrest-js's
// GenericTable constraint) for query result types to resolve at all instead
// of silently collapsing to `never`. Interfaces aren't assignable to
// Record<string, unknown> without an explicit index signature; type object
// literals are. This bit us once already — keep these as `type`.
export type Profile = {
  id: string
  username: string
  display_name: string | null
  avatar_url: string | null
  bio: string | null
  created_at: string
}

export type ItemCategory = 'project' | 'item'

export type Item = {
  id: string
  owner_id: string
  title: string
  description: string | null
  category: ItemCategory
  trade: string | null
  model_file_path: string | null
  model_file_type: string | null
  cover_image_path: string | null
  published: boolean
  created_at: string
  updated_at: string
}

export type ItemImage = {
  id: string
  item_id: string
  storage_path: string
  position: number
  created_at: string
}

export type Database = {
  public: {
    Tables: {
      profiles: {
        Row: Profile
        Insert: Partial<Profile> & Pick<Profile, 'id' | 'username'>
        Update: Partial<Profile>
        Relationships: []
      }
      items: {
        Row: Item
        Insert: Partial<Item> & Pick<Item, 'owner_id' | 'title' | 'category'>
        Update: Partial<Item>
        Relationships: [
          {
            foreignKeyName: 'items_owner_id_fkey'
            columns: ['owner_id']
            referencedRelation: 'profiles'
            referencedColumns: ['id']
          },
        ]
      }
      item_images: {
        Row: ItemImage
        Insert: Partial<ItemImage> & Pick<ItemImage, 'item_id' | 'storage_path'>
        Update: Partial<ItemImage>
        Relationships: [
          {
            foreignKeyName: 'item_images_item_id_fkey'
            columns: ['item_id']
            referencedRelation: 'items'
            referencedColumns: ['id']
          },
        ]
      }
    }
    Views: Record<string, never>
    Functions: Record<string, never>
  }
}
