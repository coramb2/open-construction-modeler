import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// Lets the app boot (and show a clear message) before Supabase env vars are
// configured, instead of every server component throwing on `!` assertions.
export function computeHasEnvVars(url: string | undefined, key: string | undefined): boolean {
  return !!url && !!key
}

export const hasEnvVars = computeHasEnvVars(
  process.env.NEXT_PUBLIC_SUPABASE_URL,
  process.env.NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY,
)
