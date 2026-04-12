<script lang="ts">
import type { BreweryState } from "./App.svelte";

let {
	brewery,
	onCommand,
}: { brewery: BreweryState; onCommand: (cmd: object) => void } = $props();

const controlSources = ["Top", "Bottom", "Average"];

type ControllerPreset = { label: string; cmd: object };
const controllerPresets: ControllerPreset[] = [
	{
		label: "PID",
		cmd: { SetController: { Pid: { kp: 0.5, ki: 0.01, kd: 0.002 } } },
	},
	{ label: "Manual", cmd: { SetController: "Manual" } },
];

let activeController = $state("PID");
let targetInput = $state(65);

function setController(preset: ControllerPreset) {
	activeController = preset.label;
	onCommand(preset.cmd);
}

function setTarget() {
	onCommand({ SetTarget: { temperature: targetInput } });
}

function setControlSource(source: string) {
	onCommand({ SetControlSource: source });
}
</script>

<div class="control-panel">
	<h3>Controller</h3>
	<div class="controls">
		{#each controllerPresets as preset}
			<button
				class:active={activeController === preset.label}
				onclick={() => setController(preset)}>{preset.label}</button
			>
		{/each}
	</div>
	<h3>Target</h3>
	<span class="target-input">
		<input
			type="number"
			bind:value={targetInput}
			min="0"
			max="100"
			step="0.5"
		/>
		<button onclick={setTarget}>Set</button>
	</span>
	<h3>Sensor source</h3>
	<div class="controls">
		{#each controlSources as source}
			<button
				class:active={brewery.control_source === source}
				onclick={() => setControlSource(source)}>{source}</button
			>
		{/each}
	</div>
</div>
