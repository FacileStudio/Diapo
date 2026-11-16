<script lang="ts">
	import { EmptyState } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
</script>

<svelte:head><title>Diapo</title></svelte:head>

<div class="mx-auto flex w-full max-w-fc-xl flex-col gap-8 px-4 py-8 pb-24 sm:px-6 md:px-10 md:py-10 md:pb-32">
	<header class="flex flex-col gap-1">
		<h1 class="text-fc-2xl font-semibold whitespace-nowrap text-fc-fg">Présentations</h1>
		<p class="text-fc-sm text-fc-fg-muted">Espace personnel</p>
	</header>

	<EmptyState
		icon={logoIcon}
		title="Aucune présentation"
		description="Vos présentations apparaîtront ici. Une présentation est un seul fichier texte."
	/>
</div>
