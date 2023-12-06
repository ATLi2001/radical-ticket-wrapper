/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npm run dev` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npm run deploy` to publish your worker
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

import { get_rw_set, anti_fraud } from './rust_radical_ticket/pkg/worker_rust';

const cache = await caches.open('RADICAL_SSR');
let env = {};
let logger = console.log;

function keyToCacheKey(key) {
	return new Request(`http://radicalcache/key/${key}`);
}

function valueToCacheValue(value) {
	let cacheResp = new Response(JSON.stringify({ value }));
	cacheResp.headers.append('Cache-Control', 'max-age=1000');
	cacheResp.headers.append('Cache-Control', 'public');
	return cacheResp;
}

async function checkCacheVersions(keys) {
	let kvMap = {};
	const randomValues = new Uint32Array(keys.length);
	crypto.getRandomValues(randomValues);
	await Promise.all(
		keys.map(async (key, idx) => {
			logger('Going to check the cache for key', key);
			const cacheKey = keyToCacheKey(key);
			let cacheValue = await cache.match(cacheKey);
			let version = 0;
			if (cacheValue != undefined) {
				let value = await cacheValue.json();
				let missProb = (env.COLD_MISS_PROB + env.CAP_MISS_PROB) * 100;
				let randomValue = randomValues[idx] / (0xffffffff + 1);
				let compRandom = Math.floor(randomValue * (100 - 0 + 1));
				if (compRandom <= missProb) {
					logger(`Triggering intentional miss on ${key} (${compRandom} vs ${missProb})`);
					cacheValue = undefined;
				}
				version = value['value'].Version;
			}
			if (cacheValue == undefined) {
				logger(`${key} not present in the storage system (or we triggered an intentional miss)`);
				version = -1;
			}
			logger(`Located version ${version} of ${key}`);
			kvMap[key] = version;
		})
	);
	return kvMap;
}

async function updateCache(key, value) {
	const cacheKey = keyToCacheKey(key);
	const cacheValue = valueToCacheValue(value);
	await cache.put(cacheKey, cacheValue);
}

async function handleConsistencyCheck(versions, remotefunc, args, remoteUrl) {
	let reqData = {
		versions: versions,
		function: remotefunc,
		args: args,
	};
	logger('Sending consistency check for', versions, 'to', remoteUrl, 'with remote func', remotefunc);
	return fetch(remoteUrl, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
		},
		body: JSON.stringify(reqData),
	}).then(async (resp) => {
		return resp.json();
	});
}

async function handleFunctionInvocation(url, args) {
	logger('Invoking function that lives at', url);
	return fetch(url, {
		method: 'POST',
		body: JSON.stringify(args),
	});
}

async function orchestrate(request) {
	const start = performance.now();
	let data = await request.json();
	// Get the current versions of the keys in question
	const versionStart = performance.now();

	// extract read write set
	let extracted = await target_rw_set(data.args);
	let checkVersions = await checkCacheVersions(extracted);
	let versionEnd = performance.now();
	logger('Result of version check:', checkVersions, 'took', versionEnd - versionStart);
	// Kick off the consistency check before running the function
	let consistencyPromise = null;
	if (env.DO_CONSISTENCY_CHECK) {
		consistencyPromise = handleConsistencyCheck(checkVersions, data.backup, data.args, data.remoteUrl);
	} else {
		logger('Skipping consistency check that would have used:', data.backup, data.remoteUrl, data.args);
	}
	// Now run the function while we have the http request to the consistency check fired off
	logger('Args to function', data.args);
	let functionResult = await target_function(data.args);
	logger('Result of function invocation', functionResult);
	const consistencyStart = performance.now();
	let consistencyResult = consistencyPromise != null ? await consistencyPromise : { checkResult: true };
	const consistencyEnd = performance.now();
	logger('Result of consistency check', consistencyResult, 'took', consistencyEnd - consistencyStart);
	let endResult = {
		success: consistencyResult.checkResult,
	};
	if (consistencyResult.checkResult) {
		endResult.result = functionResult;
	} else {
		endResult.result = consistencyResult.result;
	}
	logger('End result we should return to the client', endResult);
	if (!consistencyResult.checkResult) {
		logger('Consistency result failed');
		let updateStart = performance.now();
		await Promise.all(
			consistencyResult.updatedKeys.map(async (obj) => {
				let { ID, Key, Value, Version } = obj;
				await updateCache(Key, { ID, Key, Value, Version });
				logger(`Updated ${ID}`);
			})
		);
		let updateEnd = performance.now();
		logger('Finished updating the keys', updateEnd - updateStart);
	}
	// If we failed, we also need to update storage
	let end = performance.now();
	logger('Returning to user', end - start);
	return new Response(JSON.stringify(endResult), {
		headers: {
			'content-type': 'application/json;charset=UTF-8',
		},
	});
}

