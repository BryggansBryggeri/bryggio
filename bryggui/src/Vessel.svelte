<script lang="ts">
import type { BreweryState } from "./App.svelte";

let {
	brewery,
	onTogglePump,
}: { brewery: BreweryState; onTogglePump: () => void } = $props();

function formatTemp(t: number | null): string {
	return t !== null ? `${t.toFixed(1)} °C` : "--.- °C";
}
</script>

<svg class="vessel-svg" viewBox="0 0 400 300">
	<!-- Vessel body -->
	<rect
		id="vessel"
		x="100"
		y="40"
		width="200"
		height="200"
		rx="10"
		fill="none"
		stroke="#666"
		stroke-width="2"
	/>

	<!-- Heater at bottom -->
	<line
		id="vessel-heater"
		x1="130"
		y1="220"
		x2="270"
		y2="220"
		stroke={brewery.heater_power > 0 ? "#e8a838" : "#555"}
		stroke-width="4"
		stroke-linecap="round"
	/>

	<!-- Top temp sensor -->
	<circle
		id="vessel-temp-top"
		cx="160"
		cy="90"
		r="6"
		fill={brewery.vessel_temp_top !== null ? "#4a9" : "#555"}
	/>
	<text
		id="vessel-temp-top-value"
		x="175"
		y="95"
		font-family="monospace"
		font-size="14"
		fill="#e0e0e0"
	>
		{formatTemp(brewery.vessel_temp_top)}
	</text>

	<!-- Bottom temp sensor -->
	<circle
		id="vessel-temp-bottom"
		cx="160"
		cy="190"
		r="6"
		fill={brewery.vessel_temp_bottom !== null ? "#4a9" : "#555"}
	/>
	<text
		id="vessel-temp-bottom-value"
		x="175"
		y="195"
		font-family="monospace"
		font-size="14"
		fill="#e0e0e0"
	>
		{formatTemp(brewery.vessel_temp_bottom)}
	</text>

	<!-- Target temperature -->
	<text x="310" y="140" font-family="monospace" font-size="12" fill="#aaa">
		Target: {brewery.target_temperature !== null
			? `${brewery.target_temperature.toFixed(1)} °C`
			: "not set"}
	</text>

	<!-- Labels -->
	<text
		x="200"
		y="30"
		text-anchor="middle"
		font-family="monospace"
		font-size="12"
		fill="#888">VESSEL</text
	>

	<!-- Pump indicator (clickable) -->
	<g class="pump-toggle" role="button" tabindex="0" onclick={onTogglePump} onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") onTogglePump(); }}>
		<circle
			id="pump"
			cx="50"
			cy="140"
			r="15"
			fill={brewery.pump_on ? "#4a9" : "#555"}
			stroke="#666"
			stroke-width="1"
		/>
		<text
			x="50"
			y="175"
			text-anchor="middle"
			font-family="monospace"
			font-size="10"
			fill="#888">PUMP</text
		>
	</g>
</svg>
