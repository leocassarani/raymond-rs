import { Image, Scene } from "raymond";
import { memory } from "raymond/raymond_bg";

const scene = new Scene();

const canvas = document.getElementById("canvas");
const image = new Image(canvas.width, canvas.height);

const length = canvas.width * canvas.height;
const pixels = new Uint8ClampedArray(memory.buffer, image.pixels(), length << 2);
const imageData = new ImageData(pixels, canvas.width, canvas.height);

const ctx = canvas.getContext('2d');

scene.render(image);
ctx.putImageData(imageData, 0, 0);
