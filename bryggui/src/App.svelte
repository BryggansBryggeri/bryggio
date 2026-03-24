<script lang="ts">
interface BreweryState {
	phase: string;
	vessel_temp_top: number | null;
	vessel_temp_bottom: number | null;
	heater_power: number;
	pump_on: boolean;
	timestamp: number;
}

let brewery: BreweryState = $state({
	phase: "Idle",
	vessel_temp_top: null,
	vessel_temp_bottom: null,
	heater_power: 0,
	pump_on: false,
	timestamp: 0,
});

let isConnected = $state(false);

$effect(() => {
	const es = new EventSource("/events");
	es.onmessage = (e) => {
		brewery = JSON.parse(e.data);
		isConnected = true;
	};
	es.onerror = () => {
		isConnected = false;
	};
	return () => es.close();
});

function sendCommand(cmd: object) {
	fetch("/command", {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(cmd),
	});
}

function setPhase(phase: string) {
	sendCommand({ SetPhase: phase });
}

function togglePump() {
	sendCommand({ SetPump: !brewery.pump_on });
}

function formatTemp(t: number | null): string {
	return t !== null ? `${t.toFixed(1)} °C` : "--.- °C";
}

const phases = [
	"Idle",
	"Prep",
	"Mashing",
	"Lautering",
	"Boiling",
	"Cooling",
	"Done",
];
</script>

<h1>bryggui {isConnected ? '' : '(disconnected)'}</h1>

<!-- Dumb SVG vessel -->
<svg class="vessel-svg" viewBox="0 0 400 300" width="400" height="300">
  <!-- Vessel body -->
  <rect id="vessel" x="100" y="40" width="200" height="200" rx="10"
    fill="none" stroke="#666" stroke-width="2" />

  <!-- Heater at bottom -->
  <line id="vessel-heater" x1="130" y1="220" x2="270" y2="220"
    stroke={brewery.heater_power > 0 ? '#e8a838' : '#555'}
    stroke-width="4" stroke-linecap="round" />

  <!-- Top temp sensor -->
  <circle id="vessel-temp-top" cx="160" cy="90" r="6"
    fill={brewery.vessel_temp_top !== null ? '#4a9' : '#555'} />
  <text id="vessel-temp-top-value" x="175" y="95"
    font-family="monospace" font-size="14" fill="#e0e0e0">
    {formatTemp(brewery.vessel_temp_top)}
  </text>

  <!-- Bottom temp sensor -->
  <circle id="vessel-temp-bottom" cx="160" cy="190" r="6"
    fill={brewery.vessel_temp_bottom !== null ? '#4a9' : '#555'} />
  <text id="vessel-temp-bottom-value" x="175" y="195"
    font-family="monospace" font-size="14" fill="#e0e0e0">
    {formatTemp(brewery.vessel_temp_bottom)}
  </text>

  <!-- Labels -->
  <text x="200" y="30" text-anchor="middle" font-family="monospace"
    font-size="12" fill="#888">VESSEL</text>

  <!-- Pump indicator -->
  <circle id="pump" cx="50" cy="140" r="15"
    fill={brewery.pump_on ? '#4a9' : '#555'} stroke="#666" stroke-width="1" />
  <text x="50" y="175" text-anchor="middle" font-family="monospace"
    font-size="10" fill="#888">PUMP</text>
</svg>

<!-- Phase controls -->
<div class="controls">
  {#each phases as phase}
    <button class:active={brewery.phase === phase}
      onclick={() => setPhase(phase)}>{phase}</button>
  {/each}
  <button class:active={brewery.pump_on}
    onclick={togglePump}>Pump {brewery.pump_on ? 'ON' : 'OFF'}</button>
</div>

<!-- Raw state -->
<div class="state-panel">
  <pre>{JSON.stringify(brewery, null, 2)}</pre>
</div>
