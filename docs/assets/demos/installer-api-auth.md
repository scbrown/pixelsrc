# Installer release API authentication

[Terminal recording](installer-api-auth.log) · [Replay timing](installer-api-auth.time)

Before: two synthetic-token controls fail on the actual embedded installer. After: all 21 installer fixtures and both header-boundary controls pass. Only release metadata requests to `api.github.com` use an optional `GITHUB_TOKEN`; asset requests and redirects to asset hosts omit Authorization. Without a token, requests remain anonymous.

The redirect control injects a synthetic header to test the boundary. The previous anonymous installer did not send a token in production. These are offline transport controls; actual hosted release downloads remain a separate CI gate.
