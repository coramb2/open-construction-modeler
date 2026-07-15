import type { NextConfig } from "next";
import { securityHeaders } from "./src/lib/security-headers";

const nextConfig: NextConfig = {
  // Apply the security headers to every route.
  async headers() {
    return [
      {
        source: "/:path*",
        headers: securityHeaders,
      },
    ];
  },
  webpack: (config) => {
    // The wasm-pack glue (src/wasm/ocm) loads the engine via
    // `new URL('ocm_wasm_bg.wasm', import.meta.url)`. Emit that .wasm as a
    // served asset so the URL resolves at runtime in the browser bundle.
    config.module.rules.push({
      test: /ocm_wasm_bg\.wasm$/,
      type: "asset/resource",
    });
    return config;
  },
};

export default nextConfig;
