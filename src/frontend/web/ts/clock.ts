const clock = document.getElementById("clock");

function tick(): void {
	if (clock) {
		clock.textContent = new Date().toLocaleTimeString("pl-PL");
	}
}

tick();
setInterval(tick, 1000);
