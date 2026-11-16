<script lang="ts">
	import { Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import DemoSlide from './DemoSlide.svelte';
	import { coverLines } from './demo-source';

	let typed = $state<string[]>([]);

	const tail = $derived(['---', 'layout: center', '---', '', `## ${t('demo.thanks')}`]);
	const second = $derived(typed.length >= 3);

	const lines = (compact: boolean) => [...coverLines(t('demo.deck'), compact), ...typed];
	const thanks = $derived(typed[4]?.slice(3) ?? '');

	function reset() {
		typed = [];
	}

	function poster() {
		typed = [...tail];
	}

	async function script(act: Act) {
		await act.wait(900);
		for (const line of tail) {
			typed.push('');
			const at = typed.length - 1;
			await act.type(line, (value) => (typed[at] = value), 60);
			await act.wait(line ? 260 : 80);
		}
		await act.wait(2600);
		reset();
		await act.wait(500);
	}
</script>

<Tile title={t('demo.file.title')} body={t('demo.file.body')}>
	<Stage width={476} height={290} compact={{ below: 440, width: 326, height: 250 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div
				class="ft-card absolute top-5 bottom-5 left-5 overflow-hidden rounded-fc-lg bg-fc-page py-3 text-fc-fg"
				style:width={compact ? '158px' : '232px'}
			>
				{#each lines(compact) as line, index (index)}
					<div class="flex h-5 items-center text-[12px] leading-5 whitespace-pre">
						<span class="w-6 flex-none pr-2 text-right text-fc-fg-muted">{index + 1}</span>
						<span
							class={line.startsWith('#')
								? 'font-semibold'
								: line === '---' || line.includes(': ')
									? 'text-fc-fg-muted'
									: ''}>{line}</span
						>{#if index === lines(compact).length - 1}<span class="ft-caret"></span>{/if}
					</div>
				{/each}
			</div>
			<div class="ft-zoom absolute top-5 right-5 flex flex-col gap-3" style:width={compact ? '116px' : '176px'}>
				<DemoSlide layout="cover" title={t('demo.deck')} class="ft-slide" />
				<DemoSlide layout="center" title={thanks} class="ft-slide {second ? '' : 'ft-off'}" />
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.ft-zoom {
		--ds-zoom: 1.5;
	}
	.ft-card {
		font-family: var(--font-fc-mono);
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.25);
	}
	.ft-caret {
		display: inline-block;
		width: 1.5px;
		height: 13px;
		margin-left: 1px;
		background: var(--color-fc-fg);
	}
	:global(.ft-slide) {
		border-radius: 6px;
		box-shadow: 0 10px 22px -12px rgb(0 0 0 / 0.3);
		transition:
			opacity 0.3s ease,
			transform 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	:global(.ft-slide.ft-off) {
		opacity: 0;
		transform: translateY(6px) scale(0.97);
	}
	@media (prefers-reduced-motion: reduce) {
		:global(.ft-slide) {
			transition: none;
		}
	}
</style>
