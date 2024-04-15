const wsm = require("ws");
const fs = require("fs");

const readFileLines = filename =>
    fs.readFileSync(filename).toString('UTF8').split('\n');

let keys = readFileLines("keys.txt");
let values = readFileLines("values.txt");

const ws = new wsm.WebSocket("ws://127.0.0.1:3000/ws?password=balls")

function sleep(ms = 0) {
  return new Promise(resolve => setTimeout(resolve, ms));
}


ws.on('error', console.error);

ws.on('open', async function open() {
    for (let i = 0; i < keys.length; i++) {
        ws.send(`ADD ${keys[i]} ${values[i]}`)
        await sleep(50);
    }
})
