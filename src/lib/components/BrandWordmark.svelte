<script lang="ts">
  let { height = 22 }: { height?: number } = $props();
</script>

<span
  class="brand-wordmark"
  role="img"
  aria-label="LithoMark"
  style:font-size={`${height}px`}
>
  <span class="litho">Litho</span><span class="mark">Mark</span>
</span>

<style>
  .brand-wordmark {
    display: inline-flex;
    align-items: center;
    flex: none;
    font-family: "Libre Baskerville", Georgia, serif;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.035em;
    white-space: nowrap;
  }

  .litho {
    color: #b88a44;
  }

  .mark {
    color: #4e6a85;
  }
</style>
