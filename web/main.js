const loadingPopup = document.querySelector(".loading-popup");
const progressLine = loadingPopup.querySelector("#progress-line");
const progressBytes = loadingPopup.querySelector("#progress-bytes");

function handleProgress(loadedBytes, totalBytes) {
	const LINE_LEN = 30;

	const progress = loadedBytes / totalBytes;

	// Progress line
	const filled = Math.floor(LINE_LEN * progress);
	progressLine.textContent = "#".repeat(filled) + "-".repeat(LINE_LEN - filled);

	// Progress bytes
	const loadedMb = (loadedBytes / 1024 / 1024).toFixed(2);
	const totalMb = (totalBytes / 1024 / 1024).toFixed(2);
	progressBytes.textContent = `${loadedMb}M / ${totalMb}M`;

	if (progress >= 1) {
		loadingPopup.classList.add("hidden");
	}
}

async function load(wasmPath) {
	if (typeof WebAssembly.compileStreaming !== 'function') {
		// TODO: show error in ui
		console.error("No `WebAssembly.compileStreaming` method found. May be your browser is ass??");
		return;
	}

	// Thanks to https://stackoverflow.com/a/65529994
	const response = await fetch(wasmPath);

	const contentLen = response.headers.get("Content-Length");
	const totalBytes = parseInt(contentLen);
	let loadedBytes = 0;

	const opts = {
		async start(controller) {
			const reader = response.body.getReader();
			while (true) {
				const { done, value } = await reader.read();

				if (done) {
					handleProgress(totalBytes, totalBytes);
					break;
				}

				loadedBytes += value.byteLength;
				handleProgress(loadedBytes, totalBytes);

				controller.enqueue(value);
			}

			controller.close();
		}
	};
	const res = new Response(new ReadableStream(opts, {
		status: response.status,
		statusText: response.statusText,
	}));

	// Copy headers
	for (const pair of response.headers.entries()) {
		res.headers.set(pair[0], pair[1]);
	}

	register_plugins(plugins);

	WebAssembly.compileStreaming(res)
		.then(obj => {
			add_missing_functions_stabs(obj);
			return WebAssembly.instantiate(obj, importObject);
		})
		.then(obj => {
			wasm_memory = obj.exports.memory;
			wasm_exports = obj.exports;

			const crate_version = wasm_exports.crate_version();
			if (version != crate_version) {
				console.error(
					`Version mismatch: gl.js version is: ${version},`,
					`miniquad crate version is: ${crate_version}`
				);
			}

			init_plugins(plugins);
			obj.exports.main();
		})
		.catch(err => {
			// TODO: show error in ui
			console.error(err);
		})
}

load("chip-maximator.wasm");
