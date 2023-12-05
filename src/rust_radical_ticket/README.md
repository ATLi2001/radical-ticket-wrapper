To build:

```
wasm-pack build
```

Then patch JS ([reference](https://developers.cloudflare.com/workers/runtime-apis/webassembly/rust/#javascript-plumbing-wasm-bindgen))

```
import * as imports from './worker_rust_bg.js';

// switch between both syntax for node and for workerd
import wkmod from './worker_rust_bg.wasm';
import * as nodemod from './worker_rust_bg.wasm';
if (typeof process !== 'undefined' && process.release.name === 'node') {
	imports.__wbg_set_wasm(nodemod);
} else {
	const instance = new WebAssembly.Instance(wkmod, { './worker_rust_bg.js': imports });
	imports.__wbg_set_wasm(instance.exports);
}

export * from './worker_rust_bg.js';

```
