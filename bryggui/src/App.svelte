<script lang="ts" module>
    export interface BreweryState {
        phase: string;
        vessel_temp_top: number | null;
        vessel_temp_bottom: number | null;
        heater_power: number;
        pump_on: boolean;
        target_temperature: number | null;
        control_source: string;
        timestamp: number;
    }
</script>

<script lang="ts">
    import ControlPanel from "./ControlPanel.svelte";
    import TemperatureChart from "./TemperatureChart.svelte";
    import Vessel from "./Vessel.svelte";

    let brewery: BreweryState = $state({
        phase: "Idle",
        vessel_temp_top: null,
        vessel_temp_bottom: null,
        heater_power: 0,
        pump_on: false,
        target_temperature: null,
        control_source: "Average",
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

    function togglePump() {
        sendCommand({ SetPump: !brewery.pump_on });
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

<h1>bryggui {isConnected ? "" : "(disconnected)"}</h1>

<!-- Top row: vessel + chart -->
<div class="top-row">
    <Vessel {brewery} onTogglePump={togglePump} />
    <div class="chart-wrapper">
        <TemperatureChart {brewery} />
    </div>
</div>

<!-- Controls -->
<div class="controls">
    {#each phases as phase}
        <button
            class:active={brewery.phase === phase}
            onclick={() => sendCommand({ SetPhase: phase })}>{phase}</button
        >
    {/each}
</div>

<!-- Aux panel -->
<div class="aux-row">
    <ControlPanel {brewery} onCommand={sendCommand} />
    <div class="state-panel">
        <pre>{JSON.stringify(brewery, null, 2)}</pre>
    </div>
</div>
