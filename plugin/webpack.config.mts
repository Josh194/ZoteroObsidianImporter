import webpack from "webpack";

import ESLintPlugin from "eslint-webpack-plugin";
import ZoteroPackPlugin from "./build/pack.mjs";

import { config as eslint_config, files as eslint_files } from "./build/eslint.mjs";

import path from "path";

import { fileURLToPath } from "url";
import { dirname } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export default async function config(): Promise<webpack.Configuration> {
	return {
		entry: {
			bootstrap: ["./src/entry.ts"]
		},
		plugins: [
			new ESLintPlugin({
				files: eslint_files, // ? Bug in eslint-webpack-config? Nothing is linted if this is not included.
				configType: "flat",
				overrideConfig: await eslint_config(), // Plugin has trouble resolving files when using a normal ESLint config setup, and this also gives validation over the config file.
				overrideConfigFile: true
			}),
			new ZoteroPackPlugin(
				"zo_importer",
				path.resolve(__dirname, "./src/manifest.json")
			)
		],
		module: {
			rules: [
				{
					test: /\.([cm]?ts|tsx)$/, // Covers `.ts`, `.cts`, `.mts`, and `.tsx`
					exclude: /node_modules/, // Is this useful?
					use: "ts-loader"
				}
			]
		},
		resolve: {
			extensions: [".tsx", ".ts", ".js"],
			extensionAlias: {
				".js": [".js", ".ts"],
				".cjs": [".cjs", ".cts"],
				".mjs": [".mjs", ".mts"]
			},
			alias: {}
		},
		output: {
			library: {
				name: "globalThis",
				type: "assign-properties"
			},
			filename: "[name].js",
			path: path.resolve(__dirname, "./dist"),
			clean: true
		},
		cache: {
			type: "filesystem"
		},
	}
};