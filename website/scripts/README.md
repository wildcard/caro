# Website Build Scripts

## extract-version.mjs

This script automatically extracts the version number from the root `Cargo.toml` file and generates a TypeScript file that can be imported by the website.

### How it works

1. Reads `../Cargo.toml` (relative to the website directory)
2. Extracts the `version = "x.y.z"` line using regex
3. Generates `src/config/version.ts` with the extracted version
4. This file is imported by `src/config/site.ts`

### When it runs

The script runs automatically:
- Before `npm run build` (via `prebuild` script)
- Before `npm run dev` (via `predev` script)

This ensures the website always displays the correct version from Cargo.toml without manual updates.

### Generated file

The generated `src/config/version.ts` file is:
- Auto-generated (marked with a comment)
- Git-ignored (in `.gitignore`)
- Should never be edited manually

### Troubleshooting

If you see a version mismatch:
1. Delete `website/src/config/version.ts`
2. Run `npm run build` or `npm run dev`
3. The script will regenerate the file with the correct version
