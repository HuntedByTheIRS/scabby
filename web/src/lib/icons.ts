// Monochrome inline-SVG icon set. Each entry is the inner markup of a 24x24
// stroke-based glyph; color comes from currentColor, never a hard-coded value.

export type IconName =
	| 'repo'
	| 'branch'
	| 'commit'
	| 'issue'
	| 'star'
	| 'copy'
	| 'check'
	| 'settings'
	| 'plus'
	| 'trash'
	| 'lock'
	| 'globe'
	| 'chevron-up'
	| 'chevron-down'
	| 'chevron-left'
	| 'chevron-right'
	| 'spinner'
	| 'search'
	| 'user'
	| 'logout'
	| 'key'
	| 'tag'
	| 'alert'
	| 'close'
	| 'book'
	| 'external'
	| 'file'
	| 'folder'
	| 'code'
	| 'filter'
	| 'upload'
	| 'clock';

export const ICONS: Record<IconName, string> = {
	repo: '<path d="M6 3h9.5A2.5 2.5 0 0 1 18 5.5v13A2.5 2.5 0 0 1 15.5 21H6"/><path d="M6 3a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2"/><path d="M8 7.5h6"/>',
	branch:
		'<circle cx="6" cy="5" r="2.4"/><circle cx="6" cy="19" r="2.4"/><circle cx="18" cy="7" r="2.4"/><path d="M6 7.4v9.2"/><path d="M18 9.4a8 8 0 0 1-8 8"/>',
	commit:
		'<circle cx="12" cy="12" r="3.6"/><path d="M2.5 12h5.9M15.6 12h5.9"/>',
	issue: '<circle cx="12" cy="12" r="8.5"/><circle cx="12" cy="12" r="2.6"/>',
	star: '<path d="M12 3.4l2.6 5.4 5.9.8-4.3 4.2 1 5.9-5.2-2.8-5.2 2.8 1-5.9L3.5 9.6l5.9-.8z"/>',
	copy: '<rect x="9" y="9" width="11.5" height="11.5" rx="2"/><path d="M5.5 15H4.5A1.5 1.5 0 0 1 3 13.5v-9A1.5 1.5 0 0 1 4.5 3h9A1.5 1.5 0 0 1 15 4.5v1"/>',
	check: '<path d="M4.5 12.6l5 5L19.5 6.4"/>',
	settings:
		'<circle cx="12" cy="12" r="3"/><path d="M12 2.5v2.8M12 18.7v2.8M2.5 12h2.8M18.7 12h2.8M5.3 5.3l2 2M16.7 16.7l2 2M18.7 5.3l-2 2M7.3 16.7l-2 2"/>',
	plus: '<path d="M12 5.2v13.6M5.2 12h13.6"/>',
	trash: '<path d="M4 6.5h16M9.5 3.5h5M6.4 6.5l.9 13.1A1.5 1.5 0 0 0 8.8 21h6.4a1.5 1.5 0 0 0 1.5-1.4l.9-13.1M10 10.5v6.5M14 10.5v6.5"/>',
	lock: '<rect x="4.8" y="10.5" width="14.4" height="9.7" rx="2"/><path d="M8 10.5V7.6a4 4 0 0 1 8 0v2.9"/>',
	globe:
		'<circle cx="12" cy="12" r="8.7"/><path d="M3.3 12h17.4"/><path d="M12 3.3a14 14 0 0 1 0 17.4M12 3.3a14 14 0 0 0 0 17.4"/>',
	'chevron-up': '<path d="M6 15l6-6 6 6"/>',
	'chevron-down': '<path d="M6 9.5l6 6 6-6"/>',
	'chevron-left': '<path d="M15 5.5l-6 6.5 6 6.5"/>',
	'chevron-right': '<path d="M9 5.5l6 6.5-6 6.5"/>',
	spinner: '<path d="M12 3.2a8.8 8.8 0 1 0 8.8 8.8"/>',
	search: '<circle cx="10.8" cy="10.8" r="7"/><path d="M15.9 15.9L21 21"/>',
	user: '<circle cx="12" cy="8" r="3.8"/><path d="M4.5 20.5a7.5 7.5 0 0 1 15 0"/>',
	logout:
		'<path d="M14.5 4H18a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-3.5M9.5 16.5L5 12l4.5-4.5M5 12h11"/>',
	key: '<circle cx="8" cy="15" r="4"/><path d="M10.9 12.1L20 3M16.6 6.4l2 2M14.4 8.6l2 2"/>',
	tag: '<path d="M3.5 12V5.5A2 2 0 0 1 5.5 3.5H12l8.5 8.5-8.5 8.5z"/><circle cx="7.6" cy="7.6" r="1.3"/>',
	alert: '<path d="M12 3.2l9.3 16.6H2.7z"/><path d="M12 9.3v4.6M12 16.9v.9"/>',
	close: '<path d="M6 6l12 12M18 6L6 18"/>',
	book: '<path d="M5 4a2 2 0 0 1 2-2h12.5v18H7a2 2 0 0 1-2-2z"/><path d="M5 18a2 2 0 0 1 2-2h12.5"/>',
	external:
		'<path d="M14 4h6v6M20 4l-9 9M18 13.5V18a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4.5"/>',
	file: '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/>',
	folder: '<path d="M3 7.5A2 2 0 0 1 5 5.5h4l2 2.2h8a2 2 0 0 1 2 2v7.8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
	code: '<path d="M8.5 6.5L3 12l5.5 5.5M15.5 6.5L21 12l-5.5 5.5"/>',
	filter: '<path d="M3 5h18l-7 8.2V20l-4-2.2v-4.6z"/>',
	upload: '<path d="M12 20V5.5M6.5 11L12 5.5 17.5 11"/>',
	clock: '<circle cx="12" cy="12" r="8.7"/><path d="M12 7.2V12l3.4 2"/>'
};
