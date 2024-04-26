const wsm = require("ws");
const fs = require("fs");

const readFileLines = (filename) =>
  fs.readFileSync(filename).toString("UTF8").split("\n");

let keys = readFileLines("keys.txt");
let values = readFileLines("values.txt");

const ws = new wsm.WebSocket("ws://127.0.0.1:3000/ws?password=balls");

function sleep(ms = 0) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

ws.on("error", console.error);

ws.on("open", async function open() {
  let timeAvg = 0;

  for (let i = 0; i <= 100; i++) {
    let startTime = performance.now();

    for (let k = 0; k < keys.length; k++) {
      ws.send(`ADD ${keys[i]} ${values[i]}`);
    }

    let endTime = performance.now();

    console.log(`Run ${i} took ${endTime - startTime}ms`);

    timeAvg += endTime - startTime;
  }

  timeAvg /= 100;

  console.log(`Took on average: ${timeAvg}ms`);

  ws.close();
});
