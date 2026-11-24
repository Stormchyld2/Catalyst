<script lang="ts">
  import catalystLogo from "../../assets/catalyst-logo.png";

  let { height = 22 }: { height?: number } = $props();
</script>

<img
  class="brand-wordmark"
  src={catalystLogo}
  alt="Catalyst"
  style:height={`${height}px`}
/>

<style>
  .brand-wordmark {
    display: block;
    flex: none;
    width: auto;
    user-select: none;
  }
</style>
