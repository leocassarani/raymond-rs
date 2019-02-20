import { Scene } from "raymond";
import { memory } from "raymond/raymond_bg";

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext('2d');

const scene = Scene.new();
const length = canvas.width * canvas.height;
const buf = new Uint8ClampedArray(memory.buffer, scene.render(), length << 2);
const imageData = new ImageData(buf, canvas.width, canvas.height);
ctx.putImageData(imageData, 0, 0);
