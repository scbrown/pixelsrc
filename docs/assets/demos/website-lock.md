# Clean website install

Replay: `scriptreplay --log-timing website-lock.time --log-out website-lock.log`.

Before: saved actual `npm ci` failure with release-base d6382dc manifest/lock files, reporting local WASM package 0.1.0 versus 0.2.0. After: actual clean install and production TypeScript/Vite build with the synchronized local dependency entry. No external dependency versions change. Uses a generated WASM module; this verifies installation and build rather than published-site behavior. Both probes ran under the HOME escape guard.
