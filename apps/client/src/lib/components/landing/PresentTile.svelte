<script lang="ts">
	import { Roll, Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { clock } from '$lib/deck/show.svelte';
	import { t } from '$lib/i18n';
	import DemoSlide from './DemoSlide.svelte';

	const START = 192;

	let step = $state(1);
	let seconds = $state(START);

	const bullets = $derived([t('demo.first'), t('demo.second')]);

	function reset() {
		step = 1;
		seconds = START;
	}

	function poster() {
		step = 2;
		seconds = START + 4;
	}

	async function script(act: Act) {
		for (let tick = 0; tick < 7; tick += 1) {
			await act.wait(1000);
			seconds += 1;
			if (tick === 2) step = 2;
		}
		reset();
	}
</script>

<Tile title={t('demo.present.title')} body={t('demo.present.body')}>
	<Stage width={476} height={290} compact={{ below: 440, width: 326, height: 250 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div
				class="pt-screen absolute top-5 left-5 overflow-hidden rounded-fc-lg"
				style="--ds-zoom: 1.5"
				style:width={compact ? '236px' : '344px'}
			>
				<DemoSlide title={t('demo.heading')} {bullets} shown={step} />
				<span class="pt-bar absolute bottom-0 left-0 h-[3px]" style:width={step === 2 ? '66%' : '50%'}></span>
			</div>
			<div
				class="pt-card absolute right-5 bottom-5 flex flex-col gap-1.5 rounded-fc-lg bg-fc-page p-2.5 text-fc-fg"
				style:width={compact ? '156px' : '204px'}
			>
				<span class="flex items-center justify-between text-[10px] leading-3 text-fc-fg-muted">
					{t('presenter.notes')}
					<span class="text-[13px] leading-4 font-semibold text-fc-fg tabular-nums"
						><Roll value={clock(seconds * 1000)} /></span
					>
				</span>
				<span class="text-[12px] leading-4">{t('demo.present.note')}</span>
				<span class="mt-0.5 flex items-center gap-2 text-[10px] leading-3 text-fc-fg-muted">
					<DemoSlide layout="center" title={t('demo.thanks')} class="pt-next" />
					<span class="min-w-0">{t('presenter.next')}</span>
				</span>
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.pt-screen {
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.35);
	}
	.pt-bar {
		background: var(--color-fc-inverse);
		opacity: 0.55;
		transition: width 0.4s ease;
	}
	.pt-card {
		box-shadow: 0 12px 28px -10px rgb(0 0 0 / 0.3);
	}
	:global(.pt-next) {
		--ds-zoom: 2.6;
		width: 52px;
		flex: none;
		border-radius: 3px;
		box-shadow: 0 2px 6px rgb(0 0 0 / 0.2);
	}
	@media (prefers-reduced-motion: reduce) {
		.pt-bar {
			transition: none;
		}
	}
</style>
