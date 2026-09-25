type Problem = {
	id: string;
	name: string;
	title: string;
	time_limit: number;
	memory_limit: number;
	public: boolean;
	created_at: string;
};

const body = document.getElementById("problems") as HTMLTableSectionElement;
const params = new URLSearchParams(location.search);
const q = params.get("q") ?? "";

function time(ms: number): string {
	return ms % 1000 === 0 ? `${ms / 1000} s` : `${ms} ms`;
}

function memory(kib: number): string {
	return kib % 1024 === 0 ? `${kib / 1024} MB` : `${kib} KB`;
}

function link(id: string): string {
	const [name, hash] = id.split("#");
	return `/p/${encodeURIComponent(name)}/${encodeURIComponent(hash ?? "")}`;
}

function cell(text: string, cls: string): HTMLTableCellElement {
	const td = document.createElement("td");
	td.className = cls;
	td.textContent = text;
	return td;
}

function message(text: string): void {
	const tr = document.createElement("tr");
	const td = cell(text, "text-center text-muted");
	td.colSpan = 4;
	tr.append(td);
	body.replaceChildren(tr);
}

function row(p: Problem): HTMLTableRowElement {
	const tr = document.createElement("tr");
	const name = document.createElement("td");
	const a = document.createElement("a");
	a.href = link(p.id);
	a.textContent = p.title;
	name.append(a);
	tr.append(
		cell(p.id, "font-mono whitespace-nowrap"),
		name,
		cell(time(p.time_limit), "text-center whitespace-nowrap"),
		cell(memory(p.memory_limit), "text-center whitespace-nowrap"),
	);
	return tr;
}

function sortLinks(): void {
	for (const order of ["id", "name"]) {
		const a = document.getElementById(`sort-${order}`) as HTMLAnchorElement;
		const next = new URLSearchParams({ order_by: order });
		if (q) {
			next.set("q", q);
		}
		a.href = `?${next}`;
	}
}

async function load(): Promise<void> {
	(document.getElementById("q") as HTMLInputElement).value = q;
	sortLinks();
	try {
		const res = await fetch(`/api/problems?${params}`);
		if (!res.ok) {
			message(`Error ${res.status}`);
			return;
		}
		const list: Problem[] = await res.json();
		if (list.length === 0) {
			message("No problems found.");
			return;
		}
		body.replaceChildren(...list.map(row));
	} catch {
		message("Can't reach the server.");
	}
}

load();
