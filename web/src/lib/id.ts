let counter = 0;

/** Stable, collision-free id for label/control/error wiring. */
export function uid(prefix = 'f'): string {
	counter += 1;
	return `${prefix}-${counter}`;
}
