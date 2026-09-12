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
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;');
}

export function noteToEditorContent(corps: string): string {
	if (!corps.trim()) return '<p></p>';
	if (noteLooksLikeHtml(corps)) return corps;
	return corps
		.split('\n')
		.map((line) => `<p>${escapeHtml(line) || '<br>'}</p>`)
		.join('');
}
