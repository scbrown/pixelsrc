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

npm supports OIDC trusted publishing (GA since July 2025). Classic tokens were deprecated December 2025.

**Setup on npmjs.com:**
1. Go to https://www.npmjs.com/package/@stiwi/pixelsrc-wasm/access
2. Under "Trusted Publishers", click "Add a trusted publisher"
3. Select "GitHub Actions" and configure:
   - Repository owner: `scbrown` (case-sensitive!)
   - Repository name: `pixelsrc`
   - Workflow filename: `wasm.yml`
   - Environment: (leave blank)

**Important:** The `repository.url` in package.json must exactly match. Use the normalized format:
```json
"repository": {
  "type": "git",
  "url": "git+https://github.com/scbrown/pixelsrc.git"
}
```

**Workflow requirements:**
- Permission: `id-token: write`
- Node.js 22+ and run `npm install -g npm@latest` for OIDC support
- Use `--provenance` flag (triggers OIDC authentication)
- Do NOT use `registry-url` in `setup-node` (creates .npmrc expecting tokens)
- Do NOT set `NODE_AUTH_TOKEN` (OIDC only works when no token is present)

## Troubleshooting

### npm 404 errors with OIDC

A 404 usually means npm couldn't match your workflow to the Trusted Publisher config:

1. **Case sensitivity**: Repository owner must match exactly (check for capital letters)
2. **repository.url mismatch**: Must be `git+https://github.com/owner/repo.git` format
3. **Stale token interfering**: Delete any `NPM_TOKEN` secret from the repo
4. **registry-url in setup-node**: Remove it - creates .npmrc expecting tokens
5. **Workflow filename**: Must match exactly what's configured on npm

### "ENEEDAUTH" error

This means no authentication was provided. For OIDC:
- Ensure `id-token: write` permission is set
- Ensure NO token/secret is being used (OIDC won't activate if a token exists)

## Important Notes

- **No secrets needed**: Do not add `NPM_TOKEN`, `CARGO_REGISTRY_TOKEN`, or similar secrets
- **First publish**: May need manual publishing to create the package initially
- **OIDC detection**: npm CLI auto-detects OIDC and uses it before falling back to tokens
- **Cloud runners only**: OIDC trusted publishing only works on cloud-hosted runners (not self-hosted)
