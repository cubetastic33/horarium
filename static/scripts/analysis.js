var ctx = document.getElementById('analysisChart').getContext('2d');

var datasets = [{
		label: 'Hours last week',
		data: aggregate_times_last_week,
		backgroundColor: [
			'#66bb6a',
			'#c75b39',
			'#2286c3',
			'#cabf45',
			'#b4004e',
			'#7e57c2',
			'#ba6b6c'
		],
		borderColor: [
			'#98ee99',
			'#ff8a65',
			'#64b5f6',
			'#fff176',
			'#ec407a',
			'#b085f5',
			'#ef9a9a'
		],
		borderWidth: 2
	}, {
		label: 'Hours this week',
		data: aggregate_times_this_week,
		backgroundColor: [
			'#98ee99',
			'#ff8a65',
			'#64b5f6',
			'#fff176',
			'#ec407a',
			'#b085f5',
			'#ef9a9a'
		],
		borderColor: [
			'#66bb6a',
			'#c75b39',
			'#2286c3',
			'#cabf45',
			'#b4004e',
			'#7e57c2',
			'#ba6b6c'
		],
		borderWidth: 2
	}];

var comparisonChart = new Chart(ctx, {
	type: 'bar',
	data: {
		labels: subjects,
		datasets: datasets,
	},
	options: {
		aspectRatio: 1.6,
		scales: {
			yAxes: [{
				ticks: {
					beginAtZero: true
				}
			}]
		}
	}
});

var individualChart = new Chart($('#thisWeekAnalysisChart'), {
	type: 'pie',
	data: {
		labels: subjects,
		datasets: [datasets[1]]
	},
	options: {
		legend: {
			display: false
		}
	}
});
