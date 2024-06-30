import { WebSocket } from "ws";
import { readFileSync } from "fs";

const readFileLines = (filename: string) =>
  readFileSync(filename).toString().split("\n");

let keys = readFileLines("keys.txt");
let values = readFileLines("values.txt");

const ws = new WebSocket("ws://127.0.0.1:3000?password=balls");

ws.on("error", console.error);

ws.on("open", async function open() {
  let timeAvg = 0;

  for (let i = 0; i <= 999; i++) {
    let startTime = performance.now();

    for (let k = 0; k < keys.length; k++) {
      ws.send(`ADD ${keys[i]} ${values[i]}`);
    }

    let endTime = performance.now();

    console.log(`Run ${i} took ${endTime - startTime}ms`);

    timeAvg += endTime - startTime;
  }

  console.log(`Took ${timeAvg/1000}s to ADD 1000 values`);

  timeAvg /= 1000;

  console.log(`Took on average: ${timeAvg}ms`);

  ws.close();
});
