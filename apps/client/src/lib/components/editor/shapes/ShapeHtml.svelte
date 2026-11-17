<script lang="ts">
	let { html }: { html: string } = $props();
</script>

<iframe
	title=""
	srcdoc={html}
	sandbox="allow-scripts"
	referrerpolicy="no-referrer"
	tabindex="-1"
	class="pointer-events-none block size-full border-0 bg-transparent"
></iframe>
