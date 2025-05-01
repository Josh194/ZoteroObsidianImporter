import webpack from 'webpack';
import path from 'path';

const config: webpack.Configuration = {
	entry: {
		bootstrap: ['./src/main.ts']
	},
	plugins: [],
	module: {
		rules: [
			{
				test: /\.([cm]?ts|tsx)$/, // Covers `.ts`, `.cts`, `.mts`, and `.tsx`
				exclude: /node_modules/, // Is this useful?
				use: 'ts-loader',
			}
		]
	},
	resolve: {
		extensions: ['.tsx', '.ts', '.js'],
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
		filename: '[name].js',
		path: path.resolve(__dirname, './build'),
		clean: true
	}
};

export default config;