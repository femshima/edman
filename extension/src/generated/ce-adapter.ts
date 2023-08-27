const EDMAN_UNIQUE_NAME = "edman";
/*
 Generated by typeshare 1.7.0
*/

export interface ConfigRequest {
}

export interface Config {
	download_directory: string;
	download_subdirectory: string;
	save_file_directory: string;
	allowed_origins: string[];
	allowed_extensions: string[];
}

export interface ConfigReply {
	config?: Config;
}

export interface RegisterFileRequest {
	path: string;
	key: string;
}

export interface RegisterFileReply {
	id: number;
}

export interface GetFileStatesRequest {
	keys: string[];
}

export interface GetFileStatesReply {
	result: boolean[];
}

export type NativeMessage = 
	| { type: "register_file", data: {
	downloadPath: string;
	savePath: string[];
	key: string;
}}
	| { type: "fetch_file_states", data: {
	query: string[];
}};

export type NativeResult = 
	| { type: "register_file", data: RegisterFileReply }
	| { type: "fetch_file_states", data: GetFileStatesReply }
	| { type: "err", data: string };


type NativeTypes = NativeMessage['type'];
type INativeMessage<T extends NativeTypes> = NativeMessage & { type: T };
type INativeResult<T extends NativeTypes> = NativeResult & { type: T };
export async function sendNativeMessage<T extends NativeTypes>(type: T, data: INativeMessage<T>["data"]): Promise<INativeResult<T>> {
    const result = await chrome.runtime.sendNativeMessage(EDMAN_UNIQUE_NAME, { type, data }) as NativeResult;
    if (result.type === type) {
    return result as NativeResult & { type: T };
    } else if (result.type === 'err') {
    throw new Error(`Native process returned error: ${result.data}`);
    } else {
    throw new Error(`Return type mismatch: expected ${type} but got ${result.type}`);
    }
}

