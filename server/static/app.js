console.log("app.js loaded");

const socket = new WebSocket(
    `ws://${window.location.host}/ws`
);

socket.onopen = () => {
    console.log("Connected");
};

socket.onclose = () => {
    console.log("Disconnected");
};

socket.onerror = (error) => {
    console.error("WebSocket error:", error);
};

socket.onmessage = (event) => {
    const reading = JSON.parse(event.data);

    switch (reading.sensor.type) {

        case "Temperature":
            updateTemperatureChart(
                reading.timestamp,
                reading.sensor.value
            );
            break;

        case "Humidity":
            updateHumidityChart(
                reading.timestamp,
                reading.sensor.value
            );
            break;

        case "Pressure":
            updatePressureChart(
                reading.timestamp,
                reading.sensor.value
            );
            break;

        case "GasResistance":
            updateGasResistanceChart(
                reading.timestamp,
                reading.sensor.profile,
                reading.sensor.resistance
            );
            break;

        case "CO2":
            updateCO2Chart(
                reading.timestamp,
                reading.sensor.value
            );
            break;

        case "Lux": 
            updateLuxChart(
                reading.timestamp,
                reading.sensor.value
            );
            break;

        case "Noise":
            updateNoiseChart(
                reading.timestamp,
                reading.sensor.value
            );
            break;
    }
};

const temperatureChart = new Chart(
    document.getElementById("temperature-chart"),
    {
        type: "line",

        data: {
            datasets: [{
                label: "Temperature",
                data: [],
                borderWidth: 2,
                pointRadius: 0
            }]
        },

        options: {
            animation: false,

            parsing: false,

            scales: {
                x: {
                    type: 'linear',
                    min: Date.now() - 60_000,
                    max: Date.now(),
                    ticks: {
                        stepSize: 5000,
                        callback: (value) => `${Math.floor((Date.now() - value) / 1000)}s`
                    },
                    title: {
                        display: true,
                        text: 'Seconds ago'
                    }
                },

                y: {
                    type: 'linear',
                    title: {
                        display: true,
                        text: "Temperature (°C)"
                    },
                }
            },

            plugins: {
                legend: {
                    display: false
                }
            }
        }
    }
);

function updateTemperatureChart(timestamp, value) {

    const now = timestamp;
    const cutoff = now - 60_000;
    const dataset = temperatureChart.data.datasets[0];
    
    dataset.data.push({
        x: now,
        y: value
    });
    dataset.data = dataset.data.filter(point => point.x >= cutoff);

    const values = dataset.data.map(v => v.y);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const padding = 1;

    temperatureChart.options.scales.x.min = cutoff;
    temperatureChart.options.scales.x.max = now;
    temperatureChart.options.scales.y.min = Math.floor(min - padding);
    temperatureChart.options.scales.y.max = Math.ceil(max + padding);
    temperatureChart.update("none");
}

const humidityChart = new Chart(
    document.getElementById("humidity-chart"),
    {
        type: "line",

        data: {
            datasets: [{
                label: "Humidity",
                data: [],
                borderWidth: 2,
                pointRadius: 0
            }]
        },

        options: {
            animation: false,

            parsing: false,

            scales: {
                x: {
                    type: 'linear',
                    min: Date.now() - 60_000,
                    max: Date.now(),
                    ticks: {
                        stepSize: 5000,
                        callback: (value) => `${Math.floor((Date.now() - value) / 1000)}s`
                    },
                    title: {
                        display: true,
                        text: 'Seconds ago'
                    }
                },

                y: {
                    type: 'linear',
                    title: {
                        display: true,
                        text: "Humidity (%)"
                    },
                }
            },

            plugins: {
                legend: {
                    display: false
                }
            }
        }
    }
);

function updateHumidityChart(timestamp, value) {

    const now = timestamp;
    const cutoff = now - 60_000;
    const dataset = humidityChart.data.datasets[0];
    
    dataset.data.push({
        x: now,
        y: value
    });
    dataset.data = dataset.data.filter(point => point.x >= cutoff);

    const values = dataset.data.map(v => v.y);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const padding = 1;

    humidityChart.options.scales.x.min = cutoff;
    humidityChart.options.scales.x.max = now;
    humidityChart.options.scales.y.min = Math.floor(min - padding);
    humidityChart.options.scales.y.max = Math.ceil(max + padding);
    humidityChart.update("none");
}

