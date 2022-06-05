<script context="module" lang="ts">
import type { Load } from "@sveltejs/kit";
export const load: Load = ({ error, status }) => ({
  props: {
    message: JSON.stringify(error?.message),
    status,
  },
});
</script>

<script lang="ts">
  import { InlineNotification } from "carbon-components-svelte";
  export let message: string;
  export let status: number;
</script>

{#if status == 404} <!-- Used '==' instead of '===' to match string/number status code (just to be sure) -->
  <InlineNotification>
      Page not found
  </InlineNotification>
{:else}
  <InlineNotification>
      Error: {{ message }}
  </InlineNotification>
{/if}