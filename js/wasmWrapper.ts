import wasm from "../Cargo.toml";

let greet = Function;
async function loadWasm() {
    const exports = await wasm();
    exports.greet();
    greet = exports.greet;
    // Use functions which were exported from Rust...
    return exports;
}

export default exports;
