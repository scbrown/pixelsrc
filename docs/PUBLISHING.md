# Package Publishing

This project uses **OIDC trusted publishing** for all package registries. No tokens or secrets are stored in the repository.

## How It Works

Instead of storing API tokens as GitHub secrets, we use OpenID Connect (OIDC) to establish trust between GitHub Actions and package registries. The registry verifies that the publish request comes from an authorized GitHub Actions workflow.

## Configured Registries

### crates.io (Rust)

**Workflow:** `.github/workflows/crates.yml`

**Setup on crates.io:**
1. Go to https://crates.io/settings/tokens
2. Under "Trusted Publishers", add:
   - Repository: `scbrown/pixelsrc`
   - Workflow: `crates.yml`

**Workflow requirements:**
- Permission: `id-token: write`
- Uses: `rust-lang/crates-io-auth-action@v1`

### npm (WASM)

**Workflow:** `.github/workflows/wasm.yml`

**Setup on npmjs.com:**
1. Go to https://www.npmjs.com/package/@stiwi/pixelsrc-wasm/access
2. Under "Publishing access", click "Add a trusted publisher"
3. Select "GitHub Actions" and configure:
   - Repository owner: `scbrown`
   - Repository name: `pixelsrc`
   - Workflow filename: `wasm.yml`

**Workflow requirements:**
- Permission: `id-token: write`
- Flag: `npm publish --provenance`
- Do NOT use `registry-url` in `setup-node` (it creates an .npmrc expecting tokens)

## Troubleshooting

### "Access token expired or revoked"

This error usually means:
1. A stale `NPM_TOKEN` secret exists in the repo - delete it
2. The `setup-node` action has `registry-url` set - remove it
3. Trusted publishing isn't configured on the registry

### "Not found" or 404 errors

The package/crate may need to be published manually the first time, or trusted publishing isn't configured correctly on the registry side.

## Important Notes

- **No secrets needed**: Do not add `NPM_TOKEN`, `CARGO_REGISTRY_TOKEN`, or similar secrets
- **First publish**: The first version may need manual publishing to create the package
- **Provenance**: The `--provenance` flag adds attestations proving the package came from this repo
