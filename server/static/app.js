// ---------------------------------------------------------------------------
// Connection status
// ---------------------------------------------------------------------------

const connectionDot = document.getElementById("connection-dot");
const connectionLabel = document.getElementById("connection-label");

function setConnectionState(state) {
    const colors = {
        connecting: "bg-slate-600",
        open: "bg-emerald-500",
        closed: "bg-rose-500",
    };
    const labels = {
        connecting: "Connecting…",
        open: "Connected",
        closed: "Disconnected",
    };
    connectionDot.className = `w-2.5 h-2.5 rounded-full ${colors[state]}`;
    connectionLabel.textContent = labels[state];
}

const socket = new WebSocket(`ws://${window.location.host}/ws`);

socket.onopen = () => setConnectionState("open");
socket.onclose = () => setConnectionState("closed");
socket.onerror = (error) => console.error("WebSocket error:", error);

socket.onmessage = (event) => {
    const qualityScore = JSON.parse(event.data);
    handleQualityScore(qualityScore);
};

// ---------------------------------------------------------------------------
// Presentation helpers
// ---------------------------------------------------------------------------

function scoreColor(score) {
    if (score >= 80) return "#34d399"; // emerald-400
    if (score >= 50) return "#facc15"; // yellow-400
    return "#fb7185"; // rose-400
}

function overallLabel(score) {
    if (score >= 90) return "Excellent";
    if (score >= 80) return "Good";
    if (score >= 60) return "Fair";
    if (score >= 40) return "Poor";
    return "Bad";
}

// ---------------------------------------------------------------------------
// Sensor card definitions
//
// Each field matches a `SubScore` in the `QualityScore` message broadcast by
// the server (see proto/scores.proto and server/src/domain/mod.rs).
// ---------------------------------------------------------------------------

const ICONS = {
    temperature: '<svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" class="w-full h-full"><path stroke-linecap="round" stroke-linejoin="round" d="M15.362 5.214A8.252 8.252 0 0112 21 8.25 8.25 0 016.038 7.048 8.287 8.287 0 009 9.601a8.983 8.983 0 013.361-6.867 8.21 8.21 0 003 2.48z" /><path stroke-linecap="round" stroke-linejoin="round" d="M12 12.75a2.25 2.25 0 100 4.5 2.25 2.25 0 000-4.5z" /></svg>',
    humidity: '<svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" class="w-full h-full"><path stroke-linecap="round" stroke-linejoin="round" d="M12 21a7.5 7.5 0 007.5-7.5c0-4.142-7.5-11.25-7.5-11.25S4.5 9.358 4.5 13.5A7.5 7.5 0 0012 21z" /></svg>',
    co2: '<svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" class="w-full h-full"><path stroke-linecap="round" stroke-linejoin="round" d="M2.25 15a4.5 4.5 0 004.5 4.5H18a3.75 3.75 0 001.332-7.257 3 3 0 00-3.758-3.848 5.25 5.25 0 00-10.233 2.33A4.502 4.502 0 002.25 15z" /></svg>',
    illuminance: '<svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" class="w-full h-full"><path stroke-linecap="round" stroke-linejoin="round" d="M12 3v2.25m6.364.386-1.591 1.591M21 12h-2.25m-.386 6.364-1.591-1.591M12 18.75V21m-4.773-4.227-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z" /></svg>',
    noise: '<svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" class="w-full h-full"><path stroke-linecap="round" stroke-linejoin="round" d="M19.114 5.636a9 9 0 010 12.728M16.463 8.288a5.25 5.25 0 010 7.424M6.75 8.25l4.72-4.72a.75.75 0 011.28.53v15.88a.75.75 0 01-1.28.53l-4.72-4.72H4.51c-.88 0-1.704-.507-1.938-1.354A9.01 9.01 0 012.25 12c0-.83.112-1.633.322-2.396C2.806 8.756 3.63 8.25 4.51 8.25H6.75z" /></svg>',
};

const SUBSCORES = {
    temperature: { label: "Temperature", unit: "°C" },
    humidity: { label: "Humidity", unit: "%" },
    co2: { label: "CO2", unit: "ppm" },
    illuminance: { label: "Illumination", unit: "lux" },
    noise: { label: "Noise", unit: "dB" },
};

// ---------------------------------------------------------------------------
// Card creation
// ---------------------------------------------------------------------------

const scoreCharts = {}; // field -> Chart instance

function makeDoughnut(canvas, initialScore) {
    return new Chart(canvas, {
        type: "doughnut",
        data: {
            datasets: [{
                data: [initialScore, 100 - initialScore],
                backgroundColor: [scoreColor(initialScore), "#1e293b"],
                borderWidth: 0,
            }],
        },
        options: {
            cutout: "72%",
            animation: { duration: 300 },
            plugins: { legend: { display: false }, tooltip: { enabled: false } },
        },
    });
}

function buildScoreCards() {
    const container = document.getElementById("score-cards");
    const template = document.getElementById("sensor-card-template");

    for (const [field, def] of Object.entries(SUBSCORES)) {
        const node = template.content.cloneNode(true);
        const root = node.querySelector("div");
        root.id = `card-${field}`;
        node.querySelector(".sensor-icon").innerHTML = ICONS[field];
        node.querySelector(".sensor-name").textContent = def.label;
        node.querySelector(".sensor-value").id = `value-${field}`;
        node.querySelector(".sensor-score-value").id = `score-${field}`;

        container.appendChild(node);

        scoreCharts[field] = makeDoughnut(
            document.getElementById(`card-${field}`).querySelector(".sensor-doughnut"),
            0
        );
    }
}

buildScoreCards();

const overallDoughnut = makeDoughnut(document.getElementById("overall-doughnut"), 0);

// ---------------------------------------------------------------------------
// Quality score handling
// ---------------------------------------------------------------------------

function updateSubScore(field, subScore) {
    const def = SUBSCORES[field];
    const { score, measurement } = subScore;

    document.getElementById(`value-${field}`).textContent = `${measurement.toFixed(1)} ${def.unit}`;
    document.getElementById(`score-${field}`).textContent = Math.round(score);

    const chart = scoreCharts[field];
    const ds = chart.data.datasets[0];
    ds.data = [score, 100 - score];
    ds.backgroundColor = [scoreColor(score), "#1e293b"];
    chart.update("none");
}

function updateOverallScore(score) {
    document.getElementById("overall-score-value").textContent = Math.round(score);
    document.getElementById("overall-score-label").textContent = overallLabel(score);

    const ds = overallDoughnut.data.datasets[0];
    ds.data = [score, 100 - score];
    ds.backgroundColor = [scoreColor(score), "#1e293b"];
    overallDoughnut.update("none");
}

function handleQualityScore(qualityScore) {
    for (const field of Object.keys(SUBSCORES)) {
        if (qualityScore[field]) {
            updateSubScore(field, qualityScore[field]);
        }
    }
    updateOverallScore(qualityScore.score);
}
