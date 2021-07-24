// import * as _ from 'lodash';
// // import wasm 
// // import("../pkg").catch(console.error);
// import { main_js } from "../pkg/rust_chip8_wasm";

// main_js();

// function component() {
//     const element = document.createElement('div');

//     element.innerHTML = _.join(['Hello', 'webpack'], ' ');

//     return element;
// }

// document.body.appendChild(component());



import App from './App.svelte';

const app = new App({
    target: document.body,
    props: {
        name: 'world'
    }
});

export default app;