import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@botglue/common": path.resolve(__dirname, "../ui-common"),
    },
  },
});
