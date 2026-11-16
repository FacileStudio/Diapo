<script lang="ts">
	import { Cursor, Stage, Window, cursorAt, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import DemoEditor from './DemoEditor.svelte';
	import DemoSlide from './DemoSlide.svelte';
	import { sceneLines, typedScene } from './demo-source';

	let typed = $state<string[]>([]);
	let saving = $state(false);
	let presenting = $state(false);
	let onSecond = $state(false);
	let cursor = $state(cursorAt());
	let target = $state<HTMLElement>();

	const all = $derived(sceneLines(t('demo.heading'), t('demo.first'), t('demo.second')));
	const scene = $derived(typedScene(all));

	function reset() {
		typed = [];
		saving = false;
		presenting = false;
		onSecond = false;
		cursor.visible = false;
	}

	function poster() {
		reset();
		typed = [...all];
	}

	async function write(act: Act) {
		for (const line of all) {
			typed.push('');
			const at = typed.length - 1;
			saving = true;
			await act.type(line, (value) => (typed[at] = value), 55);
			await act.wait(line ? 380 : 120);
			saving = false;
		}
	}

	async function present(act: Act) {
		cursor.visible = true;
		await act.move(cursor, act.point(target, 0.6, 0.6));
		await act.click(cursor, () => (presenting = true));
		cursor.visible = false;
		await act.wait(1500);
		onSecond = true;
		await act.wait(2400);
	}

	async function script(act: Act) {
		await act.wait(900);
		await write(act);
		await act.wait(700);
		await present(act);
		reset();
		await act.wait(700);
	}
</script>

{#snippet show(compact: boolean)}
	<div class="ed-show absolute inset-0 flex items-center justify-center" class:ed-on={presenting} aria-hidden="true">
		<div class="relative {compact ? 'w-full' : 'w-[86%]'}">
			<DemoSlide
				layout="cover"
				title={t('demo.deck')}
				sub={t('demo.sub')}
				class="ed-slide {onSecond ? 'ed-off' : ''}"
			/>
			<DemoSlide
				title={scene.heading}
				bullets={scene.bullets}
				class="ed-slide absolute inset-0 {onSecond ? '' : 'ed-off'}"
			/>
		</div>
		<span class="ed-bar absolute bottom-0 left-0 h-[3px]" style:width={onSecond ? '100%' : '50%'}></span>
	</div>
{/snippet}

<Stage
	width={960}
	height={560}
	compact={{ below: 640, width: 358, height: 520 }}
	{script}
	{reset}
	{poster}
	label={t('demo.label')}
>
	{#snippet children({ compact })}
		<Window
			style={compact ? 'top:12px;left:12px;width:334px;height:496px' : 'top:16px;left:40px;width:880px;height:528px'}
		>
			<div class="relative flex min-h-0 flex-1 {compact ? '' : 'p-3 pt-1'}">
				<DemoEditor {typed} {saving} {compact} bind:target />
				{@render show(compact)}
			</div>
		</Window>
		<Cursor {cursor} />
	{/snippet}
</Stage>

<style>
	.ed-show {
		background: var(--color-fc-inverse);
		opacity: 0;
		transition: opacity 0.35s ease;
	}
	.ed-on {
		opacity: 1;
	}
	.ed-show :global(.ed-slide) {
		transition: opacity 0.35s ease;
	}
	.ed-show :global(.ed-off) {
		opacity: 0;
	}
	.ed-bar {
		background: var(--color-fc-on-inverse);
		opacity: 0.7;
		transition: width 0.4s ease;
	}
	@media (prefers-reduced-motion: reduce) {
		.ed-show,
		.ed-bar,
		.ed-show :global(.ed-slide) {
			transition: none;
		}
	}
</style>
