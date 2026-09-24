type Pong = {
	pong: boolean;
	t: number;
};

const api = "/api/ping";
const out = document.getElementById("out") as HTMLElement;

async function ping(): Promise<void> {
	const start = performance.now();
	try {
		const res = await fetch(api);
		const json: Pong = await res.json();
		const ms = (performance.now() - start).toFixed(1);
		out.textContent = `${res.status} ${JSON.stringify(json)} ${ms}ms`;
	} catch (e) {
		out.textContent = `down: ${e}`;
	}
}

setInterval(ping, 1000);
