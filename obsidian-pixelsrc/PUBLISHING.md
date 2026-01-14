# Publishing to Obsidian Community Plugins

## Prerequisites

1. Plugin has been tested in a fresh vault
2. All files are present: `main.js`, `manifest.json`, `styles.css`
3. GitHub repository is public
4. A release has been created with the required files

## Release Workflow

The plugin uses GitHub Actions for automated releases:

1. Tag a release:
   ```bash
   git tag 0.1.0
   git push origin 0.1.0
   ```

2. The workflow at `.github/workflows/release.yml` will:
   - Build the WASM package
   - Build the plugin
   - Create a GitHub release with `main.js`, `manifest.json`, `styles.css`

## Community Plugin Submission

To submit to the Obsidian Community Plugins:

1. Fork https://github.com/obsidianmd/obsidian-releases

2. Edit `community-plugins.json` to add your plugin entry:
   ```json
   {
     "id": "pixelsrc",
     "name": "PixelSrc",
     "author": "PixelSrc",
     "description": "Render pixelsrc pixel art code blocks in your notes",
     "repo": "pixelsrc/obsidian-pixelsrc"
   }
   ```

3. Submit a pull request with:
   - Title: `Add PixelSrc plugin`
   - Description including:
     - Brief description of what the plugin does
     - Link to repository
     - Confirmation that you've tested the plugin

## Required Files for Release

| File | Description | Size |
|------|-------------|------|
| `main.js` | Bundled plugin code with WASM | ~390KB |
| `manifest.json` | Plugin metadata | <1KB |
| `styles.css` | Plugin styles | ~2KB |

## Testing Checklist

Before submitting:

- [ ] Plugin loads without errors
- [ ] Code blocks with `pixelsrc` language render sprites
- [ ] Code blocks with `pxl` language render sprites
- [ ] Live preview shows sprites while editing
- [ ] Settings tab appears and works
- [ ] Right-click "Copy as PNG" copies the sprite
- [ ] Scale setting affects sprite size
- [ ] Transparency background setting works

## Manual Testing Steps

1. Copy `main.js`, `manifest.json`, `styles.css` to `.obsidian/plugins/pixelsrc/`
2. Reload Obsidian
3. Enable the plugin in Settings > Community Plugins
4. Create a note with a pixelsrc code block:
   ~~~markdown
   ```pixelsrc
   {"type":"sprite","name":"test","palette":{"{x}":"#FF0000"},"grid":["{x}"]}
   ```
   ~~~
5. Verify the sprite renders in both reading mode and Live Preview
