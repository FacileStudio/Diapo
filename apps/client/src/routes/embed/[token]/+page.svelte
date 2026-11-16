<script lang="ts">
	import { page } from '$app/state';
	import PublicViewer from '$lib/components/PublicViewer.svelte';
</script>

{#key page.params.token}
	<PublicViewer token={page.params.token ?? ''} embedded />
{/key}
