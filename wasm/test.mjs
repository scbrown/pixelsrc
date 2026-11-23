import { readFileSync } from 'fs';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Dynamic import for WASM
const { default: init, render_to_png, render_to_rgba, render_named_to_rgba, Project, list_sprites, validate } = await import('./pkg/pixelsrc.js');

// Load WASM file manually for Node.js
const wasmPath = join(__dirname, 'pkg', 'pixelsrc_bg.wasm');
const wasmBytes = readFileSync(wasmPath);
await init({ module_or_path: wasmBytes });

console.log('Running @stiwi/pixelsrc-wasm tests...\n');

// Test 1: Minimal sprite
const minimalSprite = '{"type":"sprite","name":"dot","size":[1,1],"palette":{"x":"#FF0000"},"regions":{"x":{"points":[[0,0]]}}}';

const png = render_to_png(minimalSprite);
assert.ok(png.length > 0, 'PNG should have content');
assert.ok(png[0] === 0x89 && png[1] === 0x50 && png[2] === 0x4E && png[3] === 0x47,
  'PNG should start with magic bytes');
console.log('✓ render_to_png produces valid PNG');

// Test 2: RGBA output
const rgba = render_to_rgba(minimalSprite);
assert.ok(rgba.width === 1, `Width should be 1, got ${rgba.width}`);
assert.ok(rgba.height === 1, `Height should be 1, got ${rgba.height}`);
assert.ok(rgba.pixels.length === 4, `Should have 4 bytes (RGBA), got ${rgba.pixels.length}`);
assert.ok(rgba.pixels[0] === 255, 'Red channel should be 255');
assert.ok(rgba.pixels[1] === 0, 'Green channel should be 0');
assert.ok(rgba.pixels[2] === 0, 'Blue channel should be 0');
assert.ok(rgba.pixels[3] === 255, 'Alpha channel should be 255');
console.log('✓ render_to_rgba returns correct dimensions and pixels');

// Test 3: List sprites
const multiSprite = `{"type":"sprite","name":"one","size":[1,1],"palette":{"x":"#FF0000"},"regions":{"x":{"points":[[0,0]]}}}
{"type":"sprite","name":"two","size":[1,1],"palette":{"x":"#00FF00"},"regions":{"x":{"points":[[0,0]]}}}`;
const names = list_sprites(multiSprite);
assert.ok(names.length === 2, `Should have 2 sprites, got ${names.length}`);
assert.ok(names.includes('one'), 'Should include "one"');
assert.ok(names.includes('two'), 'Should include "two"');
console.log('✓ list_sprites returns sprite names');

// Test 4: Validate
const invalid = '{"type":"sprite","name":"bad"';
const messages = validate(invalid);
assert.ok(messages.length > 0, 'Should have validation messages');
console.log('✓ validate catches errors');

// Test 5: Warnings
const withWarning = '{"type":"sprite","name":"warn","palette":{"x":"#FF0000"},"size":[2,1],"regions":{"missing":{"points":[[0,0]]}}}';
const warnResult = render_to_rgba(withWarning);
assert.ok(warnResult.warnings.length > 0, 'Should have warnings for an unknown region token');
console.log('✓ Warnings are captured');

console.log('\n✅ All tests passed!');

// Loaded multi-source projects keep parsed registries across named renders.
const sharedSource = `{type:"palette",name:"p",colors:{x:"#ff0000"}}
{type:"sprite",name:"dot",size:[2,2],palette:"p",regions:{x:{points:[[0,0]]}}}`;
const figureSource = `{type:"composition",name:"figure",size:[4,2],cell_size:[2,2],sprites:{D:"dot"},layers:[{map:["DD"]}]}`;
const sourceFiles = [{path:'shared.pxl',text:sharedSource},{path:'figures.pxl',text:figureSource}];
const project = Project.load(JSON.stringify(sourceFiles));
assert.equal(project.source_count, 2);
const reference = render_named_to_rgba(sourceFiles.map(source => source.text).join('\n'), 'figure');
for (let i=0;i<3;i++) {
  const image=project.render('figure');
  assert.deepEqual([image.width,image.height],[4,2]);
  assert.deepEqual(image.pixels,reference.pixels);
  assert.deepEqual(image.warnings,[]);
  image.free();
}
assert.ok(project.list_renderables().includes('composition:figure'));
const included = Project.load(JSON.stringify([
  {path:'shared.pxl',text:sharedSource.split('\n')[0]},
  {path:'scenes/figure.pxl',text:sharedSource.split('\n')[1].replace('palette:"p"','palette:"@include:../shared.pxl#p"')}
]));
const includedImage = included.render('dot');
assert.deepEqual(Array.from(includedImage.pixels.slice(0,4)),[255,0,0,255]);
let missingInclude;
try { Project.load(JSON.stringify([{path:'bad.pxl',text:sharedSource.split('\n')[1].replace('palette:"p"','palette:"@include:missing.pxl"')}])); }
catch(error) { missingInclude=String(error); }
assert.match(missingInclude ?? '',/not a supplied source file/);
includedImage.free(); included.free(); reference.free(); project.free();
console.log('✓ Loaded multi-source Project renders shared named figures and resolves/diagnoses includes');
