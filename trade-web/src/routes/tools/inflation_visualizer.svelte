<script lang="ts">
	import { Grid, Row, Column, NumberInput, Slider, Button } from 'carbon-components-svelte';
	import '@carbon/charts/styles.min.css';

	import { AreaChart } from '@carbon/charts-svelte';

	let baseValue = 1000.0;
	let inflationRate = 7.4;
	let data: any = [];

	function calculateInflation() {
		const predictionYears = 20;
		let monthlyInflationRate = inflationRate / 100.0 / 12.0;

		let inflationData = [];
		let currentDate = new Date();
		let currentValue = baseValue;

		let inflation = 0.0;
		for (let i = 0; i < 12 * predictionYears; i++) {
			/*
      inflationData.push({
        group: "Inflation",
        date: currentDate.getTime(),
        value: inflation,
      });*/
			inflationData.push({
				group: 'Value with inflation',
				date: currentDate.getTime(),
				value: currentValue
			});
			inflationData.push({
				group: 'Baseline',
				date: currentDate.getTime(),
				value: baseValue
			});
			currentDate.setMonth(currentDate.getMonth() + 1);

			inflation = currentValue * monthlyInflationRate;
			currentValue = currentValue - inflation;
		}

		data = inflationData;
	}
</script>

<Grid>
	<Row>
		<NumberInput
			helperText="Optional helper text"
			id="tj-input"
			invalidText="Number is not valid"
			label="Number input label"
			min={0}
			step={1000}
			bind:value={baseValue}
		/>
		<Slider
			id="slider"
			labelText="Inflation Rate in %"
			max={100}
			min={0}
			step={0.1}
			stepMultiplier={0.4}
			value={inflationRate}
		/>
		<Button on:click={calculateInflation}>Calculate Inflation</Button>
	</Row>
	<Row>
		<Column>
			<AreaChart
				bind:data
				options={{
					title: 'Inflation',
					axes: {
						// @ts-ignore
						left: { visible: true, mapsTo: 'value', scaleType: 'linear' },
						// @ts-ignore
						bottom: { visible: true, mapsTo: 'date', scaleType: 'time' }
					}
				}}
			/>
		</Column>
	</Row>
</Grid>
