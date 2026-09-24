const api = "/api/ping";
const out = document.getElementById("out");

async function ping() {
	const start = performance.now();
	try {
		const res = await fetch(api);
		const json = await res.json();
		const ms = (performance.now() - start).toFixed(1);
		out.textContent = `${res.status} ${JSON.stringify(json)} ${ms}ms`;
	} catch (e) {
		out.textContent = `down: ${e}`;
	}
}

setInterval(ping, 1000);