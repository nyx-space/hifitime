# hifitime for JavaScript and TypeScript

WebAssembly bindings for [hifitime](https://github.com/nyx-space/hifitime), with
TypeScript declarations. Every method calls the hifitime Rust crate, so the
results match the Rust crate and the Python package.

```bash
npm install hifitime
```

```ts
import { Duration, Epoch, TimeScale } from 'hifitime';

const launch = new Epoch('2024-02-29T12:34:56.789 UTC');
launch.toGregorianString(TimeScale.GPST); // 2024-02-29T12:35:14.789000000 GPST
launch.add(Duration.fromHours(1.5)).durationSince(launch).toString(); // 1 h 30 min
```

The package holds the WebAssembly binary inline and loads it on import. It needs
no init call and no bundler plugin. It works in Node 18 and later, in a browser,
and with Angular, Vite, and webpack.

## Notes

* **Precision.** A JavaScript `number` holds 53 significant bits, so a method that
  returns a `number` can lose nanoseconds. Exact values are a `bigint`, for
  example `Duration.totalNanoseconds()` and `Epoch.toUnixNanoseconds()`.
  Arithmetic between epochs and durations is always exact.
* **Errors.** A method that fails throws an `Error` with the hifitime message.
* **Memory.** Each object owns WebAssembly memory. The garbage collector releases
  it. Call `free()` to release it earlier, for example in a loop.
* **Bundle size.** The binary adds about 150 kB with gzip. In Angular, this can
  exceed the default warning budget of 500 kB. Use a dynamic `import('hifitime')`
  or raise the budget in `angular.json`.
* **Absent features.** The `ut1` and `lts` features download files, so the
  package omits them. The IERS leap seconds are built in.

## Build and test

Run these commands in `web/`. They need `wasm-pack`, the `wasm32-unknown-unknown`
target, and Node 18 or later.

```bash
npm run build   # writes the npm package into pkg-inline/
npm test        # builds, then runs the JavaScript and the Rust tests
```

## Release

The npm version is the version in the root `Cargo.toml`. A tag runs
`.github/workflows/wasm.yml`, which publishes `pkg-inline/` to npm. The workflow
needs an npm automation token in the repository secret `NPM_TOKEN`.
