/** Unified-diff parsing for DiffView. The v1 API does not return per-line
 * patches, so this only runs when a `patch` string is actually present. */

export type DiffLineType = 'add' | 'del' | 'context' | 'hunk' | 'meta';

export interface DiffLine {
	type: DiffLineType;
	text: string;
	oldNo: number | null;
	newNo: number | null;
}

export function parseUnifiedDiff(patch: string): DiffLine[] {
	const out: DiffLine[] = [];
	let oldNo = 0;
	let newNo = 0;

	for (const raw of patch.split('\n')) {
		if (raw.startsWith('@@')) {
			const match = /@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(raw);
			if (match) {
				oldNo = Number(match[1]);
				newNo = Number(match[2]);
			}
			out.push({ type: 'hunk', text: raw, oldNo: null, newNo: null });
			continue;
		}
		if (
			raw.startsWith('diff ') ||
			raw.startsWith('index ') ||
			raw.startsWith('--- ') ||
			raw.startsWith('+++ ') ||
			raw.startsWith('new file') ||
			raw.startsWith('deleted file') ||
			raw.startsWith('similarity index') ||
			raw.startsWith('rename ')
		) {
			out.push({ type: 'meta', text: raw, oldNo: null, newNo: null });
			continue;
		}
		if (raw.startsWith('\\')) {
			out.push({ type: 'meta', text: raw, oldNo: null, newNo: null });
			continue;
		}
		if (raw.startsWith('+')) {
			out.push({ type: 'add', text: raw.slice(1), oldNo: null, newNo: newNo++ });
			continue;
		}
		if (raw.startsWith('-')) {
			out.push({ type: 'del', text: raw.slice(1), oldNo: oldNo++, newNo: null });
			continue;
		}
		if (raw.startsWith(' ') || raw === '') {
			out.push({ type: 'context', text: raw.slice(1), oldNo: oldNo++, newNo: newNo++ });
			continue;
		}
		out.push({ type: 'context', text: raw, oldNo: oldNo++, newNo: newNo++ });
	}

	return out;
}

/** Count additions/deletions from a patch, for a compact stats line. */
export function diffStats(lines: DiffLine[]): { additions: number; deletions: number } {
	let additions = 0;
	let deletions = 0;
	for (const line of lines) {
		if (line.type === 'add') additions++;
		else if (line.type === 'del') deletions++;
	}
	return { additions, deletions };
}

/** Split a full multi-file patch into per-file sections. */
export function splitPatchByFile(patch: string): { header: string; lines: DiffLine[] }[] {
	const files: { header: string; lines: DiffLine[] }[] = [];
	let current: { header: string; lines: DiffLine[] } | null = null;

	for (const raw of patch.split('\n')) {
		if (raw.startsWith('diff ')) {
			if (current) files.push(current);
			current = { header: raw, lines: [] };
			continue;
		}
		if (current) current.lines.push(...parseUnifiedDiff(raw));
	}
	if (current) files.push(current);
	return files;
}
