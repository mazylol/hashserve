import fetch from "node-fetch";
import { readFileSync } from "fs";
import { exit } from "process";

const readFileLines = (filename) =>
  readFileSync(filename).toString().split("\n");

let keys = readFileLines("keys.txt");
let values = readFileLines("values.txt");

let timeAvg = 0;

let overallStartTime = performance.now();

for (let k = 0; k < keys.length - 1; k++) {
  let startTime = performance.now();

  // url: http://localhost:3000?password=1234 body: ADD ${keys[k]} ${values[k]}
  await fetch("http://localhost:3000?password=1234", {
    method: "POST",
    body: `ADD ${keys[k]} ${values[k]}`,
    headers: {
      "Content-Type": "text/plain",
    },
  });

  let endTime = performance.now();

  console.log(`Run ${k} took ${endTime - startTime}ms`);

  timeAvg += endTime - startTime;
}

let overallEndTime = performance.now();

console.log(`Overall took: ${overallEndTime - overallStartTime}ms`);

timeAvg /= 1000;

console.log(`Took on average: ${timeAvg}ms`);

exit(0);
