import { native } from "./native";
import { registerOnMessage } from "./Messaging";

interface RegisterFileMessage {
  downloadPath: string;
  savePath: string[];
  key: string;
}
const initiatedDownloads: Map<number, RegisterFileMessage> = new Map();

registerOnMessage(async ({ type, data, callback }) => {
  switch (type) {
    case "download": {
      const config = await native.sendNativeMessage("config", undefined);

      const downloadPath = `${
        config.data.downloadSubdirectory
      }/${Date.now()}-${data.key.replace(/[./\/]/, "_")}`;
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
      const res = await native.sendNativeMessage("fetch_file_states", {
        query: data.keys,
      });
      callback({
        result: res.data.result,
      });
      break;
    }
  }
});

chrome.downloads.onChanged.addListener(async (delta) => {
  if (
    delta.state?.previous !== "complete" &&
    delta.state?.current === "complete"
  ) {
    const id = delta.id;
    const message = initiatedDownloads.get(id);
    if (!message) return;

    await native.sendNativeMessage("register_file", message);
    await chrome.downloads.erase({
      id,
    });
    initiatedDownloads.delete(id);
  }
});