const pressureChart = new Chart(
    document.getElementById("pressure-chart"),
    {
        type: "line",

        data: {
            datasets: [{
                label: "Pressure",
                data: [],
                borderWidth: 2,
                pointRadius: 0
            }]
        },

        options: {
            animation: false,

            parsing: false,

            scales: {
                x: {
                    type: 'linear',
                    min: Date.now() - 60_000,
                    max: Date.now(),
                    ticks: {
                        stepSize: 5000,
                        callback: (value) => `${Math.floor((Date.now() - value) / 1000)}s`
                    },
                    title: {
                        display: true,
                        text: 'Seconds ago'
                    }
                },

                y: {
                    type: 'linear',

                    title: {
                        display: true,
                        text: "Pressure (hPa)"
                    },
                }
            },

            plugins: {
                legend: {
                    display: false
                }
            }
        }
    }
);

function updatePressureChart(timestamp, value) {

    const now = timestamp;
    const cutoff = now - 60_000;
    const dataset = pressureChart.data.datasets[0];
    
    dataset.data.push({
        x: now,
        y: value / 100.0 // Convert Pa to hPa
    });
    dataset.data = dataset.data.filter(point => point.x >= cutoff);

    const values = dataset.data.map(v => v.y);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const padding = 1;

    pressureChart.options.scales.x.min = cutoff;
    pressureChart.options.scales.x.max = now;
    pressureChart.options.scales.y.min = Math.floor(min - padding);
    pressureChart.options.scales.y.max = Math.ceil(max + padding);
    pressureChart.update("none");
}

const luxChart = new Chart(
    document.getElementById("lux-chart"),
    {
        type: "line",

        data: {
            datasets: [{
                label: "Lux",
                data: [],
                borderWidth: 2,
                pointRadius: 0
            }]
        },

        options: {
            animation: false,

            parsing: false,

            scales: {
                x: {
                    type: 'linear',
                    min: Date.now() - 60_000,
                    max: Date.now(),
                    ticks: {
                        stepSize: 5000,
                        callback: (value) => `${Math.floor((Date.now() - value) / 1000)}s`
                    },
                    title: {
                        display: true,
                        text: 'Seconds ago'
                    }
                },

                y: {
                    type: 'linear',
                    title: {
                        display: true,
                        text: "Lux (lux)"
                    },
                }
            },

            plugins: {
                legend: {
                    display: false
                }
            }
        }
    }
);

function updateLuxChart(timestamp, value) {

    const now = timestamp;
    const cutoff = now - 60_000;
    const dataset = luxChart.data.datasets[0];
    
    dataset.data.push({
        x: now,
        y: value
    });
    dataset.data = dataset.data.filter(point => point.x >= cutoff);

    const values = dataset.data.map(v => v.y);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const padding = 5;

    luxChart.options.scales.x.min = cutoff;
    luxChart.options.scales.x.max = now;
    luxChart.options.scales.y.min = Math.floor(min - padding);
    luxChart.options.scales.y.max = Math.ceil(max + padding);
    luxChart.update("none");
}

const co2Chart = new Chart(
    document.getElementById("co2-chart"),
    {
        type: "line",

        data: {
            datasets: [{
                label: "CO2",
                data: [],
                borderWidth: 2,
                pointRadius: 0
            }]
        },

        options: {
            animation: false,

            parsing: false,

            scales: {
                x: {
                    type: 'linear',
                    min: Date.now() - 60_000,
                    max: Date.now(),
                    ticks: {
                        stepSize: 5000,
                        callback: (value) => `${Math.floor((Date.now() - value) / 1000)}s`
                    },
                    title: {
                        display: true,
                        text: 'Seconds ago'
                    }
                },

                y: {
                    type: 'linear',
                    title: {
                        display: true,
                        text: "CO2 (ppm)"
                    },
                }
            },

            plugins: {
                legend: {
                    display: false
                }
            }
        }
    }
);

