use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let generated = include_str!(concat!(env!("OUT_DIR"), "/generated.ts"));
    let function = r#"
type NativeTypes = NativeMessage['type'];
type INativeMessage<T extends NativeTypes> = NativeMessage & { type: T };
type INativeResult<T extends NativeTypes> = NativeResult & { type: T };
export async function sendNativeMessage<T extends NativeTypes>(type: T, data: INativeMessage<T>["data"]): Promise<INativeResult<T>> {
    const result = await chrome.runtime.sendNativeMessage("edman", { type, data }) as NativeResult;
    if (result.type === type) {
    return result as NativeResult & { type: T };
    } else if (result.type === 'err') {
    throw new Error(`Native process returned error: ${result.data}`);
    } else {
    throw new Error(`Return type mismatch: expected ${type} but got ${result.type}`);
    }
}

"#;

    let ts = format!("{}{}", generated, function);

    let file = std::env::args().skip(1).next().unwrap();
    std::fs::File::create(file)?.write_all(ts.as_bytes())?;

    Ok(())
}