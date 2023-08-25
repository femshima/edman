import { sendNativeMessage } from "./generated/ce-adapter";
import { registerOnMessage } from "./Messaging";

interface RegisterFileMessage {
  downloadPath: string,
  savePath: string[],
  key: string,
}
const initiatedDownloads: Map<number, RegisterFileMessage> = new Map();

registerOnMessage(async ({ type, data, callback }) => {
  switch (type) {
    case "download": {
      const downloadPath = `edman/${Date.now()}-${data.key.replace(/[./\/]/, "")}`;
      const id = await chrome.downloads.download({
        url: data.url,
        filename: downloadPath,
      });
      initiatedDownloads.set(id, {
        downloadPath,
        savePath: data.savePath,
        key: data.key,
      });
      callback(undefined);
      break;
    }
    case "file_states": {
      const res = await sendNativeMessage("fetch_file_states", {
        query: data.keys
      });
      callback({
        result: res.data.result
      });
      break;
    }
  }
})

chrome.downloads.onChanged.addListener(async (delta) => {
  if (delta.state?.previous !== "complete" && delta.state?.current === "complete") {
    const id = delta.id;
    const message = initiatedDownloads.get(id);
    if (!message) return;

    await sendNativeMessage("register_file", message);
    await chrome.downloads.erase({
      id
    });
    initiatedDownloads.delete(id);
  }
})
