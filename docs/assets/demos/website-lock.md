# Clean website install across npm versions

Replay: `scriptreplay --log-timing website-lock.time --log-out website-lock.log`.

Before: saved actual `npm ci` failure with release-base d6382dc manifest/lock files, reporting local WASM package 0.1.0 versus 0.2.0. After: actual clean install and production TypeScript/Vite builds with npm9.2.0 and npm10.8.2, preserving the original local-package symlink and synchronizing its version. The project explicitly sets `install-links=false`, so installation does not depend on each npm version's default. No external dependency versions change. Uses a generated WASM module; this verifies installation/build rather than published-site behavior. Probes ran under the HOME escape guard.

An earlier copy-style candidate passed local npm9 but hosted npm10 rejected it as missing the local package entry. That hosted failure prompted the explicit link configuration and the two-version verification above. See [npm install-links](https://docs.npmjs.com/cli/v10/npm-install/#install-links) for the package-versus-link behavior.
