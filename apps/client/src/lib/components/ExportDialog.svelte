<script lang="ts">
	import { untrack } from 'svelte';
	import { Badge, Button, Modal, Spinner } from '@facile/muse';
	import { EXPORT_FORMATS, Exporter } from '$lib/deck/exporting.svelte';
	import { t } from '$lib/i18n';
	import type { ExportFormat } from '$lib/types';

	let { open = $bindable(), id }: { open: boolean; id: string } = $props();

	const exporter = new Exporter(untrack(() => id));

	$effect(() => {
		if (open) void exporter.probe();
	});

	$effect(() => () => exporter.stop());

	const hint = (format: ExportFormat) => {
		if (format !== 'pptx' || exporter.pictures === null) return `export.${format}.hint` as const;
		return exporter.pictures ? 'export.pptx.hint.pictures' : 'export.pptx.hint.placeholders';
	};
</script>

<Modal {open} title={t('export.title')} size="md" onClose={() => (open = false)}>
	<ul class="flex flex-col gap-2" data-testid="export-dialog">
		{#each EXPORT_FORMATS as format (format)}
			{@const row = exporter.rows[format]}
			<li
				class="flex flex-wrap items-center gap-3 rounded-fc-lg bg-fc-surface p-3"
				data-testid="export-{format}"
				data-state={row.state}
			>
				<div class="min-w-0 flex-1 basis-48">
					<p class="text-fc-sm font-semibold text-fc-fg">{t(`export.${format}`)}</p>
					<p class="text-fc-xs text-fc-fg-muted">{t(hint(format))}</p>
					{#if row.state === 'failed'}
						<p class="text-fc-xs text-fc-danger" role="alert">{row.job?.error || t('export.failed')}</p>
					{/if}
				</div>
				{#if row.state === 'soon'}
					<Badge>{t('landing.soon')}</Badge>
				{:else if row.state === 'working'}
					<span class="flex items-center gap-2 text-fc-xs whitespace-nowrap text-fc-fg-muted" role="status">
						<Spinner size="sm" />{t('export.working')}
					</span>
				{:else if row.state === 'done' && row.job?.download_url}
					<Button size="sm" href={row.job.download_url} download={row.job.file_name} data-testid="export-download">
						{t('export.download')}
					</Button>
				{:else}
					<Button size="sm" variant="secondary" onclick={() => exporter.start(format)}>
						{row.state === 'failed' ? t('action.retry') : t('export.start')}
					</Button>
				{/if}
			</li>
		{/each}
	</ul>
</Modal>
