// Global variable to store the classifier
let classifier;

// Label
let label = 'listening...';

// Teachable Machine model URL:
let soundModelUrl = 'http://127.0.0.1:5000/static/model/'; // for some reason the sound req remote model but image doesn't...

let bark = new Audio("/static/bark.opus");
let song = new Audio("/static/song.opus");

let shouldRun = false;

async function preload() {
    // Load the model
    classifier = ml5.soundClassifier(soundModelUrl + 'model.json');
}

function setup() {
    createCanvas(320, 240);
  fill(255);
  textSize(32);
  textAlign(CENTER, CENTER);
  text(label, width / 2, height / 2);
    // Start classifying
    // The sound model will continuously listen to the microphone
    classifier.classify(gotResult);
}

function draw() {
    if (shouldRun && label === "Car") {
        letTheDogsOut();
    }
}

// The model recognizing a sound will trigger this event
function gotResult(error, results) {
    if (!shouldRun) {
        return;
    } else if (error) {
        console.error(error);
        return;
    }
    // The results are in an array ordered by confidence.
    // console.log(results[0]);
    label = results[0].label;
}

function letTheDogsOut() {
    bark.play();
    song.play();

    shouldRun = false;

    document.getElementById("alert").style.display = "block";
    document.getElementsByClassName("body")[0].style.background = "black";
}