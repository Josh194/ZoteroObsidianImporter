import { type Linter } from "eslint";

import stylistic from "@stylistic/eslint-plugin";
import { createRequire } from "module";

export async function config(): Promise<Linter.Config<Linter.RulesRecord>[]> {
	const require = createRequire(import.meta.url);
	const parserTs: typeof import("@typescript-eslint/parser/dist/index") = require("@typescript-eslint/parser");

	return [
		{
			files: files,
			plugins: {
				"@stylistic": stylistic
			},
			languageOptions: {
				parser: parserTs,
				ecmaVersion: 2022,
				sourceType: "module"
			},
			rules: {
				"@stylistic/indent": ["error", "tab"],
				"@stylistic/quotes": ["error", "double"],
				"@stylistic/semi": ["error", "always"],
				"@stylistic/comma-dangle": ["error", "never"],
				"@stylistic/brace-style": ["error", "1tbs", { "allowSingleLine": true }],
				"@stylistic/block-spacing": ["error", "always"],
				"@stylistic/function-call-spacing": ["error", "never"],
				"@stylistic/semi-spacing": ["error", { "before": false, "after": true }],
				"@stylistic/no-trailing-spaces": ["error"],
				"@stylistic/array-bracket-spacing": ["error", "never"]
			}
		}
	];
}

export const files: string[] = [
	"./src/**/*.ts"
];