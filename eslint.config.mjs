// import js from "@eslint/js";
import globals from "globals";
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: [
      "node_modules/",
      "dist/",
      "build/",
      "coverage/",
      "tmp/",
      "temp/",
      "*.log",
      ".vscode/",
      ".idea/",
      "*.swp",
      "*.swo",
      ".DS_Store",
      "Thumbs.db",
    ]
  },
  // js.configs.recommended,
  ...tseslint.configs.recommended,
  ...tseslint.configs.strict,
  ...tseslint.configs.stylistic,
  tseslint.configs.eslintRecommended,
  {
    files: ["**/*.{js,ts,tsx,mjs,cjs}"],
    languageOptions: {
      globals: {
        ...globals.browser,
        Dexie: "readonly"
      },
      ecmaVersion: 2022,
      sourceType: "module"
    },
    // linterOptions: {
    //   noWarnIgnored: true
    // },
    rules: {
      'no-undef': 'error', // TypeScript handles this
      '@typescript-eslint/no-unused-vars': ['error', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_'
      }],
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-var-requires': 'off',
      '@eslint/no-unused-vars': 'off'
    }
  },
  {
    files: ["**/*.js", "**/*.mjs", "**/*.cjs"],
    rules: {
      '@typescript-eslint/no-var-requires': 'off'
    }
  }
);
