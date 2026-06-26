<script lang="ts">
    import "../styles/base.css";
    import { appState } from "$lib/stores/app.svelte";
    import { onMount } from "svelte";
    import type { Snippet } from "svelte";

    let { children }: { children: Snippet } = $props();

    onMount(() => {
        appState.initialize();
    });
</script>

<svelte:head>
    {#if appState.themeCss}
        {@html `<style id="umm-theme">${appState.themeCss}</style>`}
    {/if}
</svelte:head>

{@render children()}
