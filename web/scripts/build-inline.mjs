#!/usr/bin/env node
// Builds pkg-inline/, the published npm package, from the wasm-pack output in pkg/.
//
// The entry point holds the WebAssembly binary as base64 and calls `initSync`
// when it loads, so a caller needs no init call and no bundler plugin. The
// version comes from the root hifitime Cargo.toml.

import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = join(dirname(fileURLToPath(import.meta.url)), '..');
const source = join(web, 'pkg');
const target = join(web, 'pkg-inline');

const rootManifest = readFileSync(join(web, '..', 'Cargo.toml'), 'utf8');
const packageSection = rootManifest.split(/^\[/m).find((part) => part.startsWith('package]'));
const version = packageSection?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (!version) {
  throw new Error('no [package] version in the root Cargo.toml');
}

rmSync(target, { recursive: true, force: true });
mkdirSync(target);
writeFileSync(join(target, '.gitignore'), '*\n');
copyFileSync(join(source, 'hifitime.js'), join(target, 'hifitime.js'));
copyFileSync(join(source, 'hifitime.d.ts'), join(target, 'hifitime.d.ts'));
copyFileSync(join(web, '..', 'LICENSE.txt'), join(target, 'LICENSE.txt'));
copyFileSync(join(web, 'README.md'), join(target, 'README.md'));

const base64 = readFileSync(join(source, 'hifitime_bg.wasm')).toString('base64');
writeFileSync(
  join(target, 'hifitime-inline.js'),
  `/* @ts-self-types="./hifitime-inline.d.ts" */
import { initSync } from './hifitime.js';

const bytes = Uint8Array.from(atob('${base64}'), (c) => c.charCodeAt(0));
initSync({ module: bytes });

export * from './hifitime.js';
`,
);
writeFileSync(join(target, 'hifitime-inline.d.ts'), "export * from './hifitime.js';\n");

// wasm-pack copies the description, the keywords, and the links from web/Cargo.toml.
const generated = JSON.parse(readFileSync(join(source, 'package.json'), 'utf8'));
const manifest = {
  name: 'hifitime',
  version,
  description: generated.description,
  license: generated.license,
  homepage: generated.homepage,
  repository: { type: 'git', url: 'git+https://github.com/nyx-space/hifitime.git' },
  keywords: generated.keywords,
  type: 'module',
  main: 'hifitime-inline.js',
  types: 'hifitime-inline.d.ts',
  exports: {
    '.': { types: './hifitime-inline.d.ts', default: './hifitime-inline.js' },
  },
  files: ['hifitime-inline.js', 'hifitime-inline.d.ts', 'hifitime.js', 'hifitime.d.ts'],
  sideEffects: ['./hifitime-inline.js'],
};
writeFileSync(join(target, 'package.json'), `${JSON.stringify(manifest, null, 2)}\n`);

console.log(`pkg-inline: hifitime@${version}`);
