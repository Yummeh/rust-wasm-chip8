import App from './App.svelte';
import init, { draw_to_canvas } from "chip8_rust_wasm";

await init();

const app = new App({
    target: document.body,
    props: {
        name: 'world'
    }
});

export default app;