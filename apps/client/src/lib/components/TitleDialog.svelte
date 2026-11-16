<script lang="ts">
	import { Button, Field, Input, Modal } from '@facile/muse';
	import { t } from '$lib/i18n';

	let {
		open,
		heading,
		value = '',
		confirm,
		onsubmit,
		onclose
	}: {
		open: boolean;
		heading: string;
		/** The title the field opens with. */
		value?: string;
		/** The label of the submit button. */
		confirm: string;
		onsubmit: (title: string) => void | Promise<void>;
		onclose: () => void;
	} = $props();

	const form = $props.id();

	let draft = $state('');
	let busy = $state(false);

	$effect(() => {
		if (open) draft = value;
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const title = draft.trim();
		if (!title || busy) return;
		busy = true;
		try {
			await onsubmit(title);
		} finally {
			busy = false;
		}
	}
</script>

<Modal {open} title={heading} size="sm" onClose={onclose}>
	<form id={form} class="flex flex-col gap-4" onsubmit={submit}>
		<Field label={t('deck.title')}>
			{#snippet children({ id })}
				<Input {id} bind:value={draft} maxlength={200} required autofocus placeholder={t('deck.untitled')} />
			{/snippet}
		</Field>
	</form>
	{#snippet footer()}
		<Button variant="secondary" onclick={onclose}>{t('action.cancel')}</Button>
		<Button type="submit" {form} disabled={busy || !draft.trim()}>{confirm}</Button>
	{/snippet}
</Modal>
