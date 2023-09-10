export type FileStatesRequest = {
  keys: string[];
};
export type FileStatesReply = {
  result: boolean[];
};

export type DownloadRequest = {
  savePath: string[];
  url: string;
  key: string;
};
export type DownloadReply = undefined;

interface ReqResPair<Q, R> {
  request: Q;
  response: R;
}

interface ReqRes {
  file_states: ReqResPair<FileStatesRequest, FileStatesReply>;
  download: ReqResPair<DownloadRequest, DownloadReply>;
}

export async function sendMessage<T extends keyof ReqRes>(
  type: T,
  data: ReqRes[T]["request"],
): Promise<ReqRes[T]["response"]> {
  return chrome.runtime.sendMessage({
    type,
    data,
  });
}

interface CallbackArgs<T extends keyof ReqRes> {
  type: T;
  data: ReqRes[T]["request"];
  sender: chrome.runtime.MessageSender;
  callback: (ret: ReqRes[T]["response"]) => void;
}

type ArgUnion = CallbackArgs<"download"> | CallbackArgs<"file_states">;
export async function registerOnMessage(handler: (args: ArgUnion) => void) {
  chrome.runtime.onMessage.addListener((message, sender, cb) => {
    handler({ type: message.type, data: message.data, sender, callback: cb });
    return true;
  });
}
