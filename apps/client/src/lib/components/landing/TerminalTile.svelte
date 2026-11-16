<script lang="ts">
	import { Caret, Stage, Terminal, Tile, type Act } from '@facile/mise-en-scene';
	import { Badge } from '@facile/muse';
	import { t } from '$lib/i18n';
	import { CLI_RELEASED } from '$lib/downloads';

	type Line = { kind: 'cmd' | 'ok'; text: string };

	const SCRIPT: Line[] = [
		{ kind: 'cmd', text: 'diapo new bilan' },
		{ kind: 'ok', text: 'Created bilan with diapo.md and assets/' },
		{ kind: 'cmd', text: 'diapo serve bilan' },
		{ kind: 'ok', text: 'Serving bilan at http://localhost:3030' },
		{ kind: 'cmd', text: 'diapo push bilan --create' },
		{ kind: 'ok', text: 'Pushed bilan to deck 7c1e (revision 1)' }
	];

	let lines = $state<Line[]>([]);

	function reset() {
		lines = [];
	}

	function poster() {
		lines = [...SCRIPT];
	}

	async function script(act: Act) {
		await act.wait(600);
		for (const line of SCRIPT) {
			if (line.kind === 'ok') {
				lines.push(line);
				await act.wait(900);
				continue;
			}
			lines.push({ kind: 'cmd', text: '' });
			const at = lines.length - 1;
			await act.type(line.text, (value) => (lines[at].text = value), 60);
			await act.wait(450);
		}
		await act.wait(2400);
		reset();
	}
</script>

{#snippet soon()}{#if !CLI_RELEASED}<Badge>{t('landing.soon')}</Badge>{/if}{/snippet}

<Tile title={t('demo.terminal.title')} body={t('demo.terminal.body')} badge={soon}>
	<Stage width={476} height={290} compact={{ below: 440, width: 326, height: 250 }} {script} {reset} {poster}>
		{#snippet children()}
			<Terminal style="top:20px;left:20px;right:-48px;bottom:-24px">
				{#each lines as line, index (index)}
					<div class="whitespace-nowrap">
						{#if line.kind === 'cmd'}
							<span class="tt-dim">$</span>
							{line.text}{#if index === lines.length - 1}<Caret />{/if}
						{:else}
							<span class="tt-ok">✓</span> <span class="tt-dim">{line.text}</span>
						{/if}
					</div>
				{/each}
				{#if lines.at(-1)?.kind !== 'cmd'}
					<div><span class="tt-dim">$</span> <Caret /></div>
				{/if}
			</Terminal>
		{/snippet}
	</Stage>
</Tile>

<style>
	.tt-dim {
		color: var(--color-fc-on-inverse-muted);
	}
	.tt-ok {
		color: color-mix(in oklab, var(--color-fc-success), var(--color-fc-on-inverse) 35%);
	}
</style>
