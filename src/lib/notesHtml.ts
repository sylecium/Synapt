const ALLOWED_TAGS = new Set([
	'p',
	'br',
	'strong',
	'b',
	'em',
	'i',
	'u',
	'ul',
	'ol',
	'li',
	'h2',
	'table',
	'thead',
	'tbody',
	'tr',
	'th',
	'td',
	'blockquote'
]);

const VOID_TAGS = new Set(['br']);

export function noteLooksLikeHtml(corps: string): boolean {
	return /<\/?[a-z][\s\S]*>/i.test(corps);
}

export function notePlainText(corps: string): string {
	return corps
		.replace(/<br\s*\/?>/gi, '\n')
		.replace(/<\/(p|div|h[1-6]|li|tr|blockquote)>/gi, '\n')
		.replace(/<[^>]+>/g, '')
		.replace(/&nbsp;/g, ' ')
		.replace(/&amp;/g, '&')
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&quot;/g, '"')
		.replace(/\n+/g, '\n')
		.trim();
}

export function noteIsEmpty(corps: string | null | undefined): boolean {
	return !notePlainText(corps ?? '');
}

export function noteTitle(corps: string): string {
	const line = notePlainText(corps).split('\n').find((l) => l.trim());
	return line?.trim() || 'Nouvelle note';
}

export function notePreview(corps: string): string {
	const lines = notePlainText(corps)
		.split('\n')
		.map((l) => l.trim())
		.filter(Boolean);
	return lines.slice(1).join(' ');
}

function escapeHtml(text: string): string {
	return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

export function sanitizeNoteHtml(html: string): string {
	let s = html.replace(/<!--[\s\S]*?-->/g, '');
	s = s.replace(/<(script|style|iframe|object|embed)[\s\S]*?<\/\1>/gi, '');
	s = s.replace(/<\/?(script|style|iframe|object|embed)[^>]*>/gi, '');
	return s.replace(/<\/?([a-zA-Z][a-zA-Z0-9]*)\b[^>]*>/g, (full, rawName: string) => {
		const tag = rawName.toLowerCase();
		if (!ALLOWED_TAGS.has(tag)) return '';
		if (full.startsWith('</')) return `</${tag}>`;
		if (VOID_TAGS.has(tag)) return `<${tag}>`;
		return `<${tag}>`;
	});
}

export function noteToEditorContent(corps: string): string {
	if (!corps.trim()) return '<p></p>';
	if (noteLooksLikeHtml(corps)) return sanitizeNoteHtml(corps);
	return corps
		.split('\n')
		.map((line) => `<p>${escapeHtml(line) || '<br>'}</p>`)
		.join('');
}
