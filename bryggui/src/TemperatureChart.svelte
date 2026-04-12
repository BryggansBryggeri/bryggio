<script lang="ts">
import {
	Chart,
	Filler,
	Legend,
	LinearScale,
	LineController,
	LineElement,
	PointElement,
	Tooltip,
} from "chart.js";
import type { BreweryState } from "./App.svelte";

Chart.register(
	LineController,
	LineElement,
	PointElement,
	LinearScale,
	Filler,
	Legend,
	Tooltip,
);

/** Format elapsed seconds as mm:ss or h:mm once past an hour. */
function formatElapsed(seconds: number): string {
	const s = Math.round(seconds);
	if (s < 3600) {
		const mm = Math.floor(s / 60);
		const ss = s % 60;
		return `${mm}:${String(ss).padStart(2, "0")}`;
	}
	const h = Math.floor(s / 3600);
	const mm = Math.floor((s % 3600) / 60);
	return `${h}:${String(mm).padStart(2, "0")}`;
}

/** Format a unix epoch as HH:MM:SS wall-clock time. */
function formatWallClock(epoch: number): string {
	const d = new Date(epoch * 1000);
	return d.toLocaleTimeString();
}

interface HistoryPoint {
	time: number;
	tempTop: number | null;
	tempBottom: number | null;
	target: number | null;
	heaterPower: number;
	pumpOn: boolean;
}

let { brewery }: { brewery: BreweryState } = $props();

let canvas: HTMLCanvasElement;
let chart: Chart | undefined;
let history: HistoryPoint[] = [];
let startTime = 0;
let lastDbTimestamp = 0;
let dbLoaded = $state(false);

function createChart() {
	chart = new Chart(canvas, {
		type: "line",
		data: {
			datasets: [
				{
					label: "Top temp",
					borderColor: "#4a9",
					backgroundColor: "#4a9",
					data: [],
					pointRadius: 0,
					borderWidth: 2,
					yAxisID: "y",
				},
				{
					label: "Bottom temp",
					borderColor: "#4a9fff",
					backgroundColor: "#4a9fff",
					data: [],
					pointRadius: 0,
					borderWidth: 2,
					yAxisID: "y",
				},
				{
					label: "Target",
					borderColor: "#e8a838",
					backgroundColor: "#e8a838",
					data: [],
					pointRadius: 0,
					borderWidth: 2,
					borderDash: [6, 3],
					yAxisID: "y",
				},
				{
					label: "Heater power",
					borderColor: "rgba(232, 168, 56, 0.6)",
					data: [],
					pointRadius: 0,
					borderWidth: 1,
					borderDash: [4, 3],
					yAxisID: "y1",
				},
				{
					label: "Pump",
					borderColor: "rgba(100, 180, 255, 0.8)",
					data: [],
					pointRadius: 0,
					borderWidth: 1,
					borderDash: [4, 3],
					stepped: true,
					yAxisID: "y1",
				},
			],
		},
		options: {
			animation: false,
			responsive: true,
			maintainAspectRatio: false,
			interaction: { mode: "nearest", axis: "x", intersect: false },
			plugins: {
				legend: {
					labels: { color: "#e0e0e0", font: { family: "monospace", size: 11 } },
				},
				tooltip: {
					callbacks: {
						title: (items: { parsed: { x: number | null } }[]) => {
							const first = items[0];
							const x = first?.parsed.x;
							if (x == null) return "";
							const elapsed = formatElapsed(x);
							const wall = formatWallClock(startTime + x);
							return `${elapsed}  (${wall})`;
						},
					},
				},
			},
			scales: {
				x: {
					type: "linear",
					title: {
						display: true,
						text: "elapsed",
						color: "#888",
						font: { family: "monospace" },
					},
					ticks: {
						color: "#888",
						font: { family: "monospace" },
						callback: (v: string | number) => formatElapsed(Number(v)),
					},
					grid: { color: "rgba(255,255,255,0.05)" },
				},
				y: {
					type: "linear",
					position: "right",
					min: 0,
					max: 100,
					title: { display: false },
					ticks: {
						color: "#888",
						font: { family: "monospace" },
						callback: (v: string | number) => `${v}°C`,
					},
					grid: { color: "rgba(255,255,255,0.08)" },
				},
				y1: {
					type: "linear",
					position: "left",
					min: 0,
					max: 1,
					title: {
						display: true,
						text: "power / pump",
						color: "#888",
						font: { family: "monospace" },
					},
					ticks: {
						color: "#888",
						font: { family: "monospace" },
						callback: (v: string | number) => `${Number(v) * 100}%`,
					},
					grid: { drawOnChartArea: false },
				},
			},
		},
	});
}

function stateToPoint(state: BreweryState): HistoryPoint {
	return {
		time: state.timestamp - startTime,
		tempTop: state.vessel_temp_top,
		tempBottom: state.vessel_temp_bottom,
		target: state.target_temperature,
		heaterPower: state.heater_power,
		pumpOn: state.pump_on,
	};
}

function updateChart() {
	if (!chart) return;
	const toXY = (getValue: (p: HistoryPoint) => number | null) =>
		history.flatMap((p) => {
			const v = getValue(p);
			return v !== null ? [{ x: p.time, y: v }] : [];
		});

	chart.data.datasets[0].data = toXY((p) => p.tempTop);
	chart.data.datasets[1].data = toXY((p) => p.tempBottom);
	chart.data.datasets[2].data = toXY((p) => p.target);
	chart.data.datasets[3].data = toXY((p) => p.heaterPower);
	chart.data.datasets[4].data = toXY((p) => (p.pumpOn ? 1 : 0));
	chart.update("none");
}

async function loadHistory() {
	try {
		const res = await fetch("/readings");
		if (!res.ok) return;
		const states: BreweryState[] = await res.json();
		if (states.length === 0) return;

		startTime = states[0].timestamp;
		lastDbTimestamp = states[states.length - 1].timestamp;
		history = states.map(stateToPoint);
		updateChart();
	} finally {
		dbLoaded = true;
	}
}

$effect(() => {
	createChart();
	loadHistory();
	return () => chart?.destroy();
});

$effect(() => {
	const state = brewery;
	if (!chart || !dbLoaded) return;
	if (state.timestamp <= lastDbTimestamp) return;

	if (startTime === 0) startTime = state.timestamp;
	history.push(stateToPoint(state));
	updateChart();
});
</script>

<div class="chart-container">
	<canvas bind:this={canvas}></canvas>
</div>
