import { fileURLToPath } from "node:url";
import { createJiti } from "jiti";
import type { NextConfig } from "next";
const jiti = createJiti(fileURLToPath(import.meta.url));

jiti.esmResolve("./src/env.ts");

const nextConfig: NextConfig = {
  eslint: {
    ignoreDuringBuilds: true,
  },
};

export default nextConfig;
