// The worker has its own scope and no direct access to functions/objects of the
// global scope. We import the generated JS file to make `wasm_bindgen`
// available which we need to initialize our Wasm code.
importScripts("./pkg/torii_client_wasm.js");

console.log("Initializing client worker...");

// In the worker, we have a different struct that we want to use as in
// `index.js`.
const { spawn_client } = wasm_bindgen;

async function setup() {
	// Load the wasm file by awaiting the Promise returned by `wasm_bindgen`.
	await wasm_bindgen("./pkg/torii_client_wasm_bg.wasm");

	try {
		const client = await spawn_client(
			"http://localhost:8080/grpc",
			"http://localhost:5050",
			"0x2430f23de0cd9a957e1beb7aa8ef2db2af872cc7bb3058b9be833111d5518f5",
			[
				{
					component: "Position",
					keys: [
						"0x517ececd29116499f4a1b64b094da79ba08dfd54a3edaa316134c41f8160973",
					],
				},
			]
		);

		// setup the message handler for the worker
		self.onmessage = function (e) {
			const event = e.data.type;
			const data = e.data.data;

			if (event === "getComponentValue") {
				getComponentValueHandler(client, data);
			} else if (event === "addEntityToSync") {
				addEntityToSyncHandler(client, data);
			} else {
				console.log("Sync Worker: Unknown event type", event);
			}
		};
	} catch (e) {
		console.error("error spawning client: ", e);
	}
}

// function addEntityToSyncHandler(client, data) {
// 	console.log("Sync Worker | Adding new entity to sync | data: ", data);
// 	client.addEntityToSync(data);
// }

/// Handler for the `get_entity` event from the main thread.
/// Returns back the entity data to the main thread via `postMessage`.
async function getComponentValueHandler(client, data) {
	console.log("Sync Worker | Getting component value | data: ", data);

	const component = data.component;
	const keys = data.keys;

	const values = await client.getComponentValue(component, keys);

	self.postMessage({
		type: "getComponentValue",
		data: {
			component: "Position",
			keys,
			values,
		},
	});
}

setup();
