import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Dynamic import for WASM
const { default: init, render_to_png, render_to_rgba, list_sprites, validate } = await import('./pkg/pixelsrc.js');

// Load WASM file manually for Node.js
const wasmPath = join(__dirname, 'pkg', 'pixelsrc_bg.wasm');
const wasmBytes = readFileSync(wasmPath);
await init(wasmBytes);

console.log('Running @stiwi/pixelsrc-wasm tests...\n');

// Test 1: Minimal sprite
const minimalSprite = '{"type":"sprite","name":"dot","palette":{"{x}":"#FF0000"},"grid":["{x}"]}';

const png = render_to_png(minimalSprite);
console.assert(png.length > 0, 'PNG should have content');
console.assert(png[0] === 0x89 && png[1] === 0x50 && png[2] === 0x4E && png[3] === 0x47,
  'PNG should start with magic bytes');
console.log('✓ render_to_png produces valid PNG');

// Test 2: RGBA output
const rgba = render_to_rgba(minimalSprite);
console.assert(rgba.width === 1, `Width should be 1, got ${rgba.width}`);
console.assert(rgba.height === 1, `Height should be 1, got ${rgba.height}`);
console.assert(rgba.pixels.length === 4, `Should have 4 bytes (RGBA), got ${rgba.pixels.length}`);
console.assert(rgba.pixels[0] === 255, 'Red channel should be 255');
console.assert(rgba.pixels[1] === 0, 'Green channel should be 0');
console.assert(rgba.pixels[2] === 0, 'Blue channel should be 0');
console.assert(rgba.pixels[3] === 255, 'Alpha channel should be 255');
console.log('✓ render_to_rgba returns correct dimensions and pixels');

// Test 3: List sprites
const multiSprite = `{"type":"sprite","name":"one","palette":{"{x}":"#FF0000"},"grid":["{x}"]}
{"type":"sprite","name":"two","palette":{"{x}":"#00FF00"},"grid":["{x}"]}`;
const names = list_sprites(multiSprite);
console.assert(names.length === 2, `Should have 2 sprites, got ${names.length}`);
console.assert(names.includes('one'), 'Should include "one"');
console.assert(names.includes('two'), 'Should include "two"');
console.log('✓ list_sprites returns sprite names');

// Test 4: Validate
const invalid = '{"type":"sprite","name":"bad"';
const messages = validate(invalid);
console.assert(messages.length > 0, 'Should have validation messages');
console.log('✓ validate catches errors');

// Test 5: Warnings
const withWarning = '{"type":"sprite","name":"warn","palette":{"{x}":"#FF0000"},"size":[2,1],"grid":["{x}"]}';
const warnResult = render_to_rgba(withWarning);
console.assert(warnResult.warnings.length > 0, 'Should have warnings for short row');
console.log('✓ Warnings are captured');

console.log('\n✅ All tests passed!');
