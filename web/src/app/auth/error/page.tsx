export default async function AuthErrorPage({
  searchParams,
}: {
  searchParams: Promise<{ error?: string }>
}) {
  const params = await searchParams

  return (
    <div className="flex min-h-screen w-full items-center justify-center bg-gray-900 p-6 text-gray-100">
      <div className="w-full max-w-sm rounded-lg border border-gray-700 bg-gray-800 p-6">
        <h1 className="text-lg font-bold text-red-400">Sign-in failed</h1>
        <p className="mt-2 text-sm text-gray-400">
          {params?.error ?? 'An unspecified error occurred.'}
        </p>
        <a
          href="/auth/login"
          className="mt-4 inline-block rounded bg-blue-600 px-3 py-2 text-xs font-medium text-white hover:bg-blue-500"
        >
          Try again
        </a>
      </div>
    </div>
  )
}
