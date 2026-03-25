/// <reference types="vitest/config" />
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

export default defineConfig({
	plugins: [svelte()],
	server: {
		proxy: {
			"/events": "http://localhost:8080",
			"/command": "http://localhost:8080",
		},
	},
	test: {
		environment: "jsdom",
	},
});
