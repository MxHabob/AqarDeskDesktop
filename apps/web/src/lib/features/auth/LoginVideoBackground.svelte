<script lang="ts">
	import * as m from '#lib/paraglide/messages';
</script>

<aside class="relative m-2 hidden min-h-[calc(100svh-1rem)] flex-1 overflow-hidden rounded-xl lg:flex">
	<div
		class="absolute inset-0 bg-[url('/assets/bg-login-indigo.jpg')] bg-cover bg-center"
		aria-hidden="true"
	></div>
	<div class="absolute inset-0 bg-black/20" aria-hidden="true"></div>
	<div class="relative z-10 flex h-full w-full items-center justify-center p-8 text-center">
		<div class="max-w-lg space-y-4">
			<p class="font-sans text-xl leading-relaxed text-white/90">{m.app_tagline()}</p>
			<p class="font-sans text-sm leading-relaxed text-white/70">{m.dashboard_subtitle()}</p>
		</div>
	</div>
</aside>