function updateCO2Chart(timestamp, value) {

    const now = timestamp;
    const cutoff = now - 60_000;
    const dataset = co2Chart.data.datasets[0];
    
    dataset.data.push({
        x: now,
        y: value
    });
    dataset.data = dataset.data.filter(point => point.x >= cutoff);

    const values = dataset.data.map(v => v.y);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const padding = 2;

    co2Chart.options.scales.x.min = cutoff;
    co2Chart.options.scales.x.max = now;
    co2Chart.options.scales.y.min = Math.floor(min - padding);
    co2Chart.options.scales.y.max = Math.ceil(max + padding);
    co2Chart.update("none");
}

const profileColors = [
    "#e6194b", "#3cb44b", "#ffe119", "#4363d8", "#f58231",
    "#911eb4", "#42d4f4", "#f032e6", "#bfef45", "#fabed4"
];

const gasResistanceChart = new Chart(
    document.getElementById("gas-resistance-chart"),
    {
        type: "line",

        data: {
            datasets: profileColors.map((color, i) => ({
                label: `Profile ${i}`,
                data: [],
                borderColor: color,
                backgroundColor: color,
                borderWidth: 2,
                pointRadius: 2
            }))
        },

        options: {
            animation: false,

            parsing: false,

            scales: {
                x: {
                    type: 'linear',
                    min: Date.now() - 60_000,
                    max: Date.now(),
                    ticks: {
                        stepSize: 5000,
                        callback: (value) => `${Math.floor((Date.now() - value) / 1000)}s`
                    },
                    title: {
                        display: true,
                        text: 'Seconds ago'
                    }
                },

                y: {
                    type: 'logarithmic',
                    title: {
                        display: true,
                        text: "Gas Resistance (kΩ)"
                    },
                }
            },

            plugins: {
                legend: {
                    display: true
                }
            }
        }
    }
);

function updateGasResistanceChart(timestamp, profile, resistance) {

    const now = timestamp;
    const cutoff = now - 60_000;
    
    // Prune old data across all profile datasets
    gasResistanceChart.data.datasets.forEach(ds => {
        ds.data = ds.data.filter(point => point.x >= cutoff);
    });

    if (gasResistanceChart.data.datasets[profile]) {
        gasResistanceChart.data.datasets[profile].data.push({
            x: now,
            y: resistance / 1000.0 // Convert Ω to kΩ
        });
    }

    gasResistanceChart.options.scales.x.min = cutoff;
    gasResistanceChart.options.scales.x.max = now;
    gasResistanceChart.update("none");
}

const noiseChart = new Chart(
    document.getElementById("noise-chart"),
    {
        type: "line",

        data: {
            datasets: [{
                label: "Noise Level",
                data: [],
                borderWidth: 2,
                pointRadius: 0
            }]
        },

        options: {
            animation: false,

            parsing: false,

            scales: {
                x: {
                    type: 'linear',
                    min: Date.now() - 60_000,
                    max: Date.now(),
                    ticks: {
                        stepSize: 5000,
                        callback: (value) => `${Math.floor((Date.now() - value) / 1000)}s`
                    },
                    title: {
                        display: true,
                        text: 'Seconds ago'
                    }
                },

                y: {
                    type: 'logarithmic',
                    title: {
                        display: true,
                        text: "Noise Level (dB)"
                    },
                }
            },

            plugins: {
                legend: {
                    display: false
                }
            }
        }
    }
);

function updateNoiseChart(timestamp, value) {

    const now = timestamp;
    const cutoff = now - 60_000;
    const dataset = noiseChart.data.datasets[0];
    
    dataset.data.push({
        x: now,
        y: value
    });
    dataset.data = dataset.data.filter(point => point.x >= cutoff);

    const values = dataset.data.map(v => v.y);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const padding = 1;

    noiseChart.options.scales.x.min = cutoff;
    noiseChart.options.scales.x.max = now;
    noiseChart.options.scales.y.min = Math.floor(min - padding);
    noiseChart.options.scales.y.max = Math.ceil(max + padding);
    noiseChart.update("none");
}