async function target_rw_set(args) {
	let { id, taken, res_email, res_name, res_card } = args;

	let rw_set = await get_rw_set(id, res_email, res_name, res_card);

	logger('rw_set', rw_set);

	return [rw_set];
}

async function target_function(args) {
	let { id, taken, res_email, res_name, res_card } = args;

	// get old ticket
	let key = `ticket-${id}`;
	let cacheValue = await cache.match(keyToCacheKey(key))
	// not there 
	if (cacheValue == undefined) {
		return false;
	}
	// extract info
	let val_json = await cacheValue.json();
	let old_val = val_json["value"];
	let old_version = old_val["Version"];
	let new_version = old_version + 1;
	let old_ticket = old_val["Value"];
	if (old_ticket["taken"]) {
		return false;
	}

	let new_ticket = {
		"id": id,
		"taken": true,
		"res_email": res_email,
		"res_name": res_name,
		"res_card": res_card,
	}

	let funcStart = performance.now();
	let b = anti_fraud(id, res_email, res_name, res_card);
	let funcEnd = performance.now();
	logger('anti fraud runtime', funcEnd - funcStart);

	let new_val = {
		"Key": key,
		"ID": key,
		"Version": new_version,
		"Value": new_ticket,
	};

	await updateCache(key, new_val);

	return b;
}

async function populate_tickets(n) {
	for (let i = 0; i < n; i++) {
		let key = `ticket-${i}`;
		let new_ticket = {
			"id": i,
			"taken": false,
			"res_email": null,
			"res_name": null,
			"res_card": null,
		}

		let new_val = {
			"Key": key,
			"ID": key,
			"Version": 0,
			"Value": new_ticket,
		};

		await updateCache(key, new_val);
	}

	// so we know how much to delet later
	await updateCache("count", n);
}


async function clear_cache() {
	let cacheValue = await cache.match(keyToCacheKey("count"));
	if (cacheValue != undefined) {
		let val_json = await cacheValue.json();
		let n = val_json["value"];
		for (let i = 0; i < n; i++) {
			await cache.delete(keyToCacheKey(`ticket-${i}`));
		}
	}
}

async function get_ticket(i) {
	let cacheValue =  await cache.match(keyToCacheKey(`ticket-${i}`));	
	if (cacheValue != undefined) {
		return await cacheValue.json();
	}
	return undefined;
}

export default {
	async fetch(request, environment, ctx) {
		const url = new URL(request.url);

		if (url.pathname === '/populate_tickets') {
			let n = await request.text();
			await populate_tickets(parseInt(n));
			return new Response('Success');
		} else if (url.pathname === '/clear_cache') {
			await clear_cache();
			return new Response('Success');
		} else if (url.pathname === '/get_ticket') {
			let resp = await get_ticket(0);
			logger(resp);
			return new Response('ticket 0');
		} 
		else {
			if (!environment.DEBUGPRINT) {
				logger = function () {};
			}
			logger('Starting function!');
			env = environment;
			let orchStart = performance.now();
			let orchResult = await orchestrate(request);
			let orchEnd = performance.now();
			logger('Got orchestrator result in', orchEnd - orchStart);
			return orchResult;
		}
	},
};
