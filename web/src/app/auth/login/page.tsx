import { Suspense } from 'react'
import LoginForm from './LoginForm'

export default function LoginPage() {
  return (
    <div className="flex min-h-screen w-full items-center justify-center bg-gray-900 p-6">
      <Suspense>
        <LoginForm />
      </Suspense>
    </div>
  )
}
