<script lang="ts">

type Option = {
  value: string;
  label: string;
};

let {
  value,
  options,
  onchange,
  class: className = "",
}: {
  value: string;
  options: Option[];
  onchange: (value: string) => void;
  class?: string;
} = $props();

</script>

<select
  {value}
  onchange={(event) => onchange(event.currentTarget.value)}
  class={`h-8 w-full rounded-md border border-border bg-card px-2 text-xs text-foreground ${className}`}
>
  {#each options as option (option.value)}
    <option value={option.value}>{option.label}</option>
  {/each}
</select>
