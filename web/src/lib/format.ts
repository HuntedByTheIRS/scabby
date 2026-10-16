/** Relative time, byte sizes, initials, and deterministic avatar hues. */

export function relativeTime(iso: string | null | undefined, now: number = Date.now()): string {
	if (!iso) return '';
	const t = Date.parse(iso);
	if (Number.isNaN(t)) return '';
	const diff = Math.round((t - now) / 1000);
	const abs = Math.abs(diff);
	const units: [number, string][] = [
		[60, 'second'],
		[3600, 'minute'],
		[86400, 'hour'],
		[2592000, 'day'],
		[31536000, 'month'],
		[Infinity, 'year']
	];
	const divs = [1, 60, 3600, 86400, 2592000, 31536000];
	let unit = 'second';
	let div = 1;
	for (let i = 0; i < units.length; i++) {
		if (abs < units[i][0]) {
			div = divs[i];
			unit = units[i][1];
			break;
		}
	}
	const value = Math.max(1, Math.round(abs / div));
	const plural = value === 1 ? '' : 's';
	return diff < 0 ? `${value} ${unit}${plural} ago` : `${value} ${unit}${plural} from now`;
}

export function shortDate(iso: string | null | undefined): string {
	if (!iso) return '';
	const t = Date.parse(iso);
	if (Number.isNaN(t)) return '';
	return new Date(t).toLocaleDateString(undefined, {
		year: 'numeric',
		month: 'short',
		day: 'numeric'
	});
}

export function monthYear(iso: string | null | undefined): string {
	if (!iso) return '';
	const t = Date.parse(iso);
	if (Number.isNaN(t)) return '';
	return new Date(t).toLocaleDateString(undefined, { year: 'numeric', month: 'long' });
}

export function formatBytes(bytes: number | null | undefined): string {
	if (bytes === null || bytes === undefined) return '—';
	if (bytes < 1024) return `${bytes} B`;
	const units = ['KB', 'MB', 'GB', 'TB'];
	let value = bytes / 1024;
	let i = 0;
	while (value >= 1024 && i < units.length - 1) {
		value /= 1024;
		i++;
	}
	return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[i]}`;
}

export function initials(name: string): string {
	const parts = name.trim().split(/[\s._-]+/).filter(Boolean);
	if (parts.length === 0) return '?';
	if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
	return (parts[0][0] + parts[1][0]).toUpperCase();
}

/** Deterministic hue pair from a string hash, for the two-color avatar gradient. */
export function avatarHues(key: string): [number, number] {
	let h = 0;
	for (let i = 0; i < key.length; i++) {
		h = (h * 31 + key.charCodeAt(i)) >>> 0;
	}
	const a = h % 360;
	const b = (a + 42 + (h % 60)) % 360;
	return [a, b];
}

/** Split a clone URL at the middle for truncation that keeps both ends legible. */
export function truncateMiddle(value: string, head = 26, tail = 18): string {
	if (value.length <= head + tail + 1) return value;
	return `${value.slice(0, head)}…${value.slice(-tail)}`;
}

export function pluralize(count: number, one: string, many = `${one}s`): string {
	return `${count} ${count === 1 ? one : many}`;
}

/** Strip the first line of a commit message body for list subjects. */
export function subject(message: string): string {
	const line = (message ?? '').split('\n')[0].trim();
	return line || '(no message)';
}
