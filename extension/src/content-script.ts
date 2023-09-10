import { sendMessage } from "./Messaging";

console.log("Hello from Content Script!");

sendMessage("file_states", { keys: [] }).then((data) => console.log(data));
