<script lang="ts">
	import { Blob } from '@facile/blob';
	import { Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { Icon, icons } from '@facile/muse';
	import { t } from '$lib/i18n';
	import DemoSlide from './DemoSlide.svelte';

	let asked = $state(false);
	let done = $state(false);

	const bullets = $derived([t('demo.first'), t('demo.second')]);

	function reset() {
		asked = false;
		done = false;
	}

	function poster() {
		asked = true;
		done = true;
	}

	async function script(act: Act) {
		await act.wait(1000);
		asked = true;
		await act.wait(1500);
		done = true;
		await act.wait(3000);
		reset();
		await act.wait(500);
	}
</script>

<Tile title={t('demo.bulle.title')} body={t('demo.bulle.body')}>
	<Stage width={476} height={290} compact={{ below: 440, width: 326, height: 250 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div class="absolute top-5 left-5" style="--ds-zoom: 1.5" style:width={compact ? '214px' : '304px'}>
				<DemoSlide title={t('demo.heading')} {bullets} shown={done ? 2 : 1} class="bt-slide" />
			</div>
			<div class="absolute right-5 bottom-5 flex flex-col items-end gap-2" style:left={compact ? '56px' : '180px'}>
				<span
					class="bt-row max-w-full rounded-fc-lg bg-fc-accent px-3 py-1.5 text-[12px] leading-4 text-fc-accent-fg"
					class:bt-on={asked}>{t('demo.bulle.ask')}</span
				>
				<span
					class="bt-row flex items-center gap-2 self-start rounded-fc-lg bg-fc-page py-1.5 pr-3 pl-1.5 text-[12px] leading-4 text-fc-fg"
					class:bt-on={done}
				>
					<Blob name="Bulle" size={20} />
					<span class="whitespace-nowrap">{t('demo.bulle.done')}</span>
					<Icon icon={icons.check} size={12} class="text-fc-success" />
				</span>
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	:global(.bt-slide) {
		border-radius: 6px;
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.3);
	}
	.bt-row {
		box-shadow: 0 10px 22px -12px rgb(0 0 0 / 0.3);
		opacity: 0;
		transform: translateY(6px);
		transition:
			opacity 0.3s ease,
			transform 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.bt-on {
		opacity: 1;
		transform: none;
	}
	@media (prefers-reduced-motion: reduce) {
		.bt-row {
			transition: none;
		}
	}
</style>
