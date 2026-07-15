import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import { Analytics } from "@vercel/analytics/next";
import { SpeedInsights } from "@vercel/speed-insights/next";
import "./globals.css";
import Nav from "@/components/Nav";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Open Construction Modeler",
  description:
    "Browse and share construction models — from finished projects to individual made items.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col bg-gray-900 text-gray-100">
        <Nav />
        <main className="flex-1">{children}</main>
        {/* Vercel Web Analytics (visitors/page views) + Speed Insights (real-
            user Core Web Vitals). No-ops until enabled in the Vercel project
            dashboard; privacy-friendly (no cross-site cookies). */}
        <Analytics />
        <SpeedInsights />
      </body>
    </html>
  );
}
