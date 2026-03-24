<script lang="ts">
import { Chart, LineController, LineElement, PointElement, LinearScale, Filler, Legend, Tooltip } from "chart.js";
import type { BreweryState } from "./App.svelte";

Chart.register(LineController, LineElement, PointElement, LinearScale, Filler, Legend, Tooltip);

const MAX_POINTS = 600;

interface HistoryPoint {
	time: number;
	tempTop: number | null;
	tempBottom: number | null;
	target: number | null;
	heaterPower: number;
}

let { brewery }: { brewery: BreweryState } = $props();

let canvas: HTMLCanvasElement;
let chart: Chart | undefined;
let history: HistoryPoint[] = [];
let startTime = 0;

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
					backgroundColor: "rgba(232, 168, 56, 0.15)",
					data: [],
					pointRadius: 0,
					borderWidth: 1,
					fill: true,
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
			},
			scales: {
				x: {
					type: "linear",
					title: { display: true, text: "seconds", color: "#888", font: { family: "monospace" } },
					ticks: { color: "#888", font: { family: "monospace" } },
					grid: { color: "rgba(255,255,255,0.05)" },
				},
				y: {
					type: "linear",
					position: "left",
					title: { display: true, text: "°C", color: "#888", font: { family: "monospace" } },
					ticks: { color: "#888", font: { family: "monospace" } },
					grid: { color: "rgba(255,255,255,0.08)" },
				},
				y1: {
					type: "linear",
					position: "right",
					min: 0,
					max: 1,
					title: { display: true, text: "heater", color: "#888", font: { family: "monospace" } },
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

$effect(() => {
	createChart();
	return () => chart?.destroy();
});

$effect(() => {
	// Access reactive state
	const state = brewery;
	if (!chart) return;

	if (startTime === 0) startTime = state.timestamp;
	const t = state.timestamp - startTime;

	history.push({
		time: t,
		tempTop: state.vessel_temp_top,
		tempBottom: state.vessel_temp_bottom,
		target: state.target_temperature,
		heaterPower: state.heater_power,
	});
	if (history.length > MAX_POINTS) history.shift();

	const toXY = (getValue: (p: HistoryPoint) => number | null) =>
		history.flatMap((p) => {
			const v = getValue(p);
			return v !== null ? [{ x: p.time, y: v }] : [];
		});

	chart.data.datasets[0].data = toXY((p) => p.tempTop);
	chart.data.datasets[1].data = toXY((p) => p.tempBottom);
	chart.data.datasets[2].data = toXY((p) => p.target);
	chart.data.datasets[3].data = toXY((p) => p.heaterPower);
	chart.update("none");
});
</script>

<div class="chart-container">
	<canvas bind:this={canvas}></canvas>
</div>
