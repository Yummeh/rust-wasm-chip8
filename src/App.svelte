<script lang="ts">
	export let name: string;
	import init, {
		greet,
		start,
		stop_program,
		test_comp,
	} from "chip8_rust_wasm";
	import { afterUpdate, beforeUpdate, onMount } from "svelte";

	// await init
	function start_chip8() {
		start();
		// greet();
		// draw_to_canvas();
		// console.log("Hello");
		// ctx.fillRect(10, 10, 150, 100);
	}

	function update_chip8() {}

	function draw_chip8() {}

	afterUpdate(() => {
		// draw_to_canvas();
	});

	function drawPixel(
		x: number,
		y: number,
		ctx: CanvasRenderingContext2D,
		sW: number,
		sH: number
	) {
		let pos = { x: x * sW, y: y * sH };
		// console.log(pos);
		ctx.fillRect(pos.x, pos.y, sW, sH);
	}

	function normalCanvas() {
		const canvas = document.getElementById(
			"chip8_canvas"
		) as HTMLCanvasElement;
		const ctx = canvas.getContext("2d");
		ctx.imageSmoothingEnabled = false;
		ctx.fillStyle = "green";
		// ctx.fillRect(200, 199, 1, 1);

		// Determine w & h based on ctx w & h and specified w & h

		let virtualCanvasSize = { w: 10, h: 10 };

		let w = canvas.width / virtualCanvasSize.w;
		let h = canvas.height / virtualCanvasSize.h;

		drawPixel(9, 9, ctx, w, h);
	}

	function webGLCanvas() {
		const canvas = document.getElementById(
			"chip8_canvas"
		) as HTMLCanvasElement;
		const gl = canvas.getContext("webgl");

		if (gl === null) {
			console.error("Could not start webgl rendering!");
			return;
		}

		gl.clearColor(0.0, 0.0, 0.0, 1.0);
		// gl.draw
		gl.clear(gl.COLOR_BUFFER_BIT);
	}

	onMount(() => {
		// webGLCanvas();
		// fetch("somefile.txt")
		// 	.then((res) => res.json())
		// 	.then((data) => {
		// 		console.log(data);
		// 	});
	});
</script>

<main>
	<input type="file" id="chip8-file-input" />

	<h1>Hello {name}!</h1>
	<p>
		<!-- Visit the <a href="https://svelte.dev/tutorial">Svelte tutorial</a> to learn -->
		<!-- how to build Svelte apps. Test -->
	</p>

	<canvas id="chip8_canvas" width="800" height="400" />

	<div class="next-line">
		<button id="press-me-button" on:click={start_chip8}>Start</button>
		<button id="press-me-button" on:click={update_chip8}>Update</button>
		<button id="press-me-button" on:click={draw_chip8}>Draw</button>
		<button
			id="stop_button"
			on:click={() => {
				stop_program();
			}}>Stop</button
		>
		<button id="press-me-button" on:click={test_comp}>Test compilation</button>

	</div>
</main>

<style>
	main {
		font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
			Oxygen, Ubuntu, Cantarell, "Open Sans", "Helvetica Neue", sans-serif;
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}
	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
		/* background-color: red; */
	}
	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
	.next-line {
		margin-top: 30px;
	}

	#chip8_canvas {
		width: 400px;
		height: 200px;

		margin-top: 10px;

		/* background-color: red; */
		box-shadow: 0px 0px 5px 5px rgb(160, 160, 160);
		/* background-color: red; */
	}

	.container {
		width: 100%;
		align-content: center;
		justify-content: center;
	}

	#press-me-button {
		justify-self: center;

		color: #ff3e00;
		/* width: 50px; */
		padding: 15px;
		border-radius: 5px;
		border-width: 5px;
		border-color: #ff3e00;
		background-color: white;
		-webkit-user-select: none; /* Safari */
		-moz-user-select: none; /* Firefox */
		-ms-user-select: none; /* IE10+/Edge */
		user-select: none; /* Standard */

		/* box-shadow: 0px 5px 5px 5px lightgray; */
	}

	#press-me-button:active {
		/* background-color: aliceblue; */
		color: #ffbfab;
		background-color: #f2f2f2;
		/* box-shadow: 0px 0px 0px 0px white; */
	}
</style>
