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

> **Note:** Unlike crates.io, npm does NOT support fully tokenless OIDC publishing.
> The `--provenance` flag adds attestations but doesn't replace authentication.
> A granular access token is still required.

**Setup on npmjs.com:**
1. Go to https://www.npmjs.com/ → Avatar → Access Tokens
2. Generate New Token → **Granular Access Token**
3. Configure:
   - Token name: `github-actions-pixelsrc`
   - Expiration: No expiration
   - Packages: Only select packages → `@stiwi/pixelsrc-wasm`
   - Permissions: Read and write
   - Organizations: No access
4. Copy the token

**Setup on GitHub:**
1. Go to repo Settings → Secrets → Actions
2. Add secret: `NPM_TOKEN` with the token value

**Workflow requirements:**
- Permission: `id-token: write` (for provenance attestations)
- Secret: `NPM_TOKEN` (for authentication)
- Flag: `npm publish --provenance` (adds attestations)
- Use `registry-url` in `setup-node` to configure .npmrc

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
