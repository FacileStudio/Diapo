<script lang="ts">
	import { Blob } from '@facile/blob';
	import { Icon, icons } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
	import { t } from '$lib/i18n';
	import DemoSlide from './DemoSlide.svelte';
	import { coverLines, typedScene } from './demo-source';

	let {
		typed,
		saving,
		compact,
		target = $bindable()
	}: {
		/** The lines typed so far after the cover scene. The last one carries the caret. */
		typed: string[];
		saving: boolean;
		compact: boolean;
		/** The Present button, for the cursor to aim at. */
		target?: HTMLElement;
	} = $props();

	const fixed = $derived(coverLines(t('demo.deck'), compact));
	const scene = $derived(typedScene(typed));
	const lines = $derived([...fixed, ...typed]);
	const sub = $derived(t('demo.sub'));
</script>

{#snippet slide(index: number, extra: string)}
	{#if index === 0}
		<DemoSlide layout="cover" title={t('demo.deck')} {sub} class={extra} />
	{:else}
		<DemoSlide title={scene.heading} bullets={scene.bullets} class={extra} />
	{/if}
{/snippet}

<div class="flex min-h-0 flex-1 text-fc-fg" style="font-family: var(--font-fc-body)">
	{#if !compact}
		<div class="flex w-[52px] flex-none flex-col items-center gap-1.5 rounded-fc-lg bg-(--mes-w-side) py-3">
			<Icon icon={logoIcon} size={22} />
			<span class="mt-3 flex size-8 items-center justify-center rounded-fc-md bg-fc-accent text-fc-accent-fg">
				<Icon icon={icons.home} size={16} />
			</span>
			<span class="flex size-8 items-center justify-center text-fc-fg-muted"
				><Icon icon={icons.remove} size={16} /></span
			>
			<span
				class="mt-auto flex size-7 items-center justify-center rounded-fc-full bg-fc-accent text-[11px] font-semibold text-fc-accent-fg"
				>A</span
			>
		</div>
	{/if}
	<div class="flex min-w-0 flex-1 flex-col gap-2.5 {compact ? 'p-2.5' : 'py-0 pr-0 pl-3'}">
		<div class="flex h-9 flex-none items-center gap-2 text-[12px]">
			{#if !compact}
				<span class="flex items-center gap-1 text-fc-fg-muted"
					><Icon icon={icons.chevronLeft} size={12} />{t('decks.title')}</span
				>
			{/if}
			<span class="px-1 text-[15px] font-semibold whitespace-nowrap">{t('demo.deck')}</span>
			<span class="flex items-center gap-1.5 text-[11px] whitespace-nowrap text-fc-fg-muted">
				<span class="size-1.5 rounded-fc-full {saving ? 'bg-fc-fg-muted' : 'bg-fc-success'}"></span>
				{#if !compact}{saving ? t('save.saving') : t('save.clean')}{/if}
			</span>
			<span class="ml-auto flex items-center gap-1.5">
				{#if !compact}
					<span class="flex h-8 items-center gap-1.5 rounded-fc-md bg-(--mes-w-side) px-2.5">
						<Icon icon={icons.plus} size={13} />{t('insert.label')}
					</span>
					<span class="flex size-8 items-center justify-center"><Icon icon={icons.folder} size={15} /></span>
				{/if}
				<span class="flex size-8 items-center justify-center"><Blob name="Bulle" size={20} /></span>
				<span
					bind:this={target}
					class="flex h-8 items-center gap-1.5 rounded-fc-pill bg-fc-accent px-3 font-semibold whitespace-nowrap text-fc-accent-fg"
				>
					<Icon icon={icons.monitor} size={13} />{t('present.start')}
				</span>
			</span>
		</div>
		<div class="flex min-h-0 flex-1 gap-2.5 {compact ? 'flex-col' : ''}">
			<div class="de-source min-w-0 flex-1 overflow-hidden rounded-fc-lg bg-(--mes-w-side) py-2.5">
				{#each lines as line, index (index)}
					<div class="flex h-5 items-center text-[12px] leading-5 whitespace-pre">
						<span class="w-9 flex-none pr-3 text-right text-fc-fg-muted">{index + 1}</span>
						<span
							class={line.startsWith('#')
								? 'font-semibold'
								: line === '---' || line.includes(': ')
									? 'text-fc-fg-muted'
									: ''}>{line}</span
						>{#if index === lines.length - 1 && typed.length > 0}<span class="de-caret"></span>{/if}
					</div>
				{/each}
			</div>
			<div class="flex min-w-0 flex-1 items-center justify-center rounded-fc-lg bg-(--mes-w-side) p-3">
				<div class="de-slides relative w-full">
					{@render slide(0, `de-slide ${scene.open ? 'de-off' : ''}`)}
					{@render slide(1, `de-slide absolute inset-0 ${scene.open ? '' : 'de-off'}`)}
				</div>
			</div>
		</div>
		{#if !compact}
			<div class="flex h-[84px] flex-none flex-col gap-1.5 rounded-fc-lg bg-(--mes-w-side) px-2.5 py-2">
				<span class="text-[10px] leading-3 text-fc-fg-muted">{t('strip.count', { count: scene.open ? 2 : 1 })}</span>
				<div class="flex gap-2">
					{@render slide(0, `de-thumb ${scene.open ? '' : 'de-current'}`)}
					{@render slide(1, `de-thumb de-current ${scene.open ? '' : 'de-off'}`)}
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.de-source {
		font-family: var(--font-fc-mono);
	}
	.de-caret {
		display: inline-block;
		width: 1.5px;
		height: 14px;
		margin-left: 1px;
		background: var(--color-fc-fg);
	}
	.de-slides :global(.de-slide) {
		border-radius: 6px;
		box-shadow: 0 8px 22px -8px rgb(0 0 0 / 0.3);
		transition: opacity 0.3s ease;
	}
	:global(.de-thumb) {
		width: 86px;
		flex: none;
		border-radius: 4px;
		opacity: 0.75;
		transition:
			opacity 0.3s ease,
			transform 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	:global(.de-thumb.de-current) {
		opacity: 1;
	}
	:global(.de-off) {
		opacity: 0 !important;
	}
	:global(.de-thumb.de-off) {
		transform: translateY(4px) scale(0.96);
	}
	@media (prefers-reduced-motion: reduce) {
		.de-slides :global(.de-slide),
		:global(.de-thumb) {
			transition: none;
		}
	}
</style>
